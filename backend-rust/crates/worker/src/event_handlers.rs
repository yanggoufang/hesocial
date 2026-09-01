use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::event_management::{
    EventDetailRow, RegistrationStatsRow, event_detail_json, is_bool_column, is_json_column,
    slugify_title, update_column,
};
use hesocial_core::pagination::number_json;
use hesocial_core::{ApiEnvelope, auth::UserRow};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use worker::js_sys::Date;
use worker::send::SendFuture;

use crate::AppState;
use crate::auth::{authenticate, bearer_token, internal_error, require_admin, require_super_admin};
use crate::auth_handlers::now_iso;
use crate::db::{self, Val};

const EVENT_DETAIL_SQL: &str = "SELECT e.id AS id, e.title AS name, e.description AS description, e.start_datetime AS \"dateTime\", e.registration_closes_at AS \"registrationDeadline\", e.price_platinum AS price_platinum, e.price_diamond AS price_diamond, e.price_black_card AS price_black_card, e.currency AS pricing_currency, e.dress_code AS \"dressCode\", e.capacity_max AS capacity, e.current_registrations AS \"currentAttendees\", e.gallery_images AS images, e.special_requirements AS requirements, e.created_at AS \"createdAt\", e.updated_at AS \"updatedAt\", v.id AS \"venueId\", v.name AS \"venueName\", v.address AS \"venueAddress\", v.city AS \"venueCity\", v.latitude AS latitude, v.longitude AS longitude, v.rating AS \"venueRating\", v.amenities AS \"venueAmenities\", v.images AS \"venueImages\", ec.id AS \"categoryId\", ec.name AS \"categoryName\", ec.description AS \"categoryDescription\", ec.icon AS \"categoryIcon\", (u.first_name || ' ' || u.last_name) AS \"organizerName\", e.slug AS slug, e.title AS title, e.detailed_description AS detailed_description, e.category_id AS category_id, e.venue_id AS venue_id, e.organizer_id AS organizer_id, e.start_datetime AS start_datetime, e.end_datetime AS end_datetime, e.timezone AS timezone, e.capacity_min AS capacity_min, e.capacity_max AS capacity_max, e.current_registrations AS current_registrations, e.currency AS currency, e.status AS status, e.approval_status AS approval_status, e.approved_by AS approved_by, e.approved_at AS approved_at, e.required_membership_tiers AS required_membership_tiers, e.required_verification AS required_verification, e.age_restriction AS age_restriction, e.dress_code AS dress_code, e.language AS language, e.special_requirements AS special_requirements, e.inclusions AS inclusions, e.exclusions AS exclusions, e.registration_opens_at AS registration_opens_at, e.registration_closes_at AS registration_closes_at, e.cancellation_deadline AS cancellation_deadline, e.waitlist_enabled AS waitlist_enabled, e.auto_approval AS auto_approval, e.meta_title AS meta_title, e.meta_description AS meta_description, e.featured_image AS featured_image, e.gallery_images AS gallery_images, e.internal_notes AS internal_notes, e.cost_breakdown AS cost_breakdown, e.profit_margin AS profit_margin, e.published_at AS published_at FROM events e JOIN venues v ON e.venue_id = v.id JOIN event_categories ec ON e.category_id = ec.id JOIN users u ON e.organizer_id = u.id WHERE e.id = ?";

const REGISTRATION_STATS_SQL: &str = "SELECT COUNT(*) AS total_registrations, COUNT(CASE WHEN status = 'confirmed' THEN 1 END) AS confirmed_registrations, COUNT(CASE WHEN status = 'waitlisted' THEN 1 END) AS waitlisted_registrations, COUNT(CASE WHEN status = 'pending' THEN 1 END) AS pending_registrations FROM registrations WHERE event_id = ?";

const WAITLIST_COUNT_SQL: &str = "SELECT COUNT(*) AS waitlist_count FROM event_waitlist WHERE event_id = ? AND status = 'waiting'";

const EVENT_EXISTS_SQL: &str = "SELECT id, status FROM events WHERE id = ?";

// Express reads `event_registrations`; the D1 schema names the table
// `registrations` (locked decision, see ROADMAP Phase 0.5 header).
const REGISTRATION_COUNT_SQL: &str =
    "SELECT COUNT(*) AS count FROM registrations WHERE event_id = ?";

const DELETE_EVENT_SQL: &str = "DELETE FROM events WHERE id = ?";

const PUBLISH_EVENT_SQL: &str = "UPDATE events SET status = 'published', published_at = ?, updated_at = ? WHERE id = ? AND approval_status = 'approved'";

const APPROVE_EVENT_SQL: &str = "UPDATE events SET approval_status = ?, approved_by = ?, approved_at = ?, updated_at = ? WHERE id = ?";

