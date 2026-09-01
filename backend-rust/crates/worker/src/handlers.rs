#![allow(clippy::result_large_err)]
use std::collections::HashMap;
use std::sync::OnceLock;

use axum::Json;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::events::{
    CategoryRow, EventListRow, VenueListRow, category_json, event_list_item_json,
    venue_list_item_json,
};
use hesocial_core::pagination::{js_parse_f64, number_json, pagination_json};
use hesocial_core::{ApiEnvelope, HealthResponse};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use worker::js_sys::Date;
use worker::send::SendFuture;

use crate::AppState;
use crate::auth::{authenticate, require_admin, require_super_admin};
use crate::db::{self, Val};

const EVENT_SELECT: &str = "SELECT e.id AS id, e.title AS name, e.description AS description, e.start_datetime AS \"dateTime\", e.registration_closes_at AS \"registrationDeadline\", e.price_platinum AS price_platinum, e.price_diamond AS price_diamond, e.price_black_card AS price_black_card, e.currency AS pricing_currency, e.dress_code AS \"dressCode\", e.capacity_max AS capacity, e.current_registrations AS \"currentAttendees\", e.gallery_images AS images, e.special_requirements AS requirements, e.created_at AS \"createdAt\", e.updated_at AS \"updatedAt\", v.name AS \"venueName\", v.address AS \"venueAddress\", v.city AS \"venueCity\", v.rating AS \"venueRating\", v.amenities AS \"venueAmenities\", ec.name AS \"categoryName\", ec.icon AS \"categoryIcon\", (u.first_name || ' ' || u.last_name) AS \"organizerName\" FROM events e JOIN venues v ON e.venue_id = v.id JOIN event_categories ec ON e.category_id = ec.id JOIN users u ON e.organizer_id = u.id";

const EVENT_COUNT: &str = "SELECT COUNT(*) AS total FROM events e JOIN venues v ON e.venue_id = v.id JOIN event_categories ec ON e.category_id = ec.id JOIN users u ON e.organizer_id = u.id";

const CATEGORIES_SQL: &str = "SELECT id, name, description, icon, created_at AS \"createdAt\" FROM event_categories ORDER BY name";

const VENUES_SQL: &str = "SELECT id, name, address, city, latitude, longitude, rating, amenities, images, created_at AS \"createdAt\" FROM venues";

static START_MS: OnceLock<f64> = OnceLock::new();

#[derive(Deserialize)]
struct CountRow {
    total: i64,
}

fn now_iso() -> String {
    Date::new_0()
        .to_iso_string()
        .as_string()
        .unwrap_or_default()
}

fn error_response(error: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiEnvelope::<Value>::error(error)),
    )
        .into_response()
}

fn success_with_data(data: Vec<Value>) -> Response {
    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert("data".to_owned(), Value::Array(data));
    Json(Value::Object(body)).into_response()
}

fn parse_query_number(value: Option<&String>, default: f64) -> Option<f64> {
    match value {
        None => Some(default),
        Some(raw) => js_parse_f64(raw),
    }
}

pub async fn health() -> impl IntoResponse {
    Json(HealthResponse::healthy(now_iso()))
}

pub async fn health_status(State(state): State<AppState>) -> Response {
    let start = *START_MS.get_or_init(Date::now);
    let uptime = ((Date::now() - start) / 1000.0).floor().max(0.0);
    let hours = (uptime / 3600.0).floor();
    let minutes = ((uptime % 3600.0) / 60.0).floor();
    let seconds = uptime % 60.0;
    let environment = state
        .env
        .var("NODE_ENV")
        .map(|value| value.to_string())
        .unwrap_or_else(|_| "development".to_owned());

    let body = json!({
        "success": true,
        "status": "healthy",
        "server": {
            "uptime": number_json(uptime),
            "uptimeFormatted": format!("{hours}h {minutes}m {seconds}s"),
            "memory": {
                "rss": "0MB",
                "heapUsed": "0MB",
                "heapTotal": "0MB"
            },
            "nodeVersion": Value::Null,
            "platform": Value::Null
        },
        "database": {
            "type": "Turso",
            "r2Sync": "disabled"
        },
        "environment": environment,
        "timestamp": now_iso()
    });

    Json(body).into_response()
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(list_events_inner(state, params)).await
}

