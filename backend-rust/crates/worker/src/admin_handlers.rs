//! `/api/users/*` + `GET /api/admin/database/stats` — Phase 7 port of
//! `backend/src/routes/userManagement.ts` and the stats route of
//! `backend/src/routes/admin.ts`.
#![allow(clippy::result_large_err)]
//!
//! Guards mirror Express per route: `authenticate` + `require_admin`
//! everywhere except DELETE `/api/users/{id}` and POST `.../role`, which use
//! `require_super_admin`. All other `/api/admin/*` routes (backup/restore/
//! cleanup/periodic-backup/checkpoint) intentionally stay on the guarded 501
//! fallback in `handlers.rs` (locked decision #5: DuckDB-specific endpoints
//! are not ported).
//!
//! SQL, envelopes, and deviations are in `core::admin` (module docs there).

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::admin::{
    COLUMN_COUNT_SQL, DELETE_USER_SQL, ListFilters, RECENT_REGISTRATIONS_SQL, ROLES,
    SCHEMA_VERSION_SQL, SERVER_STATE_SQL, TABLES_SQL, UPDATE_ROLE_SQL, USER_BY_ID_SQL,
    USER_COUNT_SQL, USER_EXISTS_SQL, USER_LIST_SELECT, USERS_BY_ROLE_SQL, USERS_BY_TIER_SQL,
    USERS_BY_VERIFICATION_SQL, VERIFY_STATUSES, VERIFY_USER_SQL, database_stats_envelope,
    js_parse_int_or, list_where, server_stats_json, stats_envelope, table_json, update_assignments,
    user_json, verify_message,
};
use hesocial_core::auth::UserRow;
use hesocial_core::pagination::pagination_json;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use worker::send::SendFuture;

use crate::AppState;
use crate::auth::{authenticate, require_admin, require_super_admin};
use crate::auth_handlers::now_iso;
use crate::db::{self, Val};

const LIST_ERROR: &str = "Failed to retrieve users";
const GET_ERROR: &str = "Failed to retrieve user";
const UPDATE_ERROR: &str = "Failed to update user";
const DELETE_ERROR: &str = "Failed to delete user";
const VERIFY_ERROR: &str = "Failed to verify user";
const ROLE_ERROR: &str = "Failed to update user role";
const STATS_ERROR: &str = "Failed to retrieve user statistics";
const DB_STATS_ERROR: &str = "Failed to get database statistics";
const USER_NOT_FOUND: &str = "User not found";

#[derive(Deserialize)]
struct CountRow {
    total: i64,
}

fn error_response(status: StatusCode, error: &str) -> Response {
    (status, Json(json!({ "success": false, "error": error }))).into_response()
}

fn not_found() -> Response {
    error_response(StatusCode::NOT_FOUND, USER_NOT_FOUND)
}

fn server_error(error: &str) -> Response {
    error_response(StatusCode::INTERNAL_SERVER_ERROR, error)
}

/// Express's `{success: true, message}` with no `data` key — the ApiEnvelope
/// success variant always carries data, so these are built raw.
fn message_response(message: &str) -> Response {
    Json(json!({ "success": true, "message": message })).into_response()
}

fn database(state: &AppState, error: &str) -> Result<db::Db, Response> {
    db::Db::from_env(&state.env).map_err(|_| server_error(error))
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

fn bind(db: &db::Db, sql: &str, values: &[Value]) -> Result<db::PreparedStatement, ()> {
    db.prepare(sql).bind(values).map_err(|_| ())
}

async fn all_values(statement: db::PreparedStatement) -> Result<Vec<Value>, ()> {
    let result = statement.all().await.map_err(|_| ())?;
    result.results::<Value>().map_err(|_| ())
}

async fn first_value(statement: db::PreparedStatement) -> Result<Option<Value>, ()> {
    statement.first::<Value>(None).await.map_err(|_| ())
}

fn result_changes(result: &db::QueryResult) -> usize {
    result.meta().changes
}

async fn admin_user(state: &AppState, headers: &HeaderMap) -> Result<UserRow, Response> {
    authenticate(state, headers)
        .await
        .and_then(|user| require_admin(&user).map(|()| user))
}

async fn super_admin_user(state: &AppState, headers: &HeaderMap) -> Result<UserRow, Response> {
    authenticate(state, headers)
        .await
        .and_then(|user| require_super_admin(&user).map(|()| user))
}

/// Express truthiness on query params: empty string means "no filter".
fn text_filter<'a>(params: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    SendFuture::new(list_users_inner(state, headers, params)).await
}