const CREATE_EVENT_SQL: &str = "INSERT INTO events (id, title, slug, description, detailed_description, category_id, venue_id, organizer_id, start_datetime, end_datetime, timezone, capacity_min, capacity_max, price_platinum, price_diamond, price_black_card, currency, required_membership_tiers, required_verification, age_restriction, dress_code, language, special_requirements, inclusions, exclusions, registration_opens_at, registration_closes_at, cancellation_deadline, waitlist_enabled, auto_approval, meta_title, meta_description, featured_image, internal_notes, cost_breakdown, profit_margin, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

#[derive(Deserialize)]
struct EventExistsRow {
    #[allow(dead_code)]
    id: i64,
    #[allow(dead_code)]
    status: Option<String>,
}

#[derive(Deserialize)]
struct CountRow {
    count: i64,
}

#[derive(Deserialize)]
struct WaitlistCountRow {
    waitlist_count: i64,
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(ApiEnvelope::<Value>::error(error))).into_response()
}

fn not_found() -> Response {
    json_error(StatusCode::NOT_FOUND, "Event not found")
}

fn message_response(message: &str) -> Response {
    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert("message".to_owned(), json!(message));
    Json(Value::Object(body)).into_response()
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

fn raw_field(body: &Value, key: &str) -> Value {
    body.get(key).map_or(db::NULL, to_js)
}

/// JS `value || fallback` semantics for scalar fields.
fn fallback_field(body: &Value, key: &str, fallback: &str) -> Value {
    match body.get(key).filter(|value| js_truthy(value)) {
        Some(value) => to_js(value),
        None => Val::from_str(fallback),
    }
}

fn fallback_number(body: &Value, key: &str, fallback: f64) -> Value {
    match body.get(key).filter(|value| js_truthy(value)) {
        Some(value) => to_js(value),
        None => Val::from_f64(fallback),
    }
}

/// JS `JSON.stringify(value || fallback)` for the JSON text columns.
fn json_field(body: &Value, key: &str, fallback: &str) -> Value {
    let serialized = match body.get(key).filter(|value| js_truthy(value)) {
        Some(value) => value.to_string(),
        None => fallback.to_owned(),
    };
    Val::from_str(&serialized)
}

/// JS `value !== false`.
fn unless_false(body: &Value, key: &str) -> Value {
    let flag = !matches!(body.get(key), Some(Value::Bool(false)));
    Val::from_f64(if flag { 1.0 } else { 0.0 })
}

/// JS `value === true`.
fn only_true(body: &Value, key: &str) -> Value {
    let flag = matches!(body.get(key), Some(Value::Bool(true)));
    Val::from_f64(if flag { 1.0 } else { 0.0 })
}

/// SQLite compares an INTEGER PRIMARY KEY against a numeric bind; a raw path
/// string would never match. Express passes the string through to DuckDB,
/// which casts — reproduce by parsing when possible.
fn id_bind(id: &str) -> Value {
    match id.parse::<f64>().ok().filter(|value| value.is_finite()) {
        Some(number) => Val::from_f64(number),
        None => Val::from_str(id),
    }
}

async fn run_changes(statement: db::PreparedStatement) -> Result<usize, ()> {
    let result = statement.run().await.map_err(|_| ())?;
    Ok(result.meta().changes)
}

pub async fn get_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    // optionalAuth: a missing or invalid token degrades to anonymous, never 401.
    let user = match bearer_token(&headers) {
        Some(_) => SendFuture::new(authenticate(&state, &headers)).await.ok(),
        None => None,
    };
    SendFuture::new(get_event_inner(state, user, id)).await
}

async fn get_event_inner(state: AppState, user: Option<UserRow>, id: String) -> Response {
    let is_admin = user
        .as_ref()
        .is_some_and(|user| user.role == "admin" || user.role == "super_admin");

    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get event"),
    };
    let bind = id_bind(&id);
    let query = match db
        .prepare(EVENT_DETAIL_SQL)
        .bind(std::slice::from_ref(&bind))
    {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to get event"),
    };
    let event: Option<EventDetailRow> = match query.first(None).await {
        Ok(event) => event,
        Err(_) => return internal_error("Failed to get event"),
    };
    let Some(event) = event else {
        return not_found();
    };

    // The live Express handler filters `is_active = true` only; the unified
    // schema has no such column, so non-admins get the list endpoint's
    // published+approved visibility instead (declared deviation).
    let visible = event.status.as_deref() == Some("published")
        && event.approval_status.as_deref() == Some("approved");
    if !is_admin && !visible {
        return not_found();
    }

    let admin_stats = if is_admin {
        let stats_query = db
            .prepare(REGISTRATION_STATS_SQL)
            .bind(std::slice::from_ref(&bind));
        let stats: Option<RegistrationStatsRow> = match stats_query {
            Ok(query) => match query.first(None).await {
                Ok(stats) => stats,
                Err(_) => return internal_error("Failed to get event"),
            },
            Err(_) => return internal_error("Failed to get event"),
        };
        let waitlist_query = db
            .prepare(WAITLIST_COUNT_SQL)
            .bind(std::slice::from_ref(&bind));
        let waitlist: Option<WaitlistCountRow> = match waitlist_query {
            Ok(query) => match query.first(None).await {
                Ok(row) => row,
                Err(_) => return internal_error("Failed to get event"),
            },
            Err(_) => return internal_error("Failed to get event"),
        };
        stats.map(|stats| (stats, waitlist.map_or(0, |row| row.waitlist_count)))
    } else {
        None
    };

    Json(json!({
        "success": true,
        "data": event_detail_json(&event, admin_stats.as_ref().map(|(stats, count)| (stats, *count))),
    }))
    .into_response()
}

