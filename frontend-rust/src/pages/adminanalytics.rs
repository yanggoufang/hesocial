use std::cell::Cell;
use std::rc::Rc;

use crate::adminanalytics::{
    AnalyticsPhase, AnalyticsTab, CHART_HEIGHT, CHART_WIDTH, ChartPoint, ConversionStats,
    DATE_WINDOWS, DEFAULT_DATE_WINDOW_DAYS, DEFAULT_PAD, DONUT_SIZE, DailyVisitor, DashboardData,
    EMPTY_CHART_LABEL, EventsEngagement, EventsOverview, EventsPerformance, MembersEngagement,
    PopularPage, RevenueData, SPARKLINE_HEIGHT, SPARKLINE_WIDTH, VisitorsOverview,
    admin_route_guard, analytics_phase, bar_chart, category_revenue_points, chart_aria_label,
    conversion_points, daily_visitor_points, donut_chart, donut_color, engagement_rate_points,
    events_engagement_points, fetch_dashboard, format_currency, format_one_decimal,
    format_percentage, format_whole, format_year_month, is_active_date_window, line_chart,
    membership_tier_dot_class, monthly_revenue_points, occupancy_points, overview_kpis,
    performance_occupancy_points, popular_page_points, retention_points, sparkline,
    tier_revenue_points,
};
use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session};
use crate::shell::{Presence, presence_after_animation_end, presence_class};
use dioxus::prelude::*;

#[component]
pub fn AdminAnalytics() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match admin_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { AdminAnalyticsGuardLoading {} },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "admin-analytics-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { AdminAnalyticsBody {} },
    }
}

#[component]
fn AdminAnalyticsGuardLoading() -> Element {
    rsx! {
        div {
            id: "admin-analytics-guard-loading",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                p { class: "text-luxury-platinum", "驗證存取權限中..." }
            }
        }
    }
}

#[component]
fn AdminAnalyticsBody() -> Element {
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let mut loading = use_signal(|| true);
    let mut refreshing = use_signal(|| false);
    let mut tab = use_signal(AnalyticsTab::default);
    let mut tab_presence = use_signal(|| Presence::Entering);
    let mut date_window_days = use_signal(|| DEFAULT_DATE_WINDOW_DAYS);
    let mut data = use_signal(DashboardData::default);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let days = date_window_days();
            let token = session().token.clone();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let fetched = fetch_dashboard(token.as_deref(), days).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                data.set(fetched);
                loading.set(false);
                refreshing.set(false);
            });
        }
    });

    let snapshot = data();
    rsx! {
        AdminAnalyticsScreen {
            loading: loading(),
            refreshing: refreshing(),
            error: snapshot.error.clone(),
            tab: tab(),
            tab_presence: tab_presence(),
            date_window_days: date_window_days(),
            overview: snapshot.overview.clone(),
            performance: snapshot.performance.clone(),
            revenue: snapshot.revenue.clone(),
            members: snapshot.members.clone(),
            visitors: snapshot.visitors.clone(),
            visitors_daily: snapshot.visitors_daily.clone(),
            popular_pages: snapshot.popular_pages.clone(),
            conversion: snapshot.conversion.clone(),
            events_engagement: snapshot.events_engagement.clone(),
            on_tab: move |next: AnalyticsTab| {
                tab.set(next);
                tab_presence.set(Presence::Entering);
            },
            on_tab_animation_end: move |_| {
                tab_presence.set(presence_after_animation_end(tab_presence()));
            },
            on_date_window: move |days: u32| date_window_days.set(days),
            on_refresh: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    let days = date_window_days();
                    let token = session().token.clone();
                    let request_id = fetch_gen.get() + 1;
                    fetch_gen.set(request_id);
                    refreshing.set(true);
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        let fetched = fetch_dashboard(token.as_deref(), days).await;
                        if fetch_gen.get() != request_id {
                            return;
                        }
                        data.set(fetched);
                        loading.set(false);
                        refreshing.set(false);
                    });
                }
            },
            on_export: move |_| {},
        }
    }
}

