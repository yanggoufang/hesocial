//! Visitor tracking domain ported to Cloudflare Analytics Engine (Phase 2g).
//!
//! Everything here is pure: the AE write mapping, the AE SQL builders, and the
//! Express-exact response shapers are all host-testable. The worker crate
//! (Stage 2) converts [`AnalyticsDataPoint`] into
//! `worker::AnalyticsEngineDataPointBuilder` and POSTs the SQL strings to
//! `https://api.cloudflare.com/client/v4/accounts/{ACCOUNT_ID}/analytics_engine/sql`.
//!
//! ## AE write mapping (dataset `hesocial_visitors`)
//!
//! One data point is written per tracked hit (page view from the
//! `visitorTracking` middleware path, or a custom event from
//! `POST /api/analytics/events/track`).
//!
//! | AE slot   | logical field         | source                                             |
//! |-----------|-----------------------|----------------------------------------------------|
//! | blob1     | kind                  | `"page_view"` or `"custom_event"`                  |
//! | blob2     | visitor_id            | `X-Visitor-ID` header / `visitorId` cookie / query |
//! | blob3     | session_id            | beacon body, falls back to visitor_id              |
//! | blob4     | path                  | `req.path`                                         |
//! | blob5     | method                | `req.method`                                       |
//! | blob6     | referer               | `Referer` header (`""` when absent)                |
//! | blob7     | user_agent            | `User-Agent` header                                |
//! | blob8     | event_type            | custom events only; `""` for page views            |
//! | blob9     | payload_json          | `JSON.stringify(req.query)` for page views, `JSON.stringify(event_data)` for custom events |
//! | blob10    | user_id               | linked user after `linkVisitorToUser`; `""` while anonymous |
//! | double1   | timestamp_ms          | event time, epoch milliseconds                     |
//! | double2   | is_new_visitor        | 1/0                                                |
//! | double3   | time_spent_seconds    | beacon-reported dwell; 0 when absent               |
//! | double4   | is_registered         | 1 when user_id linked, else 0                      |
//! | index1    | session_id            | AE sampling key (single indexed string)            |
//!
//! Budget: 10 of 25 blobs, 4 of 20 doubles — headroom is intentional.
//! `index1 = session_id` (not visitor_id): AE samples uniformly over index1,
//! and session granularity keeps long-lived visitor cookies from skewing the
//! sample. Express has no session concept (visitor_sessions is per-visitor),
//! so the session id is a new field the Stage 2 beacon accepts, defaulting to
//! the visitor id when the client does not send one.
//!
//! ## Documented AE deviations (vs the Express/DuckDB originals)
//!
//! - **Sampling**: AE is sampled; counts are scaled with `SUM(_sample_interval)`
//!   / `* _sample_interval` instead of `COUNT(*)`. Results are approximate.
//! - **`COUNT(DISTINCT ...)`**: used for unique-visitor counts against blob2.
//!   ClickHouse-dialect support must be verified against the live AE SQL API in
//!   Stage 2; the fallback is per-day exactness only.
//! - **avg_pages_per_visitor**: Express averages the per-session `page_views`
//!   counter; AE has no session rows, so this is total views / unique visitors.
//! - **Time buckets**: `toStartOfDay(timestamp)` buckets in UTC; DuckDB
//!   `DATE()` used the server timezone. Buckets can shift near midnight.
//! - **Visitor detail**: AE cannot faithfully replay per-visitor rows under
//!   sampling; the `session` block is rebuilt from aggregates and `page_views`
//!   is an approximate sample.
//! - **Negative `days`**: Express passed `parseInt` output straight into the
//!   date math (`days=-5` queried the future). AE `INTERVAL` rejects negatives,
//!   so the window clamps to >= 1 day.
//! - **D1-backed endpoints stay on D1**: events/overview, events/performance,
//!   events/:id/performance, revenue/events and engagement/members read
//!   events/registrations/users and get no AE SQL builders here. Their D1 SQL
//!   and envelope shapers live in the "D1-backed analytics (Stage 3)" section
//!   at the bottom of this module.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::pagination::js_parse_f64;

/// AE dataset the builders address. Final name is a Stage 3 wrangler decision.
/// The track endpoint's body validation: `visitor_id` and `event_type` are
/// required (Express 400 `visitor_id and event_type are required`).
/// Returns `(visitor_id, event_type, event_data_json)`.
pub fn validate_track_body(body: &Value) -> Result<(String, String, String), &'static str> {
    let missing = || "visitor_id and event_type are required";
    let visitor_id = body
        .get("visitor_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(missing)?;
    let event_type = body
        .get("event_type")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(missing)?;
    let event_data_json = match body.get("event_data") {
        Some(value) => value.to_string(),
        None => "{}".to_owned(),
    };
    Ok((
        visitor_id.to_owned(),
        event_type.to_owned(),
        event_data_json,
    ))
}

/// Express success body for the track endpoint — note there is NO `data` key,
/// unlike every other analytics envelope.
pub fn track_success_json() -> Value {
    json!({"success": true, "message": "Event tracked successfully"})
}

// ---------------------------------------------------------------------------
// SQL builders (AE SQL dialect — ClickHouse-flavored subset)
// ---------------------------------------------------------------------------

/// JS `parseInt(raw) || fallback`: leading-integer parse, `0` and garbage fall
/// back. Negative results clamp to 1 (see module docs — Express would have
/// queried a future window, AE rejects negative INTERVALs).
pub fn period_days(raw: Option<&str>, fallback: u32) -> u32 {
    let parsed = raw
        .map(str::trim)
        .and_then(|text| {
            let (sign, digits) = match text.strip_prefix('-') {
                Some(rest) => (-1i64, rest),
                None => (1i64, text.strip_prefix('+').unwrap_or(text)),
            };
            let digits: String = digits
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect();
            digits.parse::<i64>().ok().map(|value| sign * value)
        })
        .filter(|value| *value != 0)
        .unwrap_or(i64::from(fallback));
    parsed.clamp(1, i64::from(u32::MAX)) as u32
}

/// JS `parseInt(raw) || fallback` for LIMIT parameters.
pub fn limit_param(raw: Option<&str>, fallback: u32) -> u32 {
    period_days(raw, fallback)
}

/// Quote a string literal for AE SQL (single-quote doubling).
/// Rolling-window cutoff. Emitted in the same `strftime` shape the writer
/// stores, so the comparison is exact string ordering rather than a mix of
/// `YYYY-MM-DDTHH:MM:SS.SSSZ` against `datetime()`'s space-separated form.
fn window_cutoff(days: u32) -> String {
    format!("strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-{days} days')")
}

