use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::pagination::{js_parse_f64, pagination_json};
use hesocial_core::registrations::{
    ExistingRegistrationRow, RegistrationEventRow, RegistrationOwnerRow, RegistrationViewRow,
    registration_eligibility_error, registration_json,
};
use hesocial_core::{ApiEnvelope, auth::UserRow};
use serde::Deserialize;
use serde_json::{Value, json};
use worker::D1Database;
use worker::js_sys::Date;
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;

use crate::AppState;
use crate::auth::{authenticate, internal_error};
use crate::auth_handlers::now_iso;

const EVENT_FOR_REGISTRATION_SQL: &str = "SELECT id, registration_closes_at, start_datetime, capacity_max, current_registrations, required_membership_tiers, required_verification, waitlist_enabled FROM events WHERE id = ? AND status = 'published' AND approval_status = 'approved'";

const EXISTING_REGISTRATION_SQL: &str =
    "SELECT id FROM registrations WHERE user_id = ? AND event_id = ?";

const INSERT_CAPACITY_REGISTRATION_SQL: &str = "INSERT INTO registrations (event_id, user_id, status, payment_status, special_requests, created_at, updated_at) SELECT e.id, ?, 'pending', 'pending', ?, ?, ? FROM events e WHERE e.id = ? AND e.current_registrations < e.capacity_max";

const INCREMENT_EVENT_SQL: &str = "UPDATE events SET current_registrations = current_registrations + 1, updated_at = ? WHERE id = ? AND current_registrations < capacity_max AND EXISTS (SELECT 1 FROM registrations r WHERE r.event_id = events.id AND r.user_id = ? AND r.status = 'pending' AND r.created_at = ?)";

const INSERT_WAITLIST_REGISTRATION_SQL: &str = "INSERT INTO registrations (event_id, user_id, status, payment_status, special_requests, created_at, updated_at) SELECT e.id, ?, 'waitlisted', 'pending', ?, ?, ? FROM events e WHERE e.id = ? AND e.current_registrations >= e.capacity_max AND e.waitlist_enabled = 1";

const INSERT_WAITLIST_SQL: &str = "INSERT INTO event_waitlist (event_id, user_id, position, status, created_at, updated_at) SELECT ?, ?, COALESCE(MAX(position), 0) + 1, 'waiting', ?, ? FROM event_waitlist WHERE event_id = ? HAVING EXISTS (SELECT 1 FROM registrations r WHERE r.event_id = ? AND r.user_id = ? AND r.status = 'waitlisted' AND r.created_at = ?)";

const REGISTRATION_VIEW_SELECT: &str = "SELECT r.id, r.user_id, r.event_id, r.status, r.payment_status, r.payment_intent_id, r.special_requests, r.created_at, r.updated_at, e.title AS event_name, e.description AS event_description, e.start_datetime AS event_date_time, e.registration_closes_at AS registration_deadline, e.dress_code, e.capacity_max AS capacity, e.current_registrations AS current_attendees, e.price_platinum, e.price_diamond, e.price_black_card, e.currency AS pricing_currency, v.name AS venue_name, v.address AS venue_address, v.latitude, v.longitude, v.amenities AS venue_amenities, v.images AS venue_images, ec.name AS category_name, ec.description AS category_description FROM registrations r JOIN events e ON r.event_id = e.id JOIN venues v ON e.venue_id = v.id JOIN event_categories ec ON e.category_id = ec.id";

const REGISTRATION_OWNER_SQL: &str = "SELECT r.id, r.event_id, r.user_id, r.status, e.start_datetime AS event_date_time, e.registration_closes_at AS registration_deadline FROM registrations r JOIN events e ON r.event_id = e.id WHERE r.id = ?";

const CANCEL_ACTIVE_SQL: &str = "UPDATE registrations SET status = 'cancelled', cancelled_at = ?, updated_at = ? WHERE id = ? AND status IN ('pending', 'confirmed')";

