//! Visitor tracking and analytics, backed by the `visitor_sessions` /
//! `visitor_page_views` / `visitor_events` tables in Turso.
//!
//! Phase 2g originally put the write path in Cloudflare Analytics Engine and
//! read it back over the AE SQL API, because D1 could not absorb a write per
//! page view. Dropping D1 removed that constraint, and with it the reasons to
//! keep tracking in a second store reachable only with a separate Cloudflare
//! API token. The tables here are the ones Express already writes
//! (`database/migrations/005_visitor_tracking.sql`), so the two backends now
//! agree on the data model as well as the responses.
//!
//! Response shapes are unchanged: `crates/core`'s envelope shapers still take
//! `{alias: value}` rows and still emit `*_ms` epoch numbers, so the SQL below
//! converts stored ISO timestamps rather than the shapers changing.
//!
//! - Reads are admin-guarded; the tracking middleware and `POST
//!   /api/analytics/events/track` are not (see the route table in `lib.rs`).
//! - The middleware never resolves the user, so `user_id` stays NULL on the
//!   session row until something else links it.

use std::collections::HashMap;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{COOKIE, REFERER, SET_COOKIE, USER_AGENT};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use hesocial_core::analytics::{
    AnalyticsQuery, INSERT_PAGE_VIEW_SQL, INSERT_VISITOR_EVENT_SQL, UPSERT_VISITOR_SESSION_SQL,
    conversion_envelope, conversion_sql, events_engagement_envelope, events_engagement_sql,
    limit_param, period_days, popular_pages_envelope, popular_pages_sql, track_success_json,
    validate_track_body, visitor_detail_envelope, visitor_page_views_sql, visitor_session_sql,
    visitors_daily_envelope, visitors_daily_sql, visitors_overview_envelope, visitors_overview_sql,
};
use serde_json::{Map, Value, json};
use worker::Env;
use worker::js_sys::Date;
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;

use crate::AppState;
use crate::auth::{authenticate, require_admin};
use crate::auth_handlers::now_iso;
use crate::db::{self, Val};

const VISITOR_ID_HEADER: HeaderName = HeaderName::from_static("x-visitor-id");
/// Client IP as Cloudflare presents it; Express read it off the socket.
const CONNECTING_IP_HEADER: HeaderName = HeaderName::from_static("cf-connecting-ip");
const VISITOR_COOKIE: &str = "visitorId";
const COOKIE_MAX_AGE_SECS: u64 = 365 * 24 * 60 * 60;
const COOKIE_MAX_AGE_MS: f64 = COOKIE_MAX_AGE_SECS as f64 * 1000.0;

// ---------------------------------------------------------------------------
// Turso-backed reads
// ---------------------------------------------------------------------------

/// Run one analytics query and wrap its rows in the envelope shapers' input
/// type. Rows come back already untagged by the libSQL layer, so they are the
/// same `{alias: value}` objects the Analytics Engine SQL API used to return
/// and every shaper below is unchanged.
async fn query_rows(state: &AppState, sql: String, binds: &[Value]) -> Result<AnalyticsQuery, ()> {
    let db = db::Db::from_env(&state.env).map_err(|_| ())?;
    let result = db.prepare(sql).bind(binds)?.all().await?;
    Ok(AnalyticsQuery {
        data: result.results::<Value>()?,
    })
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Query-string decoding for `blob9` and the `visitorId` fallback: `%XX`
/// triplets plus `+` as space (qs semantics for single-value params).
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' => {
                let pair = bytes
                    .get(index + 1)
                    .and_then(|hi| hex_value(*hi))
                    .zip(bytes.get(index + 2).and_then(|lo| hex_value(*lo)));
                match pair {
                    Some((hi, lo)) => {
                        out.push(hi << 4 | lo);
                        index += 3;
                    }
                    None => {
                        out.push(b'%');
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `JSON.stringify(req.query)`: last value wins on repeated keys.
fn query_params_json(raw_query: Option<&str>) -> String {
    let mut map = Map::new();
    if let Some(raw) = raw_query {
        for pair in raw.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            map.insert(percent_decode(key), Value::String(percent_decode(value)));
        }
    }
    Value::Object(map).to_string()
}

fn query_param(raw_query: Option<&str>, name: &str) -> Option<String> {
    raw_query?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (percent_decode(key) == name).then(|| percent_decode(value))
    })
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|value| value.to_str().ok())?
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.trim().to_owned())
}