pub async fn create_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = require_admin(&user) {
        return response;
    }
    SendFuture::new(create_event_inner(state, user, body)).await
}

async fn create_event_inner(state: AppState, user: UserRow, body: Value) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to create event"),
    };

    // Express crashes on `title.toLowerCase()` when title is missing, hitting
    // the catch-all 500 — mirrored by bailing out before the INSERT.
    let Some(title) = body.get("title").and_then(Value::as_str) else {
        return internal_error("Failed to create event");
    };

    let epoch_ms = Date::now() as u64;
    let slug = slugify_title(title, epoch_ms);
    let timestamp = now_iso();

    let insert = db.prepare(CREATE_EVENT_SQL).bind(&[
        Val::from_f64(epoch_ms as f64),
        raw_field(&body, "title"),
        Val::from_str(&slug),
        raw_field(&body, "description"),
        raw_field(&body, "detailedDescription"),
        raw_field(&body, "categoryId"),
        raw_field(&body, "venueId"),
        Val::from_str(&user.id),
        raw_field(&body, "startDatetime"),
        raw_field(&body, "endDatetime"),
        fallback_field(&body, "timezone", "Asia/Taipei"),
        fallback_number(&body, "capacityMin", 1.0),
        raw_field(&body, "capacityMax"),
        raw_field(&body, "pricePlatinum"),
        raw_field(&body, "priceDiamond"),
        raw_field(&body, "priceBlackCard"),
        fallback_field(&body, "currency", "TWD"),
        json_field(&body, "requiredMembershipTiers", "[]"),
        unless_false(&body, "requiredVerification"),
        json_field(&body, "ageRestriction", "{}"),
        raw_field(&body, "dressCode"),
        fallback_field(&body, "language", "Traditional Chinese"),
        raw_field(&body, "specialRequirements"),
        json_field(&body, "inclusions", "[]"),
        json_field(&body, "exclusions", "[]"),
        raw_field(&body, "registrationOpensAt"),
        raw_field(&body, "registrationClosesAt"),
        raw_field(&body, "cancellationDeadline"),
        unless_false(&body, "waitlistEnabled"),
        only_true(&body, "autoApproval"),
        fallback_field(&body, "metaTitle", title),
        match body.get("metaDescription").filter(|value| js_truthy(value)) {
            Some(value) => to_js(value),
            None => raw_field(&body, "description"),
        },
        raw_field(&body, "featuredImage"),
        raw_field(&body, "internalNotes"),
        json_field(&body, "costBreakdown", "{}"),
        raw_field(&body, "profitMargin"),
        Val::from_str(&timestamp),
        Val::from_str(&timestamp),
    ]);
    let insert = match insert {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to create event"),
    };
    if insert.run().await.is_err() {
        return internal_error("Failed to create event");
    }

    let mut data = Map::new();
    data.insert("eventId".to_owned(), number_json(epoch_ms as f64));
    data.insert("slug".to_owned(), json!(slug));

    let mut response = Map::new();
    response.insert("success".to_owned(), json!(true));
    response.insert("message".to_owned(), json!("Event created successfully"));
    response.insert("data".to_owned(), Value::Object(data));
    (StatusCode::CREATED, Json(Value::Object(response))).into_response()
}

pub async fn update_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = require_admin(&user) {
        return response;
    }
    SendFuture::new(update_event_inner(state, id, body)).await
}

