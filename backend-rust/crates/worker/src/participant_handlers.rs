use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::ApiEnvelope;
use hesocial_core::auth::UserRow;
use hesocial_core::pagination::pagination_json;
use hesocial_core::participants::{
    ParticipantRow, ParticipantViewAccess, mask_participant, participant_view_access,
    viewer_relationship,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use worker::send::SendFuture;

use crate::AppState;
use crate::auth::{authenticate, internal_error};
use crate::auth_handlers::now_iso;
use crate::db::{self, Val};

#[derive(Deserialize)]
struct AccessRecord {
    payment_status: String,
}

#[derive(Deserialize)]
struct RegistrationAccessRow {
    status: String,
    payment_status: String,
}

#[derive(Deserialize)]
struct CountRow {
    paid: Option<i64>,
    unpaid: Option<i64>,
}

#[derive(Deserialize)]
struct VisibleCountRow {
    count: i64,
}

#[derive(Deserialize)]
struct TierCountRow {
    membership_tier: String,
    count: i64,
}

#[derive(Deserialize)]
struct PrivacySettingsRow {
    privacy_level: i64,
    allow_contact: i64,
    show_in_list: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettingsBody {
    privacy_level: Option<f64>,
    allow_contact: Option<bool>,
    show_in_list: Option<bool>,
}

#[derive(Deserialize)]
pub struct ContactBody {
    message: Option<Value>,
}

struct ParticipantCounts {
    paid: i64,
    unpaid: i64,
    by_tier: Map<String, Value>,
}

struct ParticipantList {
    participants: Vec<Value>,
    total_count: i64,
    counts: ParticipantCounts,
    viewer_access: ParticipantViewAccess,
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(ApiEnvelope::<Value>::error(error))).into_response()
}

fn id_bind(id: &str) -> Value {
    match id.parse::<f64>().ok().filter(|value| value.is_finite()) {
        Some(number) => Val::from_f64(number),
        None => Val::from_str(id),
    }
}

fn optional_bool_bind(value: Option<bool>) -> Value {
    value.map_or(db::NULL, Val::from_bool)
}

fn bind_statement(db: &db::Db, sql: &str, values: &[Value]) -> Result<db::PreparedStatement, ()> {
    db.prepare(sql).bind(values).map_err(|_| ())
}

fn js_parse_int(raw: Option<&String>, fallback: i64) -> i64 {
    let Some(raw) = raw else {
        return fallback;
    };
    let trimmed = raw.trim_start();
    let mut end = 0;
    for (index, character) in trimmed.char_indices() {
        if index == 0 && matches!(character, '+' | '-') {
            end = character.len_utf8();
        } else if character.is_ascii_digit() {
            end = index + character.len_utf8();
        } else {
            break;
        }
    }
    let parsed = trimmed
        .get(..end)
        .and_then(|value| value.parse::<i64>().ok());
    parsed.filter(|value| *value != 0).unwrap_or(fallback)
}

async fn participant_access(db: &db::Db, user: &UserRow, event_id: &str) -> ParticipantViewAccess {
    let query = bind_statement(
        db,
        "SELECT payment_status FROM event_participant_access WHERE user_id = ? AND event_id = ?",
        &[Val::from_str(&user.id), id_bind(event_id)],
    );
    let payment_status = match query {
        Ok(query) => query
            .first::<AccessRecord>(None)
            .await
            .ok()
            .flatten()
            .map(|record| record.payment_status),
        Err(()) => None,
    };
    participant_view_access(viewer_relationship(
        payment_status.as_deref(),
        &user.membership_tier,
    ))
}

async fn registration_access(
    db: &db::Db,
    user_id: &str,
    event_id: &str,
) -> Result<Option<RegistrationAccessRow>, ()> {
    let query = bind_statement(
        db,
        "SELECT r.status, r.payment_status FROM registrations r LEFT JOIN event_participant_access epa ON r.id = epa.registration_id WHERE r.user_id = ? AND r.event_id = ?",
        &[Val::from_str(user_id), id_bind(event_id)],
    )?;
    query.first(None).await.map_err(|_| ())
}

fn access_check_json(
    access: ParticipantViewAccess,
    registration: Option<&RegistrationAccessRow>,
) -> Value {
    json!({
        "hasAccess": access.can_view_participants,
        "accessLevel": access,
        "paymentRequired": !access.can_view_participants,
        "paymentStatus": registration.map_or("none", |row| row.payment_status.as_str()),
        "registrationStatus": registration.map_or("none", |row| row.status.as_str()),
    })
}

fn denied_access_check_unknown() -> Value {
    json!({
        "hasAccess": false,
        "accessLevel": ParticipantViewAccess::denied(),
        "paymentRequired": true,
        "paymentStatus": "unknown",
    })
}

async fn check_access(db: &db::Db, user: &UserRow, event_id: &str) -> Value {
    let access = participant_access(db, user, event_id).await;
    match registration_access(db, &user.id, event_id).await {
        Ok(registration) => access_check_json(access, registration.as_ref()),
        Err(()) => denied_access_check_unknown(),
    }
}

async fn participant_counts(db: &db::Db, event_id: &str) -> Result<ParticipantCounts, ()> {
    let count = bind_statement(
        db,
        "SELECT SUM(CASE WHEN payment_status = 'paid' THEN 1 ELSE 0 END) AS paid, SUM(CASE WHEN payment_status != 'paid' THEN 1 ELSE 0 END) AS unpaid FROM event_participant_access WHERE event_id = ?",
        &[id_bind(event_id)],
    )?
    .first::<CountRow>(None)
    .await
    .map_err(|_| ())?
    .unwrap_or(CountRow {
        paid: None,
        unpaid: None,
    });
    let tiers = bind_statement(
        db,
        "SELECT u.membership_tier, COUNT(*) AS count FROM users u JOIN event_participant_access epa ON u.id = epa.user_id WHERE epa.event_id = ? AND epa.payment_status = 'paid' GROUP BY u.membership_tier",
        &[id_bind(event_id)],
    )?
    .all()
    .await
    .map_err(|_| ())?
    .results::<TierCountRow>()
    .map_err(|_| ())?;
    let by_tier = tiers
        .into_iter()
        .map(|tier| (tier.membership_tier, json!(tier.count)))
        .collect();

    Ok(ParticipantCounts {
        paid: count.paid.unwrap_or(0),
        unpaid: count.unpaid.unwrap_or(0),
        by_tier,
    })
}

fn list_data(list: &ParticipantList) -> Value {
    json!({
        "participants": list.participants,
        "totalCount": list.total_count,
        "paidParticipantCount": list.counts.paid,
        "unpaidParticipantCount": list.counts.unpaid,
        "viewerAccess": list.viewer_access,
        "participantCountByTier": list.counts.by_tier,
    })
}

async fn log_views(
    db: &db::Db,
    viewer_id: &str,
    event_id: &str,
    access_level: i64,
    participants: &[Value],
) {
    let statements = participants
        .iter()
        .filter_map(|participant| participant.get("id").and_then(Value::as_str))
        .filter_map(|participant_id| {
            let timestamp = now_iso();
            bind_statement(
                db,
                "INSERT INTO participant_view_logs (viewer_id, participant_id, event_id, access_level, ip_address, user_agent, viewed_at, created_at) VALUES (?, ?, ?, ?, NULL, NULL, ?, ?)",
                &[
                    Val::from_str(viewer_id),
                    Val::from_str(participant_id),
                    id_bind(event_id),
                    Val::from_f64(access_level as f64),
                    Val::from_str(&timestamp),
                    Val::from_str(&timestamp),
                ],
            )
            .ok()
        })
        .collect::<Vec<_>>();
    if !statements.is_empty() {
        let _ = db.batch(statements).await;
    }
}

async fn get_participant_list(
    db: &db::Db,
    user: &UserRow,
    event_id: &str,
    page: i64,
    limit: i64,
    filters: &HashMap<String, String>,
    viewer_access: ParticipantViewAccess,
) -> Result<ParticipantList, ()> {
    // `viewer_access` is computed by the caller, which rejects unpaid viewers
    // with the Express 403 before reaching this paid-only query path.

    let mut conditions = vec![
        "epa.event_id = ?".to_owned(),
        "epa.payment_status = 'paid'".to_owned(),
        "COALESCE(epo.show_in_list, 1) = 1".to_owned(),
        "COALESCE(epo.privacy_level, u.privacy_level, 2) <= ?".to_owned(),
    ];
    let mut values = vec![
        id_bind(event_id),
        Val::from_f64(viewer_access.max_privacy_level_visible as f64),
    ];
    if let Some(tier) = filters
        .get("membershipTier")
        .filter(|value| !value.is_empty())
    {
        conditions.push("u.membership_tier = ?".to_owned());
        values.push(Val::from_str(tier));
    }
    if let Some(profession) = filters.get("profession").filter(|value| !value.is_empty()) {
        conditions.push("u.profession LIKE ?".to_owned());
        values.push(Val::from_str(&format!("%{profession}%")));
    }
    let min_privacy = filters.get("minPrivacyLevel").and_then(|raw| {
        js_parse_int(Some(raw), 0)
            .ne(&0)
            .then(|| js_parse_int(Some(raw), 0))
    });
    if let Some(level) = min_privacy {
        conditions.push("COALESCE(epo.privacy_level, u.privacy_level, 2) >= ?".to_owned());
        values.push(Val::from_f64(level as f64));
    }
    conditions.push("u.id != ?".to_owned());
    values.push(Val::from_str(&user.id));

    let where_clause = conditions.join(" AND ");
    let mut joined_values = vec![id_bind(event_id)];
    joined_values.extend(values.iter().cloned());
    let count_sql = format!(
        "SELECT COUNT(*) AS count FROM users u JOIN event_participant_access epa ON u.id = epa.user_id LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ? WHERE {where_clause}"
    );
    let total_count = bind_statement(db, &count_sql, &joined_values)?
        .first::<VisibleCountRow>(None)
        .await
        .map_err(|_| ())?
        .map_or(0, |row| row.count);

    let participant_sql = format!(
        "SELECT u.id, u.first_name, u.last_name, u.email, u.phone_number AS phone, u.age, u.profession, u.company, NULL AS city, u.membership_tier, u.interests, u.profile_picture, u.bio, COALESCE(epo.privacy_level, u.privacy_level, 2) AS effective_privacy_level, COALESCE(epo.allow_contact, 1) AS can_contact FROM users u JOIN event_participant_access epa ON u.id = epa.user_id LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ? WHERE {where_clause} ORDER BY u.membership_tier DESC, u.first_name ASC LIMIT ? OFFSET ?"
    );
    let mut participant_values = joined_values;
    participant_values.push(Val::from_f64(limit as f64));
    participant_values.push(Val::from_f64(
        page.saturating_sub(1).saturating_mul(limit) as f64
    ));
    let rows = bind_statement(db, &participant_sql, &participant_values)?
        .all()
        .await
        .map_err(|_| ())?
        .results::<ParticipantRow>()
        .map_err(|_| ())?;
    let participants = rows
        .iter()
        .filter_map(|participant| mask_participant(participant, viewer_access))
        .collect::<Vec<_>>();
    log_views(
        db,
        &user.id,
        event_id,
        viewer_access.access_level,
        &participants,
    )
    .await;
    let counts = participant_counts(db, event_id).await?;

    Ok(ParticipantList {
        participants,
        total_count,
        counts,
        viewer_access,
    })
}

pub async fn list_participants(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
    Query(filters): Query<HashMap<String, String>>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(list_participants_inner(state, user, event_id, filters)).await
}

async fn list_participants_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    filters: HashMap<String, String>,
) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get event participants"),
    };
    let page = js_parse_int(filters.get("page"), 1);
    let limit = js_parse_int(filters.get("limit"), 20).clamp(1, 100);
    // Unpaid/pending viewers get the same 403 the detail route and Express's
    // list route answer — not a 200 with an empty list (the pre-payment-gate
    // drift the recovered WIP's contract test caught).
    let viewer_access = participant_access(&db, &user, &event_id).await;
    if !viewer_access.can_view_participants {
        return json_error(
            StatusCode::FORBIDDEN,
            "Access denied - payment required to view participants",
        );
    }
    match get_participant_list(&db, &user, &event_id, page, limit, &filters, viewer_access).await {
        Ok(list) => Json(json!({
            "success": true,
            "data": list_data(&list),
            "pagination": pagination_json(page as f64, limit as f64, list.total_count),
        }))
        .into_response(),
        Err(()) => internal_error("Failed to get event participants"),
    }
}