async fn list_users_inner(
    state: AppState,
    headers: HeaderMap,
    params: HashMap<String, String>,
) -> Response {
    if let Err(response) = admin_user(&state, &headers).await {
        return response;
    }

    let page = js_parse_int_or(params.get("page").map(String::as_str), 1);
    let limit = js_parse_int_or(params.get("limit").map(String::as_str), 20);
    let offset = (page - 1) * limit;

    let filters = ListFilters {
        search: text_filter(&params, "search"),
        role: text_filter(&params, "role"),
        membership_tier: text_filter(&params, "membershipTier"),
        verification_status: text_filter(&params, "verificationStatus"),
    };
    let (where_clause, filter_params) = list_where(&filters);
    let filter_binds: Vec<Value> = filter_params
        .iter()
        .map(|param| Val::from_str(param))
        .collect();

    let db = match database(&state, LIST_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };

    let count_sql = format!("{USER_COUNT_SQL} {where_clause}");
    let total = match bind(&db, &count_sql, &filter_binds) {
        Ok(statement) => match statement.first::<CountRow>(None).await {
            Ok(Some(row)) => row.total,
            _ => return server_error(LIST_ERROR),
        },
        Err(()) => return server_error(LIST_ERROR),
    };

    let mut data_binds = filter_binds.clone();
    data_binds.push(Val::from_f64(limit as f64));
    data_binds.push(Val::from_f64(offset as f64));
    let data_sql =
        format!("{USER_LIST_SELECT} {where_clause} ORDER BY created_at DESC LIMIT ? OFFSET ?");
    let rows = match bind(&db, &data_sql, &data_binds) {
        Ok(statement) => match all_values(statement).await {
            Ok(rows) => rows,
            Err(()) => return server_error(LIST_ERROR),
        },
        Err(()) => return server_error(LIST_ERROR),
    };

    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert(
        "data".to_owned(),
        Value::Array(rows.iter().map(user_json).collect()),
    );
    body.insert(
        "pagination".to_owned(),
        pagination_json(page as f64, limit as f64, total),
    );
    Json(Value::Object(body)).into_response()
}

pub async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    SendFuture::new(get_user_inner(state, headers, id)).await
}

async fn get_user_inner(state: AppState, headers: HeaderMap, id: String) -> Response {
    if let Err(response) = admin_user(&state, &headers).await {
        return response;
    }
    let db = match database(&state, GET_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let row = match bind(&db, USER_BY_ID_SQL, &[Val::from_str(&id)]) {
        Ok(statement) => match first_value(statement).await {
            Ok(row) => row,
            Err(()) => return server_error(GET_ERROR),
        },
        Err(()) => return server_error(GET_ERROR),
    };
    match row {
        Some(row) => Json(json!({ "success": true, "data": user_json(&row) })).into_response(),
        None => not_found(),
    }
}

pub async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Json<Value>,
) -> Response {
    SendFuture::new(update_user_inner(state, headers, id, body.0)).await
}

async fn update_user_inner(
    state: AppState,
    headers: HeaderMap,
    id: String,
    body: Value,
) -> Response {
    if let Err(response) = admin_user(&state, &headers).await {
        return response;
    }
    let db = match database(&state, UPDATE_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };

    // Express checks existence first and 404s before touching the update.
    let exists = match bind(&db, USER_EXISTS_SQL, &[Val::from_str(&id)]) {
        Ok(statement) => match first_value(statement).await {
            Ok(row) => row.is_some(),
            Err(()) => return server_error(UPDATE_ERROR),
        },
        Err(()) => return server_error(UPDATE_ERROR),
    };
    if !exists {
        return not_found();
    }

    let Some(assignments) = update_assignments(&body) else {
        return error_response(StatusCode::BAD_REQUEST, "No valid fields to update");
    };

    let mut sets: Vec<String> = assignments
        .iter()
        .map(|(column, _)| format!("{column} = ?"))
        .collect();
    sets.push("updated_at = ?".to_owned());
    let mut binds: Vec<Value> = assignments.iter().map(|(_, value)| to_js(value)).collect();
    binds.push(Val::from_str(&now_iso()));
    binds.push(Val::from_str(&id));

    let sql = format!("UPDATE users SET {} WHERE id = ?", sets.join(", "));
    match bind(&db, &sql, &binds) {
        Ok(statement) if statement.run().await.is_ok() => {
            message_response("User updated successfully")
        }
        _ => server_error(UPDATE_ERROR),
    }
}

