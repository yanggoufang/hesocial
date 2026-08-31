//! D1-backed analytics read endpoints (Phase 2g Stage 3).
//!
//! The remaining five Express analytics endpoints read `events`,
//! `registrations` and `users` — tables that live in D1 (visitor data never
//! touches D1; it went to Analytics Engine in Stages 1-2). The pure SQL and
//! envelope shapers live in `core::analytics` (see its "D1-backed analytics
//! (Stage 3)" section for the column mapping and every deviation); this
//! module is only the wasm/host boundary: admin guard, D1 bind/execute, and
//! the Express error envelopes.
//!
//! All five GETs keep the Express admin guard (`authenticate` +
//! `require_admin`), same split as Stage 2: only the beacon
//! `POST /events/track` is unauthenticated.
//!
//! ## Documented deviations (vs Express) — full list in core::analytics
//!
//! - Old-DuckDB-only columns (`pricing_vip`, `pricing_vvip`,
//!   `registrations.tier`, `date_time`, `current_attendees`, `capacity`) map
//!   onto the D1 event-management schema instead of reproducing dead 500
//!   paths (`events/performance` 500s against DuckDB too — those columns
//!   never existed there either).
//! - `categories` -> `event_categories` in `events/:id/performance`.
//! - The `'2025-07'`/`100.0` hardcodes in `revenue/events` and
//!   `engagement/members` are REPRODUCED VERBATIM: they are cosmetic labels
//!   on all-time aggregates, exactly as half-dead in Express as they are
//!   here. Dropping them would break response parity.
//! - Timestamp predicates compare the ISO-8601 TEXT columns against
//!   `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` instead of DuckDB's
//!   `CURRENT_TIMESTAMP` — same instant, matching storage format.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::analytics::{
    CATEGORY_REVENUE_SQL, EVENT_PERFORMANCE_DETAIL_SQL, EVENTS_OVERVIEW_EVENT_STATS_SQL,
    EVENTS_OVERVIEW_POPULAR_EVENTS_SQL, EVENTS_OVERVIEW_REGISTRATION_STATS_SQL,
    MEMBER_ENGAGEMENT_SQL, MEMBERSHIP_BREAKDOWN_SQL, MONTHLY_REVENUE_SQL,
    REGISTRATION_TIMELINE_SQL, RETENTION_SQL, STATUS_BREAKDOWN_SQL, TIER_REVENUE_SQL,
    TOP_MEMBERS_SQL, event_performance_detail_envelope, events_overview_envelope,
    events_performance_envelope, events_performance_sql, members_engagement_envelope, period_days,
    revenue_events_envelope,
};
use serde_json::{Value, json};
use worker::D1Database;
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;

use crate::AppState;
use crate::analytics_handlers::{require_analytics_admin, server_error};

const OVERVIEW_ERROR: &str = "Failed to retrieve event analytics overview";
const PERFORMANCE_ERROR: &str = "Failed to retrieve event performance analytics";
const DETAIL_ERROR: &str = "Failed to fetch event performance data";
const REVENUE_ERROR: &str = "Failed to fetch revenue analytics";
const MEMBERS_ERROR: &str = "Failed to fetch member engagement data";

/// SQLite compares an INTEGER PRIMARY KEY against a numeric bind; a raw path
/// string would never match. Express passes the string through to DuckDB,
/// which casts — reproduce by parsing when possible (same helper as
/// `event_handlers`, kept private there).
fn id_bind(id: &str) -> JsValue {
    match id.parse::<f64>().ok().filter(|value| value.is_finite()) {
        Some(number) => JsValue::from_f64(number),
        None => JsValue::from_str(id),
    }
}

async fn all_values(db: &D1Database, sql: &str, binds: &[JsValue]) -> Result<Vec<Value>, ()> {
    let statement = db.prepare(sql).bind(binds).map_err(|_| ())?;
    let result = statement.all().await.map_err(|_| ())?;
    result.results::<Value>().map_err(|_| ())
}

fn database(state: &AppState, error: &'static str) -> Result<D1Database, Response> {
    state.env.d1("DB").map_err(|_| server_error(error))
}

pub async fn events_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(events_overview_inner(state, headers, params)).await
}