const DECREMENT_EVENT_SQL: &str = "UPDATE events SET current_registrations = MAX(current_registrations - 1, 0), updated_at = ? WHERE id = ? AND EXISTS (SELECT 1 FROM registrations r WHERE r.id = ? AND r.status = 'cancelled' AND r.cancelled_at = ?)";

const ACCEPT_NEXT_WAITLIST_SQL: &str = "UPDATE event_waitlist SET status = 'accepted', offered_at = ?, updated_at = ? WHERE id = (SELECT ew.id FROM event_waitlist ew JOIN registrations r ON r.event_id = ew.event_id AND r.user_id = ew.user_id WHERE ew.event_id = ? AND ew.status = 'waiting' AND r.status = 'waitlisted' ORDER BY ew.position ASC, ew.created_at ASC, ew.id ASC LIMIT 1) AND EXISTS (SELECT 1 FROM registrations cancelled WHERE cancelled.id = ? AND cancelled.status = 'cancelled' AND cancelled.cancelled_at = ?)";

const PROMOTE_WAITLIST_REGISTRATION_SQL: &str = "UPDATE registrations SET status = 'pending', updated_at = ? WHERE event_id = ? AND status = 'waitlisted' AND user_id = (SELECT user_id FROM event_waitlist WHERE event_id = ? AND status = 'accepted' AND offered_at = ? ORDER BY position DESC, id DESC LIMIT 1)";

const INCREMENT_PROMOTED_EVENT_SQL: &str = "UPDATE events SET current_registrations = current_registrations + 1, updated_at = ? WHERE id = ? AND current_registrations < capacity_max AND EXISTS (SELECT 1 FROM registrations promoted JOIN event_waitlist ew ON ew.event_id = promoted.event_id AND ew.user_id = promoted.user_id WHERE promoted.event_id = events.id AND promoted.status = 'pending' AND promoted.updated_at = ? AND ew.status = 'accepted' AND ew.offered_at = ?)";

const CANCEL_WAITLISTED_SQL: &str = "UPDATE registrations SET status = 'cancelled', cancelled_at = ?, updated_at = ? WHERE id = ? AND status = 'waitlisted'";

const DECLINE_WAITLIST_SQL: &str = "UPDATE event_waitlist SET status = 'declined', removed_at = ?, updated_at = ? WHERE event_id = ? AND user_id = ? AND status = 'waiting'";

/// Cutover blocker #4 (2e review F1): Express seeds an `event_participant_access`
/// row on every successful registration (`updateParticipantAccess`, 'pending',
/// best-effort). Without this the participants gate (`payment_status='paid'`)
/// can never be satisfied for post-cutover registrations. `access_level` is the
/// stringified Express integer, matching the D1 seed convention ('1' pending).
const UPSERT_EPA_SQL: &str = "INSERT INTO event_participant_access (user_id, event_id, registration_id, has_access, payment_status, access_level, created_at, updated_at) VALUES (?, ?, ?, 0, 'pending', '1', ?, ?) ON CONFLICT(user_id, event_id) DO UPDATE SET registration_id = excluded.registration_id, payment_status = excluded.payment_status, updated_at = excluded.updated_at";

const PAYMENT_UPDATE_SQL: &str = "UPDATE registrations SET payment_status = ?, payment_intent_id = ?, updated_at = ? WHERE id = ?";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationBody {
    special_requests: Option<String>,
}

#[derive(Deserialize)]
struct CountRow {
    total: i64,
}

#[derive(Deserialize)]
struct LegacyStatsRow {
    total_registrations: i64,
    confirmed_registrations: i64,
    pending_registrations: i64,
    waitlist_registrations: i64,
}

fn id_bind(id: &str) -> JsValue {
    match id.parse::<f64>().ok().filter(|value| value.is_finite()) {
        Some(number) => JsValue::from_f64(number),
        None => JsValue::from_str(id),
    }
}