/// Stored ISO timestamp -> epoch milliseconds. The envelopes emit `*_ms`
/// numbers, a shape the Analytics Engine port established and the frontend
/// already consumes, so the conversion happens here rather than in the shaper.
fn epoch_ms(column: &str) -> String {
    format!("CAST((julianday({column}) - 2440587.5) * 86400000.0 AS INTEGER)")
}

/// GET /api/analytics/visitors — `getVisitorAnalytics` in
/// `backend/src/middleware/visitorTracking.ts`. `total_page_views` counts
/// session rows, not page views; that is Express's behaviour, kept verbatim.
pub fn visitors_overview_sql(days: u32) -> String {
    format!(
        "SELECT \
           COUNT(DISTINCT visitor_id) AS unique_visitors, \
           COUNT(*) AS total_page_views, \
           COUNT(DISTINCT CASE WHEN user_id IS NOT NULL THEN visitor_id END) AS converted_visitors, \
           AVG(page_views) AS avg_pages_per_visitor, \
           COUNT(CASE WHEN first_seen >= {cutoff} THEN 1 END) AS new_visitors \
         FROM visitor_sessions \
         WHERE last_seen >= {cutoff}",
        cutoff = window_cutoff(days),
    )
}

/// GET /api/analytics/visitors/daily — the `visitor_analytics_daily` view from
/// migration 005, with the route's window filter and LIMIT 100 folded in.
pub fn visitors_daily_sql(days: u32) -> String {
    format!(
        "SELECT \
           DATE(last_seen) AS date, \
           COUNT(DISTINCT visitor_id) AS unique_visitors, \
           SUM(page_views) AS total_page_views, \
           COUNT(DISTINCT CASE WHEN user_id IS NOT NULL THEN visitor_id END) AS converted_visitors, \
           AVG(page_views) AS avg_pages_per_visitor \
         FROM visitor_sessions \
         WHERE last_seen >= {cutoff} \
         GROUP BY DATE(last_seen) \
         ORDER BY date DESC \
         LIMIT 100",
        cutoff = window_cutoff(days),
    )
}

/// GET /api/analytics/pages/popular — the `popular_pages` view, LIMIT from the
/// query param.
pub fn popular_pages_sql(limit: u32) -> String {
    format!(
        "SELECT \
           vpv.path AS path, \
           COUNT(*) AS views, \
           COUNT(DISTINCT vpv.visitor_id) AS unique_visitors, \
           AVG(CASE WHEN vs.user_id IS NOT NULL THEN 1.0 ELSE 0.0 END) AS conversion_rate \
         FROM visitor_page_views vpv \
         LEFT JOIN visitor_sessions vs ON vpv.visitor_id = vs.visitor_id \
         GROUP BY vpv.path \
         ORDER BY views DESC \
         LIMIT {limit}"
    )
}

/// GET /api/analytics/conversion — funnel counts; `conversion_rate` is rounded
/// in the shaper (Express did `ROUND(..., 2)` in SQL — same value, fewer
/// dialect assumptions).
pub fn conversion_sql(days: u32) -> String {
    format!(
        "SELECT \
           COUNT(DISTINCT vs.visitor_id) AS total_visitors, \
           COUNT(DISTINCT CASE WHEN vpv.path LIKE '/events/%' THEN vs.visitor_id END) AS event_viewers, \
           COUNT(DISTINCT CASE WHEN vs.user_id IS NOT NULL THEN vs.visitor_id END) AS registered_users \
         FROM visitor_sessions vs \
         LEFT JOIN visitor_page_views vpv ON vs.visitor_id = vpv.visitor_id \
         WHERE vs.last_seen >= {cutoff}",
        cutoff = window_cutoff(days),
    )
}

/// GET /api/analytics/visitors/:visitorId — the session block. The visitor id
/// is bound, not interpolated: it arrives straight off the URL path.
pub fn visitor_session_sql() -> String {
    format!(
        "SELECT \
           visitor_id, \
           {first_seen} AS first_seen_ms, \
           {last_seen} AS last_seen_ms, \
           page_views, \
           referer, \
           user_agent \
         FROM visitor_sessions \
         WHERE visitor_id = ?",
        first_seen = epoch_ms("first_seen"),
        last_seen = epoch_ms("last_seen"),
    )
}

/// GET /api/analytics/visitors/:visitorId — the page_views block, newest
/// first, LIMIT 100 like Express.
pub fn visitor_page_views_sql() -> String {
    format!(
        "SELECT \
           path, \
           method, \
           query_params, \
           referer, \
           {timestamp} AS timestamp_ms, \
           time_spent \
         FROM visitor_page_views \
         WHERE visitor_id = ? \
         ORDER BY timestamp DESC \
         LIMIT 100",
        timestamp = epoch_ms("timestamp"),
    )
}

/// GET /api/analytics/events/engagement — per-day engagement over `/events%`
/// paths.
pub fn events_engagement_sql(days: u32) -> String {
    format!(
        "SELECT \
           DATE(vpv.timestamp) AS date, \
           COUNT(DISTINCT vpv.visitor_id) AS unique_visitors, \
           COUNT(vpv.id) AS total_page_views, \
           COUNT(CASE WHEN vpv.path LIKE '/events/%' THEN 1 END) AS event_page_views, \
           COUNT(CASE WHEN vpv.path LIKE '/events/%/register' THEN 1 END) AS registration_page_views, \
           AVG(vpv.time_spent) AS avg_time_spent \
         FROM visitor_page_views vpv \
         WHERE vpv.timestamp >= {cutoff} \
           AND vpv.path LIKE '/events%' \
         GROUP BY DATE(vpv.timestamp) \
         ORDER BY date DESC",
        cutoff = window_cutoff(days),
    )
}

// ---------------------------------------------------------------------------
// Write path (replaces the Analytics Engine data points)
// ---------------------------------------------------------------------------

/// One page view. Paired with [`UPSERT_VISITOR_SESSION_SQL`] in a batch so the
/// session counter and the row land together or not at all.
pub const INSERT_PAGE_VIEW_SQL: &str = "INSERT INTO visitor_page_views (visitor_id, path, method, query_params, referer, timestamp, time_spent, ip_address, user_agent) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";