pub async fn get_participant_access(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(get_participant_access_inner(state, user, event_id)).await
}

async fn get_participant_access_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to check participant access"),
    };
    Json(json!({ "success": true, "data": check_access(&db, &user, &event_id).await }))
        .into_response()
}

pub async fn get_participant(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((event_id, participant_id)): Path<(String, String)>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(get_participant_inner(state, user, event_id, participant_id)).await
}

async fn get_participant_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    participant_id: String,
) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get participant details"),
    };
    // K3 review fix: query the target by id through the same visibility
    // predicates as the list — the Express-ported LIMIT-1 fetch + find
    // 404'd for every participant except the sort-first row, and it
    // mis-attributed the view log to that first row.
    let access = participant_access(&db, &user, &event_id).await;
    if !access.can_view_participants {
        return json_error(
            StatusCode::FORBIDDEN,
            "Access denied - payment required to view participants",
        );
    }
    let participant_sql = "SELECT u.id, u.first_name, u.last_name, u.email, u.phone_number AS phone, u.age, u.profession, u.company, NULL AS city, u.membership_tier, u.interests, u.profile_picture, u.bio, COALESCE(epo.privacy_level, u.privacy_level, 2) AS effective_privacy_level, COALESCE(epo.allow_contact, 1) AS can_contact FROM users u JOIN event_participant_access epa ON u.id = epa.user_id LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ? WHERE epa.event_id = ? AND epa.payment_status = 'paid' AND COALESCE(epo.show_in_list, 1) = 1 AND COALESCE(epo.privacy_level, u.privacy_level, 2) <= ? AND u.id != ? AND u.id = ? LIMIT 1";
    let row = match bind_statement(
        &db,
        participant_sql,
        &[
            id_bind(&event_id),
            id_bind(&event_id),
            Val::from_f64(access.max_privacy_level_visible as f64),
            Val::from_str(&user.id),
            Val::from_str(&participant_id),
        ],
    ) {
        Ok(query) => match query.first::<ParticipantRow>(None).await {
            Ok(Some(row)) => row,
            Ok(None) => {
                return json_error(
                    StatusCode::NOT_FOUND,
                    "Participant not found or not visible",
                );
            }
            Err(_) => return internal_error("Failed to get participant details"),
        },
        Err(()) => return internal_error("Failed to get participant details"),
    };
    let Some(masked) = mask_participant(&row, access) else {
        return json_error(
            StatusCode::NOT_FOUND,
            "Participant not found or not visible",
        );
    };
    log_views(
        &db,
        &user.id,
        &event_id,
        access.access_level,
        std::slice::from_ref(&masked),
    )
    .await;
    Json(json!({
        "success": true,
        "data": {
            "participant": masked,
            "viewerAccess": access.access_level,
        }
    }))
    .into_response()
}

