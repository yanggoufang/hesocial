#![allow(clippy::result_large_err)]
//! `/api/sales/*` — the sales CRM port of
//! `backend/src/routes/salesManagement.ts` + `salesController.ts`.
//!
//! Express mounts the whole router behind `authenticateToken` (any signed-in
//! user) and adds `requireAdmin` to exactly one route, the lead delete. Every
//! handler keeps the Express envelope, error text, and field whitelist.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use hesocial_core::pagination::{js_parse_f64, pagination_json};
use hesocial_core::sales::{
    ActivityRow, LeadMetricsRow, LeadRow, OpportunityMetricsRow, OpportunityRow, PipelineStageRow,
    TeamMemberRow, activity_insert_is_within_constraints, activity_json, interests_json,
    is_json_column, lead_insert_is_within_constraints, lead_json, lead_score_for,
    lead_update_column, opportunity_insert_is_within_constraints, opportunity_json,
    opportunity_update_column, period_start, pipeline_stage_json, sales_metrics_json,
    team_member_json, update_value_is_allowed,
};
use hesocial_core::{ApiEnvelope, auth::UserRow};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use worker::D1Database;
use worker::js_sys::Date;
use worker::send::SendFuture;
use worker::wasm_bindgen::JsValue;

use crate::AppState;
use crate::auth::{authenticate, internal_error, require_admin};
use crate::auth_handlers::now_iso;

const LEAD_SELECT: &str = "SELECT l.id, l.first_name, l.last_name, l.email, l.phone, l.company, l.job_title, l.annual_income, l.net_worth, l.source, l.referral_code, l.lead_score, l.status, l.interested_membership_tier, l.budget_range, l.timeline, l.pain_points, l.interests, l.notes, l.last_contact_date, l.next_follow_up_date, l.assigned_to, l.created_at, l.updated_at, u.first_name AS assigned_to_first_name, u.last_name AS assigned_to_last_name FROM sales_leads l LEFT JOIN users u ON l.assigned_to = u.id";

const LEAD_COUNT: &str = "SELECT COUNT(*) AS total FROM sales_leads l";

const LEAD_ROW_SQL: &str = "SELECT * FROM sales_leads WHERE id = ?";

const INSERT_LEAD_SQL: &str = "INSERT INTO sales_leads (first_name, last_name, email, phone, company, job_title, annual_income, net_worth, source, referral_code, lead_score, interested_membership_tier, budget_range, timeline, pain_points, interests, notes, assigned_to) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const DELETE_LEAD_SQL: &str = "DELETE FROM sales_leads WHERE id = ?";

const OPPORTUNITY_SELECT: &str = "SELECT o.id, o.lead_id, o.name, o.description, o.stage, o.probability, o.value, o.expected_close_date, o.actual_close_date, o.membership_tier, o.payment_terms, o.close_reason, o.assigned_to, o.created_at, o.updated_at, l.first_name AS lead_first_name, l.last_name AS lead_last_name, l.email AS lead_email, u.first_name AS assigned_to_first_name, u.last_name AS assigned_to_last_name FROM sales_opportunities o LEFT JOIN sales_leads l ON o.lead_id = l.id LEFT JOIN users u ON o.assigned_to = u.id";

const OPPORTUNITY_COUNT: &str = "SELECT COUNT(*) AS total FROM sales_opportunities o";

const OPPORTUNITY_ROW_SQL: &str = "SELECT * FROM sales_opportunities WHERE id = ?";

const INSERT_OPPORTUNITY_SQL: &str = "INSERT INTO sales_opportunities (lead_id, name, description, stage, probability, value, expected_close_date, membership_tier, payment_terms, assigned_to) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const ACTIVITY_SELECT: &str = "SELECT a.id, a.lead_id, a.opportunity_id, a.activity_type, a.subject, a.description, a.outcome, a.duration_minutes, a.scheduled_at, a.completed_at, a.created_by, a.created_at, a.updated_at, u.first_name AS created_by_first_name, u.last_name AS created_by_last_name FROM sales_activities a LEFT JOIN users u ON a.created_by = u.id";

const ACTIVITY_ROW_SQL: &str = "SELECT * FROM sales_activities WHERE id = ?";