async fn duplicate_registration_response(
    db: &D1Database,
    user_id: &str,
    event_id: i64,
) -> Response {
    // A UNIQUE(event_id,user_id) violation in a batch surfaces as a plain
    // error; re-check whether a registration for this pair now exists and
    // answer 400 instead of an opaque 500 for the concurrent-double-register
    // race.
    match existing_registration(db, user_id, event_id).await {
        Ok(Some(_)) => json_error(
            StatusCode::BAD_REQUEST,
            "You are already registered for this event",
        ),
        _ => internal_error("Failed to register for event"),
    }
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(ApiEnvelope::<Value>::error(error))).into_response()
}

fn registration_not_found() -> Response {
    json_error(StatusCode::NOT_FOUND, "Registration not found")
}

fn message_response(message: &str) -> Response {
    Json(json!({ "success": true, "message": message })).into_response()
}

fn special_requests_bind(value: Option<&str>) -> JsValue {
    value
        .filter(|request| !request.is_empty())
        .map_or_else(|| JsValue::NULL, JsValue::from_str)
}

fn is_admin(user: &UserRow) -> bool {
    user.role == "admin" || user.role == "super_admin"
}

fn can_access(user: &UserRow, owner_id: &str) -> bool {
    user.id == owner_id || is_admin(user)
}

fn result_changes(result: &worker::D1Result) -> usize {
    result
        .meta()
        .ok()
        .flatten()
        .and_then(|meta| meta.changes)
        .unwrap_or(0)
}

fn result_last_row_id(result: &worker::D1Result) -> Option<i64> {
    result
        .meta()
        .ok()
        .flatten()
        .and_then(|meta| meta.last_row_id)
}

fn bind_statement(
    db: &D1Database,
    sql: &str,
    values: &[JsValue],
) -> Result<worker::D1PreparedStatement, ()> {
    db.prepare(sql).bind(values).map_err(|_| ())
}

async fn event_for_registration(
    db: &D1Database,
    event_id: &str,
) -> Result<Option<RegistrationEventRow>, ()> {
    let query = bind_statement(db, EVENT_FOR_REGISTRATION_SQL, &[id_bind(event_id)])?;
    query.first(None).await.map_err(|_| ())
}

async fn existing_registration(
    db: &D1Database,
    user_id: &str,
    event_id: i64,
) -> Result<Option<ExistingRegistrationRow>, ()> {
    let query = bind_statement(
        db,
        EXISTING_REGISTRATION_SQL,
        &[
            JsValue::from_str(user_id),
            JsValue::from_f64(event_id as f64),
        ],
    )?;
    query.first(None).await.map_err(|_| ())
}

fn registration_created(registration_id: i64, status: &str, message: &str) -> Response {
    (
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": {
                "registrationId": registration_id,
                "status": status,
                "message": message,
            }
        })),
    )
        .into_response()
}

/// Best-effort participant-access seeding after a successful registration —
/// Express swallows epa failures (`Don't fail the main operation if
/// participant access creation fails`), so the write runs outside the atomic
/// registration batch and its errors are ignored here too.
async fn seed_participant_access(
    db: &D1Database,
    user_id: &str,
    event_id: i64,
    registration_id: i64,
) {
    let timestamp = now_iso();
    let upsert = bind_statement(
        db,
        UPSERT_EPA_SQL,
        &[
            JsValue::from_str(user_id),
            JsValue::from_f64(event_id as f64),
            JsValue::from_f64(registration_id as f64),
            JsValue::from_str(&timestamp),
            JsValue::from_str(&timestamp),
        ],
    );
    if let Ok(statement) = upsert {
        let _ = statement.run().await;
    }
}

/// Access-level string for a payment status, following the Express integer
/// semantics (3 paid / 1 pending / 0 refunded) stored as TEXT in D1.
fn access_level_for(payment_status: &str) -> &'static str {
    match payment_status {
        "paid" => "3",
        "refunded" => "0",
        _ => "1",
    }
}

