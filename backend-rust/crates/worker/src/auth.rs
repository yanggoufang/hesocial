#![allow(clippy::result_large_err)]
use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::auth::{LOGIN_USER_SELECT, USER_SELECT_BY_ID_ALIVE, UserRow, user_select};
use hesocial_core::{ApiEnvelope, JWT_EXPIRY_SECONDS, parse_jwt_expiry, verify_jwt};
use serde_json::Value;
use worker::js_sys::Date;
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;

use crate::AppState;

pub struct QueryUnavailable;

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(ApiEnvelope::<Value>::error(error))).into_response()
}

pub fn unauthorized(error: &str) -> Response {
    json_error(StatusCode::UNAUTHORIZED, error)
}

pub fn forbidden(error: &str) -> Response {
    json_error(StatusCode::FORBIDDEN, error)
}

pub fn internal_error(error: &str) -> Response {
    json_error(StatusCode::INTERNAL_SERVER_ERROR, error)
}

pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    header.split(' ').nth(1).filter(|token| !token.is_empty())
}

fn env_value(state: &AppState, name: &str) -> Option<String> {
    state.env.var(name).ok().map(|value| value.to_string())
}

pub fn jwt_secret(state: &AppState) -> Option<String> {
    env_value(state, "JWT_SECRET")
}

pub fn jwt_expiry_seconds(state: &AppState) -> u64 {
    env_value(state, "JWT_EXPIRES_IN")
        .and_then(|raw| parse_jwt_expiry(&raw))
        .unwrap_or(JWT_EXPIRY_SECONDS)
}

pub fn now_seconds() -> u64 {
    (Date::now() / 1000.0).floor() as u64
}

async fn query_user(
    state: &AppState,
    statement: &str,
    bind: JsValue,
) -> Result<Option<UserRow>, QueryUnavailable> {
    let db = state.env.d1("DB").map_err(|_| QueryUnavailable)?;
    let query = db
        .prepare(statement.to_owned())
        .bind(&[bind])
        .map_err(|_| QueryUnavailable)?;
    let rows = query.all().await.map_err(|_| QueryUnavailable)?;
    Ok(rows
        .results::<UserRow>()
        .map_err(|_| QueryUnavailable)?
        .into_iter()
        .next())
}

pub async fn authenticate(state: &AppState, headers: &HeaderMap) -> Result<UserRow, Response> {
    let Some(token) = bearer_token(headers) else {
        return Err(unauthorized("Access token required"));
    };
    let Some(secret) = jwt_secret(state) else {
        return Err(internal_error("Authentication failed"));
    };

    let claims = match verify_jwt(token, &secret, now_seconds()) {
        Ok(claims) => claims,
        Err(_) => return Err(unauthorized("Invalid token")),
    };

    let user = SendFuture::new(query_user(
        state,
        &user_select(USER_SELECT_BY_ID_ALIVE),
        JsValue::from_str(&claims.user_id),
    ))
    .await;

    match user {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(unauthorized("Invalid token - user not found")),
        Err(QueryUnavailable) => Err(internal_error("Authentication failed")),
    }
}

pub fn require_admin(user: &UserRow) -> Result<(), Response> {
    if user.role == "admin" || user.role == "super_admin" {
        Ok(())
    } else {
        Err(forbidden("Admin access required"))
    }
}

pub fn require_super_admin(user: &UserRow) -> Result<(), Response> {
    if user.role == "super_admin" {
        Ok(())
    } else {
        Err(forbidden("Super admin access required"))
    }
}

pub async fn find_user_by_email(
    state: &AppState,
    email: JsValue,
) -> Result<Option<UserRow>, QueryUnavailable> {
    SendFuture::new(query_user(state, &user_select(LOGIN_USER_SELECT), email)).await
}