pub async fn initiate_contact(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((event_id, _participant_id)): Path<(String, String)>,
    Json(body): Json<ContactBody>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(initiate_contact_inner(state, user, event_id, body)).await
}

async fn initiate_contact_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    body: ContactBody,
) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to initiate contact"),
    };
    let access_check = check_access(&db, &user, &event_id).await;
    let can_contact = access_check
        .pointer("/accessLevel/canInitiateContact")
        .and_then(Value::as_bool)
        == Some(true);
    if access_check.get("hasAccess").and_then(Value::as_bool) != Some(true) || !can_contact {
        return json_error(
            StatusCode::FORBIDDEN,
            "Access denied - cannot initiate contact",
        );
    }
    match body.message {
        None | Some(Value::Null) | Some(Value::Bool(false)) => {
            json_error(StatusCode::BAD_REQUEST, "Message is required")
        }
        Some(Value::Number(number)) if number.as_f64() == Some(0.0) => {
            json_error(StatusCode::BAD_REQUEST, "Message is required")
        }
        Some(Value::String(message)) if message.trim().is_empty() => {
            json_error(StatusCode::BAD_REQUEST, "Message is required")
        }
        Some(Value::String(_)) => Json(json!({
            "success": true,
            "message": "Contact request sent successfully"
        }))
        .into_response(),
        Some(_) => internal_error("Failed to initiate contact"),
    }
}

