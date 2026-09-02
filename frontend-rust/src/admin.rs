use serde_json::Value;

use crate::permissions::{AuthSnapshot, RouteGuard, permissions};

pub const HEALTH_API_PATH: &str = "/api/health";
pub const HEALTH_STATUS_API_PATH: &str = "/api/health/status";
pub const DATABASE_STATS_API_PATH: &str = "/api/admin/database/stats";
pub const USER_STATS_API_PATH: &str = "/api/users/stats/overview";
pub const EVENTS_OVERVIEW_API_PATH: &str = "/api/analytics/events/overview";

pub const NETWORK_ERROR: &str = "Network error occurred";
pub const HEALTH_FETCH_FALLBACK: &str = "Failed to load system health";
pub const SYSTEM_FETCH_FALLBACK: &str = "載入系統健康資料失敗";
pub const USER_STATS_FETCH_FALLBACK: &str = "Failed to retrieve user statistics";
pub const DATABASE_STATS_FETCH_FALLBACK: &str = "Failed to get database statistics";
pub const EVENTS_OVERVIEW_FETCH_FALLBACK: &str = "Failed to retrieve event analytics overview";
pub const UNAUTHORIZED_FALLBACK: &str = "Access token required";
pub const ADMIN_ROUTE_FALLBACK: &str = "/login";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverallStatus {
    Healthy,
    Warning,
    Error,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentStatus {
    Healthy,
    Unhealthy,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemTab {
    Overview,
    Detailed,
    Metrics,
    Diagnostics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminPhase {
    Loading,
    Error,
    Ready,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemPhase {
    Loading,
    Error,
    Empty,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct HealthCheck {
    pub success: bool,
    pub message: String,
    pub timestamp: String,
    pub version: String,
    pub database: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct HealthStatus {
    pub success: bool,
    pub status: String,
    pub uptime: Option<f64>,
    pub uptime_formatted: Option<String>,
    pub memory_rss: Option<String>,
    pub memory_heap_used: Option<String>,
    pub memory_heap_total: Option<String>,
    pub node_version: Option<String>,
    pub platform: Option<String>,
    pub database_type: Option<String>,
    pub r2_sync: Option<String>,
    pub environment: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DatabaseTable {
    pub name: String,
    pub column_count: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DatabaseStats {
    pub schema_version: Option<String>,
    pub tables: Vec<DatabaseTable>,
    pub total_tables: Option<i64>,
    pub timestamp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct NamedCount {
    pub key: String,
    pub count: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UserStats {
    pub total_users: Option<i64>,
    pub users_by_role: Vec<NamedCount>,
    pub users_by_membership_tier: Vec<NamedCount>,
    pub users_by_verification_status: Vec<NamedCount>,
    pub recent_registrations: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PopularEvent {
    pub id: String,
    pub name: String,
    pub date_time: Option<String>,
    pub capacity: Option<i64>,
    pub current_attendees: Option<i64>,
    pub occupancy_rate: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventsOverview {
    pub period_days: Option<i64>,
    pub total_events: Option<i64>,
    pub recent_events: Option<i64>,
    pub upcoming_events: Option<i64>,
    pub past_events: Option<i64>,
    pub avg_occupancy_rate: Option<f64>,
    pub total_registrations: Option<i64>,
    pub recent_registrations: Option<i64>,
    pub unique_attendees: Option<i64>,
    pub popular_events: Vec<PopularEvent>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AdminDashboardData {
    pub health: HealthCheck,
    pub status: HealthStatus,
    pub users: UserStats,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SystemHealthBundle {
    pub health: HealthCheck,
    pub status: HealthStatus,
    pub database: DatabaseStats,
    pub users: UserStats,
    pub events: EventsOverview,
}

pub fn admin_route_guard(restoring: bool, snapshot: &AuthSnapshot) -> RouteGuard {
    if restoring {
        RouteGuard::Loading
    } else if !permissions(snapshot).view_admin {
        RouteGuard::Redirect(ADMIN_ROUTE_FALLBACK)
    } else {
        RouteGuard::Allow
    }
}

pub fn welcome_copy(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "管理系統運作並監控平台健康狀況。".to_string()
    } else {
        format!("歡迎回來，{name}。管理系統運作並監控平台健康狀況。")
    }
}

pub fn admin_phase(loading: bool, error: Option<&str>) -> AdminPhase {
    if loading {
        AdminPhase::Loading
    } else if error
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        AdminPhase::Error
    } else {
        AdminPhase::Ready
    }
}

pub fn system_phase(loading: bool, error: Option<&str>, has_status: bool) -> SystemPhase {
    if loading {
        SystemPhase::Loading
    } else if error
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        SystemPhase::Error
    } else if !has_status {
        SystemPhase::Empty
    } else {
        SystemPhase::Ready
    }
}

pub fn overall_status(status: &str) -> OverallStatus {
    match status.trim() {
        "healthy" => OverallStatus::Healthy,
        "warning" => OverallStatus::Warning,
        "error" => OverallStatus::Error,
        _ => OverallStatus::Unknown,
    }
}

pub fn overall_status_label(status: OverallStatus) -> &'static str {
    match status {
        OverallStatus::Healthy => "健康",
        OverallStatus::Warning => "警告",
        OverallStatus::Error => "錯誤",
        OverallStatus::Unknown => "未知",
    }
}

pub fn health_status_color(status: &str) -> &'static str {
    match overall_status(status) {
        OverallStatus::Healthy => "text-green-400",
        OverallStatus::Warning => "text-yellow-400",
        OverallStatus::Error => "text-red-400",
        OverallStatus::Unknown => "text-gray-400",
    }
}

pub fn health_status_bg(status: &str) -> &'static str {
    match overall_status(status) {
        OverallStatus::Healthy => "bg-green-500/20",
        OverallStatus::Warning => "bg-yellow-500/20",
        OverallStatus::Error => "bg-red-500/20",
        OverallStatus::Unknown => "bg-gray-500/20",
    }
}

pub fn health_status_glyph(status: &str) -> &'static str {
    match overall_status(status) {
        OverallStatus::Healthy => "✅",
        OverallStatus::Warning => "⚠️",
        OverallStatus::Error => "❌",
        OverallStatus::Unknown => "❓",
    }
}

pub fn component_status_color(status: ComponentStatus) -> &'static str {
    match status {
        ComponentStatus::Healthy => "text-green-400",
        ComponentStatus::Unhealthy => "text-red-400",
        ComponentStatus::Disabled => "text-yellow-400",
    }
}

pub fn database_status(connected: bool) -> ComponentStatus {
    if connected {
        ComponentStatus::Healthy
    } else {
        ComponentStatus::Unhealthy
    }
}

pub fn r2_status(r2_sync: &str) -> ComponentStatus {
    match r2_sync.trim() {
        "disabled" => ComponentStatus::Disabled,
        "healthy" => ComponentStatus::Healthy,
        _ => ComponentStatus::Unhealthy,
    }
}

pub fn database_connection_label(loading: bool, connected: bool) -> &'static str {
    if loading {
        "載入中..."
    } else if connected {
        "已連線"
    } else {
        "未連線"
    }
}

pub fn r2_status_label(loading: bool, r2_sync: Option<&str>) -> &'static str {
    if loading {
        "載入中..."
    } else {
        match r2_sync.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => match r2_status(value) {
                ComponentStatus::Disabled => "已停用",
                ComponentStatus::Healthy => "正常",
                ComponentStatus::Unhealthy => "連線問題",
            },
            None => "",
        }
    }
}

pub fn format_uptime(seconds: f64) -> String {
    let total = if seconds.is_finite() {
        seconds.floor().max(0.0) as u64
    } else {
        0
    };
    let days = total / 86_400;
    let hours = (total % 86_400) / 3_600;
    let minutes = (total % 3_600) / 60;
    let secs = total % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

pub fn format_bytes(bytes: f64) -> String {
    if !bytes.is_finite() || bytes == 0.0 {
        return "0 B".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB", "TB"];
    let mut index = bytes.abs().log(k).floor() as i32;
    if index < 0 {
        index = 0;
    }
    let index = (index as usize).min(sizes.len() - 1);
    let value = bytes / k.powi(index as i32);
    format!("{} {}", trim_fixed2(value), sizes[index])
}

pub fn format_timestamp(iso: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format_timestamp_js(iso);
    }
    #[cfg(not(target_arch = "wasm32"))]
    iso.to_string()
}

pub fn parse_health_response(status: u16, body: &str) -> Result<HealthCheck, String> {
    let value = parse_root(status, body, HEALTH_FETCH_FALLBACK)?;
    require_success(&value, HEALTH_FETCH_FALLBACK)?;
    Ok(HealthCheck {
        success: true,
        message: json_string(value.get("message")).unwrap_or_default(),
        timestamp: json_string(value.get("timestamp")).unwrap_or_default(),
        version: json_string(value.get("version")).unwrap_or_default(),
        database: json_string(value.get("database")).unwrap_or_default(),
    })
}

pub fn parse_health_status_response(status: u16, body: &str) -> Result<HealthStatus, String> {
    let value = parse_root(status, body, HEALTH_FETCH_FALLBACK)?;
    require_success(&value, HEALTH_FETCH_FALLBACK)?;
    let server = value.get("server");
    let memory = server.and_then(|value| value.get("memory"));
    let database = value.get("database");
    Ok(HealthStatus {
        success: true,
        status: json_string(value.get("status")).unwrap_or_default(),
        uptime: json_f64(server.and_then(|value| value.get("uptime"))),
        uptime_formatted: json_string(server.and_then(|value| value.get("uptimeFormatted"))),
        memory_rss: json_string(memory.and_then(|value| value.get("rss"))),
        memory_heap_used: json_string(memory.and_then(|value| value.get("heapUsed"))),
        memory_heap_total: json_string(memory.and_then(|value| value.get("heapTotal"))),
        node_version: json_string(server.and_then(|value| value.get("nodeVersion"))),
        platform: json_string(server.and_then(|value| value.get("platform"))),
        database_type: json_string(database.and_then(|value| value.get("type"))),
        r2_sync: json_string(database.and_then(|value| value.get("r2Sync"))),
        environment: json_string(value.get("environment")),
        timestamp: json_string(value.get("timestamp")),
    })
}

pub fn parse_database_stats_response(status: u16, body: &str) -> Result<DatabaseStats, String> {
    let data = parse_success_data(status, body, DATABASE_STATS_FETCH_FALLBACK)?;
    let meta = data.get("meta");
    let tables = data
        .get("tables")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_database_table).collect())
        .unwrap_or_default();
    Ok(DatabaseStats {
        schema_version: json_string(data.get("schemaVersion")),
        tables,
        total_tables: json_i64(meta.and_then(|value| value.get("totalTables"))),
        timestamp: json_string(meta.and_then(|value| value.get("timestamp"))),
    })
}

pub fn parse_user_stats_response(status: u16, body: &str) -> Result<UserStats, String> {
    let data = parse_success_data(status, body, USER_STATS_FETCH_FALLBACK)?;
    Ok(UserStats {
        total_users: json_i64(data.get("totalUsers")),
        users_by_role: parse_named_counts(data.get("usersByRole"), "role"),
        users_by_membership_tier: parse_named_counts(
            data.get("usersByMembershipTier"),
            "membership_tier",
        ),
        users_by_verification_status: parse_named_counts(
            data.get("usersByVerificationStatus"),
            "verification_status",
        ),
        recent_registrations: json_i64(data.get("recentRegistrations")),
    })
}

pub fn parse_events_overview_response(status: u16, body: &str) -> Result<EventsOverview, String> {
    let data = parse_success_data(status, body, EVENTS_OVERVIEW_FETCH_FALLBACK)?;
    let event_stats = data.get("event_stats").filter(|value| !value.is_null());
    let registration_stats = data
        .get("registration_stats")
        .filter(|value| !value.is_null());
    let popular_events = data
        .get("popular_events")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_popular_event).collect())
        .unwrap_or_default();
    Ok(EventsOverview {
        period_days: json_i64(data.get("period_days")),
        total_events: json_i64(event_stats.and_then(|value| value.get("total_events"))),
        recent_events: json_i64(event_stats.and_then(|value| value.get("recent_events"))),
        upcoming_events: json_i64(event_stats.and_then(|value| value.get("upcoming_events"))),
        past_events: json_i64(event_stats.and_then(|value| value.get("past_events"))),
        avg_occupancy_rate: json_f64(event_stats.and_then(|value| value.get("avg_occupancy_rate"))),
        total_registrations: json_i64(
            registration_stats.and_then(|value| value.get("total_registrations")),
        ),
        recent_registrations: json_i64(
            registration_stats.and_then(|value| value.get("recent_registrations")),
        ),
        unique_attendees: json_i64(
            registration_stats.and_then(|value| value.get("unique_attendees")),
        ),
        popular_events,
    })
}

