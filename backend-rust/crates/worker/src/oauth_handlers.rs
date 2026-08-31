use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::header::{COOKIE, LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use hesocial_core::auth::{USER_SELECT_BY_ID, UserRow, user_select};
use hesocial_core::oauth::{
    CallbackAction, GOOGLE_CALLBACK_PATH, GOOGLE_TOKEN_URL, GOOGLE_USERINFO_URL, GoogleProfile,
    STATE_COOKIE_NAME, evaluate_callback, failure_redirect_url, google_consent_url,
    needs_profile_completion, parse_google_userinfo, random_oauth_state, read_cookie,
    state_clear_cookie, state_set_cookie, success_redirect_url, token_exchange_body,
};
use serde_json::{Value, json};
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;
use worker::{
    Fetch, Headers as WorkerHeaders, Method as WorkerMethod, Request as WorkerRequest, RequestInit,
};

use crate::AppState;
use crate::auth::find_user_by_email;
use crate::auth_handlers::{issue_token, now_iso};

const GOOGLE_PICTURE_UPDATE: &str =
    "UPDATE users SET profile_picture = COALESCE(?, profile_picture), updated_at = ? WHERE id = ?";

// Same column list and values as the Express passport strategy insert: the
// financial/profile fields start NULL and the user completes them via
// /complete-profile. password_hash/password_algo/role take the D1 defaults.
const GOOGLE_INSERT: &str = "INSERT INTO users (id, email, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, profile_picture, bio, interests, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

fn env_var(state: &AppState, name: &str) -> Option<String> {
    state
        .env
        .var(name)
        .ok()
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty())
}

fn google_config(state: &AppState) -> Option<(String, String)> {
    let client_id = env_var(state, "GOOGLE_CLIENT_ID")?;
    let client_secret = env_var(state, "GOOGLE_CLIENT_SECRET")?;
    Some((client_id, client_secret))
}

fn google_oauth_unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "success": false,
            "error": "Google OAuth is not configured",
            "message": "GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must both be configured to use Google sign-in"
        })),
    )
        .into_response()
}

fn redirect(location: &str, clear_state_cookie: bool) -> Response {
    let mut response = (StatusCode::FOUND, [(LOCATION, location)]).into_response();
    if clear_state_cookie {
        if let Ok(cookie) = HeaderValue::from_str(&state_clear_cookie()) {
            response.headers_mut().append(SET_COOKIE, cookie);
        }
    }
    response
}

fn failure_redirect(frontend_origin: &str, error: &str) -> Response {
    redirect(&failure_redirect_url(frontend_origin, error), true)
}

/// Express configures `callbackURL: '/api/auth/google/callback'` and passport
/// resolves it against the originating request URL, so the redirect URI is
/// always `<request origin>/api/auth/google/callback`.
fn callback_url(uri: &Uri) -> Option<String> {
    let scheme = uri.scheme_str()?;
    let authority = uri.authority()?;
    Some(format!("{scheme}://{authority}{GOOGLE_CALLBACK_PATH}"))
}

pub async fn google_start(State(state): State<AppState>, uri: Uri) -> Response {
    SendFuture::new(google_start_inner(state, uri)).await
}

async fn google_start_inner(state: AppState, uri: Uri) -> Response {
    let Some((client_id, _)) = google_config(&state) else {
        return google_oauth_unavailable();
    };
    let frontend_origin = state.frontend_origin().to_owned();
    let Some(redirect_uri) = callback_url(&uri) else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };
    let Ok(oauth_state) = random_oauth_state() else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };

    let consent_url = google_consent_url(&client_id, &redirect_uri, &oauth_state);
    let mut response = redirect(&consent_url, false);
    if let Ok(cookie) = HeaderValue::from_str(&state_set_cookie(&oauth_state)) {
        response.headers_mut().append(SET_COOKIE, cookie);
    }
    response
}

pub async fn google_callback(
    State(state): State<AppState>,
    uri: Uri,
    Query(query): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    SendFuture::new(google_callback_inner(state, uri, query, headers)).await
}

async fn google_callback_inner(
    state: AppState,
    uri: Uri,
    query: HashMap<String, String>,
    headers: HeaderMap,
) -> Response {
    let Some((client_id, client_secret)) = google_config(&state) else {
        return google_oauth_unavailable();
    };
    let frontend_origin = state.frontend_origin().to_owned();

    let cookie_state = headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|header| read_cookie(header, STATE_COOKIE_NAME))
        .map(str::to_owned);

    let action = evaluate_callback(
        query.get("error").map(String::as_str),
        query.get("code").map(String::as_str),
        query.get("state").map(String::as_str),
        cookie_state.as_deref(),
    );
    let CallbackAction::ExchangeCode(code) = action else {
        return failure_redirect(&frontend_origin, "oauth_failed");
    };

    let Some(redirect_uri) = callback_url(&uri) else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };
    let Some(access_token) = exchange_code(&client_id, &client_secret, &code, &redirect_uri).await
    else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };
    let Some(profile) = fetch_userinfo(&access_token).await else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };
    let Some(user) = upsert_google_user(&state, &profile).await else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };
    let Some(token) = issue_token(&state, &user) else {
        return failure_redirect(&frontend_origin, "oauth_error");
    };

    let needs_completion = needs_profile_completion(
        user.age,
        user.profession.as_deref(),
        user.annual_income,
        user.net_worth,
    );
    redirect(
        &success_redirect_url(&frontend_origin, &token, needs_completion),
        true,
    )
}

