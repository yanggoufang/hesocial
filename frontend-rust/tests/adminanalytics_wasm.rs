#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::adminanalytics::{
    AnalyticsTab, ConversionStats, DailyVisitor, EventStats, EventsEngagement, EventsOverview,
    EventsPerformance, MembersEngagement, MonthlyRevenue, PerformanceEvent, PopularEvent,
    PopularPage, RegistrationStats, RevenueData, TierEngagement, TopMember, VisitorsOverview,
};
use hesocial_frontend::pages::adminanalytics::AdminAnalyticsScreen;
use hesocial_frontend::shell::Presence;
use wasm_bindgen_test::wasm_bindgen_test;

fn sample_overview() -> EventsOverview {
    EventsOverview {
        period_days: 30,
        event_stats: EventStats {
            total_events: 12.0,
            recent_events: 4.0,
            upcoming_events: 4.0,
            past_events: 8.0,
            avg_occupancy_rate: 62.5,
        },
        registration_stats: RegistrationStats {
            total_registrations: 48.0,
            recent_registrations: 48.0,
            unique_attendees: 40.0,
        },
        popular_events: vec![PopularEvent {
            id: "7".into(),
            name: "松露季私宴".into(),
            capacity: 20.0,
            current_attendees: 16.0,
            occupancy_rate: 80.0,
            ..PopularEvent::default()
        }],
    }
}

fn sample_revenue() -> RevenueData {
    RevenueData {
        monthly_revenue: vec![MonthlyRevenue {
            month: "2025-07".into(),
            revenue: 180000.0,
            event_count: 3.0,
            total_registrations: 12.0,
        }],
        category_revenue: vec![hesocial_frontend::adminanalytics::CategoryRevenue {
            category: "私人晚宴".into(),
            revenue: 90000.0,
            event_count: 2.0,
            avg_revenue_per_event: 45000.0,
        }],
        tier_revenue: vec![hesocial_frontend::adminanalytics::TierRevenue {
            membership_tier: "Black Card".into(),
            registration_count: 4.0,
            total_revenue: 60000.0,
        }],
    }
}

fn sample_members() -> MembersEngagement {
    MembersEngagement {
        engagement: vec![TierEngagement {
            membership_tier: "Black Card".into(),
            total_members: 10.0,
            active_members: 8.0,
            engagement_rate: 80.0,
            avg_events_per_member: 2.5,
        }],
        top_members: vec![TopMember {
            first_name: "Ada".into(),
            last_name: "Lovelace".into(),
            membership_tier: "Diamond".into(),
            events_attended: 6.0,
            total_spent: 90000.0,
        }],
        retention: vec![hesocial_frontend::adminanalytics::RetentionCohort {
            cohort_month: "2025-07".into(),
            cohort_size: 12.0,
            active_this_month: 12.0,
            retention_rate: 100.0,
        }],
    }
}

#[component]
fn AnalyticsAt(
    loading: bool,
    error: Option<String>,
    tab: AnalyticsTab,
    overview: Option<EventsOverview>,
    performance: Option<EventsPerformance>,
    revenue: Option<RevenueData>,
    members: Option<MembersEngagement>,
    visitors: Option<VisitorsOverview>,
    visitors_daily: Vec<DailyVisitor>,
    popular_pages: Vec<PopularPage>,
    conversion: Option<ConversionStats>,
    events_engagement: Option<EventsEngagement>,
) -> Element {
    rsx! {
        AdminAnalyticsScreen {
            loading,
            refreshing: false,
            error,
            tab,
            tab_presence: Presence::Shown,
            date_window_days: 30,
            overview,
            performance,
            revenue,
            members,
            visitors,
            visitors_daily,
            popular_pages,
            conversion,
            events_engagement,
        }
    }
}

fn render_screen(
    loading: bool,
    error: Option<String>,
    tab: AnalyticsTab,
    overview: Option<EventsOverview>,
    revenue: Option<RevenueData>,
    members: Option<MembersEngagement>,
) -> String {
    render_full(
        loading,
        error,
        tab,
        overview,
        None,
        revenue,
        members,
        None,
        Vec::new(),
        Vec::new(),
        None,
        None,
    )
}