#[component]
pub fn AdminAnalyticsScreen(
    loading: bool,
    refreshing: bool,
    error: Option<String>,
    tab: AnalyticsTab,
    tab_presence: Presence,
    date_window_days: u32,
    overview: Option<EventsOverview>,
    performance: Option<EventsPerformance>,
    revenue: Option<RevenueData>,
    members: Option<MembersEngagement>,
    visitors: Option<VisitorsOverview>,
    visitors_daily: Vec<DailyVisitor>,
    popular_pages: Vec<PopularPage>,
    conversion: Option<ConversionStats>,
    events_engagement: Option<EventsEngagement>,
    #[props(default)] on_tab: EventHandler<AnalyticsTab>,
    #[props(default)] on_tab_animation_end: EventHandler<()>,
    #[props(default)] on_date_window: EventHandler<u32>,
    #[props(default)] on_refresh: EventHandler<()>,
    #[props(default)] on_export: EventHandler<()>,
) -> Element {
    let bundle = DashboardData {
        overview: overview.clone(),
        performance: performance.clone(),
        revenue: revenue.clone(),
        members: members.clone(),
        visitors: visitors.clone(),
        visitors_daily: visitors_daily.clone(),
        popular_pages: popular_pages.clone(),
        conversion: conversion.clone(),
        events_engagement: events_engagement.clone(),
        error: error.clone(),
    };
    match analytics_phase(loading, &bundle) {
        AnalyticsPhase::Loading => {
            return rsx! {
                div {
                    id: "admin-analytics-loading",
                    class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
                    div { class: "luxury-glass p-8 rounded-2xl text-center",
                        div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                        p { class: "text-luxury-platinum", "載入分析數據中..." }
                    }
                }
            };
        }
        AnalyticsPhase::Error(message) => {
            return rsx! {
                div {
                    id: "admin-analytics-error",
                    class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
                    div { class: "luxury-glass p-8 rounded-2xl text-center max-w-md",
                        Icon {
                            name: IconName::AlertCircle,
                            class: "h-12 w-12 text-red-400 mx-auto mb-4".to_string(),
                        }
                        h2 { class: "text-xl font-luxury text-luxury-gold mb-2", "載入失敗" }
                        p { class: "text-luxury-platinum/80 mb-4", "{message}" }
                        button {
                            id: "admin-analytics-error-retry",
                            r#type: "button",
                            class: "luxury-button",
                            onclick: move |_| on_refresh.call(()),
                            "重新整理"
                        }
                    }
                }
            };
        }
        AnalyticsPhase::Ready => {}
    }

    let kpis = overview_kpis(overview.as_ref(), revenue.as_ref(), performance.as_ref());
    let refresh_icon_class = if refreshing {
        "h-4 w-4 animate-spin"
    } else {
        "h-4 w-4"
    };
    let panel_class = format!(
        "space-y-8 {}",
        presence_class(tab_presence, "hs-enter", "hs-exit")
    );
    let visitor_spark = sparkline(
        &visitors_daily
            .iter()
            .rev()
            .map(|row| row.unique_visitors)
            .collect::<Vec<_>>(),
        SPARKLINE_WIDTH,
        SPARKLINE_HEIGHT,
    );

    rsx! {
        div { id: "admin-analytics", class: "min-h-screen bg-luxury-midnight-black py-8",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "flex items-center justify-between mb-8",
                    div {
                        h1 { id: "admin-analytics-title", class: "text-3xl font-luxury font-bold text-luxury-gold mb-2",
                            "分析儀表板"
                        }
                        p { class: "text-luxury-platinum/80", "平台營運數據與會員活動分析" }
                    }
                    div { class: "flex space-x-4",
                        button {
                            id: "admin-analytics-refresh",
                            r#type: "button",
                            class: "luxury-button-outline flex items-center gap-2",
                            disabled: refreshing,
                            onclick: move |_| on_refresh.call(()),
                            Icon { name: IconName::RefreshCw, class: refresh_icon_class.to_string() }
                            if refreshing {
                                "更新中..."
                            } else {
                                "重新整理"
                            }
                        }
                        button {
                            id: "admin-analytics-export",
                            r#type: "button",
                            class: "luxury-button flex items-center gap-2",
                            onclick: move |_| on_export.call(()),
                            Icon { name: IconName::Share2, class: "h-4 w-4".to_string() }
                            "匯出報告"
                        }
                    }
                }

                if let Some(message) = error.as_deref() {
                    if !message.is_empty() {
                        div {
                            id: "admin-analytics-partial-error",
                            class: "mb-6 luxury-glass p-4 rounded-xl text-red-300 text-sm",
                            "{message}"
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-2 mb-6",
                    for window in DATE_WINDOWS.iter() {
                        {
                            let days = window.days;
                            let active = is_active_date_window(date_window_days, days);
                            let class = if active {
                                "px-4 py-2 rounded-lg bg-luxury-gold text-luxury-midnight-black text-sm font-medium"
                            } else {
                                "px-4 py-2 rounded-lg text-luxury-platinum hover:bg-luxury-gold/10 text-sm transition-colors"
                            };
                            rsx! {
                                button {
                                    id: "admin-analytics-window-{days}",
                                    r#type: "button",
                                    class,
                                    onclick: move |_| on_date_window.call(days),
                                    "{window.label}"
                                }
                            }
                        }
                    }
                }

                div { class: "flex space-x-1 mb-8",
                    TabButton {
                        tab: AnalyticsTab::Overview,
                        current: tab,
                        icon: IconName::TrendingUp,
                        on_tab,
                    }
                    TabButton {
                        tab: AnalyticsTab::Revenue,
                        current: tab,
                        icon: IconName::DollarSign,
                        on_tab,
                    }
                    TabButton {
                        tab: AnalyticsTab::Engagement,
                        current: tab,
                        icon: IconName::Users,
                        on_tab,
                    }
                }

                div {
                    class: "{panel_class}",
                    onanimationend: move |_| on_tab_animation_end.call(()),
                    match tab {
                        AnalyticsTab::Overview => rsx! {
                            OverviewPanel {
                                kpis_total_events: kpis.total_events,
                                kpis_published_events: kpis.published_events,
                                kpis_total_registrations: kpis.total_registrations,
                                kpis_avg_registrations: kpis.avg_registrations,
                                kpis_estimated_revenue: kpis.estimated_revenue,
                                kpis_completed_events: kpis.completed_events,
                                kpis_cancelled_events: kpis.cancelled_events,
                                spark_path: visitor_spark.path.clone(),
                                spark_empty: visitor_spark.empty,
                                overview,
                                performance,
                                visitors,
                                visitors_daily,
                                popular_pages,
                                conversion,
                                events_engagement,
                            }
                        },
                        AnalyticsTab::Revenue => rsx! {
                            RevenuePanel { revenue }
                        },
                        AnalyticsTab::Engagement => rsx! {
                            EngagementPanel { members }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn TabButton(
    tab: AnalyticsTab,
    current: AnalyticsTab,
    icon: IconName,
    on_tab: EventHandler<AnalyticsTab>,
) -> Element {
    let active = tab == current;
    let class = if active {
        "flex items-center gap-2 px-6 py-3 rounded-lg transition-colors bg-luxury-gold text-luxury-midnight-black font-medium"
    } else {
        "flex items-center gap-2 px-6 py-3 rounded-lg transition-colors text-luxury-platinum hover:bg-luxury-gold/10"
    };
    rsx! {
        button {
            id: "admin-analytics-tab-{tab.key()}",
            r#type: "button",
            class,
            onclick: move |_| on_tab.call(tab),
            Icon { name: icon, class: "h-4 w-4".to_string() }
            "{tab.label()}"
        }
    }
}

#[component]
fn OverviewPanel(
    kpis_total_events: f64,
    kpis_published_events: f64,
    kpis_total_registrations: f64,
    kpis_avg_registrations: f64,
    kpis_estimated_revenue: f64,
    kpis_completed_events: f64,
    kpis_cancelled_events: f64,
    spark_path: String,
    spark_empty: bool,
    overview: Option<EventsOverview>,
    performance: Option<EventsPerformance>,
    visitors: Option<VisitorsOverview>,
    visitors_daily: Vec<DailyVisitor>,
    popular_pages: Vec<PopularPage>,
    conversion: Option<ConversionStats>,
    events_engagement: Option<EventsEngagement>,
) -> Element {
    let popular = overview
        .as_ref()
        .map(|item| item.popular_events.clone())
        .unwrap_or_default();
    let occupancy = occupancy_points(&popular);
    let performance_points = performance
        .as_ref()
        .map(|item| performance_occupancy_points(&item.events))
        .unwrap_or_default();
    let daily_points = daily_visitor_points(&visitors_daily);
    let page_points = popular_page_points(&popular_pages);
    let conversion_pts = conversion
        .as_ref()
        .map(conversion_points)
        .unwrap_or_default();
    let engagement_pts = events_engagement
        .as_ref()
        .map(|item| events_engagement_points(&item.engagement))
        .unwrap_or_default();
    rsx! {
        div { id: "admin-analytics-overview", class: "space-y-8",
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                MetricCard {
                    icon: IconName::Calendar,
                    icon_wrap: "w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center".to_string(),
                    icon_class: "h-6 w-6 text-blue-400".to_string(),
                    eyebrow: "本月".to_string(),
                    value: format_whole(kpis_total_events),
                    label: "總活動數".to_string(),
                    hint_class: "text-green-400 text-xs mt-2".to_string(),
                    hint: format!("已發布: {}", format_whole(kpis_published_events)),
                }
                MetricCard {
                    icon: IconName::Users,
                    icon_wrap: "w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center".to_string(),
                    icon_class: "h-6 w-6 text-green-400".to_string(),
                    eyebrow: "累計".to_string(),
                    value: format_whole(kpis_total_registrations),
                    label: "總報名數".to_string(),
                    hint_class: "text-green-400 text-xs mt-2".to_string(),
                    hint: format!("平均: {} 人/活動", format_whole(kpis_avg_registrations)),
                }
                MetricCard {
                    icon: IconName::DollarSign,
                    icon_wrap: "w-12 h-12 bg-luxury-gold/20 rounded-lg flex items-center justify-center".to_string(),
                    icon_class: "h-6 w-6 text-luxury-gold".to_string(),
                    eyebrow: "預估".to_string(),
                    value: format_currency(kpis_estimated_revenue),
                    label: "營收".to_string(),
                    hint_class: "text-green-400 text-xs mt-2".to_string(),
                    hint: "本月預估".to_string(),
                }
                MetricCard {
                    icon: IconName::Activity,
                    icon_wrap: "w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center".to_string(),
                    icon_class: "h-6 w-6 text-purple-400".to_string(),
                    eyebrow: "狀態".to_string(),
                    value: format_whole(kpis_completed_events),
                    label: "已完成".to_string(),
                    hint_class: "text-red-400 text-xs mt-2".to_string(),
                    hint: format!("取消: {}", format_whole(kpis_cancelled_events)),
                }
            }

            div { class: "luxury-glass p-6 rounded-2xl",
                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "月度趨勢" }
                EmptyChart { id: "admin-analytics-empty-trends".to_string(), title: "月度趨勢".to_string() }
            }

            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-8",
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "熱門活動" }
                    ChartBlock {
                        id: "admin-analytics-chart-occupancy".to_string(),
                        title: "熱門活動滿座率".to_string(),
                        kind: ChartKind::Bar,
                        points: occupancy.clone(),
                    }
                    div { class: "space-y-4 mt-4",
                        if popular.is_empty() {
                            p { id: "admin-analytics-empty-events", class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                        }
                        for (index, event) in popular.iter().take(5).enumerate() {
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                                div { class: "flex items-center space-x-3",
                                    div { class: "w-8 h-8 bg-luxury-gold/20 rounded-full flex items-center justify-center text-luxury-gold font-medium text-sm",
                                        "{index + 1}"
                                    }
                                    div {
                                        div { class: "text-luxury-platinum font-medium text-sm", "{event.name}" }
                                        div { class: "text-luxury-platinum/60 text-xs",
                                            "{format_whole(event.current_attendees)}/{format_whole(event.capacity)} 人"
                                        }
                                    }
                                }
                                div { class: "text-right",
                                    div { class: "text-luxury-gold font-medium text-sm",
                                        "{format_percentage(event.occupancy_rate)}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "類別表現" }
                    EmptyChart { id: "admin-analytics-empty-categories".to_string(), title: "類別表現".to_string() }
                }
            }

            if !performance_points.is_empty() {
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "活動表現" }
                    ChartBlock {
                        id: "admin-analytics-chart-performance".to_string(),
                        title: "活動表現滿座率".to_string(),
                        kind: ChartKind::Bar,
                        points: performance_points,
                    }
                }
            }

            if visitors.is_some() || !daily_points.is_empty() {
                div { class: "luxury-glass p-6 rounded-2xl",
                    div { class: "flex items-center justify-between mb-6",
                        h3 { class: "text-xl font-luxury font-semibold text-luxury-gold", "訪客趨勢" }
                        if !spark_empty {
                            div {
                                role: "img",
                                aria_label: "訪客趨勢 sparkline",
                                svg {
                                    class: "text-luxury-gold",
                                    width: "{SPARKLINE_WIDTH}",
                                    height: "{SPARKLINE_HEIGHT}",
                                    view_box: "0 0 {SPARKLINE_WIDTH} {SPARKLINE_HEIGHT}",
                                    title { "訪客趨勢 sparkline" }
                                    path {
                                        d: "{spark_path}",
                                        fill: "none",
                                        stroke: "#d4af37",
                                        stroke_width: "2",
                                    }
                                }
                            }
                        }
                    }
                    if let Some(visitors) = visitors.as_ref() {
                        div { class: "grid grid-cols-2 md:grid-cols-4 gap-4 mb-6 text-sm",
                            VisitorStat { label: "獨立訪客".to_string(), value: format_whole(visitors.unique_visitors) }
                            VisitorStat { label: "頁面瀏覽".to_string(), value: format_whole(visitors.total_page_views) }
                            VisitorStat { label: "新訪客".to_string(), value: format_whole(visitors.new_visitors) }
                            VisitorStat { label: "平均頁數".to_string(), value: format_one_decimal(visitors.avg_pages_per_visitor) }
                        }
                    }
                    ChartBlock {
                        id: "admin-analytics-chart-visitors".to_string(),
                        title: "每日訪客".to_string(),
                        kind: ChartKind::Line,
                        points: daily_points,
                    }
                }
            }

            if !page_points.is_empty() {
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "熱門頁面" }
                    ChartBlock {
                        id: "admin-analytics-chart-pages".to_string(),
                        title: "熱門頁面".to_string(),
                        kind: ChartKind::Bar,
                        points: page_points,
                    }
                }
            }

            if let Some(conversion) = conversion {
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "轉換漏斗" }
                    ChartBlock {
                        id: "admin-analytics-chart-conversion".to_string(),
                        title: "轉換漏斗".to_string(),
                        kind: ChartKind::Donut,
                        points: conversion_pts,
                    }
                    p { class: "text-luxury-gold text-sm mt-4",
                        "轉換率 {format_percentage(conversion.conversion_rate)}"
                    }
                }
            }

            if events_engagement.is_some() {
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "活動頁面參與" }
                    ChartBlock {
                        id: "admin-analytics-chart-event-engagement".to_string(),
                        title: "活動頁面參與".to_string(),
                        kind: ChartKind::Line,
                        points: engagement_pts,
                    }
                }
            }
        }
    }
}