const INSERT_ACTIVITY_SQL: &str = "INSERT INTO sales_activities (lead_id, opportunity_id, activity_type, subject, description, outcome, duration_minutes, scheduled_at, completed_at, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

const LEAD_METRICS_SQL: &str = "SELECT COUNT(*) AS total_leads, COUNT(CASE WHEN status IN ('qualified', 'contacted', 'nurturing') THEN 1 END) AS qualified_leads, COUNT(CASE WHEN status = 'closed_won' THEN 1 END) AS converted_leads FROM sales_leads";

const OPPORTUNITY_METRICS_SQL: &str = "SELECT COUNT(*) AS total_opportunities, SUM(value) AS total_pipeline_value, AVG(value) AS average_deal_size, COUNT(CASE WHEN stage = 'closed_won' THEN 1 END) AS won_opportunities, SUM(CASE WHEN stage = 'closed_won' THEN value ELSE 0 END) AS won_revenue FROM sales_opportunities";

const PIPELINE_STAGES_SQL: &str = "SELECT id, name, description, display_order, default_probability, is_active, color_code, created_at, updated_at FROM sales_pipeline_stages WHERE is_active = 1 ORDER BY display_order";

const SALES_TEAM_SQL: &str = "SELECT st.id, st.user_id, st.role, st.territory, st.commission_rate, st.quota_amount, st.is_active, st.hire_date, st.manager_id, st.created_at, st.updated_at, u.first_name, u.last_name, u.email, m.first_name AS manager_first_name, m.last_name AS manager_last_name FROM sales_team_members st LEFT JOIN users u ON st.user_id = u.id LEFT JOIN users m ON st.manager_id = m.id WHERE st.is_active = 1 ORDER BY st.role, u.first_name";

#[derive(Deserialize)]
struct CountRow {
    total: i64,
}

fn json_error(status: StatusCode, error: &str) -> Response {
    (status, Json(ApiEnvelope::<Value>::error(error))).into_response()
}

fn not_found(error: &str) -> Response {
    json_error(StatusCode::NOT_FOUND, error)
}

fn created(data: Value, message: &str) -> Response {
    (
        StatusCode::CREATED,
        Json(ApiEnvelope::success_with_message(data, message)),
    )
        .into_response()
}

fn data_response(data: Value, message: Option<&str>) -> Response {
    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert("data".to_owned(), data);
    if let Some(message) = message {
        body.insert("message".to_owned(), json!(message));
    }
    Json(Value::Object(body)).into_response()
}

/// Express hands back the raw rows; `pagination` is only present on the two
/// paginated list routes.
fn list_response(rows: Vec<Value>, pagination: Option<Value>) -> Response {
    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert("data".to_owned(), Value::Array(rows));
    if let Some(pagination) = pagination {
        body.insert("pagination".to_owned(), pagination);
    }
    Json(Value::Object(body)).into_response()
}

/// SQLite binds a numeric `id` against an INTEGER PRIMARY KEY; a raw path
/// string would never match, so parse what parses and pass the text through.
fn id_bind(id: &str) -> JsValue {
    match id.parse::<f64>().ok().filter(|value| value.is_finite()) {
        Some(number) => JsValue::from_f64(number),
        None => JsValue::from_str(id),
    }
}

fn to_js(value: &Value) -> JsValue {
    match value {
        Value::Null => JsValue::NULL,
        Value::Bool(flag) => JsValue::from_bool(*flag),
        Value::Number(number) => number
            .as_f64()
            .map(JsValue::from_f64)
            .unwrap_or(JsValue::NULL),
        Value::String(text) => JsValue::from_str(text),
        _ => JsValue::NULL,
    }
}

/// Express binds `req.body.field` directly, so an absent key is `undefined` and
/// the driver stores NULL.
fn raw_field(body: &Value, key: &str) -> JsValue {
    body.get(key).map_or(JsValue::NULL, to_js)
}