async fn update_event_inner(state: AppState, id: String, body: Value) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to update event"),
    };

    let bind = id_bind(&id);
    let exists_query = db
        .prepare(EVENT_EXISTS_SQL)
        .bind(std::slice::from_ref(&bind));
    let exists: Option<EventExistsRow> = match exists_query {
        Ok(query) => match query.first(None).await {
            Ok(row) => row,
            Err(_) => return internal_error("Failed to update event"),
        },
        Err(_) => return internal_error("Failed to update event"),
    };
    if exists.is_none() {
        return not_found();
    }

    let Some(updates) = body.as_object() else {
        return json_error(StatusCode::BAD_REQUEST, "No valid fields to update");
    };

    let mut assignments: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();
    for (key, value) in updates {
        let Some(column) = update_column(key) else {
            continue;
        };
        if assignments.iter().any(|set| set == column) {
            continue;
        }
        assignments.push(column.to_owned());
        if is_json_column(column) {
            binds.push(Val::from_str(&value.to_string()));
        } else if is_bool_column(column) {
            binds.push(match value {
                Value::Bool(flag) => Val::from_f64(if *flag { 1.0 } else { 0.0 }),
                other => to_js(other),
            });
        } else {
            binds.push(to_js(value));
        }
    }

    if assignments.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "No valid fields to update");
    }

    let set_clause = assignments
        .iter()
        .map(|column| format!("{column} = ?"))
        .collect::<Vec<_>>()
        .join(", ");
    binds.push(Val::from_str(&now_iso()));
    binds.push(bind);

    let update = db
        .prepare(format!(
            "UPDATE events SET {set_clause}, updated_at = ? WHERE id = ?"
        ))
        .bind(&binds);
    let update = match update {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to update event"),
    };
    if update.run().await.is_err() {
        return internal_error("Failed to update event");
    }

    message_response("Event updated successfully")
}

pub async fn delete_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = require_super_admin(&user) {
        return response;
    }
    SendFuture::new(delete_event_inner(state, id)).await
}

async fn delete_event_inner(state: AppState, id: String) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to delete event"),
    };

    let bind = id_bind(&id);
    let exists_query = db
        .prepare(EVENT_EXISTS_SQL)
        .bind(std::slice::from_ref(&bind));
    let exists: Option<EventExistsRow> = match exists_query {
        Ok(query) => match query.first(None).await {
            Ok(row) => row,
            Err(_) => return internal_error("Failed to delete event"),
        },
        Err(_) => return internal_error("Failed to delete event"),
    };
    if exists.is_none() {
        return not_found();
    }

    let count_query = db
        .prepare(REGISTRATION_COUNT_SQL)
        .bind(std::slice::from_ref(&bind));
    let count: Option<CountRow> = match count_query {
        Ok(query) => match query.first(None).await {
            Ok(row) => row,
            Err(_) => return internal_error("Failed to delete event"),
        },
        Err(_) => return internal_error("Failed to delete event"),
    };
    if count.is_some_and(|row| row.count > 0) {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Cannot delete event with existing registrations. Archive the event instead.",
        );
    }

    let delete = db
        .prepare(DELETE_EVENT_SQL)
        .bind(std::slice::from_ref(&bind));
    let delete = match delete {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to delete event"),
    };
    if delete.run().await.is_err() {
        return internal_error("Failed to delete event");
    }

    message_response("Event deleted successfully")
}

pub async fn publish_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = require_admin(&user) {
        return response;
    }
    SendFuture::new(publish_event_inner(state, id)).await
}

async fn publish_event_inner(state: AppState, id: String) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to publish event"),
    };

    let timestamp = now_iso();
    let publish = db.prepare(PUBLISH_EVENT_SQL).bind(&[
        Val::from_str(&timestamp),
        Val::from_str(&timestamp),
        id_bind(&id),
    ]);
    let publish = match publish {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to publish event"),
    };
    match run_changes(publish).await {
        Ok(0) => json_error(
            StatusCode::BAD_REQUEST,
            "Event not found or not approved for publishing",
        ),
        Ok(_) => message_response("Event published successfully"),
        Err(_) => internal_error("Failed to publish event"),
    }
}

pub async fn approve_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = require_admin(&user) {
        return response;
    }
    SendFuture::new(approve_event_inner(state, user, id, body)).await
}

async fn approve_event_inner(state: AppState, user: UserRow, id: String, body: Value) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to process event approval"),
    };

    let approved = body.get("approved").is_some_and(js_truthy);
    let approval_status = if approved { "approved" } else { "rejected" };
    let timestamp = now_iso();
    let approve = db.prepare(APPROVE_EVENT_SQL).bind(&[
        Val::from_str(approval_status),
        Val::from_str(&user.id),
        Val::from_str(&timestamp),
        Val::from_str(&timestamp),
        id_bind(&id),
    ]);
    let approve = match approve {
        Ok(query) => query,
        Err(_) => return internal_error("Failed to process event approval"),
    };
    match run_changes(approve).await {
        Ok(0) => not_found(),
        Ok(_) => message_response(if approved {
            "Event approved successfully"
        } else {
            "Event rejected successfully"
        }),
        Err(_) => internal_error("Failed to process event approval"),
    }
}