pub async fn update_privacy_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
    Json(body): Json<PrivacySettingsBody>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(update_privacy_settings_inner(state, user, event_id, body)).await
}

async fn update_privacy_settings_inner(
    state: AppState,
    user: UserRow,
    event_id: String,
    body: PrivacySettingsBody,
) -> Response {
    if body.privacy_level.is_some_and(|privacy_level| {
        !(1.0..=5.0).contains(&privacy_level) || privacy_level.fract() != 0.0
    }) {
        return json_error(
            StatusCode::BAD_REQUEST,
            "Privacy level must be between 1 and 5",
        );
    }
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to update privacy settings"),
    };
    let privacy_level = body.privacy_level.map_or(db::NULL, Val::from_f64);
    let query = bind_statement(
        &db,
        "INSERT INTO event_privacy_overrides (user_id, event_id, privacy_level, allow_contact, show_in_list, created_at, updated_at) VALUES (?, ?, COALESCE(?, (SELECT privacy_level FROM event_privacy_overrides WHERE user_id = ? AND event_id = ?), 3), ?, ?, ?, ?) ON CONFLICT(user_id, event_id) DO UPDATE SET privacy_level = COALESCE(excluded.privacy_level, privacy_level), allow_contact = COALESCE(excluded.allow_contact, allow_contact), show_in_list = COALESCE(excluded.show_in_list, show_in_list), updated_at = excluded.updated_at",
        &[
            Val::from_str(&user.id),
            id_bind(&event_id),
            privacy_level,
            Val::from_str(&user.id),
            id_bind(&event_id),
            optional_bool_bind(body.allow_contact),
            optional_bool_bind(body.show_in_list),
            Val::from_str(&now_iso()),
            Val::from_str(&now_iso()),
        ],
    );
    match query {
        Ok(query) if query.run().await.is_ok() => Json(json!({
            "success": true,
            "message": "Privacy settings updated successfully"
        }))
        .into_response(),
        _ => internal_error("Failed to update privacy settings"),
    }
}

