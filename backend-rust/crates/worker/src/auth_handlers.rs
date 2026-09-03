use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::auth::{
    LEGACY_PASSWORD_ALGORITHM, USER_SELECT_BY_ID, UserRow, membership_tier_for, new_uuid_v4,
    user_json, user_select,
};
use hesocial_core::pbkdf2::{PBKDF2_ALGORITHM, hash_password, verify_password as verify_pbkdf2};
use hesocial_core::{ApiEnvelope, JwtPayload, sign_jwt, verify_password};
use serde_json::{Value, json};
use worker::js_sys::Date;
use worker::send::SendFuture;

use crate::AppState;
use crate::auth::{
    authenticate, find_user_by_email, internal_error, jwt_expiry_seconds, jwt_secret, now_seconds,
    unauthorized,
};
use crate::db::{self, Val};

const REGISTER_INSERT: &str = "INSERT INTO users (id, email, password_hash, password_algo, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, bio, interests, gender, interested_in, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const EXISTING_EMAIL_SELECT: &str = "SELECT id FROM users WHERE email = ? AND deleted_at IS NULL";

const REHASH_UPDATE: &str = "UPDATE users SET password_hash = ?, password_algo = ? WHERE id = ?";

const UPDATE_PROFILE: &str = "UPDATE users SET first_name = COALESCE(?, first_name), last_name = COALESCE(?, last_name), age = COALESCE(?, age), profession = COALESCE(?, profession), bio = COALESCE(?, bio), interests = COALESCE(?, interests), gender = COALESCE(?, gender), interested_in = COALESCE(?, interested_in), privacy_level = COALESCE(?, privacy_level), updated_at = ? WHERE id = ?";

pub(crate) fn now_iso() -> String {
    Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_default()
}

fn to_js(value: &Value) -> Value {
    match value {
        Value::Null => db::NULL,
        Value::Bool(flag) => Val::from_bool(*flag),
        Value::Number(number) => number.as_f64().map(Val::from_f64).unwrap_or(db::NULL),
        Value::String(text) => Val::from_str(text),
        _ => db::NULL,
    }
}

fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(f64::is_normal),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

fn body_string<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    body.get(key).and_then(Value::as_str)
}

fn body_number(body: &Value, key: &str) -> f64 {
    body.get(key).and_then(Value::as_f64).unwrap_or(f64::NAN)
}

fn optional_text(body: &Value, key: &str) -> Value {
    body.get(key)
        .filter(|value| js_truthy(value))
        .map(to_js)
        .unwrap_or(db::NULL)
}

fn interests_column(body: &Value) -> String {
    let interests = match body.get("interests") {
        Some(value) if js_truthy(value) => value.clone(),
        _ => json!([]),
    };
    interests.to_string()
}

fn registration_failed() -> Response {
    internal_error("Registration failed")
}

pub(crate) fn issue_token(state: &AppState, user: &UserRow) -> Option<String> {
    let secret = jwt_secret(state)?;
    let payload = JwtPayload::with_expiry(
        user.id.clone(),
        user.email.clone(),
        user.membership_tier.clone(),
        now_seconds(),
        jwt_expiry_seconds(state),
    );
    sign_jwt(&payload, &secret).ok()
}

pub async fn register(State(state): State<AppState>, Json(body): Json<Value>) -> Response {
    SendFuture::new(register_inner(state, body)).await
}

async fn register_inner(state: AppState, body: Value) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return registration_failed(),
    };

    let email = to_js(body.get("email").unwrap_or(&Value::Null));
    let existing = match db
        .prepare(EXISTING_EMAIL_SELECT)
        .bind(std::slice::from_ref(&email))
    {
        Ok(query) => query,
        Err(_) => return registration_failed(),
    };
    match existing.first::<Value>(None).await {
        Ok(None) => (),
        Ok(Some(_)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiEnvelope::<Value>::error(
                    "User with this email already exists",
                )),
            )
                .into_response();
        }
        Err(_) => return registration_failed(),
    }

    let Some(password) = body_string(&body, "password") else {
        return registration_failed();
    };
    let hashed_password = match hash_password(password) {
        Ok(hashed) => hashed,
        Err(_) => return registration_failed(),
    };

    let user_id = match new_uuid_v4() {
        Ok(user_id) => user_id,
        Err(_) => return registration_failed(),
    };
    let membership_tier = membership_tier_for(
        body_number(&body, "netWorth"),
        body_number(&body, "annualIncome"),
    );
    let timestamp = now_iso();

    let insert = db.prepare(REGISTER_INSERT).bind(&[
        Val::from_str(&user_id),
        email,
        Val::from_str(&hashed_password),
        Val::from_str(PBKDF2_ALGORITHM),
        to_js(body.get("firstName").unwrap_or(&Value::Null)),
        to_js(body.get("lastName").unwrap_or(&Value::Null)),
        to_js(body.get("age").unwrap_or(&Value::Null)),
        to_js(body.get("profession").unwrap_or(&Value::Null)),
        to_js(body.get("annualIncome").unwrap_or(&Value::Null)),
        to_js(body.get("netWorth").unwrap_or(&Value::Null)),
        Val::from_str(membership_tier),
        Val::from_f64(3.0),
        Val::from_f64(0.0),
        Val::from_str("pending"),
        optional_text(&body, "bio"),
        Val::from_str(&interests_column(&body)),
        to_js(body.get("gender").unwrap_or(&Value::Null)),
        to_js(body.get("interestedIn").or_else(|| body.get("interested_in")).unwrap_or(&Value::Null)),
        Val::from_str(&timestamp),
        Val::from_str(&timestamp),
    ]);
    let insert = match insert {
        Ok(query) => query,
        Err(_) => return registration_failed(),
    };
    if insert.run().await.is_err() {
        return registration_failed();
    }

    let created = db
        .prepare(user_select(USER_SELECT_BY_ID))
        .bind(&[Val::from_str(&user_id)]);
    let created = match created {
        Ok(query) => query,
        Err(_) => return registration_failed(),
    };
    let user: Option<UserRow> = match created.first(None).await {
        Ok(user) => user,
        Err(_) => return registration_failed(),
    };
    let Some(user) = user else {
        return registration_failed();
    };

    let Some(token) = issue_token(&state, &user) else {
        return registration_failed();
    };

    (
        StatusCode::CREATED,
        Json(ApiEnvelope::success_with_message(
            json!({ "user": user_json(&user), "token": token }),
            "User registered successfully",
        )),
    )
        .into_response()
}

