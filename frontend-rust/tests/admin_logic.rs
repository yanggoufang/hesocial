#![cfg(not(target_arch = "wasm32"))]

use dioxus::prelude::*;
use hesocial_frontend::admin::{
    ADMIN_ROUTE_FALLBACK, AdminPhase, ComponentStatus, DATABASE_STATS_API_PATH,
    EVENTS_OVERVIEW_API_PATH, HEALTH_API_PATH, HEALTH_FETCH_FALLBACK, HEALTH_STATUS_API_PATH,
    NETWORK_ERROR, OverallStatus, SystemPhase, SystemTab, UNAUTHORIZED_FALLBACK,
    USER_STATS_API_PATH, admin_phase, admin_route_guard, component_status_color,
    database_connection_label, database_status, fetch_admin_dashboard, format_bytes, format_uptime,
    health_status_bg, health_status_color, health_status_glyph, overall_status,
    overall_status_label, parse_database_stats_response, parse_events_overview_response,
    parse_health_response, parse_health_status_response, parse_user_stats_response, r2_status,
    r2_status_label, system_phase, welcome_copy,
};
use hesocial_frontend::pages::admin::{AdminScreen, AdminSystemScreen};
use hesocial_frontend::permissions::{AuthSnapshot, Role, RouteGuard};

fn health_ok_body() -> &'static str {
    r#"{
        "success": true,
        "message": "API health check passed (Turso)",
        "timestamp": "2026-09-02T00:00:00.000Z",
        "version": "1.0.0",
        "database": "turso"
    }"#
}

fn health_status_ok_body() -> &'static str {
    r#"{
        "success": true,
        "status": "healthy",
        "server": {
            "uptime": 3661,
            "uptimeFormatted": "1h 1m 1s",
            "memory": {
                "rss": "0MB",
                "heapUsed": "0MB",
                "heapTotal": "0MB"
            },
            "nodeVersion": null,
            "platform": null
        },
        "database": {
            "type": "Turso",
            "r2Sync": "disabled"
        },
        "environment": "development",
        "timestamp": "2026-09-02T00:00:00.000Z"
    }"#
}

fn database_stats_ok_body() -> &'static str {
    r#"{
        "success": true,
        "data": {
            "schemaVersion": "unknown",
            "serverStats": {},
            "tables": [
                { "name": "users", "columnCount": 12 },
                { "name": "events", "columnCount": 8 }
            ],
            "meta": { "totalTables": 2, "timestamp": "2026-09-02T00:00:00.000Z" }
        }
    }"#
}

fn user_stats_ok_body() -> &'static str {
    r#"{
        "success": true,
        "data": {
            "totalUsers": 7,
            "usersByRole": [{ "role": "admin", "count": 1 }],
            "usersByMembershipTier": [{ "membership_tier": "Diamond", "count": 3 }],
            "usersByVerificationStatus": [{ "verification_status": "approved", "count": 6 }],
            "recentRegistrations": 2
        }
    }"#
}

fn events_overview_ok_body() -> &'static str {
    r#"{
        "success": true,
        "data": {
            "period_days": 30,
            "event_stats": {
                "total_events": 4,
                "recent_events": 1,
                "upcoming_events": 1,
                "past_events": 3,
                "avg_occupancy_rate": 50.5
            },
            "registration_stats": {
                "total_registrations": 9,
                "recent_registrations": 9,
                "unique_attendees": 5
            },
            "popular_events": [
                {
                    "id": 1,
                    "name": "Gala",
                    "date_time": "2026-10-01T18:00:00.000Z",
                    "capacity": 20,
                    "current_attendees": 10,
                    "occupancy_rate": 50.0
                }
            ]
        }
    }"#
}

fn unauthorized_body() -> &'static str {
    r#"{"success":false,"error":"Access token required"}"#
}

fn snapshot(authenticated: bool, role: Option<Role>) -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated: authenticated,
        role,
        ..AuthSnapshot::default()
    }
}

#[component]
fn AdminAt(
    loading: bool,
    error: Option<String>,
    first_name: String,
    health: Option<hesocial_frontend::admin::HealthCheck>,
    status: Option<hesocial_frontend::admin::HealthStatus>,
    users: Option<hesocial_frontend::admin::UserStats>,
) -> Element {
    rsx! {
        AdminScreen {
            loading,
            error,
            first_name,
            health,
            status,
            users,
        }
    }
}

