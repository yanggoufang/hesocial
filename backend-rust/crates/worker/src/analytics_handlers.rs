//! Visitor-tracking write path + Analytics-Engine-backed read endpoints
//! (Phase 2g Stage 2).
//!
//! The pure mapping, SQL builders, and Express-exact envelope shapers live in
//! `core::analytics`; this module is the wasm/host boundary: it turns
//! [`AnalyticsDataPoint`] into `AnalyticsEngineDataPointBuilder` writes on the
//! `TRACKING` binding and POSTs SQL strings to the Analytics Engine SQL API.
//!
//! ## Write path
//!
//! - `visitor_tracking_middleware` mirrors Express `app.use(visitorTracking)`:
//!   every request is a page view. The visitor id is read from the
//!   `X-Visitor-ID` header, then the `visitorId` cookie, then the `visitorId`
//!   query param (Express precedence order). A missing id mints
//!   `visitor_<uuid v4>` and sets the `visitorId` cookie on the response
//!   (`Max-Age=1y; Path=/; SameSite=Lax`, `Secure` when `NODE_ENV=production`,
//!   no `HttpOnly` — the frontend reads it). Tracking failures never block or
//!   fail the response, exactly like the Express middleware.
//! - `POST /api/analytics/events/track` accepts the frontend beacon
//!   (`VisitorTracker.tsx`): `{visitor_id, event_type, event_data?}` plus the
//!   new optional `session_id`. Validation mirrors the Express 400; success is
//!   `{success: true, message}` with no `data` key.
//!
//! ## Query seam and the stub mechanism
//!
//! All reads go through [`AnalyticsBackend`] on [`AppState`], chosen once per
//! request from env:
//!
//! - **Stub**: when env var `ANALYTICS_QUERY_STUB` is truthy
//!   (`1`/`true`/`yes`, case-insensitive) the backend is compiled-in and
//!   returns core's `ANALYTICS_QUERY_STUB` fixture rows for every query — no
//!   file IO, no service bindings, no network. This is the documented stub
//!   mechanism for local dev and Stage 4 contract tests.
//! - **Live**: HTTPS POST to
//!   `https://api.cloudflare.com/client/v4/accounts/{CLOUDFLARE_ACCOUNT_ID}/analytics_engine/sql`
//!   with `Authorization: Bearer CLOUDFLARE_API_TOKEN` and the SQL string as
//!   the body; the response `{meta, data: [rows]}` parses into
//!   [`AnalyticsQuery`].
//! - **Unconfigured**: stub off and either env value missing — every query
//!   fails and the handlers answer the Express 500 envelope.
//!
//! ## Documented deviations (vs Express)
//!
//! - `POST /events/track` is NOT behind `authenticateToken`/`requireAdmin`.
//!   Express mounts the whole analytics router under the admin guard, but the
//!   frontend beacon posts unauthenticated, so Express 401s its own beacon.
//!   The port accepts the beacon as the frontend actually sends it; the GET
//!   read endpoints keep the admin guard. Contract tests must special-case
//!   this.
//! - `blob10`/`double4` (linked user) stay empty on page views: AE is
//!   append-only and Express's `linkVisitorToUser` rewrites past rows, which
//!   AE cannot do. Porting the link call would also touch the frozen
//!   `auth_handlers.rs`.
//! - Invalid-JSON track bodies get axum's Json rejection (400 plain text),
//!   not Express's 400 HTML — same class as the 2b register/login deviation.
//! - Query params with repeated keys keep the LAST value in `blob9`
//!   (`JSON.stringify(req.query)` makes an array in Express). Single-value
//!   params — everything the frontend sends — are identical.
//! - Non-admin-guarded tracking middleware does not resolve the user, so
//!   `user_id` is `""` for all page views (see above).

use std::collections::HashMap;

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{COOKIE, REFERER, SET_COOKIE, USER_AGENT};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use hesocial_core::analytics::{
    ANALYTICS_QUERY_STUB, AnalyticsDataPoint, AnalyticsQuery, CustomEventFields, DEFAULT_DATASET,
    PageViewFields, conversion_envelope, conversion_sql, custom_event_data_point,
    events_engagement_envelope, events_engagement_sql, limit_param, page_view_data_point,
    period_days, popular_pages_envelope, popular_pages_sql, track_success_json,
    validate_track_body, visitor_detail_envelope, visitor_page_views_sql, visitor_session_sql,
    visitors_daily_envelope, visitors_daily_sql, visitors_overview_envelope, visitors_overview_sql,
};
use serde_json::{Map, Value, json};
use worker::js_sys::Date;
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;
use worker::{AnalyticsEngineDataPointBuilder, Env, Fetch, Headers, Method, RequestInit};

use crate::AppState;
use crate::auth::{authenticate, require_admin};

/// `[[analytics_engine_datasets]]` binding name in wrangler.toml.
pub const TRACKING_BINDING: &str = "TRACKING";

/// Env var that switches every analytics read to the compiled-in stub.
pub const STUB_ENV_VAR: &str = "ANALYTICS_QUERY_STUB";

const VISITOR_ID_HEADER: HeaderName = HeaderName::from_static("x-visitor-id");
const VISITOR_COOKIE: &str = "visitorId";
const COOKIE_MAX_AGE_SECS: u64 = 365 * 24 * 60 * 60;
const COOKIE_MAX_AGE_MS: f64 = COOKIE_MAX_AGE_SECS as f64 * 1000.0;