async fn insert_capacity_registration(
    db: &D1Database,
    event_id: i64,
    user_id: &str,
    special_requests: Option<&str>,
    timestamp: &str,
) -> Result<Option<i64>, ()> {
    let insert = bind_statement(
        db,
        INSERT_CAPACITY_REGISTRATION_SQL,
        &[
            JsValue::from_str(user_id),
            special_requests_bind(special_requests),
            JsValue::from_str(timestamp),
            JsValue::from_str(timestamp),
            JsValue::from_f64(event_id as f64),
        ],
    )?;
    let increment = bind_statement(
        db,
        INCREMENT_EVENT_SQL,
        &[
            JsValue::from_str(timestamp),
            JsValue::from_f64(event_id as f64),
            JsValue::from_str(user_id),
            JsValue::from_str(timestamp),
        ],
    )?;
    let results = db.batch(vec![insert, increment]).await.map_err(|_| ())?;
    let inserted = results
        .first()
        .is_some_and(|result| result_changes(result) == 1);
    let incremented = results
        .get(1)
        .is_some_and(|result| result_changes(result) == 1);
    if inserted && incremented {
        Ok(results.first().and_then(result_last_row_id))
    } else if !inserted && !incremented {
        Ok(None)
    } else {
        Err(())
    }
}

async fn insert_waitlisted_registration(
    db: &D1Database,
    event_id: i64,
    user_id: &str,
    special_requests: Option<&str>,
    timestamp: &str,
) -> Result<Option<i64>, ()> {
    let insert_registration = bind_statement(
        db,
        INSERT_WAITLIST_REGISTRATION_SQL,
        &[
            JsValue::from_str(user_id),
            special_requests_bind(special_requests),
            JsValue::from_str(timestamp),
            JsValue::from_str(timestamp),
            JsValue::from_f64(event_id as f64),
        ],
    )?;
    let insert_waitlist = bind_statement(
        db,
        INSERT_WAITLIST_SQL,
        &[
            JsValue::from_f64(event_id as f64),
            JsValue::from_str(user_id),
            JsValue::from_str(timestamp),
            JsValue::from_str(timestamp),
            JsValue::from_f64(event_id as f64),
            JsValue::from_f64(event_id as f64),
            JsValue::from_str(user_id),
            JsValue::from_str(timestamp),
        ],
    )?;
    let results = db
        .batch(vec![insert_registration, insert_waitlist])
        .await
        .map_err(|_| ())?;
    let registered = results
        .first()
        .is_some_and(|result| result_changes(result) == 1);
    let waitlisted = results
        .get(1)
        .is_some_and(|result| result_changes(result) == 1);
    if registered && waitlisted {
        Ok(results.first().and_then(result_last_row_id))
    } else if !registered && !waitlisted {
        Ok(None)
    } else {
        Err(())
    }
}

pub async fn register_for_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
    Json(body): Json<RegistrationBody>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(register_for_event_inner(state, user, event_id, body)).await
}