pub async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    SendFuture::new(delete_user_inner(state, headers, id)).await
}

async fn delete_user_inner(state: AppState, headers: HeaderMap, id: String) -> Response {
    if let Err(response) = super_admin_user(&state, &headers).await {
        return response;
    }
    let db = match database(&state, DELETE_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };

    let exists = match bind(&db, USER_EXISTS_SQL, &[Val::from_str(&id)]) {
        Ok(statement) => match first_value(statement).await {
            Ok(row) => row.is_some(),
            Err(()) => return server_error(DELETE_ERROR),
        },
        Err(()) => return server_error(DELETE_ERROR),
    };
    if !exists {
        return not_found();
    }

    // Hard delete, exactly like Express.
    match bind(&db, DELETE_USER_SQL, &[Val::from_str(&id)]) {
        Ok(statement) if statement.run().await.is_ok() => {
            message_response("User deleted successfully")
        }
        _ => server_error(DELETE_ERROR),
    }
}

pub async fn verify_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Json<Value>,
) -> Response {
    SendFuture::new(verify_user_inner(state, headers, id, body.0)).await
}

async fn verify_user_inner(
    state: AppState,
    headers: HeaderMap,
    id: String,
    body: Value,
) -> Response {
    if let Err(response) = admin_user(&state, &headers).await {
        return response;
    }

    let status = body.get("status").and_then(Value::as_str).unwrap_or("");
    if !VERIFY_STATUSES.contains(&status) {
        return error_response(StatusCode::BAD_REQUEST, "Invalid verification status");
    }

    let db = match database(&state, VERIFY_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let binds = [
        Val::from_str(status),
        Val::from_bool(status == "approved"),
        Val::from_str(&now_iso()),
        Val::from_str(&id),
    ];
    let statement = match bind(&db, VERIFY_USER_SQL, &binds) {
        Ok(statement) => statement,
        Err(()) => return server_error(VERIFY_ERROR),
    };
    // Express checks `result.changes === 0` instead of a pre-existence query.
    match statement.run().await {
        Ok(result) if result_changes(&result) > 0 => message_response(&verify_message(status)),
        Ok(_) => not_found(),
        Err(_) => server_error(VERIFY_ERROR),
    }
}

pub async fn update_user_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Json<Value>,
) -> Response {
    SendFuture::new(update_user_role_inner(state, headers, id, body.0)).await
}