async fn list_events_inner(state: AppState, params: HashMap<String, String>) -> Response {
    let (Some(page), Some(limit)) = (
        parse_query_number(params.get("page"), 1.0),
        parse_query_number(params.get("limit"), 20.0),
    ) else {
        return error_response("Failed to get events");
    };

    let sort = params.get("sort").map_or("date_time", String::as_str);
    let order = params.get("order").map_or("asc", String::as_str);
    let upcoming = params.get("upcoming").map_or("true", String::as_str);

    let mut conditions = vec![
        "e.status = 'published'".to_owned(),
        "e.approval_status = 'approved'".to_owned(),
    ];
    let mut binds: Vec<Value> = Vec::new();

    if upcoming == "true" {
        conditions.push("e.start_datetime > ?".to_owned());
        binds.push(Val::from_str(&now_iso()));
    }
    if let Some(search) = params.get("search").filter(|value| !value.is_empty()) {
        conditions.push("(e.title LIKE ? OR e.description LIKE ?)".to_owned());
        let pattern = format!("%{search}%");
        binds.push(Val::from_str(&pattern));
        binds.push(Val::from_str(&pattern));
    }
    if let Some(category) = params.get("category").filter(|value| !value.is_empty()) {
        conditions.push("(ec.slug = ? OR ec.name = ?)".to_owned());
        binds.push(Val::from_str(category));
        binds.push(Val::from_str(category));
    }

    let where_clause = format!("WHERE {}", conditions.join(" AND "));
    let sort_column = match sort {
        "created_at" => "e.created_at",
        "name" => "e.title",
        "capacity" => "e.capacity_max",
        "current_attendees" => "e.current_registrations",
        _ => "e.start_datetime",
    };
    let sort_order = if order == "desc" { "DESC" } else { "ASC" };
    let offset = (page - 1.0) * limit;

    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return error_response("Failed to get events"),
    };

    let mut data_binds = binds.clone();
    data_binds.push(Val::from_f64(limit));
    data_binds.push(Val::from_f64(offset));

    let events_query = db
        .prepare(format!(
            "{EVENT_SELECT} {where_clause} ORDER BY {sort_column} {sort_order} LIMIT ? OFFSET ?"
        ))
        .bind(&data_binds);
    let count_query = db
        .prepare(format!("{EVENT_COUNT} {where_clause}"))
        .bind(&binds);

    let (Ok(events_query), Ok(count_query)) = (events_query, count_query) else {
        return error_response("Failed to get events");
    };

    let rows = match events_query.all().await {
        Ok(result) => match result.results::<EventListRow>() {
            Ok(rows) => rows,
            Err(_) => return error_response("Failed to get events"),
        },
        Err(_) => return error_response("Failed to get events"),
    };
    let total = match count_query.all().await {
        Ok(result) => match result.results::<CountRow>() {
            Ok(rows) => rows.first().map_or(0, |row| row.total),
            Err(_) => return error_response("Failed to get events"),
        },
        Err(_) => return error_response("Failed to get events"),
    };

    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert(
        "data".to_owned(),
        Value::Array(rows.iter().map(event_list_item_json).collect()),
    );
    body.insert("pagination".to_owned(), pagination_json(page, limit, total));
    Json(Value::Object(body)).into_response()
}

pub async fn list_categories(State(state): State<AppState>) -> Response {
    SendFuture::new(list_categories_inner(state)).await
}

async fn list_categories_inner(state: AppState) -> Response {
    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return error_response("Failed to get event categories"),
    };

    let rows = match db.prepare(CATEGORIES_SQL).all().await {
        Ok(result) => match result.results::<CategoryRow>() {
            Ok(rows) => rows,
            Err(_) => return error_response("Failed to get event categories"),
        },
        Err(_) => return error_response("Failed to get event categories"),
    };

    success_with_data(rows.iter().map(category_json).collect())
}

pub async fn list_venues(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(list_venues_inner(state, params)).await
}

async fn list_venues_inner(state: AppState, params: HashMap<String, String>) -> Response {
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Value> = Vec::new();

    if let Some(city) = params.get("city").filter(|value| !value.is_empty()) {
        conditions.push("city LIKE ?".to_owned());
        binds.push(Val::from_str(&format!("%{city}%")));
    }
    if let Some(rating) = params.get("rating").filter(|value| !value.is_empty()) {
        let Some(rating) = parse_query_number(Some(rating), 0.0) else {
            return error_response("Failed to get venues");
        };
        conditions.push("rating >= ?".to_owned());
        binds.push(Val::from_f64(rating));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let db = match db::Db::from_env(&state.env) {
        Ok(db) => db,
        Err(_) => return error_response("Failed to get venues"),
    };

    let query = db
        .prepare(format!(
            "{VENUES_SQL} {where_clause} ORDER BY rating DESC, name"
        ))
        .bind(&binds);
    let Ok(query) = query else {
        return error_response("Failed to get venues");
    };

    let rows = match query.all().await {
        Ok(result) => match result.results::<VenueListRow>() {
            Ok(rows) => rows,
            Err(_) => return error_response("Failed to get venues"),
        },
        Err(_) => return error_response("Failed to get venues"),
    };

    success_with_data(rows.iter().map(venue_list_item_json).collect())
}

pub async fn fallback(State(state): State<AppState>, request: Request<Body>) -> Response {
    let path = request.uri().path().to_owned();
    let method = request.method().as_str().to_owned();

    if path == "/api/admin" || path.starts_with("/api/admin/") {
        let guard = SendFuture::new(async {
            authenticate(&state, request.headers())
                .await
                .and_then(|user| {
                    if path == "/api/admin/restore" {
                        require_super_admin(&user).map(|()| user)
                    } else {
                        require_admin(&user).map(|()| user)
                    }
                })
        })
        .await;
        if let Err(response) = guard {
            return response;
        }
    }

    if path.starts_with("/api") {
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({
                "success": false,
                "error": "Endpoint not implemented yet",
                "message": "This endpoint will be available in a later remediation phase",
                "availableEndpoints": ["/events", "/categories", "/venues", "/auth/register", "/auth/login"]
            })),
        )
            .into_response();
    }

    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "error": "Route not found",
            "message": format!("{method} {path} is not a valid endpoint")
        })),
    )
        .into_response()
}