/// Session upsert. `user_id` arrives as '' while the visitor is anonymous;
/// `NULLIF` keeps the column NULL so the converted-visitor aggregates stay
/// honest, and `converted_at` is stamped the first time an id appears.
pub const UPSERT_VISITOR_SESSION_SQL: &str = "INSERT INTO visitor_sessions (visitor_id, user_id, ip_address, user_agent, referer, first_seen, last_seen, page_views, session_count, converted_at, created_at, updated_at) VALUES (?, NULLIF(?, ''), ?, ?, ?, ?, ?, 1, 1, NULL, ?, ?) ON CONFLICT(visitor_id) DO UPDATE SET last_seen = excluded.last_seen, page_views = visitor_sessions.page_views + 1, user_id = COALESCE(excluded.user_id, visitor_sessions.user_id), converted_at = CASE WHEN visitor_sessions.converted_at IS NULL AND excluded.user_id IS NOT NULL THEN excluded.last_seen ELSE visitor_sessions.converted_at END, referer = COALESCE(visitor_sessions.referer, excluded.referer), updated_at = excluded.updated_at";

/// POST /api/analytics/events/track.
pub const INSERT_VISITOR_EVENT_SQL: &str = "INSERT INTO visitor_events (visitor_id, event_type, event_data, timestamp) VALUES (?, ?, ?, ?)";

// ---------------------------------------------------------------------------
// Response shapers (AE SQL API rows -> Express envelopes)
// ---------------------------------------------------------------------------

/// Row format of the AE SQL API response that Stage 3's contract tests stub:
/// `POST .../analytics_engine/sql` answers `{"data": [ {alias: value, ...} ]}`
/// where each row object's keys are the SELECT aliases above and values are
/// JSON numbers or strings (AE returns some aggregates as strings).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AnalyticsQuery {
    #[serde(default)]
    pub data: Vec<Value>,
}