/// POST the authorization code to Google's token endpoint; returns the
/// access token. Any transport or protocol failure is `None` (-> oauth_error).
async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Option<String> {
    let headers = WorkerHeaders::new();
    headers
        .set("Content-Type", "application/x-www-form-urlencoded")
        .ok()?;
    headers.set("Accept", "application/json").ok()?;

    let body = token_exchange_body(client_id, client_secret, code, redirect_uri);
    let mut init = RequestInit::new();
    init.with_method(WorkerMethod::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(&body)));

    let request = WorkerRequest::new_with_init(GOOGLE_TOKEN_URL, &init).ok()?;
    let mut response = Fetch::Request(request).send().await.ok()?;
    if response.status_code() != 200 {
        return None;
    }
    let json: Value = response.json().await.ok()?;
    json.get("access_token")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

async fn fetch_userinfo(access_token: &str) -> Option<GoogleProfile> {
    let headers = WorkerHeaders::new();
    headers
        .set("Authorization", &format!("Bearer {access_token}"))
        .ok()?;
    headers.set("Accept", "application/json").ok()?;

    let mut init = RequestInit::new();
    init.with_method(WorkerMethod::Get).with_headers(headers);

    let request = WorkerRequest::new_with_init(GOOGLE_USERINFO_URL, &init).ok()?;
    let mut response = Fetch::Request(request).send().await.ok()?;
    if response.status_code() != 200 {
        return None;
    }
    let json: Value = response.json().await.ok()?;
    parse_google_userinfo(&json)
}

/// Find-or-create by email, mirroring the Express passport strategy exactly:
/// existing users only get `profile_picture = COALESCE(new, existing)` +
/// `updated_at`; new users are inserted with NULL financial/profile fields,
/// tier Platinum, privacy 3, unverified/pending, empty interests.
async fn upsert_google_user(state: &AppState, profile: &GoogleProfile) -> Option<UserRow> {
    let db = state.env.d1("DB").ok()?;
    let timestamp = now_iso();

    let existing = find_user_by_email(state, JsValue::from_str(&profile.email))
        .await
        .ok()?;

    if let Some(existing) = existing {
        let picture = profile
            .picture
            .as_deref()
            .map(JsValue::from_str)
            .unwrap_or(JsValue::NULL);
        let update = db
            .prepare(GOOGLE_PICTURE_UPDATE)
            .bind(&[
                picture,
                JsValue::from_str(&timestamp),
                JsValue::from_str(&existing.id),
            ])
            .ok()?;
        update.run().await.ok()?;
        return select_user(&db, &existing.id).await;
    }

    let user_id = hesocial_core::auth::new_uuid_v4().ok()?;
    // Express binds `profile.photos?.[0]?.value || null`: an empty string is
    // stored as NULL, unlike the update branch's plain COALESCE.
    let picture = profile
        .picture
        .as_deref()
        .filter(|picture| !picture.is_empty())
        .map(JsValue::from_str)
        .unwrap_or(JsValue::NULL);
    let insert = db
        .prepare(GOOGLE_INSERT)
        .bind(&[
            JsValue::from_str(&user_id),
            JsValue::from_str(&profile.email),
            JsValue::from_str(&profile.first_name),
            JsValue::from_str(&profile.last_name),
            JsValue::NULL, // age - to be filled by user
            JsValue::NULL, // profession - to be filled by user
            JsValue::NULL, // annual_income - to be filled by user
            JsValue::NULL, // net_worth - to be filled by user
            JsValue::from_str("Platinum"),
            JsValue::from_f64(3.0),
            JsValue::from_f64(0.0),
            JsValue::from_str("pending"),
            picture,
            JsValue::NULL,           // bio
            JsValue::from_str("[]"), // empty interests array
            JsValue::from_str(&timestamp),
            JsValue::from_str(&timestamp),
        ])
        .ok()?;
    insert.run().await.ok()?;
    select_user(&db, &user_id).await
}

async fn select_user(db: &worker::D1Database, user_id: &str) -> Option<UserRow> {
    let query = db
        .prepare(user_select(USER_SELECT_BY_ID))
        .bind(&[JsValue::from_str(user_id)])
        .ok()?;
    query.first(None).await.ok()?
}

pub async fn linkedin_unavailable() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "success": false,
            "error": "LinkedIn OAuth not implemented yet",
            "message": "LinkedIn OAuth will be added in next phase"
        })),
    )
        .into_response()
}