async fn register_for_event_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    body: RegistrationBody,
) -> Response {
    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to register for event"),
    };
    let event = match event_for_registration(&db, &event_id).await {
        Ok(Some(event)) => event,
        Ok(None) => {
            return json_error(
                StatusCode::NOT_FOUND,
                "Event not found or not available for registration",
            );
        }
        Err(()) => return internal_error("Failed to register for event"),
    };
    let timestamp = now_iso();
    if let Some(error) = registration_eligibility_error(
        &event,
        &user.membership_tier,
        user.is_verified != 0,
        &user.verification_status,
        &timestamp,
    ) {
        let status = if error.starts_with("This event requires") {
            StatusCode::FORBIDDEN
        } else {
            StatusCode::BAD_REQUEST
        };
        return json_error(status, &error);
    }
    match existing_registration(&db, &user.id, event.id).await {
        Ok(Some(existing)) => {
            let _ = existing.id;
            return json_error(
                StatusCode::BAD_REQUEST,
                "You are already registered for this event",
            );
        }
        Ok(None) => {}
        Err(()) => return internal_error("Failed to register for event"),
    }

    let special_requests = body.special_requests.as_deref();
    if event.current_registrations < event.capacity_max {
        match insert_capacity_registration(&db, event.id, &user.id, special_requests, &timestamp)
            .await
        {
            Ok(Some(id)) => {
                seed_participant_access(&db, &user.id, event.id, id).await;
                return registration_created(
                    id,
                    "pending",
                    "Registration submitted successfully. Pending approval.",
                );
            }
            Ok(None) => {}
            Err(()) => {
                return duplicate_registration_response(&db, &user.id, event.id).await;
            }
        }
    }

    if event.waitlist_enabled == 0 {
        return json_error(StatusCode::BAD_REQUEST, "Event is at full capacity");
    }
    match insert_waitlisted_registration(&db, event.id, &user.id, special_requests, &timestamp)
        .await
    {
        Ok(Some(id)) => {
            seed_participant_access(&db, &user.id, event.id, id).await;
            registration_created(
                id,
                "waitlisted",
                "Event is full. You have been added to the waitlist.",
            )
        }
        Ok(None) => {
            // Capacity may have reopened between the eligibility read and the
            // waitlist batch. One guarded retry chooses the now-current state.
            match insert_capacity_registration(
                &db,
                event.id,
                &user.id,
                special_requests,
                &timestamp,
            )
            .await
            {
                Ok(Some(id)) => {
                    seed_participant_access(&db, &user.id, event.id, id).await;
                    registration_created(
                        id,
                        "pending",
                        "Registration submitted successfully. Pending approval.",
                    )
                }
                Ok(None) => json_error(StatusCode::BAD_REQUEST, "Event is at full capacity"),
                Err(()) => duplicate_registration_response(&db, &user.id, event.id).await,
            }
        }
        Err(()) => duplicate_registration_response(&db, &user.id, event.id).await,
    }
}

pub async fn get_user_registrations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(get_user_registrations_inner(state, user, params)).await
}

async fn get_user_registrations_inner(
    state: AppState,
    user: UserRow,
    params: HashMap<String, String>,
) -> Response {
    let Some(page) = params
        .get("page")
        .map_or(Some(1.0), |value| js_parse_f64(value))
    else {
        return internal_error("Failed to fetch registrations");
    };
    let Some(limit) = params
        .get("limit")
        .map_or(Some(20.0), |value| js_parse_f64(value))
    else {
        return internal_error("Failed to fetch registrations");
    };
    let offset = (page - 1.0) * limit;
    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to fetch registrations"),
    };
    let mut condition = "r.user_id = ?".to_owned();
    let mut binds = vec![JsValue::from_str(&user.id)];
    if let Some(status) = params.get("status").filter(|status| !status.is_empty()) {
        condition.push_str(" AND r.status = ?");
        binds.push(JsValue::from_str(status));
    }

    let count = match bind_statement(
        &db,
        &format!("SELECT COUNT(*) AS total FROM registrations r WHERE {condition}"),
        &binds,
    ) {
        Ok(query) => match query.first::<CountRow>(None).await {
            Ok(Some(row)) => row.total,
            _ => return internal_error("Failed to fetch registrations"),
        },
        Err(()) => return internal_error("Failed to fetch registrations"),
    };

    let mut data_binds = binds;
    data_binds.push(JsValue::from_f64(limit));
    data_binds.push(JsValue::from_f64(offset));
    let query = format!(
        "{REGISTRATION_VIEW_SELECT} WHERE {condition} ORDER BY r.created_at DESC LIMIT ? OFFSET ?"
    );
    let rows = match bind_statement(&db, &query, &data_binds) {
        Ok(query) => match query.all().await {
            Ok(result) => match result.results::<RegistrationViewRow>() {
                Ok(rows) => rows,
                Err(_) => return internal_error("Failed to fetch registrations"),
            },
            Err(_) => return internal_error("Failed to fetch registrations"),
        },
        Err(()) => return internal_error("Failed to fetch registrations"),
    };
    Json(json!({
        "success": true,
        "data": rows.iter().map(registration_json).collect::<Vec<_>>(),
        "pagination": pagination_json(page, limit, count),
    }))
    .into_response()
}

