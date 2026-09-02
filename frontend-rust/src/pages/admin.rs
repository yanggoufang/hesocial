use std::cell::Cell;
use std::rc::Rc;

use crate::admin::{
    AdminDashboardData, ComponentStatus, EventsOverview, HealthCheck, HealthStatus,
    SystemHealthBundle, SystemPhase, SystemTab, UserStats, admin_route_guard,
    component_status_color, database_connection_label, database_status, fetch_admin_dashboard,
    fetch_system_health, format_timestamp, format_uptime, overall_status, overall_status_label,
    r2_status, r2_status_label, system_phase, welcome_copy,
};
use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session};
use dioxus::prelude::*;

#[component]
pub fn Admin() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match admin_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            AdminGuardLoading { id: "admin-guard-loading".to_string() }
        },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "admin-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { AdminBody { session } },
    }
}

#[component]
pub fn AdminSystem() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match admin_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            AdminGuardLoading { id: "admin-system-guard-loading".to_string() }
        },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "admin-system-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { AdminSystemBody {} },
    }
}

#[component]
fn AdminBody(session: Signal<Session>) -> Element {
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut data = use_signal(|| None::<AdminDashboardData>);
    let mut refresh_tick = use_signal(|| 0u32);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let _ = refresh_tick();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let result = fetch_admin_dashboard().await;
                if fetch_gen.get() != request_id {
                    return;
                }
                match result {
                    Ok(fetched) => {
                        data.set(Some(fetched));
                        error.set(None);
                    }
                    Err(message) => {
                        error.set(Some(message));
                    }
                }
                loading.set(false);
            });
        }
    });

    let current = session();
    let first_name = current
        .user
        .as_ref()
        .and_then(|user| user.email.clone())
        .unwrap_or_default();
    let snapshot = data();
    rsx! {
        AdminScreen {
            loading: loading(),
            error: error(),
            first_name,
            health: snapshot.as_ref().map(|item| item.health.clone()),
            status: snapshot.as_ref().map(|item| item.status.clone()),
            users: snapshot.as_ref().map(|item| item.users.clone()),
            on_refresh: move |_| refresh_tick.set(refresh_tick() + 1),
        }
    }
}

#[component]
fn AdminSystemBody() -> Element {
    let mut loading = use_signal(|| true);
    let mut refreshing = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut data = use_signal(|| None::<SystemHealthBundle>);
    let mut tab = use_signal(|| SystemTab::Overview);
    let mut refresh_tick = use_signal(|| 0u32);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let _ = refresh_tick();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            if data.peek().is_some() {
                refreshing.set(true);
            } else {
                loading.set(true);
            }
            error.set(None);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let result = fetch_system_health().await;
                if fetch_gen.get() != request_id {
                    return;
                }
                match result {
                    Ok(fetched) => {
                        data.set(Some(fetched));
                        error.set(None);
                    }
                    Err(message) => {
                        error.set(Some(message));
                    }
                }
                loading.set(false);
                refreshing.set(false);
            });
        }
    });

    let snapshot = data();
    rsx! {
        AdminSystemScreen {
            loading: loading(),
            refreshing: refreshing(),
            error: error(),
            health: snapshot.as_ref().map(|item| item.health.clone()),
            status: snapshot.as_ref().map(|item| item.status.clone()),
            database: snapshot.as_ref().map(|item| item.database.clone()),
            users: snapshot.as_ref().map(|item| item.users.clone()),
            events: snapshot.as_ref().map(|item| item.events.clone()),
            tab: tab(),
            on_tab: move |next: SystemTab| tab.set(next),
            on_refresh: move |_| refresh_tick.set(refresh_tick() + 1),
        }
    }
}