/// Coerce a result cell to f64 (numbers pass through, numeric strings parse
/// the JS way — SQLite hands back some aggregates as text).
fn numeric(value: &Value) -> f64 {
    match value {
        Value::Number(number) => number.as_f64().unwrap_or(0.0),
        Value::String(text) => js_parse_f64(text).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn numeric_field(row: &Value, key: &str) -> f64 {
    row.get(key).map(numeric).unwrap_or(0.0)
}

fn first_row(rows: &[Value]) -> Value {
    rows.first().cloned().unwrap_or(Value::Object(Map::new()))
}

/// DuckDB returns aggregates as BigInt which Express JSON-serializes via
/// `Number(...)`; integral f64s serialize identically (`3` not `3.0`).
fn integral_json(value: f64) -> Value {
    if value.fract() == 0.0 && value.abs() <= 9_007_199_254_740_992.0 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

/// JS `Math.round(x * 100) / 100` — matches DuckDB `ROUND(x, 2)` for the
/// non-negative rates these endpoints produce.
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// `{success: true, data: {period_days, unique_visitors, total_page_views,
/// converted_visitors, avg_pages_per_visitor, new_visitors}}`.
pub fn visitors_overview_envelope(days: u32, query: &AnalyticsQuery) -> Value {
    let row = first_row(&query.data);
    json!({
        "success": true,
        "data": {
            "period_days": days,
            "unique_visitors": integral_json(numeric_field(&row, "unique_visitors")),
            "total_page_views": integral_json(numeric_field(&row, "total_page_views")),
            "converted_visitors": integral_json(numeric_field(&row, "converted_visitors")),
            "avg_pages_per_visitor": numeric_field(&row, "avg_pages_per_visitor"),
            "new_visitors": integral_json(numeric_field(&row, "new_visitors")),
        }
    })
}

/// `{success: true, data: [ {date, unique_visitors, total_page_views,
/// converted_visitors, avg_pages_per_visitor}, ... ]}` — rows pass through in
/// query order (Express returned the view rows as-is).
pub fn visitors_daily_envelope(query: &AnalyticsQuery) -> Value {
    let rows: Vec<Value> = query
        .data
        .iter()
        .map(|row| {
            json!({
                "date": row.get("date").cloned().unwrap_or(Value::Null),
                "unique_visitors": integral_json(numeric_field(row, "unique_visitors")),
                "total_page_views": integral_json(numeric_field(row, "total_page_views")),
                "converted_visitors": integral_json(numeric_field(row, "converted_visitors")),
                "avg_pages_per_visitor": numeric_field(row, "avg_pages_per_visitor"),
            })
        })
        .collect();
    json!({"success": true, "data": rows})
}

/// `{success: true, data: [ {path, views, unique_visitors, conversion_rate} ]}`.
pub fn popular_pages_envelope(query: &AnalyticsQuery) -> Value {
    let rows: Vec<Value> = query
        .data
        .iter()
        .map(|row| {
            json!({
                "path": row.get("path").cloned().unwrap_or(Value::Null),
                "views": integral_json(numeric_field(row, "views")),
                "unique_visitors": integral_json(numeric_field(row, "unique_visitors")),
                "conversion_rate": numeric_field(row, "conversion_rate"),
            })
        })
        .collect();
    json!({"success": true, "data": rows})
}

/// `{success: true, data: {period_days, total_visitors, event_viewers,
/// registered_users, conversion_rate}}` — rate computed and rounded here.
pub fn conversion_envelope(days: u32, query: &AnalyticsQuery) -> Value {
    let row = first_row(&query.data);
    let total = numeric_field(&row, "total_visitors");
    let registered = numeric_field(&row, "registered_users");
    let rate = if total > 0.0 {
        round2(registered * 100.0 / total)
    } else {
        0.0
    };
    json!({
        "success": true,
        "data": {
            "period_days": days,
            "total_visitors": integral_json(total),
            "event_viewers": integral_json(numeric_field(&row, "event_viewers")),
            "registered_users": integral_json(registered),
            "conversion_rate": rate,
        }
    })
}

/// `{success: true, data: {period_days, engagement: [...]}}`.
pub fn events_engagement_envelope(days: u32, query: &AnalyticsQuery) -> Value {
    let rows: Vec<Value> = query
        .data
        .iter()
        .map(|row| {
            json!({
                "date": row.get("date").cloned().unwrap_or(Value::Null),
                "unique_visitors": integral_json(numeric_field(row, "unique_visitors")),
                "total_page_views": integral_json(numeric_field(row, "total_page_views")),
                "event_page_views": integral_json(numeric_field(row, "event_page_views")),
                "registration_page_views": integral_json(numeric_field(row, "registration_page_views")),
                "avg_time_spent": numeric_field(row, "avg_time_spent"),
            })
        })
        .collect();
    json!({"success": true, "data": {"period_days": days, "engagement": rows}})
}

/// `{success: true, data: {session, page_views}}`. The session block is the
/// aggregate row; `page_views` carries the sampled row set. A missing session
/// row serializes as `null` session with empty page views — Express answered
/// 404 there, which is the worker layer's call (Stage 2), not the shaper's.
pub fn visitor_detail_envelope(session: &AnalyticsQuery, page_views: &AnalyticsQuery) -> Value {
    let session_row = first_row(&session.data);
    let session_json = if session.data.is_empty() {
        Value::Null
    } else {
        json!({
            "visitor_id": session_row.get("visitor_id").cloned().unwrap_or(Value::Null),
            "first_seen_ms": numeric_field(&session_row, "first_seen_ms"),
            "last_seen_ms": numeric_field(&session_row, "last_seen_ms"),
            "page_views": integral_json(numeric_field(&session_row, "page_views")),
            "referer": session_row.get("referer").cloned().unwrap_or(Value::Null),
            "user_agent": session_row.get("user_agent").cloned().unwrap_or(Value::Null),
        })
    };
    let rows: Vec<Value> = page_views
        .data
        .iter()
        .map(|row| {
            json!({
                "path": row.get("path").cloned().unwrap_or(Value::Null),
                "method": row.get("method").cloned().unwrap_or(Value::Null),
                "query_params": row.get("query_params").cloned().unwrap_or(Value::Null),
                "referer": row.get("referer").cloned().unwrap_or(Value::Null),
                "timestamp_ms": numeric_field(row, "timestamp_ms"),
                "time_spent": numeric_field(row, "time_spent"),
            })
        })
        .collect();
    json!({"success": true, "data": {"session": session_json, "page_views": rows}})
}

// ---------------------------------------------------------------------------
// D1-backed analytics (Phase 2g Stage 3)
// ---------------------------------------------------------------------------
//
// The remaining five Express analytics endpoints read `events`,
// `registrations` and `users` — tables that ARE in D1 (locked decision #3
// only exiled the visitor tables to AE). The SQL below is SQLite-flavored and
// schema-checked against `backend-rust/sql/schema.sql`; the envelope shapers
// pass D1 rows through untouched, exactly like Express passed DuckDB rows
// through `convertBigIntToNumber` (an identity for non-BigInt values — D1
// already returns JS numbers).
//
// ## Column mapping (old DuckDB shape -> D1 event-management shape)
//
// | Express/DuckDB        | D1                       |
// |-----------------------|--------------------------|
// | events.date_time      | events.start_datetime    |
// | events.name           | events.title             |
// | events.capacity       | events.capacity_max      |
// | events.current_attendees | events.current_registrations |
// | events.pricing_vip    | events.price_platinum    |
// | events.pricing_vvip   | events.price_diamond     |
// | registrations.tier    | registrations.registration_type |
// | categories (table)    | event_categories (table) |
//
// ## Documented deviations (vs Express)
//
// - **Timestamp comparisons**: DuckDB `CURRENT_TIMESTAMP` compared against a
//   TIMESTAMP column; D1 stores ISO-8601 TEXT (`...T...Z`), so the port
//   compares against `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` — same instant,
//   matching text format.
// - **`pricing_vip`/`pricing_vvip`/`registrations.tier` do not exist in the
//   old DuckDB schema either** (grep `database/duckdb-schema.sql`): against
//   DuckDB, GET /events/performance was a dead 500 path. The port maps them
//   onto the D1 tier prices (`vip -> price_platinum`, `vvip ->
//   price_diamond`, `r.tier -> r.registration_type`) instead of reproducing a
//   500. The `'vvip'` arm is dead in D1 (`registration_type` CHECK is
//   member/guest/vip); it is kept so the query semantics stay Express-shaped.
// - **events/:id/performance**: Express's `categories`/`venues` join failed
//   against DuckDB (no `categories` table) but is nearly schema-correct in
//   D1; the only change is `categories -> event_categories`. The INNER JOINs
//   are kept verbatim (D1 `category_id`/`venue_id` are NOT NULL FKs, so they
//   never filter a real event out).
// - **`'2025-07'` hardcodes (revenue/events, engagement/members): REPRODUCED
//   VERBATIM.** The literal is a cosmetic label — Express aggregates ALL
//   rows regardless of period, so the constant never filters anything and is
//   not meaningless in D1, just as half-dead as it was in Express. Dropping
//   it would change the response keys' values, breaking parity.

/// events/overview #1: event aggregates. `recent_events` and
/// `upcoming_events` are the same expression in Express (dead duplicate) —
/// reproduced verbatim.
pub const EVENTS_OVERVIEW_EVENT_STATS_SQL: &str = "SELECT \
     COUNT(*) AS total_events, \
     COUNT(CASE WHEN start_datetime >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now') THEN 1 END) AS recent_events, \
     COUNT(CASE WHEN start_datetime >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now') THEN 1 END) AS upcoming_events, \
     COUNT(CASE WHEN start_datetime < strftime('%Y-%m-%dT%H:%M:%fZ', 'now') THEN 1 END) AS past_events, \
     AVG(CASE WHEN capacity_max > 0 THEN (current_registrations * 100.0 / capacity_max) END) AS avg_occupancy_rate \
   FROM events";

/// events/overview #2: registration aggregates. `recent_registrations` is
/// another Express dead duplicate — reproduced verbatim.
pub const EVENTS_OVERVIEW_REGISTRATION_STATS_SQL: &str = "SELECT \
     COUNT(*) AS total_registrations, \
     COUNT(*) AS recent_registrations, \
     COUNT(DISTINCT user_id) AS unique_attendees \
   FROM registrations \
   WHERE status = 'confirmed'";

/// events/overview #3: top-10 upcoming events by attendance. SELECT aliases
/// keep the Express/DuckDB output key names (`name`, `date_time`, `capacity`,
/// `current_attendees`).
pub const EVENTS_OVERVIEW_POPULAR_EVENTS_SQL: &str = "SELECT \
     e.id AS id, \
     e.title AS name, \
     e.start_datetime AS date_time, \
     e.capacity_max AS capacity, \
     e.current_registrations AS current_attendees, \
     ROUND((e.current_registrations * 100.0 / e.capacity_max), 2) AS occupancy_rate \
   FROM events e \
   WHERE e.start_datetime >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
   ORDER BY e.current_registrations DESC \
   LIMIT 10";

/// events/performance: per-event metrics over the trailing `days` window.
/// See the module section docs for the vip/vvip column mapping.
pub fn events_performance_sql(days: u32) -> String {
    format!(
        "SELECT \
           e.id AS id, \
           e.title AS name, \
           e.start_datetime AS date_time, \
           e.capacity_max AS capacity, \
           e.current_registrations AS current_attendees, \
           e.price_platinum AS pricing_vip, \
           e.price_diamond AS pricing_vvip, \
           ROUND((e.current_registrations * 100.0 / e.capacity_max), 2) AS occupancy_rate, \
           COUNT(r.id) AS total_registrations, \
           COUNT(CASE WHEN r.status = 'confirmed' THEN 1 END) AS confirmed_registrations, \
           COUNT(CASE WHEN r.status = 'pending' THEN 1 END) AS pending_registrations, \
           COUNT(CASE WHEN r.status = 'cancelled' THEN 1 END) AS cancelled_registrations, \
           AVG(CASE WHEN r.registration_type = 'vip' THEN e.price_platinum WHEN r.registration_type = 'vvip' THEN e.price_diamond END) AS avg_revenue_per_attendee \
         FROM events e \
         LEFT JOIN registrations r ON e.id = r.event_id \
         WHERE e.start_datetime >= strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-{days} days') \
         GROUP BY e.id, e.title, e.start_datetime, e.capacity_max, e.current_registrations, e.price_platinum, e.price_diamond \
         ORDER BY occupancy_rate DESC"
    )
}

/// events/:id/performance #1: the event row (`e.*`) plus category/venue names
/// and the fill/revenue derivations. Binds the event id once.
pub const EVENT_PERFORMANCE_DETAIL_SQL: &str = "SELECT \
     e.*, \
     c.name AS category_name, \
     v.name AS venue_name, \
     (CAST(e.current_registrations AS REAL) / e.capacity_max * 100) AS fill_rate, \
     (e.price_platinum * e.current_registrations) AS current_revenue, \
     (e.price_platinum * e.capacity_max) AS potential_revenue \
   FROM events e \
   JOIN event_categories c ON e.category_id = c.id \
   JOIN venues v ON e.venue_id = v.id \
   WHERE e.id = ?";

/// events/:id/performance #2: per-day registration counts with a running
/// total. Binds the event id once.
pub const REGISTRATION_TIMELINE_SQL: &str = "SELECT \
     date(created_at) AS date, \
     COUNT(*) AS registrations, \
     SUM(COUNT(*)) OVER (ORDER BY date(created_at)) AS cumulative_registrations \
   FROM registrations \
   WHERE event_id = ? AND status != 'cancelled' \
   GROUP BY date(created_at) \
   ORDER BY date";

/// events/:id/performance #3: membership-tier mix of non-cancelled
/// registrations. Binds the event id TWICE (subquery first, outer second) —
/// same order as Express's `[id, id]`.
pub const MEMBERSHIP_BREAKDOWN_SQL: &str = "SELECT \
     u.membership_tier AS membership_tier, \
     COUNT(*) AS count, \
     (CAST(COUNT(*) AS REAL) / (SELECT COUNT(*) FROM registrations WHERE event_id = ? AND status != 'cancelled') * 100) AS percentage \
   FROM registrations r \
   JOIN users u ON r.user_id = u.id \
   WHERE r.event_id = ? AND r.status != 'cancelled' \
   GROUP BY u.membership_tier \
   ORDER BY count DESC";

/// events/:id/performance #4: registration counts per status. Binds once.
pub const STATUS_BREAKDOWN_SQL: &str =
    "SELECT status, COUNT(*) AS count FROM registrations WHERE event_id = ? GROUP BY status";

/// revenue/events #1: monthly revenue. The `'2025-07'` literal is an Express
/// hardcode reproduced verbatim (see section docs); the query aggregates ALL
/// events with attendees.
pub const MONTHLY_REVENUE_SQL: &str = "SELECT \
     '2025-07' AS month, \
     COUNT(e.id) AS event_count, \
     SUM(e.current_registrations) AS total_registrations, \
     SUM(e.current_registrations * 15000) AS revenue \
   FROM events e \
   WHERE e.current_registrations > 0 \
   GROUP BY 1 \
   ORDER BY month DESC";

/// revenue/events #2: revenue by category.
pub const CATEGORY_REVENUE_SQL: &str = "SELECT \
     ec.name AS category, \
     SUM(e.current_registrations * 15000) AS revenue, \
     COUNT(e.id) AS event_count, \
     AVG(e.current_registrations * 15000) AS avg_revenue_per_event \
   FROM events e \
   JOIN event_categories ec ON e.category_id = ec.id \
   WHERE e.current_registrations > 0 \
   GROUP BY ec.id, ec.name \
   ORDER BY revenue DESC";

/// revenue/events #3: revenue by membership tier. `SUM(15000)` and the
/// NOT NULL filter are Express quirks, reproduced verbatim.
pub const TIER_REVENUE_SQL: &str = "SELECT \
     u.membership_tier AS membership_tier, \
     COUNT(*) AS registration_count, \
     SUM(15000) AS total_revenue \
   FROM users u \
   WHERE u.membership_tier IS NOT NULL \
   GROUP BY u.membership_tier \
   ORDER BY total_revenue DESC";

/// engagement/members #1: per-tier engagement rates.
pub const MEMBER_ENGAGEMENT_SQL: &str = "SELECT \
     u.membership_tier AS membership_tier, \
     COUNT(DISTINCT u.id) AS total_members, \
     COUNT(DISTINCT r.user_id) AS active_members, \
     COALESCE((CAST(COUNT(DISTINCT r.user_id) AS REAL) / COUNT(DISTINCT u.id) * 100), 0) AS engagement_rate, \
     AVG(COALESCE(user_stats.event_count, 0)) AS avg_events_per_member \
   FROM users u \
   LEFT JOIN registrations r ON u.id = r.user_id AND r.status != 'cancelled' \
   LEFT JOIN ( \
     SELECT user_id, COUNT(*) AS event_count \
     FROM registrations \
     WHERE status != 'cancelled' \
     GROUP BY user_id \
   ) user_stats ON u.id = user_stats.user_id \
   GROUP BY u.membership_tier \
   ORDER BY engagement_rate DESC";

/// engagement/members #2: top-20 most active members. `SUM(15000)` counts one
/// row even for members with zero registrations (Express quirk — their
/// `total_spent` reads 15000); reproduced verbatim.
pub const TOP_MEMBERS_SQL: &str = "SELECT \
     u.first_name AS first_name, \
     u.last_name AS last_name, \
     u.membership_tier AS membership_tier, \
     COUNT(r.id) AS events_attended, \
     SUM(15000) AS total_spent \
   FROM users u \
   LEFT JOIN registrations r ON u.id = r.user_id \
   WHERE r.status = 'confirmed' OR r.status IS NULL \
   GROUP BY u.id, u.first_name, u.last_name, u.membership_tier \
   ORDER BY events_attended DESC, total_spent DESC \
   LIMIT 20";

/// engagement/members #3: retention cohorts. `'2025-07'` and `100.0` are
/// Express hardcodes reproduced verbatim (see section docs).
pub const RETENTION_SQL: &str = "SELECT \
     '2025-07' AS cohort_month, \
     COUNT(DISTINCT u.id) AS cohort_size, \
     COUNT(DISTINCT u.id) AS active_this_month, \
     100.0 AS retention_rate \
   FROM users u \
   LEFT JOIN registrations r ON u.id = r.user_id AND r.status != 'cancelled' \
   GROUP BY 1 \
   ORDER BY cohort_month DESC";

/// `{success: true, data: {period_days, event_stats, registration_stats,
/// popular_events}}`. Aggregate queries always return exactly one row; a
/// missing row serializes as `null` (Express would have crashed on
/// `rows[0]`'s `undefined` only if the query itself failed — a 500 path).
pub fn events_overview_envelope(
    days: u32,
    event_stats: &[Value],
    registration_stats: &[Value],
    popular_events: &[Value],
) -> Value {
    json!({
        "success": true,
        "data": {
            "period_days": days,
            "event_stats": first_row(event_stats),
            "registration_stats": first_row(registration_stats),
            "popular_events": popular_events,
        }
    })
}

/// `{success: true, data: {period_days, events: [...]}}`.
pub fn events_performance_envelope(days: u32, events: &[Value]) -> Value {
    json!({"success": true, "data": {"period_days": days, "events": events}})
}

/// `{success: true, data: {event, registrationTimeline, membershipBreakdown,
/// statusBreakdown}}` — camelCase keys, exactly like Express.
pub fn event_performance_detail_envelope(
    event: &Value,
    registration_timeline: &[Value],
    membership_breakdown: &[Value],
    status_breakdown: &[Value],
) -> Value {
    json!({
        "success": true,
        "data": {
            "event": event,
            "registrationTimeline": registration_timeline,
            "membershipBreakdown": membership_breakdown,
            "statusBreakdown": status_breakdown,
        }
    })
}

/// `{success: true, data: {monthlyRevenue, categoryRevenue, tierRevenue}}`.
pub fn revenue_events_envelope(
    monthly_revenue: &[Value],
    category_revenue: &[Value],
    tier_revenue: &[Value],
) -> Value {
    json!({
        "success": true,
        "data": {
            "monthlyRevenue": monthly_revenue,
            "categoryRevenue": category_revenue,
            "tierRevenue": tier_revenue,
        }
    })
}

/// `{success: true, data: {engagement, topMembers, retention}}`.
pub fn members_engagement_envelope(
    engagement: &[Value],
    top_members: &[Value],
    retention: &[Value],
) -> Value {
    json!({
        "success": true,
        "data": {
            "engagement": engagement,
            "topMembers": top_members,
            "retention": retention,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn track_body_validation_mirrors_the_express_400() {
        assert!(validate_track_body(&json!({})).is_err());
        assert!(validate_track_body(&json!({"visitor_id": "v1"})).is_err());
        assert!(validate_track_body(&json!({"event_type": "click"})).is_err());
        assert!(validate_track_body(&json!({"visitor_id": "", "event_type": "click"})).is_err());

        let (visitor, kind, data) =
            validate_track_body(&json!({"visitor_id": "v1", "event_type": "click"}))
                .expect("valid body");
        assert_eq!(
            (visitor.as_str(), kind.as_str(), data.as_str()),
            ("v1", "click", "{}")
        );

        let (_, _, data) = validate_track_body(
            &json!({"visitor_id": "v1", "event_type": "click", "event_data": {"a": 1}}),
        )
        .expect("valid body");
        assert_eq!(data, r#"{"a":1}"#);
    }

    #[test]
    fn track_success_has_no_data_key() {
        assert_eq!(
            serde_json::to_string(&track_success_json()).expect("track json"),
            r#"{"success":true,"message":"Event tracked successfully"}"#
        );
    }

    #[test]
    fn period_days_follows_parse_int_or_default() {
        assert_eq!(period_days(None, 30), 30);
        assert_eq!(period_days(Some(""), 30), 30);
        assert_eq!(period_days(Some("7"), 30), 7);
        assert_eq!(period_days(Some(" 12 "), 30), 12);
        assert_eq!(period_days(Some("7days"), 30), 7); // parseInt prefix parse
        assert_eq!(period_days(Some("abc"), 30), 30);
        assert_eq!(period_days(Some("0"), 30), 30); // 0 is falsy in JS
        assert_eq!(period_days(Some("-5"), 30), 1); // clamped; Express queried the future
        assert_eq!(limit_param(Some("20"), 20), 20);
    }

    #[test]
    fn overview_sql_targets_the_window_over_sessions() {
        let sql = visitors_overview_sql(30);
        assert!(sql.contains("FROM visitor_sessions"));
        assert!(sql.contains("'now', '-30 days'"));
        assert!(sql.contains("COUNT(DISTINCT visitor_id) AS unique_visitors"));
        assert!(sql.contains("COUNT(CASE WHEN first_seen >="));
        assert!(sql.contains("AS new_visitors"));
    }

    #[test]
    fn daily_sql_buckets_by_day_newest_first() {
        let sql = visitors_daily_sql(7);
        assert!(sql.contains("DATE(last_seen) AS date"));
        assert!(sql.contains("GROUP BY DATE(last_seen)"));
        assert!(sql.contains("ORDER BY date DESC"));
        assert!(sql.contains("LIMIT 100"));
        assert!(sql.contains("'-7 days'"));
    }

    #[test]
    fn popular_pages_sql_groups_by_path_with_limit() {
        let sql = popular_pages_sql(20);
        assert!(sql.contains("vpv.path AS path"));
        assert!(sql.contains("LEFT JOIN visitor_sessions"));
        assert!(sql.contains("ORDER BY views DESC"));
        assert!(sql.contains("LIMIT 20"));
    }

    #[test]
    fn engagement_sql_scopes_to_event_paths() {
        let sql = events_engagement_sql(30);
        assert!(sql.contains("vpv.path LIKE '/events%'"));
        assert!(sql.contains("vpv.path LIKE '/events/%/register'"));
        assert!(sql.contains("avg_time_spent"));
        assert!(sql.contains("registration_page_views"));
    }

    /// The visitor id arrives off the URL path, so it is bound rather than
    /// interpolated — there is no quoting for an injection to escape.
    #[test]
    fn visitor_sql_binds_the_identifier() {
        let sql = visitor_session_sql();
        assert!(sql.contains("WHERE visitor_id = ?"));
        assert!(sql.contains("AS first_seen_ms"));
        assert!(sql.contains("AS last_seen_ms"));
        let views = visitor_page_views_sql();
        assert!(views.contains("WHERE visitor_id = ?"));
        assert!(views.contains("ORDER BY timestamp DESC"));
        assert!(views.contains("LIMIT 100"));
    }

    /// The three write statements replace the Analytics Engine data points.
    #[test]
    fn write_statements_target_the_visitor_tables() {
        assert!(UPSERT_VISITOR_SESSION_SQL.contains("INSERT INTO visitor_sessions"));
        assert!(UPSERT_VISITOR_SESSION_SQL.contains("ON CONFLICT(visitor_id) DO UPDATE"));
        assert!(
            UPSERT_VISITOR_SESSION_SQL.contains("page_views = visitor_sessions.page_views + 1")
        );
        assert!(UPSERT_VISITOR_SESSION_SQL.contains("NULLIF(?, '')"));
        assert!(INSERT_PAGE_VIEW_SQL.contains("INSERT INTO visitor_page_views"));
        assert!(INSERT_VISITOR_EVENT_SQL.contains("INSERT INTO visitor_events"));
    }

    #[test]
    fn overview_envelope_coerces_and_prefixes_period_days() {
        let query = AnalyticsQuery {
            data: vec![json!({
                "unique_visitors": "5",
                "total_page_views": 20,
                "converted_visitors": 2,
                "avg_pages_per_visitor": 4.0,
                "new_visitors": 3,
            })],
        };
        let envelope = visitors_overview_envelope(7, &query);
        assert_eq!(
            serde_json::to_string(&envelope).expect("overview json"),
            r#"{"success":true,"data":{"period_days":7,"unique_visitors":5,"total_page_views":20,"converted_visitors":2,"avg_pages_per_visitor":4.0,"new_visitors":3}}"#
        );
    }

    #[test]
    fn empty_rows_shape_zero_aggregates_like_express() {
        let query = AnalyticsQuery::default();
        let overview = visitors_overview_envelope(30, &query);
        assert_eq!(overview["data"]["unique_visitors"], json!(0));
        assert_eq!(overview["data"]["total_page_views"], json!(0));
        assert_eq!(overview["data"]["period_days"], json!(30));

        let conversion = conversion_envelope(30, &query);
        assert_eq!(conversion["data"]["conversion_rate"], json!(0.0));
        assert_eq!(conversion["data"]["total_visitors"], json!(0));
    }

    #[test]
    fn conversion_rate_rounds_like_round_two() {
        let query = AnalyticsQuery {
            data: vec![json!({
                "total_visitors": 3,
                "event_viewers": 2,
                "registered_users": 1,
            })],
        };
        let envelope = conversion_envelope(30, &query);
        assert_eq!(envelope["data"]["conversion_rate"], json!(33.33));
        assert_eq!(envelope["data"]["event_viewers"], json!(2));
    }

    #[test]
    fn popular_pages_and_engagement_keep_row_order_and_names() {
        let pages = AnalyticsQuery {
            data: vec![json!({
                "path": "/events",
                "views": "9",
                "unique_visitors": 4,
                "conversion_rate": 0.25,
            })],
        };
        let envelope = popular_pages_envelope(&pages);
        assert_eq!(envelope["data"][0]["path"], json!("/events"));
        assert_eq!(envelope["data"][0]["views"], json!(9));

        let engagement = AnalyticsQuery {
            data: vec![json!({
                "date": "2026-08-31T00:00:00Z",
                "unique_visitors": 2,
                "total_page_views": 6,
                "event_page_views": 5,
                "registration_page_views": 1,
                "avg_time_spent": 42.5,
            })],
        };
        let envelope = events_engagement_envelope(30, &engagement);
        assert_eq!(envelope["data"]["period_days"], json!(30));
        assert_eq!(
            envelope["data"]["engagement"][0]["avg_time_spent"],
            json!(42.5)
        );
        assert_eq!(
            envelope["data"]["engagement"][0]["registration_page_views"],
            json!(1)
        );
    }

    #[test]
    fn visitor_detail_splits_session_from_page_views() {
        let session = AnalyticsQuery {
            data: vec![json!({
                "visitor_id": "visitor_abc",
                "first_seen_ms": 1.0,
                "last_seen_ms": 2.0,
                "page_views": "4",
                "referer": "https://google.com",
                "user_agent": "Mozilla/5.0",
            })],
        };
        let views = AnalyticsQuery {
            data: vec![json!({
                "path": "/events/42",
                "method": "GET",
                "query_params": "{}",
                "referer": "",
                "timestamp_ms": 2.0,
                "time_spent": 3.0,
            })],
        };
        let envelope = visitor_detail_envelope(&session, &views);
        assert_eq!(
            envelope["data"]["session"]["visitor_id"],
            json!("visitor_abc")
        );
        assert_eq!(envelope["data"]["session"]["page_views"], json!(4));
        assert_eq!(
            envelope["data"]["page_views"][0]["path"],
            json!("/events/42")
        );

        let missing =
            visitor_detail_envelope(&AnalyticsQuery::default(), &AnalyticsQuery::default());
        assert_eq!(missing["data"]["session"], Value::Null);
        assert_eq!(missing["data"]["page_views"], json!([]));
    }

    // -----------------------------------------------------------------------
    // Stage 3: D1-backed endpoints
    // -----------------------------------------------------------------------

    #[test]
    fn overview_sql_uses_d1_columns_and_keeps_the_dead_duplicates() {
        assert!(EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("FROM events"));
        assert!(EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("start_datetime"));
        assert!(EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("capacity_max"));
        assert!(EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("current_registrations"));
        assert!(!EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("date_time"));
        assert!(!EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("current_attendees"));
        // Express dead duplicates, reproduced verbatim.
        assert!(EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("AS recent_events"));
        assert!(EVENTS_OVERVIEW_EVENT_STATS_SQL.contains("AS upcoming_events"));
        assert!(EVENTS_OVERVIEW_REGISTRATION_STATS_SQL.contains("AS recent_registrations"));
        assert!(EVENTS_OVERVIEW_REGISTRATION_STATS_SQL.contains("status = 'confirmed'"));

        let popular = EVENTS_OVERVIEW_POPULAR_EVENTS_SQL;
        assert!(popular.contains("e.title AS name"));
        assert!(popular.contains("e.start_datetime AS date_time"));
        assert!(popular.contains("e.capacity_max AS capacity"));
        assert!(popular.contains("e.current_registrations AS current_attendees"));
        assert!(popular.contains("LIMIT 10"));
    }

    #[test]
    fn performance_sql_interpolates_the_window_and_maps_the_tier_columns() {
        let sql = events_performance_sql(30);
        assert!(sql.contains("'now', '-30 days'"));
        assert!(sql.contains("e.price_platinum AS pricing_vip"));
        assert!(sql.contains("e.price_diamond AS pricing_vvip"));
        assert!(sql.contains("r.registration_type = 'vip'"));
        assert!(sql.contains("LEFT JOIN registrations r ON e.id = r.event_id"));
        assert!(sql.contains("ORDER BY occupancy_rate DESC"));

        let other = events_performance_sql(7);
        assert!(other.contains("'-7 days'"));
    }

    #[test]
    fn detail_sql_joins_event_categories_and_binds_like_express() {
        assert!(EVENT_PERFORMANCE_DETAIL_SQL.contains("JOIN event_categories c"));
        assert!(EVENT_PERFORMANCE_DETAIL_SQL.contains("JOIN venues v"));
        assert!(EVENT_PERFORMANCE_DETAIL_SQL.contains("WHERE e.id = ?"));
        assert!(!EVENT_PERFORMANCE_DETAIL_SQL.contains("JOIN categories"));
        assert!(EVENT_PERFORMANCE_DETAIL_SQL.contains("AS fill_rate"));
        assert!(EVENT_PERFORMANCE_DETAIL_SQL.contains("AS potential_revenue"));

        // Two binds, subquery first — Express's [id, id] order.
        assert_eq!(MEMBERSHIP_BREAKDOWN_SQL.matches('?').count(), 2);
        assert!(MEMBERSHIP_BREAKDOWN_SQL.contains("status != 'cancelled'"));
        assert_eq!(REGISTRATION_TIMELINE_SQL.matches('?').count(), 1);
        assert!(REGISTRATION_TIMELINE_SQL.contains("OVER (ORDER BY date(created_at))"));
        assert_eq!(STATUS_BREAKDOWN_SQL.matches('?').count(), 1);
    }

    #[test]
    fn revenue_and_members_sql_reproduce_the_2025_07_hardcode_verbatim() {
        assert!(MONTHLY_REVENUE_SQL.contains("'2025-07' AS month"));
        assert!(MONTHLY_REVENUE_SQL.contains("SUM(e.current_registrations * 15000) AS revenue"));
        assert!(RETENTION_SQL.contains("'2025-07' AS cohort_month"));
        assert!(RETENTION_SQL.contains("100.0 AS retention_rate"));
        assert!(CATEGORY_REVENUE_SQL.contains("JOIN event_categories ec"));
        assert!(TIER_REVENUE_SQL.contains("SUM(15000) AS total_revenue"));
        assert!(MEMBER_ENGAGEMENT_SQL.contains("AS engagement_rate"));
        assert!(MEMBER_ENGAGEMENT_SQL.contains("COUNT(DISTINCT r.user_id)"));
        assert!(TOP_MEMBERS_SQL.contains("LIMIT 20"));
        assert!(TOP_MEMBERS_SQL.contains("SUM(15000) AS total_spent"));
        assert!(TOP_MEMBERS_SQL.contains("r.status = 'confirmed' OR r.status IS NULL"));
    }

    #[test]
    fn overview_envelope_shapes_the_three_query_results() {
        let event_stats = vec![json!({"total_events": 4, "upcoming_events": 1})];
        let registration_stats = vec![json!({"total_registrations": 9})];
        let popular = vec![json!({"id": 1, "name": "Gala", "occupancy_rate": 50.0})];
        let envelope = events_overview_envelope(30, &event_stats, &registration_stats, &popular);
        assert_eq!(envelope["data"]["period_days"], json!(30));
        assert_eq!(envelope["data"]["event_stats"]["total_events"], json!(4));
        assert_eq!(
            envelope["data"]["registration_stats"]["total_registrations"],
            json!(9)
        );
        assert_eq!(envelope["data"]["popular_events"][0]["name"], json!("Gala"));
    }

    #[test]
    fn detail_envelope_uses_the_express_camel_case_keys() {
        let envelope = event_performance_detail_envelope(
            &json!({"id": 7, "fill_rate": 25.0}),
            &[json!({"date": "2026-08-01", "registrations": 2})],
            &[json!({"membership_tier": "Diamond", "count": 2})],
            &[json!({"status": "confirmed", "count": 2})],
        );
        assert_eq!(envelope["data"]["event"]["id"], json!(7));
        assert_eq!(
            envelope["data"]["registrationTimeline"][0]["registrations"],
            json!(2)
        );
        assert_eq!(
            envelope["data"]["membershipBreakdown"][0]["membership_tier"],
            json!("Diamond")
        );
        assert_eq!(
            envelope["data"]["statusBreakdown"][0]["status"],
            json!("confirmed")
        );
    }

    #[test]
    fn revenue_and_members_envelopes_keep_row_order_and_key_names() {
        let revenue = revenue_events_envelope(
            &[json!({"month": "2025-07", "revenue": 30000})],
            &[json!({"category": "Gala", "revenue": 15000})],
            &[json!({"membership_tier": "Platinum", "total_revenue": 15000})],
        );
        assert_eq!(
            revenue["data"]["monthlyRevenue"][0]["month"],
            json!("2025-07")
        );
        assert_eq!(
            revenue["data"]["categoryRevenue"][0]["category"],
            json!("Gala")
        );
        assert_eq!(
            revenue["data"]["tierRevenue"][0]["total_revenue"],
            json!(15000)
        );

        let members = members_engagement_envelope(
            &[json!({"membership_tier": "Diamond", "engagement_rate": 66.67})],
            &[json!({"first_name": "A", "events_attended": 3})],
            &[json!({"cohort_month": "2025-07", "retention_rate": 100.0})],
        );
        assert_eq!(
            members["data"]["engagement"][0]["engagement_rate"],
            json!(66.67)
        );
        assert_eq!(
            members["data"]["topMembers"][0]["events_attended"],
            json!(3)
        );
        assert_eq!(members["data"]["retention"][0]["cohort_size"], Value::Null);
    }
}