pub async fn fetch_health() -> Result<HealthCheck, String> {
    authorized_get(
        HEALTH_API_PATH,
        HEALTH_FETCH_FALLBACK,
        parse_health_response,
    )
    .await
}

pub async fn fetch_health_status() -> Result<HealthStatus, String> {
    authorized_get(
        HEALTH_STATUS_API_PATH,
        HEALTH_FETCH_FALLBACK,
        parse_health_status_response,
    )
    .await
}

pub async fn fetch_database_stats() -> Result<DatabaseStats, String> {
    authorized_get(
        DATABASE_STATS_API_PATH,
        DATABASE_STATS_FETCH_FALLBACK,
        parse_database_stats_response,
    )
    .await
}

pub async fn fetch_user_stats() -> Result<UserStats, String> {
    authorized_get(
        USER_STATS_API_PATH,
        USER_STATS_FETCH_FALLBACK,
        parse_user_stats_response,
    )
    .await
}

pub async fn fetch_events_overview() -> Result<EventsOverview, String> {
    authorized_get(
        EVENTS_OVERVIEW_API_PATH,
        EVENTS_OVERVIEW_FETCH_FALLBACK,
        parse_events_overview_response,
    )
    .await
}

pub async fn fetch_admin_dashboard() -> Result<AdminDashboardData, String> {
    let health = fetch_health().await?;
    let status = fetch_health_status().await?;
    let users = fetch_user_stats().await?;
    Ok(AdminDashboardData {
        health,
        status,
        users,
    })
}