async fn registration_owner(db: &D1Database, id: &str) -> Result<Option<RegistrationOwnerRow>, ()> {
    let query = bind_statement(db, REGISTRATION_OWNER_SQL, &[id_bind(id)])?;
    query.first(None).await.map_err(|_| ())
}

pub async fn get_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(get_registration_inner(state, user, id)).await
}

async fn get_registration_inner(state: AppState, user: UserRow, id: String) -> Response {
    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to fetch registration details"),
    };
    let owner = match registration_owner(&db, &id).await {
        Ok(Some(owner)) if can_access(&user, &owner.user_id) => owner,
        Ok(_) => return registration_not_found(),
        Err(()) => return internal_error("Failed to fetch registration details"),
    };
    let query = format!("{REGISTRATION_VIEW_SELECT} WHERE r.id = ?");
    let row = match bind_statement(&db, &query, &[JsValue::from_f64(owner.id as f64)]) {
        Ok(query) => match query.first::<RegistrationViewRow>(None).await {
            Ok(Some(row)) => row,
            Ok(None) => return registration_not_found(),
            Err(_) => return internal_error("Failed to fetch registration details"),
        },
        Err(()) => return internal_error("Failed to fetch registration details"),
    };
    Json(json!({ "success": true, "data": registration_json(&row) })).into_response()
}

pub async fn update_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<RegistrationBody>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(update_registration_inner(state, user, id, body)).await
}

async fn update_registration_inner(
    state: AppState,
    user: UserRow,
    id: String,
    body: RegistrationBody,
) -> Response {
    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to update registration"),
    };
    let registration = match registration_owner(&db, &id).await {
        Ok(Some(row)) if can_access(&user, &row.user_id) => row,
        Ok(_) => return registration_not_found(),
        Err(()) => return internal_error("Failed to update registration"),
    };
    if matches!(registration.status.as_str(), "confirmed" | "cancelled") {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Cannot modify registration after approval/rejection",
        );
    }
    let timestamp = now_iso();
    if registration
        .registration_deadline
        .as_deref()
        .is_some_and(|deadline| timestamp.as_str() > deadline)
    {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Cannot modify registration after deadline",
        );
    }
    let update = bind_statement(
        &db,
        "UPDATE registrations SET special_requests = ?, updated_at = ? WHERE id = ?",
        &[
            special_requests_bind(body.special_requests.as_deref()),
            JsValue::from_str(&timestamp),
            JsValue::from_f64(registration.id as f64),
        ],
    );
    match update {
        Ok(query) if query.run().await.is_ok() => {
            message_response("Registration updated successfully")
        }
        _ => internal_error("Failed to update registration"),
    }
}

pub async fn cancel_registration(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(cancel_registration_inner(state, user, id)).await
}