fn render_admin(
    loading: bool,
    error: Option<String>,
    first_name: &str,
    health: Option<hesocial_frontend::admin::HealthCheck>,
    status: Option<hesocial_frontend::admin::HealthStatus>,
    users: Option<hesocial_frontend::admin::UserStats>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        AdminAt,
        AdminAtProps {
            loading,
            error,
            first_name: first_name.to_string(),
            health,
            status,
            users,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[component]
fn SystemAt(
    loading: bool,
    error: Option<String>,
    health: Option<hesocial_frontend::admin::HealthCheck>,
    status: Option<hesocial_frontend::admin::HealthStatus>,
    database: Option<hesocial_frontend::admin::DatabaseStats>,
    users: Option<hesocial_frontend::admin::UserStats>,
    events: Option<hesocial_frontend::admin::EventsOverview>,
    tab: SystemTab,
) -> Element {
    rsx! {
        AdminSystemScreen {
            loading,
            refreshing: false,
            error,
            health,
            status,
            database,
            users,
            events,
            tab,
        }
    }
}

fn render_system(
    loading: bool,
    error: Option<String>,
    health: Option<hesocial_frontend::admin::HealthCheck>,
    status: Option<hesocial_frontend::admin::HealthStatus>,
    database: Option<hesocial_frontend::admin::DatabaseStats>,
    users: Option<hesocial_frontend::admin::UserStats>,
    events: Option<hesocial_frontend::admin::EventsOverview>,
    tab: SystemTab,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        SystemAt,
        SystemAtProps {
            loading,
            error,
            health,
            status,
            database,
            users,
            events,
            tab,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[test]
fn api_paths_match_backend_routes() {
    assert_eq!(HEALTH_API_PATH, "/api/health");
    assert_eq!(HEALTH_STATUS_API_PATH, "/api/health/status");
    assert_eq!(DATABASE_STATS_API_PATH, "/api/admin/database/stats");
    assert_eq!(USER_STATS_API_PATH, "/api/users/stats/overview");
    assert_eq!(EVENTS_OVERVIEW_API_PATH, "/api/analytics/events/overview");
}

#[test]
fn parse_health_reads_flat_envelope() {
    let health = parse_health_response(200, health_ok_body()).expect("health");
    assert!(health.success);
    assert_eq!(health.database, "turso");
    assert_eq!(health.version, "1.0.0");
    assert_eq!(health.message, "API health check passed (Turso)");
}

#[test]
fn parse_health_401_uses_backend_error() {
    let err = parse_health_response(401, unauthorized_body()).expect_err("401");
    assert_eq!(err, UNAUTHORIZED_FALLBACK);
}

#[test]
fn parse_health_empty_and_invalid_bodies_fail() {
    assert_eq!(
        parse_health_response(200, "").expect_err("empty"),
        HEALTH_FETCH_FALLBACK
    );
    assert_eq!(
        parse_health_response(200, "{not-json").expect_err("invalid"),
        HEALTH_FETCH_FALLBACK
    );
    assert_eq!(
        parse_health_response(500, r#"{"success":false,"error":"boom"}"#).expect_err("500"),
        "boom"
    );
}

#[test]
fn parse_health_status_reads_server_and_r2_fields() {
    let status = parse_health_status_response(200, health_status_ok_body()).expect("status");
    assert_eq!(status.status, "healthy");
    assert_eq!(status.uptime, Some(3661.0));
    assert_eq!(status.uptime_formatted.as_deref(), Some("1h 1m 1s"));
    assert_eq!(status.memory_rss.as_deref(), Some("0MB"));
    assert!(status.node_version.is_none());
    assert!(status.platform.is_none());
    assert_eq!(status.database_type.as_deref(), Some("Turso"));
    assert_eq!(status.r2_sync.as_deref(), Some("disabled"));
    assert_eq!(status.environment.as_deref(), Some("development"));
}

#[test]
fn parse_health_status_401_and_empty() {
    assert_eq!(
        parse_health_status_response(401, unauthorized_body()).expect_err("401"),
        UNAUTHORIZED_FALLBACK
    );
    assert_eq!(
        parse_health_status_response(200, "{}").expect_err("empty"),
        HEALTH_FETCH_FALLBACK
    );
}

#[test]
fn parse_database_stats_populated_and_empty_tables() {
    let stats = parse_database_stats_response(200, database_stats_ok_body()).expect("stats");
    assert_eq!(stats.schema_version.as_deref(), Some("unknown"));
    assert_eq!(stats.total_tables, Some(2));
    assert_eq!(stats.tables.len(), 2);
    assert_eq!(stats.tables[0].name, "users");
    assert_eq!(stats.tables[0].column_count, Some(12));

    let empty = parse_database_stats_response(
        200,
        r#"{"success":true,"data":{"schemaVersion":"unknown","serverStats":{},"tables":[],"meta":{"totalTables":0,"timestamp":"t"}}}"#,
    )
    .expect("empty tables");
    assert!(empty.tables.is_empty());
    assert_eq!(empty.total_tables, Some(0));
}

#[test]
fn parse_database_stats_401() {
    assert_eq!(
        parse_database_stats_response(401, unauthorized_body()).expect_err("401"),
        UNAUTHORIZED_FALLBACK
    );
}

#[test]
fn parse_user_stats_populated_and_empty() {
    let stats = parse_user_stats_response(200, user_stats_ok_body()).expect("users");
    assert_eq!(stats.total_users, Some(7));
    assert_eq!(stats.recent_registrations, Some(2));
    assert_eq!(stats.users_by_role[0].key, "admin");
    assert_eq!(stats.users_by_role[0].count, Some(1));
    assert_eq!(stats.users_by_membership_tier[0].key, "Diamond");
    assert_eq!(stats.users_by_verification_status[0].key, "approved");

    let empty = parse_user_stats_response(
        200,
        r#"{"success":true,"data":{"totalUsers":0,"usersByRole":[],"usersByMembershipTier":[],"usersByVerificationStatus":[],"recentRegistrations":0}}"#,
    )
    .expect("empty users");
    assert_eq!(empty.total_users, Some(0));
    assert!(empty.users_by_role.is_empty());
}

#[test]
fn parse_user_stats_401() {
    assert_eq!(
        parse_user_stats_response(401, unauthorized_body()).expect_err("401"),
        UNAUTHORIZED_FALLBACK
    );
}

#[test]
fn parse_events_overview_populated_null_and_401() {
    let overview = parse_events_overview_response(200, events_overview_ok_body()).expect("events");
    assert_eq!(overview.period_days, Some(30));
    assert_eq!(overview.total_events, Some(4));
    assert_eq!(overview.total_registrations, Some(9));
    assert_eq!(overview.popular_events.len(), 1);
    assert_eq!(overview.popular_events[0].name, "Gala");

    let empty = parse_events_overview_response(
        200,
        r#"{"success":true,"data":{"period_days":30,"event_stats":null,"registration_stats":null,"popular_events":[]}}"#,
    )
    .expect("null stats");
    assert!(empty.total_events.is_none());
    assert!(empty.popular_events.is_empty());

    assert_eq!(
        parse_events_overview_response(401, unauthorized_body()).expect_err("401"),
        UNAUTHORIZED_FALLBACK
    );
}

#[test]
fn format_uptime_matches_react_thresholds() {
    assert_eq!(format_uptime(0.0), "0s");
    assert_eq!(format_uptime(45.0), "45s");
    assert_eq!(format_uptime(60.0), "1m 0s");
    assert_eq!(format_uptime(3661.0), "1h 1m");
    assert_eq!(format_uptime(90_061.0), "1d 1h 1m");
}

#[test]
fn format_bytes_matches_react_thresholds() {
    assert_eq!(format_bytes(0.0), "0 B");
    assert_eq!(format_bytes(1024.0), "1 KB");
    assert_eq!(format_bytes(1536.0), "1.5 KB");
    assert_eq!(format_bytes(1_048_576.0), "1 MB");
}

#[test]
fn health_status_thresholds_map_colour_classes() {
    assert_eq!(overall_status("healthy"), OverallStatus::Healthy);
    assert_eq!(overall_status("warning"), OverallStatus::Warning);
    assert_eq!(overall_status("error"), OverallStatus::Error);
    assert_eq!(overall_status(""), OverallStatus::Unknown);
    assert_eq!(overall_status_label(OverallStatus::Healthy), "健康");
    assert_eq!(overall_status_label(OverallStatus::Warning), "警告");
    assert_eq!(overall_status_label(OverallStatus::Error), "錯誤");
    assert_eq!(overall_status_label(OverallStatus::Unknown), "未知");
    assert_eq!(health_status_color("healthy"), "text-green-400");
    assert_eq!(health_status_color("warning"), "text-yellow-400");
    assert_eq!(health_status_color("error"), "text-red-400");
    assert_eq!(health_status_color("other"), "text-gray-400");
    assert_eq!(health_status_bg("healthy"), "bg-green-500/20");
    assert_eq!(health_status_bg("warning"), "bg-yellow-500/20");
    assert_eq!(health_status_bg("error"), "bg-red-500/20");
    assert_eq!(health_status_bg("other"), "bg-gray-500/20");
    assert_eq!(health_status_glyph("healthy"), "✅");
    assert_eq!(health_status_glyph("warning"), "⚠️");
    assert_eq!(health_status_glyph("error"), "❌");
    assert_eq!(health_status_glyph("other"), "❓");
}

#[test]
fn component_status_thresholds_and_labels() {
    assert_eq!(database_status(true), ComponentStatus::Healthy);
    assert_eq!(database_status(false), ComponentStatus::Unhealthy);
    assert_eq!(r2_status("disabled"), ComponentStatus::Disabled);
    assert_eq!(r2_status("healthy"), ComponentStatus::Healthy);
    assert_eq!(r2_status("down"), ComponentStatus::Unhealthy);
    assert_eq!(
        component_status_color(ComponentStatus::Healthy),
        "text-green-400"
    );
    assert_eq!(
        component_status_color(ComponentStatus::Unhealthy),
        "text-red-400"
    );
    assert_eq!(
        component_status_color(ComponentStatus::Disabled),
        "text-yellow-400"
    );
    assert_eq!(database_connection_label(true, false), "載入中...");
    assert_eq!(database_connection_label(false, true), "已連線");
    assert_eq!(database_connection_label(false, false), "未連線");
    assert_eq!(r2_status_label(true, None), "載入中...");
    assert_eq!(r2_status_label(false, Some("disabled")), "已停用");
    assert_eq!(r2_status_label(false, Some("healthy")), "正常");
    assert_eq!(r2_status_label(false, Some("down")), "連線問題");
    assert_eq!(r2_status_label(false, None), "");
}

#[test]
fn admin_guard_has_loading_redirect_and_allow() {
    assert_eq!(
        admin_route_guard(true, &snapshot(false, None)),
        RouteGuard::Loading
    );
    assert_eq!(
        admin_route_guard(false, &snapshot(false, None)),
        RouteGuard::Redirect(ADMIN_ROUTE_FALLBACK)
    );
    assert_eq!(
        admin_route_guard(false, &snapshot(true, Some(Role::User))),
        RouteGuard::Redirect(ADMIN_ROUTE_FALLBACK)
    );
    assert_eq!(
        admin_route_guard(false, &snapshot(true, Some(Role::Admin))),
        RouteGuard::Allow
    );
    assert_eq!(
        admin_route_guard(false, &snapshot(true, Some(Role::SuperAdmin))),
        RouteGuard::Allow
    );
    assert_eq!(ADMIN_ROUTE_FALLBACK, "/login");
}

#[test]
fn welcome_copy_keeps_react_sentence() {
    assert_eq!(
        welcome_copy("Ada"),
        "歡迎回來，Ada。管理系統運作並監控平台健康狀況。"
    );
    assert_eq!(welcome_copy("  "), "管理系統運作並監控平台健康狀況。");
}

#[test]
fn phases_cover_loading_error_empty_ready() {
    assert_eq!(admin_phase(true, None), AdminPhase::Loading);
    assert_eq!(admin_phase(false, Some("boom")), AdminPhase::Error);
    assert_eq!(admin_phase(false, None), AdminPhase::Ready);
    assert_eq!(system_phase(true, None, false), SystemPhase::Loading);
    assert_eq!(
        system_phase(false, Some("載入系統健康資料失敗"), false),
        SystemPhase::Error
    );
    assert_eq!(system_phase(false, None, false), SystemPhase::Empty);
    assert_eq!(system_phase(false, None, true), SystemPhase::Ready);
}

#[tokio::test]
async fn native_fetch_does_not_hit_network() {
    let err = fetch_admin_dashboard().await.expect_err("native");
    assert_eq!(err, NETWORK_ERROR);
}

#[test]
fn admin_screen_loading_empty_populated_and_error() {
    let loading = render_admin(true, None, "Ada", None, None, None);
    assert!(loading.contains("id=\"admin-dashboard\""));
    assert!(loading.contains("管理後台"));
    assert!(loading.contains("載入中..."));
    assert!(loading.contains("歡迎回來，Ada。管理系統運作並監控平台健康狀況。"));
    assert!(!loading.contains("最佳"));
    assert!(!loading.contains("活躍"));

    let empty = render_admin(false, None, "", None, None, None);
    assert!(empty.contains("id=\"admin-dashboard\""));
    assert!(empty.contains("未連線"));
    assert!(empty.contains("管理系統運作並監控平台健康狀況。"));
    assert!(!empty.contains("id=\"admin-error\""));

    let health = parse_health_response(200, health_ok_body()).unwrap();
    let status = parse_health_status_response(200, health_status_ok_body()).unwrap();
    let users = parse_user_stats_response(200, user_stats_ok_body()).unwrap();
    let populated = render_admin(false, None, "Ada", Some(health), Some(status), Some(users));
    assert!(populated.contains("已連線"));
    assert!(populated.contains("已停用"));
    assert!(populated.contains(">7<") || populated.contains("7"));
    assert!(populated.contains("查看分析數據"));
    assert!(populated.contains("href=\"/admin/system\""));
    assert!(populated.contains("href=\"/admin/users\""));
    assert!(populated.contains("href=\"/events/manage\""));

    let errored = render_admin(
        false,
        Some("Failed to load system health".to_string()),
        "Ada",
        None,
        None,
        None,
    );
    assert!(errored.contains("id=\"admin-error\""));
    assert!(errored.contains("Failed to load system health"));
}

#[test]
fn system_screen_loading_empty_populated_and_error() {
    let loading = render_system(
        true,
        None,
        None,
        None,
        None,
        None,
        None,
        SystemTab::Overview,
    );
    assert!(loading.contains("id=\"admin-system-loading\""));
    assert!(loading.contains("animate-spin"));

    let empty = render_system(
        false,
        None,
        None,
        None,
        None,
        None,
        None,
        SystemTab::Overview,
    );
    assert!(empty.contains("id=\"admin-system\""));
    assert!(empty.contains("System Health Dashboard"));
    assert!(empty.contains("系統狀態：未知"));
    assert!(!empty.contains("id=\"admin-system-user-count\""));

    let health = parse_health_response(200, health_ok_body()).unwrap();
    let status = parse_health_status_response(200, health_status_ok_body()).unwrap();
    let database = parse_database_stats_response(200, database_stats_ok_body()).unwrap();
    let users = parse_user_stats_response(200, user_stats_ok_body()).unwrap();
    let events = parse_events_overview_response(200, events_overview_ok_body()).unwrap();
    let populated = render_system(
        false,
        None,
        Some(health.clone()),
        Some(status.clone()),
        Some(database.clone()),
        Some(users.clone()),
        Some(events.clone()),
        SystemTab::Overview,
    );
    assert!(populated.contains("系統狀態：健康"));
    assert!(populated.contains("Disabled"));
    assert!(populated.contains("id=\"admin-system-user-count\""));
    assert!(populated.contains("id=\"admin-system-event-count\""));
    assert!(!populated.contains("OAuth Providers"));
    assert!(!populated.contains("Stripe"));

    let detailed = render_system(
        false,
        None,
        Some(health.clone()),
        Some(status.clone()),
        Some(database),
        Some(users.clone()),
        Some(events.clone()),
        SystemTab::Detailed,
    );
    assert!(detailed.contains("Detailed Health Check"));
    assert!(detailed.contains("users"));
    assert!(detailed.contains("schemaVersion: unknown"));

    let metrics = render_system(
        false,
        None,
        Some(health),
        Some(status),
        None,
        Some(users),
        Some(events),
        SystemTab::Metrics,
    );
    assert!(metrics.contains("System Information"));
    assert!(metrics.contains("Memory Usage"));
    assert!(metrics.contains("0MB"));
    assert!(!metrics.contains("CPU Usage"));

    let diagnostics = render_system(
        false,
        None,
        None,
        Some(hesocial_frontend::admin::HealthStatus {
            success: true,
            status: "healthy".into(),
            ..Default::default()
        }),
        None,
        None,
        None,
        SystemTab::Diagnostics,
    );
    assert!(diagnostics.contains("id=\"admin-system-diagnostics\""));
    assert!(!diagnostics.contains("Run Tests"));

    let errored = render_system(
        false,
        Some("載入系統健康資料失敗".to_string()),
        None,
        None,
        None,
        None,
        None,
        SystemTab::Overview,
    );
    assert!(errored.contains("id=\"admin-system-error\""));
    assert!(errored.contains("系統健康狀況無法使用"));
    assert!(errored.contains("載入系統健康資料失敗"));
    assert!(errored.contains("重試"));
}