/// `visitor_${uuidv4()}` — tracking ids only, never security-sensitive.
fn new_visitor_id() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::getrandom(&mut bytes).is_err() {
        bytes[..8].copy_from_slice(&Date::now().to_bits().to_le_bytes());
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let mut id = String::with_capacity(8 + 36);
    id.push_str("visitor_");
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            id.push('-');
        }
        id.push_str(&format!("{byte:02x}"));
    }
    id
}

/// Express `res.cookie('visitorId', ..., {maxAge: 1y, sameSite: 'lax'})`.
fn visitor_cookie_header(visitor_id: &str, secure: bool) -> String {
    let expires = Date::new(&JsValue::from_f64(Date::now() + COOKIE_MAX_AGE_MS))
        .to_utc_string()
        .as_string()
        .unwrap_or_default();
    let mut header = format!(
        "{VISITOR_COOKIE}={visitor_id}; Max-Age={COOKIE_MAX_AGE_SECS}; Path=/; Expires={expires}; SameSite=Lax"
    );
    if secure {
        header.push_str("; Secure");
    }
    header
}

fn is_production(env: &Env) -> bool {
    env.var("NODE_ENV")
        .is_ok_and(|value| value.to_string() == "production")
}

/// Port of Express `visitorTracking`: record one page-view data point per
/// request, mint + set the visitor cookie for new visitors, and NEVER let a
/// tracking failure affect the response.
pub async fn visitor_tracking_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    SendFuture::new(track_visitor(state, request, next)).await
}

async fn track_visitor(state: AppState, request: Request<Body>, next: Next) -> Response {
    let headers = request.headers();
    let raw_query = request.uri().query();
    let existing = headers
        .get(VISITOR_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .or_else(|| cookie_value(headers, VISITOR_COOKIE))
        .or_else(|| query_param(raw_query, VISITOR_COOKIE));
    let is_new_visitor = existing.is_none();
    let visitor_id = existing.unwrap_or_else(new_visitor_id);

    let timestamp = now_iso();
    let referer = headers
        .get(REFERER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();
    let ip_address = headers
        .get(CONNECTING_IP_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();
    let path = request.uri().path().to_owned();
    let method = request.method().as_str().to_owned();
    let query_params = query_params_json(raw_query);

    // Batched so a visitor never gains a page-view row without the session
    // counter moving with it. Tracking stays advisory: a failure is logged and
    // the request continues, exactly as the Analytics Engine write did.
    if let Ok(db) = db::Db::from_env(&state.env) {
        let session = db.prepare(UPSERT_VISITOR_SESSION_SQL).bind(&[
            Val::from_str(&visitor_id),
            // The middleware never resolves the user, so the visitor stays
            // anonymous here and NULLIF turns this into a NULL user_id.
            Val::from_str(""),
            Val::from_str(&ip_address),
            Val::from_str(&user_agent),
            Val::from_str(&referer),
            Val::from_str(&timestamp),
            Val::from_str(&timestamp),
            Val::from_str(&timestamp),
            Val::from_str(&timestamp),
        ]);
        let page_view = db.prepare(INSERT_PAGE_VIEW_SQL).bind(&[
            Val::from_str(&visitor_id),
            Val::from_str(&path),
            Val::from_str(&method),
            Val::from_str(&query_params),
            Val::from_str(&referer),
            Val::from_str(&timestamp),
            // Express leaves time_spent unset on the middleware insert.
            Val::from_f64(0.0),
            Val::from_str(&ip_address),
            Val::from_str(&user_agent),
        ]);
        match (session, page_view) {
            (Ok(session), Ok(page_view)) => {
                if db.batch(vec![session, page_view]).await.is_err() {
                    worker::console_warn!("visitor tracking write failed; continuing");
                }
            }
            _ => worker::console_warn!("visitor tracking bind failed; continuing"),
        }
    }

    let mut response = next.run(request).await;
    if is_new_visitor
        && let Ok(value) = HeaderValue::from_str(&visitor_cookie_header(
            &visitor_id,
            is_production(&state.env),
        ))
    {
        response.headers_mut().insert(SET_COOKIE, value);
    }
    response
}

// ---------------------------------------------------------------------------
// Read endpoints (AE-backed)
// ---------------------------------------------------------------------------

pub(crate) async fn require_analytics_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), Response> {
    authenticate(state, headers)
        .await
        .and_then(|user| require_admin(&user))
}

pub(crate) fn server_error(error: &'static str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"success": false, "error": error})),
    )
        .into_response()
}

async fn run_envelope(
    state: &AppState,
    sql: String,
    shape: impl FnOnce(AnalyticsQuery) -> Value,
    error: &'static str,
) -> Response {
    match query_rows(state, sql, &[]).await {
        Ok(query) => Json(shape(query)).into_response(),
        Err(()) => server_error(error),
    }
}

pub async fn visitors_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(visitors_overview_inner(state, headers, params)).await
}