async fn cancel_registration_inner(state: AppState, user: UserRow, id: String) -> Response {
    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to cancel registration"),
    };
    let registration = match registration_owner(&db, &id).await {
        Ok(Some(row)) if can_access(&user, &row.user_id) => row,
        Ok(_) => return registration_not_found(),
        Err(()) => return internal_error("Failed to cancel registration"),
    };
    let now_ms = Date::now();
    let event_ms = Date::new(&JsValue::from_str(&registration.event_date_time)).get_time();
    if now_ms > event_ms {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Cannot cancel registration for past events",
        );
    }
    if (event_ms - now_ms) / 3_600_000.0 < 24.0 {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Cannot cancel registration within 24 hours of event",
        );
    }
    if registration.status == "cancelled" {
        return message_response("Registration cancelled successfully");
    }

    let timestamp = now_iso();
    let statements = if registration.status == "waitlisted" {
        let cancel = bind_statement(
            &db,
            CANCEL_WAITLISTED_SQL,
            &[
                JsValue::from_str(&timestamp),
                JsValue::from_str(&timestamp),
                JsValue::from_f64(registration.id as f64),
            ],
        );
        let decline = bind_statement(
            &db,
            DECLINE_WAITLIST_SQL,
            &[
                JsValue::from_str(&timestamp),
                JsValue::from_str(&timestamp),
                JsValue::from_f64(registration.event_id as f64),
                JsValue::from_str(&registration.user_id),
            ],
        );
        match (cancel, decline) {
            (Ok(cancel), Ok(decline)) => vec![cancel, decline],
            _ => return internal_error("Failed to cancel registration"),
        }
    } else {
        let values = || JsValue::from_str(&timestamp);
        let cancel = bind_statement(
            &db,
            CANCEL_ACTIVE_SQL,
            &[
                values(),
                values(),
                JsValue::from_f64(registration.id as f64),
            ],
        );
        let decrement = bind_statement(
            &db,
            DECREMENT_EVENT_SQL,
            &[
                values(),
                JsValue::from_f64(registration.event_id as f64),
                JsValue::from_f64(registration.id as f64),
                values(),
            ],
        );
        let accept = bind_statement(
            &db,
            ACCEPT_NEXT_WAITLIST_SQL,
            &[
                values(),
                values(),
                JsValue::from_f64(registration.event_id as f64),
                JsValue::from_f64(registration.id as f64),
                values(),
            ],
        );
        let promote = bind_statement(
            &db,
            PROMOTE_WAITLIST_REGISTRATION_SQL,
            &[
                values(),
                JsValue::from_f64(registration.event_id as f64),
                JsValue::from_f64(registration.event_id as f64),
                values(),
            ],
        );
        let increment = bind_statement(
            &db,
            INCREMENT_PROMOTED_EVENT_SQL,
            &[
                values(),
                JsValue::from_f64(registration.event_id as f64),
                values(),
                values(),
            ],
        );
        match (cancel, decrement, accept, promote, increment) {
            (Ok(cancel), Ok(decrement), Ok(accept), Ok(promote), Ok(increment)) => {
                vec![cancel, decrement, accept, promote, increment]
            }
            _ => return internal_error("Failed to cancel registration"),
        }
    };

    match db.batch(statements).await {
        Ok(results)
            if results
                .first()
                .is_some_and(|result| result_changes(result) == 1) =>
        {
            message_response("Registration cancelled successfully")
        }
        _ => internal_error("Failed to cancel registration"),
    }
}

pub async fn registration_stats(
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> Response {
    SendFuture::new(registration_stats_inner(state, event_id)).await
}

async fn registration_stats_inner(state: AppState, event_id: String) -> Response {
    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get registration stats"),
    };
    // Deliberately preserve the live Express drift. That route queries the
    // nonexistent `event_registrations` table and therefore returns 500.
    let query = bind_statement(
        &db,
        "SELECT COUNT(*) AS total_registrations, COUNT(CASE WHEN status = 'confirmed' THEN 1 END) AS confirmed_registrations, COUNT(CASE WHEN status = 'pending' THEN 1 END) AS pending_registrations, COUNT(CASE WHEN status = 'waitlist' THEN 1 END) AS waitlist_registrations FROM event_registrations WHERE event_id = ?",
        &[id_bind(&event_id)],
    );
    match query {
        Ok(query) => match query.first::<LegacyStatsRow>(None).await {
            Ok(Some(row)) => Json(json!({
                "success": true,
                "data": {
                    "total_registrations": row.total_registrations,
                    "confirmed_registrations": row.confirmed_registrations,
                    "pending_registrations": row.pending_registrations,
                    "waitlist_registrations": row.waitlist_registrations,
                }
            }))
            .into_response(),
            Ok(None) => Json(json!({
                "success": true,
                "data": {
                    "total_registrations": 0,
                    "confirmed_registrations": 0,
                    "pending_registrations": 0,
                    "waitlist_registrations": 0,
                }
            }))
            .into_response(),
            Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()),
        },
        Err(()) => internal_error("Failed to get registration stats"),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBody {
    payment_status: Option<String>,
    payment_intent_id: Option<String>,
}

/// `POST /api/registrations/{id}/payment` — admin-gated port of Express's
/// unmounted `updatePaymentStatus` controller (the frontend service calls it;
/// Express never mounted the route, so its participant-access rows could never
/// reach `paid`). Deviation: Express 404s this path, Rust implements it.
/// The epa upsert mirrors `updateParticipantAccess` (level 3/1/0 as TEXT).
pub async fn update_payment_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<PaymentBody>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    if let Err(response) = crate::auth::require_admin(&user) {
        return response;
    }
    SendFuture::new(update_payment_status_inner(state, id, body)).await
}