// ---------------------------------------------------------------------------
// Query seam
// ---------------------------------------------------------------------------

/// Injectable AE query seam, wired through [`AppState`]. An enum (not
/// `Box<dyn>`) so the state stays `Clone` with no `async_trait` dependency.
#[derive(Clone)]
pub enum AnalyticsBackend {
    /// `ANALYTICS_QUERY_STUB` truthy: compiled-in fixture, zero IO.
    Stub,
    /// Live AE SQL API over HTTPS.
    Live {
        account_id: String,
        api_token: String,
    },
    /// Stub off and credentials missing: every query errors (Express 500s).
    Unconfigured,
}

fn truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes"
    )
}

impl AnalyticsBackend {
    pub fn from_env(env: &Env) -> Self {
        if env
            .var(STUB_ENV_VAR)
            .is_ok_and(|value| truthy(&value.to_string()))
        {
            return Self::Stub;
        }
        let account_id = env
            .var("CLOUDFLARE_ACCOUNT_ID")
            .map(|value| value.to_string())
            .unwrap_or_default();
        let api_token = env
            .secret("CLOUDFLARE_API_TOKEN")
            .map(|secret| secret.to_string())
            .unwrap_or_default();
        if account_id.is_empty() || api_token.is_empty() {
            Self::Unconfigured
        } else {
            Self::Live {
                account_id,
                api_token,
            }
        }
    }

    /// Run one AE SQL statement; `Err(())` maps to the Express 500 envelope.
    pub async fn query(&self, sql: String) -> Result<AnalyticsQuery, ()> {
        match self {
            Self::Stub => serde_json::from_str(ANALYTICS_QUERY_STUB).map_err(|_| ()),
            Self::Unconfigured => Err(()),
            Self::Live {
                account_id,
                api_token,
            } => run_sql_api(account_id, api_token, &sql).await,
        }
    }
}

/// POST the SQL string to the Analytics Engine SQL API and parse the
/// `{meta, data: [...]}` response.
async fn run_sql_api(account_id: &str, api_token: &str, sql: &str) -> Result<AnalyticsQuery, ()> {
    let url =
        format!("https://api.cloudflare.com/client/v4/accounts/{account_id}/analytics_engine/sql");
    let headers = Headers::new();
    headers
        .set("Authorization", &format!("Bearer {api_token}"))
        .map_err(|_| ())?;
    headers.set("Content-Type", "text/plain").map_err(|_| ())?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(sql)));
    let request = worker::Request::new_with_init(&url, &init).map_err(|_| ())?;
    let mut response = Fetch::Request(request).send().await.map_err(|_| ())?;
    if response.status_code() != 200 {
        return Err(());
    }
    response.json::<AnalyticsQuery>().await.map_err(|_| ())
}

// ---------------------------------------------------------------------------
// AE write path
// ---------------------------------------------------------------------------

/// Convert a core [`AnalyticsDataPoint`] into a `TRACKING` binding write.
/// Synchronous and fire-and-forget, like the Express insert's `.catch()`.
fn write_data_point(env: &Env, point: &AnalyticsDataPoint) -> Result<(), ()> {
    let dataset = env.analytics_engine(TRACKING_BINDING).map_err(|_| ())?;
    let blobs: Vec<&str> = point.blobs.iter().map(String::as_str).collect();
    AnalyticsEngineDataPointBuilder::new()
        .blobs(blobs)
        .doubles(point.doubles.iter().copied())
        .indexes([point.index1.as_str()])
        .write_to(&dataset)
        .map_err(|_| ())
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

    let point = page_view_data_point(&PageViewFields {
        visitor_id: &visitor_id,
        // No session concept yet: core falls back to the visitor id.
        session_id: "",
        path: request.uri().path(),
        method: request.method().as_str(),
        referer: headers.get(REFERER).and_then(|value| value.to_str().ok()),
        user_agent: headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown"),
        query_params_json: &query_params_json(raw_query),
        timestamp_ms: Date::now(),
        is_new_visitor,
        // Express leaves time_spent NULL on the middleware insert; AE takes 0.
        time_spent_seconds: 0.0,
        user_id: "",
    });
    if write_data_point(&state.env, &point).is_err() {
        worker::console_warn!("visitor tracking write failed; continuing");
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
    match state.analytics.query(sql).await {
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
        visitors_overview_sql(DEFAULT_DATASET, days),
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
        visitors_daily_sql(DEFAULT_DATASET, days),
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
        popular_pages_sql(DEFAULT_DATASET, limit),
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
        conversion_sql(DEFAULT_DATASET, days),
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
    let session = match state
        .analytics
        .query(visitor_session_sql(DEFAULT_DATASET, &visitor_id))
        .await
    {
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
    let page_views = match state
        .analytics
        .query(visitor_page_views_sql(DEFAULT_DATASET, &visitor_id))
        .await
    {
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
        events_engagement_sql(DEFAULT_DATASET, days),
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
    // New optional field the Express schema never had; core falls back to the
    // visitor id when it is absent.
    let session_id = body
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let point = custom_event_data_point(&CustomEventFields {
        visitor_id: &visitor_id,
        session_id,
        event_type: &event_type,
        event_data_json: &event_data_json,
        timestamp_ms: Date::now(),
    });
    // Express 500s when the insert throws; here only a missing binding or a
    // build error can fail — the write itself is fire-and-forget.
    if write_data_point(&state.env, &point).is_err() {
        return server_error("Failed to track event");
    }
    Json(track_success_json()).into_response()
}