fn bind_statement(
    db: &D1Database,
    sql: &str,
    values: &[JsValue],
) -> Result<worker::D1PreparedStatement, ()> {
    db.prepare(sql).bind(values).map_err(|_| ())
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

fn database(state: &AppState, error: &str) -> Result<D1Database, Response> {
    state.env.d1("DB").map_err(|_| internal_error(error))
}

async fn all_rows<T>(statement: worker::D1PreparedStatement) -> Result<Vec<T>, ()>
where
    for<'de> T: Deserialize<'de>,
{
    let result = statement.all().await.map_err(|_| ())?;
    result.results::<T>().map_err(|_| ())
}

async fn first_row<T>(statement: worker::D1PreparedStatement) -> Result<Option<T>, ()>
where
    for<'de> T: Deserialize<'de>,
{
    statement.first(None).await.map_err(|_| ())
}

fn query_number(params: &HashMap<String, String>, key: &str, default: f64) -> Option<f64> {
    match params.get(key) {
        None => Some(default),
        Some(raw) => js_parse_f64(raw),
    }
}

/// Express tests each filter with a bare `if (value)`, so an empty query-string
/// value is skipped and everything else is bound verbatim.
fn text_filter<'a>(params: &'a HashMap<String, String>, key: &str) -> Option<&'a String> {
    params.get(key).filter(|value| !value.is_empty())
}

/// `page`/`limit` come off the query string unparsed. Express hands the driver
/// a NaN and the route answers with its 500 envelope, so the worker bails the
/// same way before touching D1.
fn page_and_limit(
    params: &HashMap<String, String>,
    error: &str,
) -> Result<(f64, f64, JsValue, JsValue), Response> {
    let page = query_number(params, "page", 1.0).ok_or_else(|| internal_error(error))?;
    let limit = query_number(params, "limit", 20.0).ok_or_else(|| internal_error(error))?;
    Ok((
        page,
        limit,
        JsValue::from_f64(limit),
        JsValue::from_f64((page - 1.0) * limit),
    ))
}

fn scalar_bind(column: &str, value: &Value) -> JsValue {
    if is_json_column(column) {
        return match value {
            Value::Null => JsValue::NULL,
            Value::String(text) => JsValue::from_str(text),
            other => JsValue::from_str(&other.to_string()),
        };
    }
    to_js(value)
}

/// Express interpolates every defined body key straight into the `SET` clause.
/// The worker resolves keys through the column map, drops unknown keys, and
/// pre-empts the D1 CHECK constraints with the 500 envelope a failed write
/// would have produced. `updated_at` is always refreshed, exactly like Express.
fn build_update(
    body: &Value,
    resolve: fn(&str) -> Option<&'static str>,
    failure_message: &str,
    timestamp: &str,
) -> Result<(Vec<String>, Vec<JsValue>), Response> {
    let mut assignments = vec!["updated_at = ?".to_owned()];
    let mut binds = vec![JsValue::from_str(timestamp)];

    let Some(fields) = body.as_object() else {
        return Ok((assignments, binds));
    };
    for (key, value) in fields {
        let Some(column) = resolve(key) else {
            continue;
        };
        if assignments
            .iter()
            .any(|set| set.split(" = ").next() == Some(column))
        {
            continue;
        }
        if !update_value_is_allowed(column, value) {
            return Err(internal_error(failure_message));
        }
        assignments.push(format!("{column} = ?"));
        binds.push(scalar_bind(column, value));
    }

    Ok((assignments, binds))
}

async fn fetch_row<T>(db: &D1Database, sql: &str, bind: JsValue) -> Result<Option<T>, ()>
where
    for<'de> T: Deserialize<'de>,
{
    let statement = bind_statement(db, sql, &[bind])?;
    first_row::<T>(statement).await
}

pub async fn list_leads(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(list_leads_inner(state, params)).await
}