pub async fn fetch_system_health() -> Result<SystemHealthBundle, String> {
    let health = fetch_health().await.map_err(system_fetch_error)?;
    let status = fetch_health_status().await.map_err(system_fetch_error)?;
    let database = fetch_database_stats().await.map_err(system_fetch_error)?;
    let users = fetch_user_stats().await.map_err(system_fetch_error)?;
    let events = fetch_events_overview().await.map_err(system_fetch_error)?;
    Ok(SystemHealthBundle {
        health,
        status,
        database,
        users,
        events,
    })
}

fn system_fetch_error(error: String) -> String {
    if error.trim().is_empty() {
        SYSTEM_FETCH_FALLBACK.to_string()
    } else {
        error
    }
}

async fn authorized_get<T>(
    url: &str,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let token = crate::auth::read_stored_token().ok_or_else(|| NETWORK_ERROR.to_string())?;
        let response = gloo_net::http::Request::get(url)
            .header("Authorization", &crate::auth::bearer_authorization(&token))
            .send()
            .await
            .map_err(|_| NETWORK_ERROR.to_string())?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return parse(status, &text).map_err(|err| {
            if err.trim().is_empty() {
                fallback.to_string()
            } else {
                err
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, fallback, parse);
        Err(NETWORK_ERROR.to_string())
    }
}

fn parse_root(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    if status == 401 {
        return Err(json_error_from_body(body, UNAUTHORIZED_FALLBACK));
    }
    if !(200..300).contains(&status) {
        return Err(json_error_from_body(body, fallback));
    }
    if body.trim().is_empty() {
        return Err(fallback.to_string());
    }
    serde_json::from_str(body).map_err(|_| fallback.to_string())
}

fn parse_success_data(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    let value = parse_root(status, body, fallback)?;
    require_success(&value, fallback)?;
    match value.get("data") {
        Some(Value::Null) | None => Err(fallback.to_string()),
        Some(data) => Ok(data.clone()),
    }
}

fn require_success(value: &Value, fallback: &'static str) -> Result<(), String> {
    if value.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(json_error_message(value, fallback))
    }
}