async fn update_payment_status_inner(state: AppState, id: String, body: PaymentBody) -> Response {
    const ERROR: &str = "Failed to update payment status";
    let Some(payment_status) = body
        .payment_status
        .as_deref()
        .filter(|status| matches!(*status, "pending" | "paid" | "refunded" | "failed"))
    else {
        return json_error(StatusCode::BAD_REQUEST, "Invalid payment status");
    };

    let db = match state.env.d1("DB") {
        Ok(db) => db,
        Err(_) => return internal_error(ERROR),
    };

    let owner = match bind_statement(&db, REGISTRATION_OWNER_SQL, &[id_bind(&id)]) {
        Ok(statement) => match statement.first::<Value>(None).await {
            Ok(row) => row,
            Err(_) => return internal_error(ERROR),
        },
        Err(()) => return internal_error(ERROR),
    };
    let Some(owner) = owner else {
        return json_error(StatusCode::NOT_FOUND, "Registration not found");
    };
    let Some(user_id) = owner
        .get("user_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return internal_error(ERROR);
    };
    let Some(event_id) = owner.get("event_id").and_then(Value::as_f64) else {
        return internal_error(ERROR);
    };

    let update = bind_statement(
        &db,
        PAYMENT_UPDATE_SQL,
        &[
            JsValue::from_str(payment_status),
            body.payment_intent_id
                .as_deref()
                .filter(|intent| !intent.is_empty())
                .map_or(JsValue::NULL, JsValue::from_str),
            JsValue::from_str(&now_iso()),
            id_bind(&id),
        ],
    );
    let updated = match update {
        Ok(statement) => match statement.run().await {
            Ok(result) => result_changes(&result) > 0,
            Err(_) => return internal_error(ERROR),
        },
        Err(()) => return internal_error(ERROR),
    };
    if !updated {
        return json_error(StatusCode::NOT_FOUND, "Registration not found");
    }

    // Mirror `updateParticipantAccess`: upsert the epa row with the new
    // payment status and the stringified access level.
    let upsert = bind_statement(
        &db,
        "INSERT INTO event_participant_access (user_id, event_id, registration_id, has_access, payment_status, access_level, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?, ?, ?) ON CONFLICT(user_id, event_id) DO UPDATE SET payment_status = excluded.payment_status, access_level = excluded.access_level, registration_id = excluded.registration_id, updated_at = excluded.updated_at",
        &[
            JsValue::from_str(&user_id),
            JsValue::from_f64(event_id),
            id_bind(&id),
            JsValue::from_str(payment_status),
            JsValue::from_str(access_level_for(payment_status)),
            JsValue::from_str(&now_iso()),
            JsValue::from_str(&now_iso()),
        ],
    );
    if let Ok(statement) = upsert {
        let _ = statement.run().await;
    }

    message_response("Payment status updated successfully")
}