async fn list_leads_inner(state: AppState, params: HashMap<String, String>) -> Response {
    let error = "Failed to fetch sales leads";
    let (page, limit, limit_bind, offset_bind) = match page_and_limit(&params, error) {
        Ok(values) => values,
        Err(response) => return response,
    };

    let mut conditions = vec!["1=1".to_owned()];
    let mut binds: Vec<JsValue> = Vec::new();

    if let Some(status) = text_filter(&params, "status") {
        conditions.push("l.status = ?".to_owned());
        binds.push(JsValue::from_str(status));
    }
    if let Some(source) = text_filter(&params, "source") {
        conditions.push("l.source = ?".to_owned());
        binds.push(JsValue::from_str(source));
    }
    if let Some(assigned_to) = text_filter(&params, "assignedTo") {
        conditions.push("l.assigned_to = ?".to_owned());
        binds.push(JsValue::from_str(assigned_to));
    }
    if let Some(tier) = text_filter(&params, "membershipTier") {
        conditions.push("l.interested_membership_tier = ?".to_owned());
        binds.push(JsValue::from_str(tier));
    }
    if let Some(search) = text_filter(&params, "search") {
        conditions.push(
            "(l.first_name LIKE ? OR l.last_name LIKE ? OR l.email LIKE ? OR l.company LIKE ?)"
                .to_owned(),
        );
        let pattern = JsValue::from_str(&format!("%{search}%"));
        for _ in 0..4 {
            binds.push(pattern.clone());
        }
    }

    let where_clause = format!("WHERE {}", conditions.join(" AND "));
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };

    let mut data_binds = binds.clone();
    data_binds.push(limit_bind);
    data_binds.push(offset_bind);

    let data_query = bind_statement(
        &db,
        &format!("{LEAD_SELECT} {where_clause} ORDER BY l.created_at DESC LIMIT ? OFFSET ?"),
        &data_binds,
    );
    let count_query = bind_statement(&db, &format!("{LEAD_COUNT} {where_clause}"), &binds);
    let (Ok(data_query), Ok(count_query)) = (data_query, count_query) else {
        return internal_error(error);
    };

    let batch = match db.batch(vec![data_query, count_query]).await {
        Ok(batch) => batch,
        Err(_) => return internal_error(error),
    };
    let (Some(data_result), Some(count_result)) = (batch.first(), batch.get(1)) else {
        return internal_error(error);
    };
    let rows = match data_result.results::<LeadRow>() {
        Ok(rows) => rows,
        Err(_) => return internal_error(error),
    };
    let counts = match count_result.results::<CountRow>() {
        Ok(rows) => rows,
        Err(_) => return internal_error(error),
    };

    list_response(
        rows.iter().map(lead_json).collect(),
        Some(pagination_json(
            page,
            limit,
            counts.first().map_or(0, |row| row.total),
        )),
    )
}

pub async fn get_lead(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(get_lead_inner(state, id)).await
}

async fn get_lead_inner(state: AppState, id: String) -> Response {
    let error = "Failed to fetch lead";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let query = bind_statement(
        &db,
        &format!("{LEAD_SELECT} WHERE l.id = ?"),
        &[id_bind(&id)],
    );
    let Ok(query) = query else {
        return internal_error(error);
    };
    match first_row::<LeadRow>(query).await {
        Ok(Some(row)) => data_response(lead_json(&row), None),
        Ok(None) => not_found("Lead not found"),
        Err(_) => internal_error(error),
    }
}

pub async fn create_lead(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(create_lead_inner(state, body)).await
}

async fn create_lead_inner(state: AppState, body: Value) -> Response {
    let error = "Failed to create lead";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };

    if !lead_insert_is_within_constraints(&body) {
        return internal_error(error);
    }

    let score = lead_score_for(&body);
    let interests = JsValue::from_str(&interests_json(&body));
    let binds = [
        raw_field(&body, "firstName"),
        raw_field(&body, "lastName"),
        raw_field(&body, "email"),
        raw_field(&body, "phone"),
        raw_field(&body, "company"),
        raw_field(&body, "jobTitle"),
        raw_field(&body, "annualIncome"),
        raw_field(&body, "netWorth"),
        raw_field(&body, "source"),
        raw_field(&body, "referralCode"),
        JsValue::from_f64(score as f64),
        raw_field(&body, "interestedMembershipTier"),
        raw_field(&body, "budgetRange"),
        raw_field(&body, "timeline"),
        raw_field(&body, "painPoints"),
        interests,
        raw_field(&body, "notes"),
        raw_field(&body, "assignedTo"),
    ];

    let insert = match bind_statement(&db, INSERT_LEAD_SQL, &binds) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    let inserted = match insert.run().await {
        Ok(result) => result,
        Err(_) => return internal_error(error),
    };
    let Some(row_id) = result_last_row_id(&inserted) else {
        return internal_error(error);
    };

    let row = fetch_row::<LeadRow>(&db, LEAD_ROW_SQL, JsValue::from_f64(row_id as f64)).await;
    match row {
        Ok(Some(row)) => created(lead_json(&row), "Lead created successfully"),
        _ => internal_error(error),
    }
}