#[component]
pub fn AdminScreen(
    loading: bool,
    error: Option<String>,
    first_name: String,
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    users: Option<UserStats>,
    #[props(default)] on_refresh: EventHandler<()>,
) -> Element {
    let welcome = welcome_copy(&first_name);
    let db_connected = health.as_ref().map(|item| item.success).unwrap_or(false);
    let db_status = health.as_ref().map(|item| database_status(item.success));
    let db_label = database_connection_label(loading, db_connected);
    let db_color = db_status
        .map(component_status_color)
        .unwrap_or("text-gray-400");
    let r2_sync = status.as_ref().and_then(|item| item.r2_sync.as_deref());
    let r2_component = r2_sync.map(r2_status);
    let r2_label = r2_status_label(loading, r2_sync);
    let r2_color = r2_component
        .map(component_status_color)
        .unwrap_or("text-gray-400");
    let user_total = users.as_ref().and_then(|item| item.total_users);
    let recent_registrations = users.as_ref().and_then(|item| item.recent_registrations);

    rsx! {
        div { id: "admin-dashboard", class: "min-h-screen bg-luxury-midnight-black py-8 px-4",
            div { class: "max-w-7xl mx-auto",
                div { class: "hs-enter",
                    div { class: "mb-8",
                        div { class: "flex items-center space-x-3 mb-4",
                            Icon { name: IconName::Shield, class: "h-8 w-8 text-luxury-gold".to_string() }
                            h1 { class: "text-3xl font-luxury font-bold text-luxury-gold", "管理後台" }
                        }
                        p { id: "admin-welcome", class: "text-luxury-platinum/80", "{welcome}" }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8",
                        div { id: "admin-db-card", class: "luxury-glass p-6 rounded-xl hs-enter",
                            div { class: "flex items-center justify-between mb-4",
                                Icon { name: IconName::Building, class: "h-8 w-8 text-luxury-gold".to_string() }
                                ComponentStatusIcon { status: db_status }
                            }
                            h3 { class: "text-luxury-platinum text-lg font-medium mb-2", "資料庫" }
                            p { id: "admin-db-status", class: "text-sm font-medium {db_color}", "{db_label}" }
                        }
                        div { id: "admin-r2-card", class: "luxury-glass p-6 rounded-xl hs-enter",
                            div { class: "flex items-center justify-between mb-4",
                                Icon { name: IconName::ExternalLink, class: "h-8 w-8 text-luxury-gold".to_string() }
                                ComponentStatusIcon { status: r2_component }
                            }
                            h3 { class: "text-luxury-platinum text-lg font-medium mb-2", "R2 備份" }
                            p { id: "admin-r2-status", class: "text-sm font-medium {r2_color}", "{r2_label}" }
                        }
                        div { id: "admin-users-card", class: "luxury-glass p-6 rounded-xl hs-enter",
                            div { class: "flex items-center justify-between mb-4",
                                Icon { name: IconName::Users, class: "h-8 w-8 text-luxury-gold".to_string() }
                            }
                            h3 { class: "text-luxury-platinum text-lg font-medium mb-2", "使用者" }
                            if loading {
                                p { id: "admin-users-status", class: "text-sm font-medium text-gray-400", "載入中..." }
                            } else if let Some(total) = user_total {
                                p { id: "admin-users-status", class: "text-sm font-medium text-luxury-platinum", "{total}" }
                                if let Some(recent) = recent_registrations {
                                    p { id: "admin-users-recent", class: "text-luxury-platinum/60 text-xs mt-1",
                                        "{recent}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "grid grid-cols-1 lg:grid-cols-4 gap-6 mb-8",
                        QuickAction {
                            id: "admin-quick-analytics".to_string(),
                            icon: IconName::TrendingUp,
                            title: "分析儀表板".to_string(),
                            body: "查看活動表現、營收分析和會員參與度等關鍵業務指標。".to_string(),
                            primary_href: "/admin/analytics".to_string(),
                            primary_label: "查看分析數據".to_string(),
                            secondary_left_href: Some("/admin/analytics".to_string()),
                            secondary_left: "營收報告".to_string(),
                            secondary_right_href: Some("/admin/analytics".to_string()),
                            secondary_right: "會員分析".to_string(),
                        }
                        QuickAction {
                            id: "admin-quick-backups".to_string(),
                            icon: IconName::Building,
                            title: "備份管理".to_string(),
                            body: "建立、還原和管理資料庫備份。建議使用手動備份以獲得完整控制。".to_string(),
                            primary_href: "/admin/backups".to_string(),
                            primary_label: "管理備份".to_string(),
                            secondary_left_href: None,
                            secondary_left: "快速備份".to_string(),
                            secondary_right_href: None,
                            secondary_right: "檢視健康狀況".to_string(),
                        }
                        div { id: "admin-quick-system", class: "luxury-glass p-6 rounded-xl hs-enter",
                            div { class: "flex items-center space-x-3 mb-4",
                                Icon { name: IconName::Activity, class: "h-6 w-6 text-luxury-gold".to_string() }
                                h2 { class: "text-xl font-luxury font-bold text-luxury-gold", "系統監控" }
                            }
                            p { class: "text-luxury-platinum/80 mb-6",
                                "即時監控系統健康狀況、效能指標和運作狀態。"
                            }
                            div { class: "space-y-3",
                                a {
                                    href: "/admin/system",
                                    class: "w-full luxury-button text-center block",
                                    "檢視系統健康狀況"
                                }
                                div { class: "grid grid-cols-2 gap-3",
                                    button {
                                        id: "admin-refresh-status",
                                        r#type: "button",
                                        class: "px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm",
                                        onclick: move |_| on_refresh.call(()),
                                        "重新整理狀態"
                                    }
                                    a {
                                        href: "/admin/system",
                                        class: "px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm text-center",
                                        "健康儀表板"
                                    }
                                }
                            }
                        }
                        QuickAction {
                            id: "admin-quick-users".to_string(),
                            icon: IconName::Users,
                            title: "使用者管理".to_string(),
                            body: "管理使用者帳戶、角色、驗證狀態和會員等級。".to_string(),
                            primary_href: "/admin/users".to_string(),
                            primary_label: "管理使用者".to_string(),
                            secondary_left_href: None,
                            secondary_left: "待審核驗證".to_string(),
                            secondary_right_href: None,
                            secondary_right: "使用者統計".to_string(),
                        }
                        QuickAction {
                            id: "admin-quick-events".to_string(),
                            icon: IconName::Calendar,
                            title: "活動管理".to_string(),
                            body: "為平台建立和管理頂級社交活動、場地和類別。".to_string(),
                            primary_href: "/events/manage".to_string(),
                            primary_label: "管理活動".to_string(),
                            secondary_left_href: Some("/events/venues".to_string()),
                            secondary_left: "場地".to_string(),
                            secondary_right_href: Some("/events/categories".to_string()),
                            secondary_right: "類別".to_string(),
                        }
                    }
                    if let Some(message) = error.clone() {
                        div {
                            id: "admin-error",
                            class: "mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm hs-enter",
                            "{message}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AdminSystemScreen(
    loading: bool,
    refreshing: bool,
    error: Option<String>,
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    database: Option<crate::admin::DatabaseStats>,
    users: Option<UserStats>,
    events: Option<EventsOverview>,
    tab: SystemTab,
    #[props(default)] on_tab: EventHandler<SystemTab>,
    #[props(default)] on_refresh: EventHandler<()>,
) -> Element {
    match system_phase(loading, error.as_deref(), status.is_some()) {
        SystemPhase::Loading => {
            return rsx! {
                div { id: "admin-system-loading", class: "min-h-screen bg-luxury-midnight-black py-12",
                    div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                        div { class: "flex items-center justify-center h-64",
                            div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-luxury-gold" }
                        }
                    }
                }
            };
        }
        SystemPhase::Error => {
            let message = error.unwrap_or_default();
            return rsx! {
                div { id: "admin-system-error", class: "min-h-screen bg-luxury-midnight-black py-12",
                    div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                        div { class: "luxury-glass p-8 rounded-2xl text-center hs-enter",
                            Icon {
                                name: IconName::AlertCircle,
                                class: "h-16 w-16 text-red-400 mx-auto mb-4".to_string(),
                            }
                            h2 { class: "text-2xl font-luxury font-bold text-luxury-gold mb-2",
                                "系統健康狀況無法使用"
                            }
                            p { class: "text-luxury-platinum/80 mb-6", "{message}" }
                            button {
                                id: "admin-system-retry",
                                r#type: "button",
                                class: "luxury-button inline-flex items-center",
                                onclick: move |_| on_refresh.call(()),
                                Icon { name: IconName::RefreshCw, class: "h-4 w-4 mr-2".to_string() }
                                "重試"
                            }
                        }
                    }
                }
            };
        }
        SystemPhase::Empty | SystemPhase::Ready => {}
    }

    let refresh_icon_class = if refreshing {
        "h-4 w-4 animate-spin"
    } else {
        "h-4 w-4"
    };

    rsx! {
        div { id: "admin-system", class: "min-h-screen bg-luxury-midnight-black py-12",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "text-center mb-8 hs-enter",
                    h1 { class: "text-4xl font-luxury font-bold text-luxury-gold mb-4",
                        "System Health Dashboard"
                    }
                    p { class: "text-luxury-platinum/80 text-lg",
                        "Real-time monitoring and diagnostics for the HeSocial platform"
                    }
                }
                div { class: "flex flex-wrap justify-center space-x-1 mb-8 hs-enter-filters",
                    TabButton {
                        id: "admin-system-tab-overview".to_string(),
                        label: "Overview".to_string(),
                        icon: IconName::Activity,
                        active: tab == SystemTab::Overview,
                        on_click: move |_| on_tab.call(SystemTab::Overview),
                    }
                    TabButton {
                        id: "admin-system-tab-detailed".to_string(),
                        label: "Detailed".to_string(),
                        icon: IconName::Eye,
                        active: tab == SystemTab::Detailed,
                        on_click: move |_| on_tab.call(SystemTab::Detailed),
                    }
                    TabButton {
                        id: "admin-system-tab-metrics".to_string(),
                        label: "Metrics".to_string(),
                        icon: IconName::TrendingUp,
                        active: tab == SystemTab::Metrics,
                        on_click: move |_| on_tab.call(SystemTab::Metrics),
                    }
                    TabButton {
                        id: "admin-system-tab-diagnostics".to_string(),
                        label: "Diagnostics".to_string(),
                        icon: IconName::Settings,
                        active: tab == SystemTab::Diagnostics,
                        on_click: move |_| on_tab.call(SystemTab::Diagnostics),
                    }
                }
                div { class: "tab-content",
                    match tab {
                        SystemTab::Overview => rsx! {
                            SystemOverviewTab {
                                refreshing,
                                refresh_icon_class: refresh_icon_class.to_string(),
                                health: health.clone(),
                                status: status.clone(),
                                users: users.clone(),
                                events: events.clone(),
                                on_refresh,
                            }
                        },
                        SystemTab::Detailed => rsx! {
                            SystemDetailedTab { database: database.clone() }
                        },
                        SystemTab::Metrics => rsx! {
                            SystemMetricsTab { status: status.clone() }
                        },
                        SystemTab::Diagnostics => rsx! {
                            div { id: "admin-system-diagnostics", class: "space-y-6" }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn SystemOverviewTab(
    refreshing: bool,
    refresh_icon_class: String,
    health: Option<HealthCheck>,
    status: Option<HealthStatus>,
    users: Option<UserStats>,
    events: Option<EventsOverview>,
    on_refresh: EventHandler<()>,
) -> Element {
    let overall = status
        .as_ref()
        .map(|item| overall_status(&item.status))
        .unwrap_or(crate::admin::OverallStatus::Unknown);
    let overall_label = overall_status_label(overall);
    let timestamp = status
        .as_ref()
        .and_then(|item| item.timestamp.as_deref())
        .filter(|value| !value.is_empty())
        .map(format_timestamp)
        .unwrap_or_else(|| "未知".to_string());
    let uptime = status
        .as_ref()
        .and_then(|item| item.uptime_formatted.clone())
        .or_else(|| {
            status
                .as_ref()
                .and_then(|item| item.uptime)
                .map(format_uptime)
        })
        .unwrap_or_else(|| "N/A".to_string());
    let memory = status
        .as_ref()
        .and_then(|item| item.memory_rss.clone())
        .or_else(|| {
            status
                .as_ref()
                .and_then(|item| item.memory_heap_used.clone())
        });
    let environment = status
        .as_ref()
        .and_then(|item| item.environment.clone())
        .filter(|value| !value.is_empty());
    let db_connected = health.as_ref().map(|item| item.success).unwrap_or(false);
    let db_connected_label = if db_connected {
        "Connected"
    } else {
        "Disconnected"
    };
    let db_connected_color = if db_connected {
        "text-green-400"
    } else {
        "text-red-400"
    };
    let user_count = users.as_ref().and_then(|item| item.total_users);
    let event_count = events.as_ref().and_then(|item| item.total_events);
    let r2_sync = status.as_ref().and_then(|item| item.r2_sync.clone());
    let r2_component = r2_sync.as_deref().map(r2_status);
    let r2_label = match r2_component {
        Some(ComponentStatus::Disabled) => "Disabled",
        Some(ComponentStatus::Healthy) => "Healthy",
        Some(ComponentStatus::Unhealthy) => "Error",
        None => "",
    };
    let r2_color = r2_component
        .map(component_status_color)
        .unwrap_or("text-gray-400");

    rsx! {
        div { id: "admin-system-overview", class: "space-y-6",
            div { id: "admin-system-overall", class: "luxury-glass p-6 rounded-2xl hs-enter",
                div { class: "flex items-center justify-between",
                    div { class: "flex items-center space-x-4",
                        OverallStatusIcon { status: overall }
                        div {
                            h3 { class: "text-2xl font-luxury font-bold text-luxury-gold",
                                "系統狀態：{overall_label}"
                            }
                            p { class: "text-luxury-platinum/80", "最後更新：{timestamp}" }
                        }
                    }
                    button {
                        id: "admin-system-refresh",
                        r#type: "button",
                        class: "luxury-button-outline flex items-center space-x-2",
                        disabled: refreshing,
                        onclick: move |_| on_refresh.call(()),
                        Icon { name: IconName::RefreshCw, class: "{refresh_icon_class}".to_string() }
                        span { "重新整理" }
                    }
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                div { class: "luxury-glass p-6 rounded-2xl hs-enter",
                    div { class: "flex items-center space-x-3 mb-4",
                        Icon { name: IconName::Activity, class: "h-6 w-6 text-luxury-gold".to_string() }
                        h4 { class: "text-lg font-luxury font-semibold text-luxury-platinum", "系統" }
                    }
                    div { class: "space-y-2",
                        div { class: "flex justify-between",
                            span { class: "text-luxury-platinum/80", "運行時間：" }
                            span { id: "admin-system-uptime", class: "text-luxury-platinum font-medium", "{uptime}" }
                        }
                        if let Some(memory) = memory.clone() {
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80", "記憶體：" }
                                span { id: "admin-system-memory", class: "text-luxury-platinum font-medium", "{memory}" }
                            }
                        }
                        if let Some(environment) = environment.clone() {
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80", "環境：" }
                                span { id: "admin-system-environment", class: "text-luxury-platinum font-medium", "{environment}" }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl hs-enter",
                    div { class: "flex items-center space-x-3 mb-4",
                        Icon { name: IconName::Building, class: "h-6 w-6 text-luxury-gold".to_string() }
                        h4 { class: "text-lg font-luxury font-semibold text-luxury-platinum", "資料庫" }
                    }
                    div { class: "space-y-2",
                        div { class: "flex justify-between",
                            span { class: "text-luxury-platinum/80", "Status:" }
                            span { class: "font-medium {db_connected_color}", "{db_connected_label}" }
                        }
                        if let Some(user_count) = user_count {
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80", "Users:" }
                                span { id: "admin-system-user-count", class: "text-luxury-platinum font-medium", "{user_count}" }
                            }
                        }
                        if let Some(event_count) = event_count {
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80", "Events:" }
                                span { id: "admin-system-event-count", class: "text-luxury-platinum font-medium", "{event_count}" }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl hs-enter",
                    div { class: "flex items-center space-x-3 mb-4",
                        Icon { name: IconName::ExternalLink, class: "h-6 w-6 text-luxury-gold".to_string() }
                        h4 { class: "text-lg font-luxury font-semibold text-luxury-platinum", "R2 Backup" }
                    }
                    div { class: "space-y-2",
                        if r2_component.is_some() {
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80", "Status:" }
                                span { id: "admin-system-r2-status", class: "font-medium {r2_color}", "{r2_label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SystemDetailedTab(database: Option<crate::admin::DatabaseStats>) -> Element {
    let Some(stats) = database else {
        return rsx! { div { id: "admin-system-detailed", class: "space-y-6" } };
    };
    let schema = stats.schema_version.clone();
    let tables = stats.tables.clone();
    rsx! {
        div { id: "admin-system-detailed", class: "space-y-6",
            div { class: "luxury-glass p-6 rounded-2xl hs-enter",
                h3 { class: "text-xl font-luxury font-bold text-luxury-gold mb-6", "Detailed Health Check" }
                if let Some(schema) = schema {
                    div { class: "mb-6 p-4 rounded-lg bg-luxury-midnight-black/50",
                        span { class: "text-lg font-medium text-luxury-platinum", "schemaVersion: {schema}" }
                    }
                }
                div { class: "space-y-4",
                    for table in tables.iter() {
                        {
                            let name = table.name.clone();
                            let count = table.column_count;
                            rsx! {
                                div { class: "p-4 rounded-lg bg-luxury-midnight-black/50",
                                    h4 { class: "font-medium text-luxury-platinum", "{name}" }
                                    if let Some(count) = count {
                                        p { class: "text-sm text-luxury-platinum/80", "{count}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SystemMetricsTab(status: Option<HealthStatus>) -> Element {
    let Some(status) = status else {
        return rsx! { div { id: "admin-system-metrics", class: "space-y-6" } };
    };
    let environment = status.environment.clone();
    let node_version = status.node_version.clone();
    let platform = status.platform.clone();
    let rss = status.memory_rss.clone();
    let heap_total = status.memory_heap_total.clone();
    let heap_used = status.memory_heap_used.clone();
    rsx! {
        div { id: "admin-system-metrics", class: "space-y-6",
            div { class: "luxury-glass p-6 rounded-2xl hs-enter",
                h3 { class: "text-xl font-luxury font-bold text-luxury-gold mb-6", "System Information" }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                    if let Some(environment) = environment {
                        div {
                            h4 { class: "text-sm font-medium text-luxury-platinum/80 mb-2", "Environment" }
                            p { class: "text-lg font-semibold text-luxury-platinum", "{environment}" }
                        }
                    }
                    if let Some(node_version) = node_version {
                        div {
                            h4 { class: "text-sm font-medium text-luxury-platinum/80 mb-2", "Node Version" }
                            p { class: "text-lg font-semibold text-luxury-platinum", "{node_version}" }
                        }
                    }
                    if let Some(platform) = platform {
                        div {
                            h4 { class: "text-sm font-medium text-luxury-platinum/80 mb-2", "Platform" }
                            p { class: "text-lg font-semibold text-luxury-platinum", "{platform}" }
                        }
                    }
                }
            }
            div { class: "luxury-glass p-6 rounded-2xl hs-enter",
                h3 { class: "text-xl font-luxury font-bold text-luxury-gold mb-6", "Memory Usage" }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                    if let Some(rss) = rss {
                        div {
                            h4 { class: "text-sm font-medium text-luxury-platinum/80 mb-2", "RSS" }
                            p { class: "text-lg font-semibold text-luxury-platinum", "{rss}" }
                        }
                    }
                    if let Some(heap_total) = heap_total {
                        div {
                            h4 { class: "text-sm font-medium text-luxury-platinum/80 mb-2", "Heap Total" }
                            p { class: "text-lg font-semibold text-luxury-platinum", "{heap_total}" }
                        }
                    }
                    if let Some(heap_used) = heap_used {
                        div {
                            h4 { class: "text-sm font-medium text-luxury-platinum/80 mb-2", "Heap Used" }
                            p { class: "text-lg font-semibold text-luxury-platinum", "{heap_used}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TabButton(
    id: String,
    label: String,
    icon: IconName,
    active: bool,
    on_click: EventHandler<()>,
) -> Element {
    let class = if active {
        "flex items-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors bg-luxury-gold text-luxury-midnight-black"
    } else {
        "flex items-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors text-luxury-platinum hover:bg-luxury-gold/10"
    };
    rsx! {
        button {
            id: "{id}",
            r#type: "button",
            class: "{class}",
            onclick: move |_| on_click.call(()),
            Icon { name: icon, class: "h-4 w-4".to_string() }
            span { "{label}" }
        }
    }
}

#[component]
fn QuickAction(
    id: String,
    icon: IconName,
    title: String,
    body: String,
    primary_href: String,
    primary_label: String,
    secondary_left_href: Option<String>,
    secondary_left: String,
    secondary_right_href: Option<String>,
    secondary_right: String,
) -> Element {
    let secondary_class = "px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors text-sm text-center";
    rsx! {
        div { id: "{id}", class: "luxury-glass p-6 rounded-xl hs-enter",
            div { class: "flex items-center space-x-3 mb-4",
                Icon { name: icon, class: "h-6 w-6 text-luxury-gold".to_string() }
                h2 { class: "text-xl font-luxury font-bold text-luxury-gold", "{title}" }
            }
            p { class: "text-luxury-platinum/80 mb-6", "{body}" }
            div { class: "space-y-3",
                a { href: "{primary_href}", class: "w-full luxury-button text-center block", "{primary_label}" }
                div { class: "grid grid-cols-2 gap-3",
                    if let Some(href) = secondary_left_href.clone() {
                        a { href: "{href}", class: "{secondary_class}", "{secondary_left}" }
                    } else {
                        button { r#type: "button", class: "{secondary_class}", "{secondary_left}" }
                    }
                    if let Some(href) = secondary_right_href.clone() {
                        a { href: "{href}", class: "{secondary_class}", "{secondary_right}" }
                    } else {
                        button { r#type: "button", class: "{secondary_class}", "{secondary_right}" }
                    }
                }
            }
        }
    }
}

#[component]
fn ComponentStatusIcon(status: Option<ComponentStatus>) -> Element {
    match status {
        Some(ComponentStatus::Healthy) => rsx! {
            Icon { name: IconName::Check, class: "h-5 w-5 text-green-400".to_string() }
        },
        Some(ComponentStatus::Unhealthy) => rsx! {
            Icon { name: IconName::AlertCircle, class: "h-5 w-5 text-red-400".to_string() }
        },
        Some(ComponentStatus::Disabled) => rsx! {
            Icon { name: IconName::AlertTriangle, class: "h-5 w-5 text-yellow-400".to_string() }
        },
        None => rsx! {},
    }
}

#[component]
fn OverallStatusIcon(status: crate::admin::OverallStatus) -> Element {
    match status {
        crate::admin::OverallStatus::Healthy => rsx! {
            Icon { name: IconName::Check, class: "h-8 w-8 text-green-400".to_string() }
        },
        crate::admin::OverallStatus::Warning => rsx! {
            Icon { name: IconName::AlertTriangle, class: "h-8 w-8 text-yellow-400".to_string() }
        },
        crate::admin::OverallStatus::Error => rsx! {
            Icon { name: IconName::AlertCircle, class: "h-8 w-8 text-red-400".to_string() }
        },
        crate::admin::OverallStatus::Unknown => rsx! {
            Icon { name: IconName::Activity, class: "h-8 w-8 text-gray-400".to_string() }
        },
    }
}

#[component]
fn AdminGuardLoading(id: String) -> Element {
    rsx! {
        div {
            id: "{id}",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                p { class: "text-luxury-platinum", "驗證存取權限中..." }
            }
        }
    }
}