pub async fn login(State(state): State<AppState>, Json(body): Json<Value>) -> Response {
    SendFuture::new(login_inner(state, body)).await
}

async fn login_inner(state: AppState, body: Value) -> Response {
    let email = to_js(body.get("email").unwrap_or(&Value::Null));
    let user = match find_user_by_email(&state, email).await {
        Ok(user) => user,
        Err(_) => return internal_error("Login failed"),
    };
    let Some(user) = user else {
        return unauthorized("Invalid email or password");
    };

    let Some(password) = body_string(&body, "password") else {
        return internal_error("Login failed");
    };
    let Some(stored_hash) = user.password_hash.as_deref() else {
        return internal_error("Login failed");
    };

    let algorithm = user
        .password_algo
        .as_deref()
        .unwrap_or(LEGACY_PASSWORD_ALGORITHM);
    let valid = if algorithm == PBKDF2_ALGORITHM {
        verify_pbkdf2(password, stored_hash)
    } else {
        verify_password(password, stored_hash)
    };
    if !valid {
        return unauthorized("Invalid email or password");
    }

    if algorithm != PBKDF2_ALGORITHM {
        if let Ok(rehashed) = hash_password(password) {
            if let Ok(db) = db::Db::from_env(&state.env) {
                if let Ok(update) = db.prepare(REHASH_UPDATE).bind(&[
                    Val::from_str(&rehashed),
                    Val::from_str(PBKDF2_ALGORITHM),
                    Val::from_str(&user.id),
                ]) {
                    let _ = update.run().await;
                }
            }
        }
    }

    let Some(token) = issue_token(&state, &user) else {
        return internal_error("Login failed");
    };

    Json(ApiEnvelope::success_with_message(
        json!({ "user": user_json(&user), "token": token }),
        "Login successful",
    ))
    .into_response()
}

pub async fn profile(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    Json(ApiEnvelope::success_with_message(
        json!({ "user": user_json(&user) }),
        "Profile retrieved successfully",
    ))
    .into_response()
}

pub async fn refresh(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let Some(token) = issue_token(&state, &user) else {
        return internal_error("Failed to refresh token");
    };

    Json(ApiEnvelope::success_with_message(
        json!({ "token": token }),
        "Token refreshed successfully",
    ))
    .into_response()
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(_) => Json(json!({
            "success": true,
            "message": "Logged out successfully"
        }))
        .into_response(),
        Err(response) => response,
    }
}

pub async fn validate(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    Json(ApiEnvelope::success_with_message(
        json!({ "user": user_json(&user), "valid": true }),
        "Token is valid",
    ))
    .into_response()
}

pub async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    SendFuture::new(update_profile_inner(state, user, body)).await
}

async fn update_profile_inner(state: AppState, user: UserRow, body: Value) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to update profile"),
    };

    // Express: `interests ? JSON.stringify(interests) : null` (JS truthiness);
    // every other field binds the raw value and COALESCE keeps the column on
    // JSON null/absent.
    let interests = match body.get("interests") {
        Some(value) if js_truthy(value) => Val::from_str(&value.to_string()),
        _ => db::NULL,
    };
    let update = db.prepare(UPDATE_PROFILE).bind(&[
        to_js(body.get("firstName").unwrap_or(&Value::Null)),
        to_js(body.get("lastName").unwrap_or(&Value::Null)),
        to_js(body.get("age").unwrap_or(&Value::Null)),
        to_js(body.get("profession").unwrap_or(&Value::Null)),
        to_js(body.get("bio").unwrap_or(&Value::Null)),
        interests,
        to_js(body.get("gender").unwrap_or(&Value::Null)),
        to_js(body.get("interestedIn").or_else(|| body.get("interested_in")).unwrap_or(&Value::Null)),
        to_js(body.get("privacyLevel").unwrap_or(&Value::Null)),
        Val::from_str(&now_iso()),
        Val::from_str(&user.id),
    ]);
    let update = match update {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to update profile"),
    };
    if update.run().await.is_err() {
        return internal_error("Failed to update profile");
    }

    let updated = db
        .prepare(user_select(USER_SELECT_BY_ID))
        .bind(&[Val::from_str(&user.id)]);
    let updated = match updated {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to update profile"),
    };
    let updated: Option<UserRow> = match updated.first(None).await {
        Ok(user) => user,
        Err(_) => return internal_error("Failed to update profile"),
    };
    let Some(updated) = updated else {
        return internal_error("Failed to update profile");
    };

    Json(ApiEnvelope::success_with_message(
        json!({ "user": user_json(&updated) }),
        "Profile updated successfully",
    ))
    .into_response()
}