pub async fn update_lead(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(update_lead_inner(state, id, body)).await
}

async fn update_lead_inner(state: AppState, id: String, body: Value) -> Response {
    let error = "Failed to update lead";
    let timestamp = now_iso();
    let (assignments, mut binds) = match build_update(&body, lead_update_column, error, &timestamp)
    {
        Ok(update) => update,
        Err(response) => return response,
    };
    binds.push(id_bind(&id));

    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let sql = format!(
        "UPDATE sales_leads SET {} WHERE id = ?",
        assignments.join(", ")
    );
    let update = match bind_statement(&db, &sql, &binds) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    let updated = match update.run().await {
        Ok(result) => result,
        Err(_) => return internal_error(error),
    };
    if result_changes(&updated) == 0 {
        return not_found("Lead not found");
    }

    let row = fetch_row::<LeadRow>(&db, LEAD_ROW_SQL, id_bind(&id)).await;
    match row {
        Ok(Some(row)) => data_response(lead_json(&row), Some("Lead updated successfully")),
        _ => not_found("Lead not found"),
    }
}

pub async fn delete_lead(
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
    SendFuture::new(delete_lead_inner(state, id)).await
}

async fn delete_lead_inner(state: AppState, id: String) -> Response {
    let error = "Failed to delete lead";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let statement = match bind_statement(&db, DELETE_LEAD_SQL, &[id_bind(&id)]) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    // Express branches on `result.rowCount`, which its DuckDB adapter never
    // populates, so a delete that matches nothing still answers 200. Pinned.
    if statement.run().await.is_err() {
        return internal_error(error);
    }

    Json(json!({
        "success": true,
        "message": "Lead deleted successfully",
    }))
    .into_response()
}

pub async fn list_opportunities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(list_opportunities_inner(state, params)).await
}

async fn list_opportunities_inner(state: AppState, params: HashMap<String, String>) -> Response {
    let error = "Failed to fetch sales opportunities";
    let (page, limit, limit_bind, offset_bind) = match page_and_limit(&params, error) {
        Ok(values) => values,
        Err(response) => return response,
    };

    let mut conditions = vec!["1=1".to_owned()];
    let mut binds: Vec<JsValue> = Vec::new();
    if let Some(stage) = text_filter(&params, "stage") {
        conditions.push("o.stage = ?".to_owned());
        binds.push(JsValue::from_str(stage));
    }
    if let Some(assigned_to) = text_filter(&params, "assignedTo") {
        conditions.push("o.assigned_to = ?".to_owned());
        binds.push(JsValue::from_str(assigned_to));
    }
    if let Some(tier) = text_filter(&params, "membershipTier") {
        conditions.push("o.membership_tier = ?".to_owned());
        binds.push(JsValue::from_str(tier));
    }

    let where_clause = format!("WHERE {}", conditions.join(" AND "));
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };

    let mut data_binds = binds.clone();
    data_binds.push(limit_bind);
    data_binds.push(offset_bind);
    let data_query = bind_statement(
        &db,
        &format!("{OPPORTUNITY_SELECT} {where_clause} ORDER BY o.created_at DESC LIMIT ? OFFSET ?"),
        &data_binds,
    );
    let count_query = bind_statement(&db, &format!("{OPPORTUNITY_COUNT} {where_clause}"), &binds);
    let (Ok(data_query), Ok(count_query)) = (data_query, count_query) else {
        return internal_error(error);
    };

    let batch = match db.batch(vec![data_query, count_query]).await {
        Ok(batch) => batch,
        Err(_) => return internal_error(error),
    };
    let (Some(data_result), Some(count_result)) = (batch.first(), batch.get(1)) else {
        return internal_error(error);
    };
    let rows = match data_result.results::<OpportunityRow>() {
        Ok(rows) => rows,
        Err(_) => return internal_error(error),
    };
    let counts = match count_result.results::<CountRow>() {
        Ok(rows) => rows,
        Err(_) => return internal_error(error),
    };

    list_response(
        rows.iter().map(opportunity_json).collect(),
        Some(pagination_json(
            page,
            limit,
            counts.first().map_or(0, |row| row.total),
        )),
    )
}

pub async fn create_opportunity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(create_opportunity_inner(state, body)).await
}