async fn events_overview_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let days = period_days(params.get("days").map(String::as_str), 30);
    let db = match database(&state, OVERVIEW_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    // Express runs the three queries sequentially; any failure 500s.
    let event_stats = match all_values(&db, EVENTS_OVERVIEW_EVENT_STATS_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(OVERVIEW_ERROR),
    };
    let registration_stats =
        match all_values(&db, EVENTS_OVERVIEW_REGISTRATION_STATS_SQL, &[]).await {
            Ok(rows) => rows,
            Err(()) => return server_error(OVERVIEW_ERROR),
        };
    let popular_events = match all_values(&db, EVENTS_OVERVIEW_POPULAR_EVENTS_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(OVERVIEW_ERROR),
    };
    Json(events_overview_envelope(
        days,
        &event_stats,
        &registration_stats,
        &popular_events,
    ))
    .into_response()
}

pub async fn events_performance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(events_performance_inner(state, headers, params)).await
}

async fn events_performance_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let days = period_days(params.get("days").map(String::as_str), 30);
    let db = match database(&state, PERFORMANCE_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let events = match all_values(&db, &events_performance_sql(days), &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(PERFORMANCE_ERROR),
    };
    Json(events_performance_envelope(days, &events)).into_response()
}

pub async fn event_performance_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    SendFuture::new(event_performance_detail_inner(state, headers, id)).await
}

async fn event_performance_detail_inner(
    state: AppState,
    headers: HeaderMap,
    id: String,
) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let db = match database(&state, DETAIL_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    // Express queries the event first and 404s before touching the breakdowns.
    let events = match all_values(&db, EVENT_PERFORMANCE_DETAIL_SQL, &[id_bind(&id)]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(DETAIL_ERROR),
    };
    let Some(event) = events.into_iter().next() else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"success": false, "error": "Event not found"})),
        )
            .into_response();
    };
    let timeline = match all_values(&db, REGISTRATION_TIMELINE_SQL, &[id_bind(&id)]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(DETAIL_ERROR),
    };
    // Two binds, subquery first — same order as Express's `[id, id]`.
    let membership =
        match all_values(&db, MEMBERSHIP_BREAKDOWN_SQL, &[id_bind(&id), id_bind(&id)]).await {
            Ok(rows) => rows,
            Err(()) => return server_error(DETAIL_ERROR),
        };
    let status = match all_values(&db, STATUS_BREAKDOWN_SQL, &[id_bind(&id)]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(DETAIL_ERROR),
    };
    Json(event_performance_detail_envelope(
        &event,
        &timeline,
        &membership,
        &status,
    ))
    .into_response()
}

pub async fn revenue_events(State(state): State<AppState>, headers: HeaderMap) -> Response {
    SendFuture::new(revenue_events_inner(state, headers)).await
}

async fn revenue_events_inner(state: AppState, headers: HeaderMap) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let db = match database(&state, REVENUE_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let monthly = match all_values(&db, MONTHLY_REVENUE_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(REVENUE_ERROR),
    };
    let category = match all_values(&db, CATEGORY_REVENUE_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(REVENUE_ERROR),
    };
    let tier = match all_values(&db, TIER_REVENUE_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(REVENUE_ERROR),
    };
    Json(revenue_events_envelope(&monthly, &category, &tier)).into_response()
}

pub async fn members_engagement(State(state): State<AppState>, headers: HeaderMap) -> Response {
    SendFuture::new(members_engagement_inner(state, headers)).await
}

async fn members_engagement_inner(state: AppState, headers: HeaderMap) -> Response {
    if let Err(response) = require_analytics_admin(&state, &headers).await {
        return response;
    }
    let db = match database(&state, MEMBERS_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let engagement = match all_values(&db, MEMBER_ENGAGEMENT_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(MEMBERS_ERROR),
    };
    let top_members = match all_values(&db, TOP_MEMBERS_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(MEMBERS_ERROR),
    };
    let retention = match all_values(&db, RETENTION_SQL, &[]).await {
        Ok(rows) => rows,
        Err(()) => return server_error(MEMBERS_ERROR),
    };
    Json(members_engagement_envelope(
        &engagement,
        &top_members,
        &retention,
    ))
    .into_response()
}