#[component]
fn RevenuePanel(revenue: Option<RevenueData>) -> Element {
    let monthly = revenue
        .as_ref()
        .map(|item| item.monthly_revenue.clone())
        .unwrap_or_default();
    let categories = revenue
        .as_ref()
        .map(|item| item.category_revenue.clone())
        .unwrap_or_default();
    let tiers = revenue
        .as_ref()
        .map(|item| item.tier_revenue.clone())
        .unwrap_or_default();
    let monthly_points = monthly_revenue_points(&monthly);
    let category_points = category_revenue_points(&categories);
    let tier_points = tier_revenue_points(&tiers);
    rsx! {
        div { id: "admin-analytics-revenue", class: "space-y-8",
            div { class: "luxury-glass p-6 rounded-2xl",
                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "月度營收趨勢" }
                ChartBlock {
                    id: "admin-analytics-chart-revenue".to_string(),
                    title: "月度營收趨勢".to_string(),
                    kind: ChartKind::Bar,
                    points: monthly_points,
                }
                div { class: "space-y-4 mt-4",
                    if monthly.is_empty() {
                        p { id: "admin-analytics-empty-revenue", class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                    }
                    for month in monthly.iter().take(6) {
                        div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                            div { class: "text-luxury-platinum font-medium", "{format_year_month(&month.month)}" }
                            div { class: "flex items-center space-x-6 text-sm",
                                div { class: "text-center",
                                    div { class: "text-luxury-gold font-medium", "{format_currency(month.revenue)}" }
                                    div { class: "text-luxury-platinum/60", "營收" }
                                }
                                div { class: "text-center",
                                    div { class: "text-luxury-gold font-medium", "{format_whole(month.event_count)}" }
                                    div { class: "text-luxury-platinum/60", "活動數" }
                                }
                                div { class: "text-center",
                                    div { class: "text-luxury-gold font-medium", "{format_whole(month.total_registrations)}" }
                                    div { class: "text-luxury-platinum/60", "報名數" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-8",
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "類別營收" }
                    ChartBlock {
                        id: "admin-analytics-chart-category-revenue".to_string(),
                        title: "類別營收".to_string(),
                        kind: ChartKind::Bar,
                        points: category_points,
                    }
                    div { class: "space-y-4 mt-4",
                        if categories.is_empty() {
                            p { class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                        }
                        for category in categories.iter() {
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                                div {
                                    div { class: "text-luxury-platinum font-medium text-sm", "{category.category}" }
                                    div { class: "text-luxury-platinum/60 text-xs",
                                        "{format_whole(category.event_count)} 個活動"
                                    }
                                }
                                div { class: "text-right",
                                    div { class: "text-luxury-gold font-medium text-sm",
                                        "{format_currency(category.revenue)}"
                                    }
                                    div { class: "text-luxury-platinum/60 text-xs",
                                        "平均 {format_currency(category.avg_revenue_per_event)}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "會員等級營收" }
                    ChartBlock {
                        id: "admin-analytics-chart-tier-revenue".to_string(),
                        title: "會員等級營收".to_string(),
                        kind: ChartKind::Donut,
                        points: tier_points,
                    }
                    div { class: "space-y-4 mt-4",
                        if tiers.is_empty() {
                            p { class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                        }
                        for tier in tiers.iter() {
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                                div { class: "flex items-center space-x-3",
                                    div { class: "w-3 h-3 rounded-full {membership_tier_dot_class(&tier.membership_tier)}" }
                                    div {
                                        div { class: "text-luxury-platinum font-medium text-sm",
                                            "{tier.membership_tier}"
                                        }
                                        div { class: "text-luxury-platinum/60 text-xs",
                                            "{format_whole(tier.registration_count)} 次報名"
                                        }
                                    }
                                }
                                div { class: "text-luxury-gold font-medium text-sm",
                                    "{format_currency(tier.total_revenue)}"
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
fn EngagementPanel(members: Option<MembersEngagement>) -> Element {
    let engagement = members
        .as_ref()
        .map(|item| item.engagement.clone())
        .unwrap_or_default();
    let top_members = members
        .as_ref()
        .map(|item| item.top_members.clone())
        .unwrap_or_default();
    let retention = members
        .as_ref()
        .map(|item| item.retention.clone())
        .unwrap_or_default();
    let rate_points = engagement_rate_points(&engagement);
    let retention_pts = retention_points(&retention);
    rsx! {
        div { id: "admin-analytics-engagement", class: "space-y-8",
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                if engagement.is_empty() {
                    p { id: "admin-analytics-empty-engagement", class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                }
                for tier in engagement.iter() {
                    div { class: "luxury-glass p-6 rounded-2xl",
                        div { class: "flex items-center justify-between mb-4",
                            h4 { class: "text-lg font-medium text-luxury-gold", "{tier.membership_tier}" }
                            div { class: "w-3 h-3 rounded-full {membership_tier_dot_class(&tier.membership_tier)}" }
                        }
                        div { class: "space-y-3",
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80 text-sm", "總會員" }
                                span { class: "text-luxury-platinum font-medium", "{format_whole(tier.total_members)}" }
                            }
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80 text-sm", "活躍會員" }
                                span { class: "text-luxury-platinum font-medium", "{format_whole(tier.active_members)}" }
                            }
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80 text-sm", "參與率" }
                                span { class: "text-luxury-gold font-medium", "{format_percentage(tier.engagement_rate)}" }
                            }
                            div { class: "flex justify-between",
                                span { class: "text-luxury-platinum/80 text-sm", "平均活動數" }
                                span { class: "text-luxury-platinum font-medium", "{format_one_decimal(tier.avg_events_per_member)}" }
                            }
                        }
                    }
                }
            }
            ChartBlock {
                id: "admin-analytics-chart-engagement".to_string(),
                title: "會員參與率".to_string(),
                kind: ChartKind::Donut,
                points: rate_points,
            }
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-8",
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "最活躍會員" }
                    div { class: "space-y-4",
                        if top_members.is_empty() {
                            p { class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                        }
                        for (index, member) in top_members.iter().take(8).enumerate() {
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                                div { class: "flex items-center space-x-3",
                                    div { class: "w-8 h-8 bg-luxury-gold/20 rounded-full flex items-center justify-center text-luxury-gold font-medium text-sm",
                                        "{index + 1}"
                                    }
                                    div {
                                        div { class: "text-luxury-platinum font-medium text-sm",
                                            "{member.first_name} {member.last_name}"
                                        }
                                        div { class: "text-luxury-platinum/60 text-xs", "{member.membership_tier}" }
                                    }
                                }
                                div { class: "text-right",
                                    div { class: "text-luxury-gold font-medium text-sm",
                                        "{format_whole(member.events_attended)} 活動"
                                    }
                                    div { class: "text-luxury-platinum/60 text-xs",
                                        "{format_currency(member.total_spent)}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl",
                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "會員留存率" }
                    ChartBlock {
                        id: "admin-analytics-chart-retention".to_string(),
                        title: "會員留存率".to_string(),
                        kind: ChartKind::Bar,
                        points: retention_pts,
                    }
                    div { class: "space-y-4 mt-4",
                        if retention.is_empty() {
                            p { class: "text-luxury-platinum/60 text-sm", "{EMPTY_CHART_LABEL}" }
                        }
                        for cohort in retention.iter().take(6) {
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                                div {
                                    div { class: "text-luxury-platinum font-medium text-sm",
                                        "{format_year_month(&cohort.cohort_month)}"
                                    }
                                    div { class: "text-luxury-platinum/60 text-xs",
                                        "{format_whole(cohort.cohort_size)} 新會員"
                                    }
                                }
                                div { class: "text-right",
                                    div { class: "text-luxury-gold font-medium text-sm",
                                        "{format_percentage(cohort.retention_rate)}"
                                    }
                                    div { class: "text-luxury-platinum/60 text-xs",
                                        "{format_whole(cohort.active_this_month)} 活躍"
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChartKind {
    Bar,
    Line,
    Donut,
}

#[component]
fn ChartBlock(id: String, title: String, kind: ChartKind, points: Vec<ChartPoint>) -> Element {
    let aria = chart_aria_label(&title, &points);
    if points.is_empty() {
        return rsx! {
            EmptyChart { id, title }
        };
    }
    match kind {
        ChartKind::Bar => {
            let geom = bar_chart(&points, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
            rsx! {
                div { role: "img", aria_label: "{aria}",
                    svg {
                        id: "{id}",
                        class: "w-full h-52 text-luxury-gold",
                        view_box: "0 0 {geom.width} {geom.height}",
                        title { "{aria}" }
                        for tick in geom.ticks.iter() {
                            path {
                                d: "M{geom.pad.left} {tick.y} H{geom.width - geom.pad.right}",
                                stroke: "rgba(212,175,55,0.18)",
                                fill: "none",
                            }
                            text {
                                x: "{geom.pad.left - 8.0}",
                                y: "{tick.y + 3.0}",
                                text_anchor: "end",
                                fill: "#c5c6c7",
                                font_size: "10",
                                "{tick.label}"
                            }
                        }
                        for bar in geom.bars.iter() {
                            rect {
                                x: "{bar.x}",
                                y: "{bar.y}",
                                width: "{bar.width}",
                                height: "{bar.height}",
                                fill: "#d4af37",
                                rx: "2",
                            }
                            text {
                                x: "{bar.label_x}",
                                y: "{bar.label_y}",
                                text_anchor: "middle",
                                fill: "#c5c6c7",
                                font_size: "10",
                                "{bar.label}"
                            }
                        }
                    }
                }
            }
        }
        ChartKind::Line => {
            let geom = line_chart(&points, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
            rsx! {
                div { role: "img", aria_label: "{aria}",
                    svg {
                        id: "{id}",
                        class: "w-full h-52 text-luxury-gold",
                        view_box: "0 0 {geom.width} {geom.height}",
                        title { "{aria}" }
                        for tick in geom.ticks.iter() {
                            path {
                                d: "M{geom.pad.left} {tick.y} H{geom.width - geom.pad.right}",
                                stroke: "rgba(212,175,55,0.18)",
                                fill: "none",
                            }
                        }
                        path {
                            d: "{geom.path}",
                            fill: "none",
                            stroke: "#d4af37",
                            stroke_width: "2",
                        }
                        for point in geom.points.iter() {
                            circle {
                                cx: "{point.x}",
                                cy: "{point.y}",
                                r: "3",
                                fill: "#d4af37",
                            }
                        }
                    }
                }
            }
        }
        ChartKind::Donut => {
            let geom = donut_chart(&points, DONUT_SIZE / 2.0, DONUT_SIZE / 2.0, 48.0, 80.0);
            rsx! {
                div { class: "flex flex-col md:flex-row items-center gap-6", role: "img", aria_label: "{aria}",
                    svg {
                        id: "{id}",
                        class: "w-48 h-48",
                        view_box: "0 0 {DONUT_SIZE} {DONUT_SIZE}",
                        title { "{aria}" }
                        if geom.empty {
                            circle {
                                cx: "{geom.cx}",
                                cy: "{geom.cy}",
                                r: "{geom.outer_radius}",
                                fill: "none",
                                stroke: "rgba(212,175,55,0.2)",
                                stroke_width: "16",
                            }
                        }
                        for slice in geom.slices.iter() {
                            path {
                                d: "{slice.path}",
                                fill: "{donut_color(slice.color_index)}",
                            }
                        }
                    }
                    div { class: "space-y-2 text-sm",
                        for slice in geom.slices.iter() {
                            div { class: "flex items-center gap-2 text-luxury-platinum",
                                span {
                                    class: "inline-block w-3 h-3 rounded-sm",
                                    style: "background:{donut_color(slice.color_index)}",
                                }
                                span { "{slice.label}" }
                                span { class: "text-luxury-gold", "{format_percentage(slice.percent)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyChart(id: String, title: String) -> Element {
    let aria = chart_aria_label(&title, &[]);
    rsx! {
        div {
            id: "{id}",
            class: "flex items-center justify-center h-40 rounded-xl border border-luxury-gold/10 text-luxury-platinum/60 text-sm",
            role: "img",
            aria_label: "{aria}",
            "{EMPTY_CHART_LABEL}"
        }
    }
}

#[component]
fn MetricCard(
    icon: IconName,
    icon_wrap: String,
    icon_class: String,
    eyebrow: String,
    value: String,
    label: String,
    hint_class: String,
    hint: String,
) -> Element {
    rsx! {
        div { class: "luxury-glass p-6 rounded-2xl",
            div { class: "flex items-center justify-between mb-4",
                div { class: "{icon_wrap}",
                    Icon { name: icon, class: icon_class }
                }
                span { class: "text-luxury-platinum/60 text-sm", "{eyebrow}" }
            }
            div { class: "text-2xl font-bold text-luxury-gold mb-1", "{value}" }
            div { class: "text-luxury-platinum/80 text-sm", "{label}" }
            div { class: "{hint_class}", "{hint}" }
        }
    }
}

#[component]
fn VisitorStat(label: String, value: String) -> Element {
    rsx! {
        div {
            div { class: "text-luxury-platinum/60", "{label}" }
            div { class: "text-luxury-gold font-medium", "{value}" }
        }
    }
}