async fn create_opportunity_inner(state: AppState, body: Value) -> Response {
    let error = "Failed to create opportunity";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };

    if !opportunity_insert_is_within_constraints(&body) {
        return internal_error(error);
    }

    let binds = [
        raw_field(&body, "leadId"),
        raw_field(&body, "name"),
        raw_field(&body, "description"),
        raw_field(&body, "stage"),
        raw_field(&body, "probability"),
        raw_field(&body, "value"),
        raw_field(&body, "expectedCloseDate"),
        raw_field(&body, "membershipTier"),
        raw_field(&body, "paymentTerms"),
        raw_field(&body, "assignedTo"),
    ];
    let insert = match bind_statement(&db, INSERT_OPPORTUNITY_SQL, &binds) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    let inserted = match insert.run().await {
        Ok(result) => result,
        Err(_) => return internal_error(error),
    };
    let Some(row_id) = result_last_row_id(&inserted) else {
        return internal_error(error);
    };

    let row =
        fetch_row::<OpportunityRow>(&db, OPPORTUNITY_ROW_SQL, JsValue::from_f64(row_id as f64))
            .await;
    match row {
        Ok(Some(row)) => created(opportunity_json(&row), "Opportunity created successfully"),
        _ => internal_error(error),
    }
}

pub async fn update_opportunity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(update_opportunity_inner(state, id, body)).await
}

async fn update_opportunity_inner(state: AppState, id: String, body: Value) -> Response {
    let error = "Failed to update opportunity";
    let timestamp = now_iso();
    let (assignments, mut binds) =
        match build_update(&body, opportunity_update_column, error, &timestamp) {
            Ok(update) => update,
            Err(response) => return response,
        };
    binds.push(id_bind(&id));

    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let sql = format!(
        "UPDATE sales_opportunities SET {} WHERE id = ?",
        assignments.join(", ")
    );
    let update = match bind_statement(&db, &sql, &binds) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    let updated = match update.run().await {
        Ok(result) => result,
        Err(_) => return internal_error(error),
    };
    if result_changes(&updated) == 0 {
        return not_found("Opportunity not found");
    }

    let row = fetch_row::<OpportunityRow>(&db, OPPORTUNITY_ROW_SQL, id_bind(&id)).await;
    match row {
        Ok(Some(row)) => data_response(
            opportunity_json(&row),
            Some("Opportunity updated successfully"),
        ),
        _ => not_found("Opportunity not found"),
    }
}

pub async fn list_activities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(list_activities_inner(state, params)).await
}

async fn list_activities_inner(state: AppState, params: HashMap<String, String>) -> Response {
    let error = "Failed to fetch sales activities";
    let (_page, _limit, limit_bind, offset_bind) = match page_and_limit(&params, error) {
        Ok(values) => values,
        Err(response) => return response,
    };

    let mut conditions = vec!["1=1".to_owned()];
    let mut binds: Vec<JsValue> = Vec::new();
    if let Some(lead_id) = text_filter(&params, "leadId") {
        conditions.push("a.lead_id = ?".to_owned());
        binds.push(id_bind(lead_id));
    }
    if let Some(opportunity_id) = text_filter(&params, "opportunityId") {
        conditions.push("a.opportunity_id = ?".to_owned());
        binds.push(id_bind(opportunity_id));
    }
    binds.push(limit_bind);
    binds.push(offset_bind);

    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let query = bind_statement(
        &db,
        &format!(
            "{ACTIVITY_SELECT} WHERE {} ORDER BY a.created_at DESC LIMIT ? OFFSET ?",
            conditions.join(" AND ")
        ),
        &binds,
    );
    let Ok(query) = query else {
        return internal_error(error);
    };
    match all_rows::<ActivityRow>(query).await {
        // This list route has no pagination block in the Express envelope.
        Ok(rows) => list_response(rows.iter().map(activity_json).collect(), None),
        Err(_) => internal_error(error),
    }
}

pub async fn create_activity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let user = match SendFuture::new(authenticate(&state, &headers)).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    SendFuture::new(create_activity_inner(state, user, body)).await
}