fn json_error_from_body(body: &str, fallback: &'static str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .map(|value| json_error_message(&value, fallback))
        .unwrap_or_else(|| fallback.to_string())
}

fn json_error_message(value: &Value, fallback: &'static str) -> String {
    value
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|error| !error.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn parse_database_table(value: &Value) -> Option<DatabaseTable> {
    let name = json_string(value.get("name")).filter(|name| !name.is_empty())?;
    Some(DatabaseTable {
        name,
        column_count: json_i64(value.get("columnCount")),
    })
}

fn parse_named_counts(value: Option<&Value>, key: &str) -> Vec<NamedCount> {
    value
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    let name = json_string(row.get(key))?;
                    Some(NamedCount {
                        key: name,
                        count: json_i64(row.get("count")),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_popular_event(value: &Value) -> Option<PopularEvent> {
    let id = json_id(value.get("id"))?;
    Some(PopularEvent {
        id,
        name: json_string(value.get("name")).unwrap_or_default(),
        date_time: json_string(value.get("date_time")),
        capacity: json_i64(value.get("capacity")),
        current_attendees: json_i64(value.get("current_attendees")),
        occupancy_rate: json_f64(value.get("occupancy_rate")),
    })
}

fn json_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn json_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

fn json_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|n| n as i64)),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn trim_fixed2(value: f64) -> String {
    let formatted = format!("{value:.2}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(target_arch = "wasm32")]
fn format_timestamp_js(iso: &str) -> String {
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_str(iso));
    if date.get_time().is_nan() {
        return iso.to_string();
    }
    date.to_locale_string("zh-TW", &wasm_bindgen::JsValue::from(js_sys::Object::new()))
        .into()
}
