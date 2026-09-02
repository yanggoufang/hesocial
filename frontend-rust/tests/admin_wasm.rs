#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::admin::{
    DatabaseStats, DatabaseTable, EventsOverview, HealthCheck, HealthStatus, SystemTab, UserStats,
};
use hesocial_frontend::pages::admin::{AdminScreen, AdminSystemScreen};
use wasm_bindgen_test::wasm_bindgen_test;

fn populated_health() -> HealthCheck {
    HealthCheck {
        success: true,
        message: "API health check passed (Turso)".to_string(),
        timestamp: "2026-09-02T00:00:00.000Z".to_string(),
        version: "1.0.0".to_string(),
        database: "turso".to_string(),
    }
}

fn populated_status() -> HealthStatus {
    HealthStatus {
        success: true,
        status: "healthy".to_string(),
        uptime: Some(90.0),
        uptime_formatted: Some("1m 30s".to_string()),
        memory_rss: Some("0MB".to_string()),
        memory_heap_used: Some("0MB".to_string()),
        memory_heap_total: Some("0MB".to_string()),
        node_version: None,
        platform: None,
        database_type: Some("Turso".to_string()),
        r2_sync: Some("disabled".to_string()),
        environment: Some("development".to_string()),
        timestamp: Some("2026-09-02T00:00:00.000Z".to_string()),
    }
}

fn populated_users() -> UserStats {
    UserStats {
        total_users: Some(7),
        recent_registrations: Some(2),
        ..UserStats::default()
    }
}

fn populated_events() -> EventsOverview {
    EventsOverview {
        total_events: Some(4),
        ..EventsOverview::default()
    }
}

fn populated_database() -> DatabaseStats {
    DatabaseStats {
        schema_version: Some("unknown".to_string()),
        tables: vec![DatabaseTable {
            name: "users".to_string(),
            column_count: Some(12),
        }],
        total_tables: Some(1),
        timestamp: Some("2026-09-02T00:00:00.000Z".to_string()),
    }
}

#[component]
fn AdminAt(
    loading: bool,
    error: Option<String>,
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    users: Option<UserStats>,
) -> Element {
    rsx! {
        AdminScreen {
            loading,
            error,
            first_name: "Ada".to_string(),
            health,
            status,
            users,
        }
    }
}

fn render_admin(
    loading: bool,
    error: Option<String>,
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    users: Option<UserStats>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        AdminAt,
        AdminAtProps {
            loading,
            error,
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
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    database: Option<DatabaseStats>,
    users: Option<UserStats>,
    events: Option<EventsOverview>,
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
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    database: Option<DatabaseStats>,
    users: Option<UserStats>,
    events: Option<EventsOverview>,
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

#[wasm_bindgen_test]
fn admin_ssr_loading_empty_populated_error() {
    let loading = render_admin(true, None, None, None, None);
    assert!(loading.contains("id=\"admin-dashboard\""));
    assert!(loading.contains("載入中..."));

    let empty = render_admin(false, None, None, None, None);
    assert!(empty.contains("未連線"));
    assert!(!empty.contains("id=\"admin-error\""));

    let populated = render_admin(
        false,
        None,
        Some(populated_health()),
        Some(populated_status()),
        Some(populated_users()),
    );
    assert!(populated.contains("已連線"));
    assert!(populated.contains("已停用"));
    assert!(populated.contains("管理後台"));

    let errored = render_admin(
        false,
        Some("Failed to load system health".to_string()),
        None,
        None,
        None,
    );
    assert!(errored.contains("id=\"admin-error\""));
}

#[wasm_bindgen_test]
fn system_ssr_loading_empty_populated_error() {
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
    assert!(empty.contains("系統狀態：未知"));

    let populated = render_system(
        false,
        None,
        Some(populated_health()),
        Some(populated_status()),
        Some(populated_database()),
        Some(populated_users()),
        Some(populated_events()),
        SystemTab::Overview,
    );
    assert!(populated.contains("系統狀態：健康"));
    assert!(populated.contains("Disabled"));

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
    assert!(errored.contains("重試"));
}