async fn visitors_overview_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let days = period_days(params.get("days").map(String::as_str), 30);
    run_envelope(
        &state,
        visitors_overview_sql(days),
        |query| visitors_overview_envelope(days, &query),
        "Failed to retrieve visitor analytics",
    )
    .await
}

pub async fn visitors_daily(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(visitors_daily_inner(state, headers, params)).await
}

async fn visitors_daily_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let days = period_days(params.get("days").map(String::as_str), 30);
    run_envelope(
        &state,
        visitors_daily_sql(days),
        |query| visitors_daily_envelope(&query),
        "Failed to retrieve daily analytics",
    )
    .await
}

pub async fn popular_pages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(popular_pages_inner(state, headers, params)).await
}

async fn popular_pages_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let limit = limit_param(params.get("limit").map(String::as_str), 20);
    run_envelope(
        &state,
        popular_pages_sql(limit),
        |query| popular_pages_envelope(&query),
        "Failed to retrieve page analytics",
    )
    .await
}

pub async fn conversion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(conversion_inner(state, headers, params)).await
}

async fn conversion_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let days = period_days(params.get("days").map(String::as_str), 30);
    run_envelope(
        &state,
        conversion_sql(days),
        |query| conversion_envelope(days, &query),
        "Failed to retrieve conversion analytics",
    )
    .await
}

pub async fn visitor_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(visitor_id): Path<String>,
) -> Response {
    SendFuture::new(visitor_detail_inner(state, headers, visitor_id)).await
}

async fn visitor_detail_inner(state: AppState, headers: HeaderMap, visitor_id: String) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    // Express queries the session first and 404s before touching page views.
    let binds = [Val::from_str(&visitor_id)];
    let session = match query_rows(&state, visitor_session_sql(), &binds).await {
        Ok(session) => session,
        Err(()) => return server_error("Failed to retrieve visitor details"),
    };
    if session.data.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"success": false, "error": "Visitor not found"})),
        )
            .into_response();
    }
    let page_views = match query_rows(&state, visitor_page_views_sql(), &binds).await {
        Ok(page_views) => page_views,
        Err(()) => return server_error("Failed to retrieve visitor details"),
    };
    Json(visitor_detail_envelope(&session, &page_views)).into_response()
}

pub async fn events_engagement(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(events_engagement_inner(state, headers, params)).await
}

async fn events_engagement_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let days = period_days(params.get("days").map(String::as_str), 30);
    run_envelope(
        &state,
        events_engagement_sql(days),
        |query| events_engagement_envelope(days, &query),
        "Failed to retrieve event engagement analytics",
    )
    .await
}

// ---------------------------------------------------------------------------
// Custom-event track endpoint (write path, NOT admin-guarded — see module docs)
// ---------------------------------------------------------------------------

pub async fn track_event(State(state): State<AppState>, Json(body): Json<Value>) -> Response {
    SendFuture::new(track_event_inner(state, body)).await
}

async fn track_event_inner(state: AppState, body: Value) -> Response {
    let (visitor_id, event_type, event_data_json) = match validate_track_body(&body) {
        Ok(values) => values,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"success": false, "error": message})),
            )
                .into_response();
        }
    };
    let timestamp = now_iso();
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return server_error("Failed to track event"),
    };
    let insert = db.prepare(INSERT_VISITOR_EVENT_SQL).bind(&[
        Val::from_str(&visitor_id),
        Val::from_str(&event_type),
        Val::from_str(&event_data_json),
        Val::from_str(&timestamp),
    ]);
    // Express 500s when the insert throws, and so does this.
    match insert {
        Ok(insert) if insert.run().await.is_ok() => {}
        _ => return server_error("Failed to track event"),
    }
    Json(track_success_json()).into_response()
}