async fn update_user_role_inner(
    state: AppState,
    headers: HeaderMap,
    id: String,
    body: Value,
) -> Response {
    if let Err(response) = super_admin_user(&state, &headers).await {
        return response;
    }

    let role = body.get("role").and_then(Value::as_str).unwrap_or("");
    if !ROLES.contains(&role) {
        return error_response(StatusCode::BAD_REQUEST, "Invalid role");
    }

    let db = match database(&state, ROLE_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let binds = [
        Val::from_str(role),
        Val::from_str(&now_iso()),
        Val::from_str(&id),
    ];
    let statement = match bind(&db, UPDATE_ROLE_SQL, &binds) {
        Ok(statement) => statement,
        Err(()) => return server_error(ROLE_ERROR),
    };
    match statement.run().await {
        Ok(result) if result_changes(&result) > 0 => {
            message_response("User role updated successfully")
        }
        Ok(_) => not_found(),
        Err(_) => server_error(ROLE_ERROR),
    }
}

pub async fn user_stats_overview(State(state): State<AppState>, headers: HeaderMap) -> Response {
    SendFuture::new(user_stats_overview_inner(state, headers)).await
}

async fn user_stats_overview_inner(state: AppState, headers: HeaderMap) -> Response {
    if let Err(response) = admin_user(&state, &headers).await {
        return response;
    }
    let db = match database(&state, STATS_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };

    // Express runs the five queries via Promise.all; sequential is
    // observably identical here and any failure 500s the whole endpoint.
    let total = match bind(&db, USER_COUNT_SQL, &[]) {
        Ok(statement) => match statement.first::<CountRow>(None).await {
            Ok(Some(row)) => row.total,
            _ => return server_error(STATS_ERROR),
        },
        Err(()) => return server_error(STATS_ERROR),
    };

    let mut groups: Vec<Vec<Value>> = Vec::new();
    for sql in [
        USERS_BY_ROLE_SQL,
        USERS_BY_TIER_SQL,
        USERS_BY_VERIFICATION_SQL,
    ] {
        match bind(&db, sql, &[]) {
            Ok(statement) => match all_values(statement).await {
                Ok(rows) => groups.push(rows),
                Err(()) => return server_error(STATS_ERROR),
            },
            Err(()) => return server_error(STATS_ERROR),
        }
    }

    let recent = match bind(&db, RECENT_REGISTRATIONS_SQL, &[]) {
        Ok(statement) => match first_value(statement).await {
            Ok(Some(row)) => row.get("recent").and_then(Value::as_i64).unwrap_or(0),
            Ok(None) => 0,
            Err(()) => return server_error(STATS_ERROR),
        },
        Err(()) => return server_error(STATS_ERROR),
    };

    let mut groups = groups.into_iter();
    Json(stats_envelope(
        total,
        groups.next().unwrap_or_default(),
        groups.next().unwrap_or_default(),
        groups.next().unwrap_or_default(),
        recent,
    ))
    .into_response()
}

pub async fn database_stats(State(state): State<AppState>, headers: HeaderMap) -> Response {
    SendFuture::new(database_stats_inner(state, headers)).await
}

async fn database_stats_inner(state: AppState, headers: HeaderMap) -> Response {
    if let Err(response) = admin_user(&state, &headers).await {
        return response;
    }
    let db = match database(&state, DB_STATS_ERROR) {
        Ok(db) => db,
        Err(response) => return response,
    };

    // Pinned Express quirk (core::admin docs): the query must succeed — a
    // failure 500s the whole endpoint — but the returned row is never read;
    // the live Express response carries `serverStats: {}`.
    match bind(&db, SERVER_STATE_SQL, &[]) {
        Ok(statement) if statement.all().await.is_ok() => {}
        _ => return server_error(DB_STATS_ERROR),
    }

    // Turso has no schema_migrations table; the schema is applied from sql/schema.sql.
    // The probe fails and we report 'unknown', the Express fallback path.
    let schema_version = match bind(&db, SCHEMA_VERSION_SQL, &[]) {
        Ok(statement) => match first_value(statement).await {
            Ok(Some(row)) => row
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_owned(),
            _ => "unknown".to_owned(),
        },
        Err(()) => "unknown".to_owned(),
    };

    let table_names = match bind(&db, TABLES_SQL, &[]) {
        Ok(statement) => match all_values(statement).await {
            Ok(rows) => rows,
            Err(()) => return server_error(DB_STATS_ERROR),
        },
        Err(()) => return server_error(DB_STATS_ERROR),
    };

    let mut tables: Vec<Value> = Vec::new();
    for name_row in &table_names {
        let Some(name) = name_row.get("name").and_then(Value::as_str) else {
            return server_error(DB_STATS_ERROR);
        };
        let column_count = match bind(&db, COLUMN_COUNT_SQL, &[Val::from_str(name)]) {
            Ok(statement) => match first_value(statement).await {
                Ok(Some(row)) => row.get("column_count").and_then(Value::as_i64).unwrap_or(0),
                _ => 0,
            },
            Err(()) => 0,
        };
        tables.push(table_json(name, column_count));
    }

    Json(database_stats_envelope(
        &schema_version,
        server_stats_json(),
        tables,
        &now_iso(),
    ))
    .into_response()
}