fn render_full(
    loading: bool,
    error: Option<String>,
    tab: AnalyticsTab,
    overview: Option<EventsOverview>,
    performance: Option<EventsPerformance>,
    revenue: Option<RevenueData>,
    members: Option<MembersEngagement>,
    visitors: Option<VisitorsOverview>,
    visitors_daily: Vec<DailyVisitor>,
    popular_pages: Vec<PopularPage>,
    conversion: Option<ConversionStats>,
    events_engagement: Option<EventsEngagement>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        AnalyticsAt,
        AnalyticsAtProps {
            loading,
            error,
            tab,
            overview,
            performance,
            revenue,
            members,
            visitors,
            visitors_daily,
            popular_pages,
            conversion,
            events_engagement,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn loading_copy_matches_react() {
    let html = render_screen(true, None, AnalyticsTab::Overview, None, None, None);
    assert!(
        html.contains("id=\"admin-analytics-loading\""),
        "loading id missing: {html}"
    );
    assert!(
        html.contains("載入分析數據中..."),
        "loading copy missing: {html}"
    );
    assert!(
        !html.contains("分析儀表板"),
        "heading must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn error_state_shows_retry() {
    let html = render_screen(
        false,
        Some("Access token required".into()),
        AnalyticsTab::Overview,
        None,
        None,
        None,
    );
    assert!(html.contains("id=\"admin-analytics-error\""));
    assert!(html.contains("載入失敗"));
    assert!(html.contains("Access token required"));
    assert!(html.contains("重新整理"));
}

#[wasm_bindgen_test]
fn empty_overview_has_accessible_chart_fallback() {
    let html = render_screen(
        false,
        None,
        AnalyticsTab::Overview,
        Some(EventsOverview::default()),
        None,
        None,
    );
    assert!(html.contains("id=\"admin-analytics-title\""));
    assert!(html.contains("分析儀表板"));
    assert!(html.contains("平台營運數據與會員活動分析"));
    assert!(html.contains("總覽"));
    assert!(html.contains("營收分析"));
    assert!(html.contains("會員參與"));
    assert!(html.contains("總活動數"));
    assert!(html.contains("總報名數"));
    assert!(html.contains("營收"));
    assert!(html.contains("已完成"));
    assert!(html.contains("月度趨勢"));
    assert!(html.contains("熱門活動"));
    assert!(html.contains("類別表現"));
    assert!(html.contains("目前沒有資料"));
    assert!(html.contains("id=\"admin-analytics-empty-trends\""));
    assert!(
        html.contains("aria-label=\"月度趨勢：目前沒有資料\"")
            || html.contains("月度趨勢：目前沒有資料")
    );
    assert!(html.contains("重新整理"));
    assert!(html.contains("匯出報告"));
    assert!(html.contains("7 天"));
    assert!(html.contains("30 天"));
}

#[wasm_bindgen_test]
fn populated_overview_renders_lists_and_svg() {
    let html = render_full(
        false,
        None,
        AnalyticsTab::Overview,
        Some(sample_overview()),
        Some(EventsPerformance {
            period_days: 30,
            events: vec![PerformanceEvent {
                name: "Yacht Night".into(),
                occupancy_rate: 55.0,
                ..PerformanceEvent::default()
            }],
        }),
        Some(sample_revenue()),
        None,
        Some(VisitorsOverview {
            unique_visitors: 5.0,
            total_page_views: 20.0,
            new_visitors: 3.0,
            avg_pages_per_visitor: 4.0,
            ..VisitorsOverview::default()
        }),
        vec![DailyVisitor {
            date: "2026-08-31".into(),
            unique_visitors: 2.0,
            ..DailyVisitor::default()
        }],
        vec![PopularPage {
            path: "/events".into(),
            views: 9.0,
            unique_visitors: 4.0,
            conversion_rate: 0.25,
        }],
        Some(ConversionStats {
            total_visitors: 10.0,
            event_viewers: 4.0,
            registered_users: 1.0,
            conversion_rate: 10.0,
            ..ConversionStats::default()
        }),
        Some(EventsEngagement {
            period_days: 30,
            engagement: vec![Default::default()],
        }),
    );
    assert!(html.contains("松露季私宴"));
    assert!(html.contains("已發布: 4"));
    assert!(html.contains("本月預估"));
    assert!(html.contains("<svg"));
    assert!(html.contains("id=\"admin-analytics-chart-occupancy\""));
    assert!(html.contains("role=\"img\""));
    assert!(html.contains("訪客趨勢"));
    assert!(html.contains("熱門頁面"));
    assert!(html.contains("轉換漏斗"));
    assert!(html.contains("NT$180,000") || html.contains("NT$180,000"));
}

#[wasm_bindgen_test]
fn revenue_tab_populated_and_empty() {
    let populated = render_screen(
        false,
        None,
        AnalyticsTab::Revenue,
        None,
        Some(sample_revenue()),
        None,
    );
    assert!(populated.contains("id=\"admin-analytics-revenue\""));
    assert!(populated.contains("月度營收趨勢"));
    assert!(populated.contains("類別營收"));
    assert!(populated.contains("會員等級營收"));
    assert!(populated.contains("2025年7月"));
    assert!(populated.contains("私人晚宴"));
    assert!(populated.contains("Black Card"));
    assert!(populated.contains("id=\"admin-analytics-chart-revenue\""));
    assert!(populated.contains("<svg"));

    let empty = render_screen(false, None, AnalyticsTab::Revenue, None, None, None);
    assert!(empty.contains("月度營收趨勢"));
    assert!(empty.contains("目前沒有資料"));
}

#[wasm_bindgen_test]
fn engagement_tab_populated_and_empty() {
    let populated = render_screen(
        false,
        None,
        AnalyticsTab::Engagement,
        None,
        None,
        Some(sample_members()),
    );
    assert!(populated.contains("id=\"admin-analytics-engagement\""));
    assert!(populated.contains("總會員"));
    assert!(populated.contains("活躍會員"));
    assert!(populated.contains("參與率"));
    assert!(populated.contains("平均活動數"));
    assert!(populated.contains("最活躍會員"));
    assert!(populated.contains("會員留存率"));
    assert!(populated.contains("Ada Lovelace"));
    assert!(populated.contains("6 活動"));
    assert!(populated.contains("12 新會員"));
    assert!(populated.contains("id=\"admin-analytics-chart-retention\""));

    let empty = render_screen(false, None, AnalyticsTab::Engagement, None, None, None);
    assert!(empty.contains("目前沒有資料"));
}