pub async fn get_privacy_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(get_privacy_settings_inner(state, user, event_id)).await
}

async fn get_privacy_settings_inner(state: AppState, user: UserRow, event_id: String) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return internal_error("Failed to get privacy settings"),
    };
    let query = bind_statement(
        &db,
        "SELECT COALESCE(epo.privacy_level, u.privacy_level, 2) AS privacy_level, COALESCE(epo.allow_contact, 1) AS allow_contact, COALESCE(epo.show_in_list, 1) AS show_in_list FROM users u LEFT JOIN event_privacy_overrides epo ON u.id = epo.user_id AND epo.event_id = ? WHERE u.id = ?",
        &[id_bind(&event_id), Val::from_str(&user.id)],
    );
    match query {
        Ok(query) => match query.first::<PrivacySettingsRow>(None).await {
            Ok(Some(settings)) => Json(json!({
                "success": true,
                "data": {
                    "privacy_level": settings.privacy_level,
                    "allow_contact": settings.allow_contact != 0,
                    "show_in_list": settings.show_in_list != 0,
                }
            }))
            .into_response(),
            Ok(None) => Json(json!({ "success": true, "data": Value::Null })).into_response(),
            Err(_) => internal_error("Failed to get privacy settings"),
        },
        Err(()) => internal_error("Failed to get privacy settings"),
    }
}