async fn create_activity_inner(state: AppState, user: UserRow, body: Value) -> Response {
    let error = "Failed to create activity";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };

    if !activity_insert_is_within_constraints(&body) {
        return internal_error(error);
    }

    let binds = [
        raw_field(&body, "leadId"),
        raw_field(&body, "opportunityId"),
        raw_field(&body, "activityType"),
        raw_field(&body, "subject"),
        raw_field(&body, "description"),
        raw_field(&body, "outcome"),
        raw_field(&body, "durationMinutes"),
        raw_field(&body, "scheduledAt"),
        raw_field(&body, "completedAt"),
        JsValue::from_str(&user.id),
    ];
    let insert = match bind_statement(&db, INSERT_ACTIVITY_SQL, &binds) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    let inserted = match insert.run().await {
        Ok(result) => result,
        Err(_) => return internal_error(error),
    };
    let Some(row_id) = result_last_row_id(&inserted) else {
        return internal_error(error);
    };

    let row =
        fetch_row::<ActivityRow>(&db, ACTIVITY_ROW_SQL, JsValue::from_f64(row_id as f64)).await;
    match row {
        Ok(Some(row)) => created(activity_json(&row), "Activity created successfully"),
        _ => internal_error(error),
    }
}

pub async fn get_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(get_metrics_inner(state, params)).await
}

async fn get_metrics_inner(state: AppState, params: HashMap<String, String>) -> Response {
    let error = "Failed to fetch sales metrics";

    // Express destructures `{ period = 'monthly' }`, so an empty value keeps
    // the filter off entirely rather than falling back to the default.
    let period = params
        .get("period")
        .map(String::as_str)
        .unwrap_or("monthly");
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<JsValue> = Vec::new();
    if let Some(start) = period_start(period, Date::now()) {
        conditions.push("created_at >= ?".to_owned());
        binds.push(JsValue::from_str(&start));
    }
    if let Some(sales_rep) = text_filter(&params, "salesRepId") {
        conditions.push("assigned_to = ?".to_owned());
        binds.push(id_bind(sales_rep));
    }
    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let leads_query = bind_statement(&db, &format!("{LEAD_METRICS_SQL} {where_clause}"), &binds);
    let opportunities_query = bind_statement(
        &db,
        &format!("{OPPORTUNITY_METRICS_SQL} {where_clause}"),
        &binds,
    );
    let (Ok(leads_query), Ok(opportunities_query)) = (leads_query, opportunities_query) else {
        return internal_error(error);
    };

    let batch = match db.batch(vec![leads_query, opportunities_query]).await {
        Ok(batch) => batch,
        Err(_) => return internal_error(error),
    };
    let (Some(lead_result), Some(opportunity_result)) = (batch.first(), batch.get(1)) else {
        return internal_error(error);
    };
    let leads = match lead_result.results::<LeadMetricsRow>() {
        Ok(rows) => rows.first().cloned().unwrap_or_default(),
        Err(_) => return internal_error(error),
    };
    let opportunities = match opportunity_result.results::<OpportunityMetricsRow>() {
        Ok(rows) => rows.first().cloned().unwrap_or_default(),
        Err(_) => return internal_error(error),
    };

    data_response(sales_metrics_json(&leads, &opportunities), None)
}

pub async fn get_pipeline_stages(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(get_pipeline_stages_inner(state)).await
}

async fn get_pipeline_stages_inner(state: AppState) -> Response {
    let error = "Failed to fetch pipeline stages";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let statement = match bind_statement(&db, PIPELINE_STAGES_SQL, &[]) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    match all_rows::<PipelineStageRow>(statement).await {
        Ok(rows) => list_response(rows.iter().map(pipeline_stage_json).collect(), None),
        Err(_) => internal_error(error),
    }
}

pub async fn get_sales_team(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = SendFuture::new(authenticate(&state, &headers)).await {
        return response;
    }
    SendFuture::new(get_sales_team_inner(state)).await
}

async fn get_sales_team_inner(state: AppState) -> Response {
    let error = "Failed to fetch sales team";
    let db = match database(&state, error) {
        Ok(db) => db,
        Err(response) => return response,
    };
    let statement = match bind_statement(&db, SALES_TEAM_SQL, &[]) {
        Ok(statement) => statement,
        Err(_) => return internal_error(error),
    };
    match all_rows::<TeamMemberRow>(statement).await {
        Ok(rows) => list_response(rows.iter().map(team_member_json).collect(), None),
        Err(_) => internal_error(error),
    }
}
