#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::adminanalytics::{
    ADMIN_ROUTE_FALLBACK, AnalyticsPhase, AnalyticsTab, CHART_HEIGHT, CHART_WIDTH,
    CONVERSION_FALLBACK, ConversionStats, DATE_WINDOWS, DEFAULT_DATE_WINDOW_DAYS, DEFAULT_PAD,
    DONUT_SIZE, DailyVisitor, DashboardData, EMPTY_CHART_LABEL, EVENT_DETAIL_FALLBACK,
    EVENTS_ENGAGEMENT_PATH, EVENTS_OVERVIEW_PATH, EVENTS_PERFORMANCE_PATH, EventsOverview,
    MEMBERS_ENGAGEMENT_PATH, NETWORK_ERROR, OVERVIEW_FALLBACK, POPULAR_PAGES_LIMIT,
    POPULAR_PAGES_PATH, PerformanceEvent, PopularEvent, REVENUE_EVENTS_PATH, SPARKLINE_HEIGHT,
    SPARKLINE_WIDTH, UNAUTHORIZED, VISITORS_DAILY_PATH, VISITORS_PATH, admin_route_guard,
    analytics_phase, bar_chart, chart_aria_label, chart_points, conversion_points, conversion_url,
    daily_visitor_points, dashboard_has_payload, days_query, donut_chart, event_performance_url,
    events_engagement_url, events_overview_url, events_performance_url, export_endpoint_available,
    fetch_dashboard, fetch_event_performance, format_currency, format_one_decimal,
    format_percentage, format_tick, format_whole, format_year_month, is_active_date_window,
    line_chart, membership_tier_dot_class, monthly_revenue_points, occupancy_points, overview_kpis,
    parse_conversion_response, parse_event_performance_detail_response,
    parse_events_engagement_response, parse_events_overview_response,
    parse_events_performance_response, parse_members_engagement_response,
    parse_popular_pages_response, parse_revenue_response, parse_visitors_daily_response,
    parse_visitors_overview_response, period_days, popular_pages_url, scale_max,
    select_date_window, series_max, slice_percent, sparkline, visitors_daily_url, visitors_url,
    y_ticks,
};
use hesocial_frontend::permissions::{AuthSnapshot, Role, RouteGuard};

fn overview_body() -> &'static str {
    r#"{
        "success": true,
        "data": {
            "period_days": 30,
            "event_stats": {
                "total_events": 12,
                "recent_events": 4,
                "upcoming_events": 4,
                "past_events": 8,
                "avg_occupancy_rate": 62.5
            },
            "registration_stats": {
                "total_registrations": 48,
                "recent_registrations": 48,
                "unique_attendees": 40
            },
            "popular_events": [
                {
                    "id": 7,
                    "name": "松露季私宴",
                    "date_time": "2026-10-01T18:00:00Z",
                    "capacity": 20,
                    "current_attendees": 16,
                    "occupancy_rate": 80
                }
            ]
        }
    }"#
}

fn revenue_body() -> &'static str {
    r#"{
        "success": true,
        "data": {
            "monthlyRevenue": [
                {
                    "month": "2025-07",
                    "event_count": 3,
                    "total_registrations": 12,
                    "revenue": 180000
                }
            ],
            "categoryRevenue": [
                {
                    "category": "私人晚宴",
                    "revenue": 90000,
                    "event_count": 2,
                    "avg_revenue_per_event": 45000
                }
            ],
            "tierRevenue": [
                {
                    "membership_tier": "Black Card",
                    "registration_count": 4,
                    "total_revenue": 60000
                },
                {
                    "membership_tier": "Diamond",
                    "registration_count": 5,
                    "total_revenue": 75000
                }
            ]
        }
    }"#
}

fn members_body() -> &'static str {
    r#"{
        "success": true,
        "data": {
            "engagement": [
                {
                    "membership_tier": "Black Card",
                    "total_members": 10,
                    "active_members": 8,
                    "engagement_rate": 80,
                    "avg_events_per_member": 2.5
                }
            ],
            "topMembers": [
                {
                    "first_name": "Ada",
                    "last_name": "Lovelace",
                    "membership_tier": "Diamond",
                    "events_attended": 6,
                    "total_spent": 90000
                }
            ],
            "retention": [
                {
                    "cohort_month": "2025-07",
                    "cohort_size": 12,
                    "active_this_month": 12,
                    "retention_rate": 100.0
                }
            ]
        }
    }"#
}

#[test]
fn admin_route_guard_three_states() {
    assert_eq!(ADMIN_ROUTE_FALLBACK, "/login");
    assert_eq!(
        admin_route_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading
    );
    assert_eq!(
        admin_route_guard(false, &AuthSnapshot::default()),
        RouteGuard::Redirect("/login")
    );
    let user = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        ..AuthSnapshot::default()
    };
    assert_eq!(
        admin_route_guard(false, &user),
        RouteGuard::Redirect("/login")
    );
    let admin = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::Admin),
        ..AuthSnapshot::default()
    };
    assert_eq!(admin_route_guard(false, &admin), RouteGuard::Allow);
    let super_admin = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::SuperAdmin),
        ..AuthSnapshot::default()
    };
    assert_eq!(admin_route_guard(false, &super_admin), RouteGuard::Allow);
}

#[test]
fn date_window_selection_matches_backend_period_days() {
    assert_eq!(DEFAULT_DATE_WINDOW_DAYS, 30);
    assert_eq!(DATE_WINDOWS.len(), 3);
    assert_eq!(DATE_WINDOWS[0].label, "7 天");
    assert_eq!(select_date_window(None), 30);
    assert_eq!(select_date_window(Some("")), 30);
    assert_eq!(select_date_window(Some("7")), 7);
    assert_eq!(select_date_window(Some(" 90 ")), 90);
    assert_eq!(select_date_window(Some("7days")), 7);
    assert_eq!(select_date_window(Some("0")), 30);
    assert_eq!(select_date_window(Some("-5")), 1);
    assert_eq!(period_days(Some("abc"), 30), 30);
    assert!(is_active_date_window(30, 30));
    assert!(!is_active_date_window(7, 30));
    assert_eq!(days_query(7), "days=7");
}

#[test]
fn api_paths_match_worker_routes() {
    assert_eq!(
        events_overview_url(30),
        "/api/analytics/events/overview?days=30"
    );
    assert_eq!(
        events_performance_url(7),
        "/api/analytics/events/performance?days=7"
    );
    assert_eq!(
        event_performance_url("42"),
        "/api/analytics/events/42/performance"
    );
    assert_eq!(
        events_engagement_url(90),
        "/api/analytics/events/engagement?days=90"
    );
    assert_eq!(REVENUE_EVENTS_PATH, "/api/analytics/revenue/events");
    assert_eq!(MEMBERS_ENGAGEMENT_PATH, "/api/analytics/engagement/members");
    assert_eq!(visitors_url(30), "/api/analytics/visitors?days=30");
    assert_eq!(
        visitors_daily_url(7),
        "/api/analytics/visitors/daily?days=7"
    );
    assert_eq!(
        popular_pages_url(POPULAR_PAGES_LIMIT),
        "/api/analytics/pages/popular?limit=20"
    );
    assert_eq!(conversion_url(30), "/api/analytics/conversion?days=30");
    assert_eq!(EVENTS_OVERVIEW_PATH, "/api/analytics/events/overview");
    assert_eq!(EVENTS_PERFORMANCE_PATH, "/api/analytics/events/performance");
    assert_eq!(EVENTS_ENGAGEMENT_PATH, "/api/analytics/events/engagement");
    assert_eq!(VISITORS_PATH, "/api/analytics/visitors");
    assert_eq!(VISITORS_DAILY_PATH, "/api/analytics/visitors/daily");
    assert_eq!(POPULAR_PAGES_PATH, "/api/analytics/pages/popular");
    assert!(!export_endpoint_available());
}

#[test]
fn parse_overview_populated_and_numeric_ids() {
    let overview = parse_events_overview_response(200, overview_body()).expect("ok");
    assert_eq!(overview.period_days, 30);
    assert_eq!(overview.event_stats.total_events, 12.0);
    assert_eq!(overview.event_stats.recent_events, 4.0);
    assert_eq!(overview.event_stats.past_events, 8.0);
    assert_eq!(overview.registration_stats.total_registrations, 48.0);
    assert_eq!(overview.popular_events.len(), 1);
    assert_eq!(overview.popular_events[0].id, "7");
    assert_eq!(overview.popular_events[0].name, "松露季私宴");
    assert_eq!(overview.popular_events[0].occupancy_rate, 80.0);
}

#[test]
fn parse_overview_empty_object_and_empty_lists() {
    let body = r#"{
        "success": true,
        "data": {
            "period_days": 30,
            "event_stats": {},
            "registration_stats": {},
            "popular_events": []
        }
    }"#;
    let overview = parse_events_overview_response(200, body).expect("ok");
    assert_eq!(overview.event_stats.total_events, 0.0);
    assert!(overview.popular_events.is_empty());
}

#[test]
fn parse_overview_error_empty_and_401() {
    assert_eq!(
        parse_events_overview_response(200, "not-json").unwrap_err(),
        OVERVIEW_FALLBACK
    );
    assert_eq!(
        parse_events_overview_response(200, r#"{"success":false}"#).unwrap_err(),
        OVERVIEW_FALLBACK
    );
    assert_eq!(
        parse_events_overview_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_events_overview_response(401, "").unwrap_err(),
        UNAUTHORIZED
    );
    assert_eq!(
        parse_events_overview_response(500, r#"{"success":false,"error":"boom"}"#).unwrap_err(),
        "boom"
    );
    assert_eq!(
        parse_events_overview_response(0, "").unwrap_err(),
        OVERVIEW_FALLBACK
    );
}

#[test]
fn parse_performance_and_detail_shapes() {
    let body = r#"{
        "success": true,
        "data": {
            "period_days": 7,
            "events": [
                {
                    "id": "9",
                    "name": "Yacht Night",
                    "occupancy_rate": "55.5",
                    "cancelled_registrations": 2,
                    "current_attendees": 11,
                    "capacity": 20
                }
            ]
        }
    }"#;
    let performance = parse_events_performance_response(200, body).expect("ok");
    assert_eq!(performance.period_days, 7);
    assert_eq!(performance.events[0].occupancy_rate, 55.5);
    assert_eq!(performance.events[0].cancelled_registrations, 2.0);

    let empty = parse_events_performance_response(
        200,
        r#"{"success":true,"data":{"period_days":30,"events":[]}}"#,
    )
    .expect("ok");
    assert!(empty.events.is_empty());

    let detail_body = r#"{
        "success": true,
        "data": {
            "event": {
                "id": 42,
                "title": "Autumn Salon",
                "fill_rate": 40,
                "current_revenue": 80000,
                "potential_revenue": 200000,
                "category_name": "藝術沙龍",
                "venue_name": "Gallery",
                "current_registrations": 8,
                "capacity_max": 20
            },
            "registrationTimeline": [
                {"date": "2026-08-01", "registrations": 2, "cumulative_registrations": 2}
            ],
            "membershipBreakdown": [
                {"membership_tier": "Diamond", "count": 5, "percentage": 62.5}
            ],
            "statusBreakdown": [{"status": "confirmed", "count": 8}]
        }
    }"#;
    let detail = parse_event_performance_detail_response(200, detail_body).expect("ok");
    assert_eq!(detail.event.id, "42");
    assert_eq!(detail.event.title, "Autumn Salon");
    assert_eq!(detail.registration_timeline.len(), 1);
    assert_eq!(detail.membership_breakdown[0].percentage, 62.5);
    assert_eq!(
        parse_event_performance_detail_response(
            404,
            r#"{"success":false,"error":"Event not found"}"#
        )
        .unwrap_err(),
        "Event not found"
    );
    assert_eq!(
        parse_event_performance_detail_response(401, "").unwrap_err(),
        UNAUTHORIZED
    );
    assert_eq!(
        EVENT_DETAIL_FALLBACK,
        "Failed to fetch event performance data"
    );
}

#[test]
fn parse_revenue_and_members() {
    let revenue = parse_revenue_response(200, revenue_body()).expect("ok");
    assert_eq!(revenue.monthly_revenue[0].month, "2025-07");
    assert_eq!(revenue.monthly_revenue[0].revenue, 180000.0);
    assert_eq!(revenue.category_revenue[0].category, "私人晚宴");
    assert_eq!(revenue.tier_revenue[1].membership_tier, "Diamond");

    let empty = parse_revenue_response(
        200,
        r#"{"success":true,"data":{"monthlyRevenue":[],"categoryRevenue":[],"tierRevenue":[]}}"#,
    )
    .expect("ok");
    assert!(empty.monthly_revenue.is_empty());

    let members = parse_members_engagement_response(200, members_body()).expect("ok");
    assert_eq!(members.engagement[0].avg_events_per_member, 2.5);
    assert_eq!(members.top_members[0].first_name, "Ada");
    assert_eq!(members.retention[0].retention_rate, 100.0);
}

#[test]
fn parse_visitor_page_and_conversion_envelopes() {
    let visitors = parse_visitors_overview_response(
        200,
        r#"{"success":true,"data":{"period_days":7,"unique_visitors":"5","total_page_views":20,"converted_visitors":2,"avg_pages_per_visitor":4.0,"new_visitors":3}}"#,
    )
    .expect("ok");
    assert_eq!(visitors.unique_visitors, 5.0);
    assert_eq!(visitors.avg_pages_per_visitor, 4.0);

    let daily = parse_visitors_daily_response(
        200,
        r#"{"success":true,"data":[{"date":"2026-08-31","unique_visitors":2,"total_page_views":6,"converted_visitors":1,"avg_pages_per_visitor":3}]}"#,
    )
    .expect("ok");
    assert_eq!(daily[0].date, "2026-08-31");
    assert!(
        parse_visitors_daily_response(200, r#"{"success":true,"data":[]}"#)
            .expect("ok")
            .is_empty()
    );

    let pages = parse_popular_pages_response(
        200,
        r#"{"success":true,"data":[{"path":"/events","views":"9","unique_visitors":4,"conversion_rate":0.25}]}"#,
    )
    .expect("ok");
    assert_eq!(pages[0].views, 9.0);

    let conversion = parse_conversion_response(
        200,
        r#"{"success":true,"data":{"period_days":30,"total_visitors":3,"event_viewers":2,"registered_users":1,"conversion_rate":33.33}}"#,
    )
    .expect("ok");
    assert_eq!(conversion.conversion_rate, 33.33);

    let engagement = parse_events_engagement_response(
        200,
        r#"{"success":true,"data":{"period_days":30,"engagement":[{"date":"2026-08-31","unique_visitors":2,"total_page_views":6,"event_page_views":5,"registration_page_views":1,"avg_time_spent":42.5}]}}"#,
    )
    .expect("ok");
    assert_eq!(engagement.engagement[0].event_page_views, 5.0);

    assert_eq!(
        parse_conversion_response(401, r#"{"success":false,"error":"Invalid token"}"#).unwrap_err(),
        "Invalid token"
    );
    assert_eq!(
        parse_visitors_overview_response(200, r#"{"success":true}"#).unwrap_err(),
        "Failed to retrieve visitor analytics"
    );
    assert_eq!(
        CONVERSION_FALLBACK,
        "Failed to retrieve conversion analytics"
    );
}

#[test]
fn overview_kpis_use_handler_fields_not_react_hardcodes() {
    let overview = parse_events_overview_response(200, overview_body()).expect("ok");
    let revenue = parse_revenue_response(200, revenue_body()).expect("ok");
    let performance = cancelled_performance();
    let kpis = overview_kpis(Some(&overview), Some(&revenue), Some(&performance));
    assert_eq!(kpis.total_events, 12.0);
    assert_eq!(kpis.published_events, 4.0);
    assert_eq!(kpis.total_registrations, 48.0);
    assert_eq!(kpis.avg_registrations, 4.0);
    assert_eq!(kpis.estimated_revenue, 180000.0);
    assert_eq!(kpis.completed_events, 8.0);
    assert_eq!(kpis.cancelled_events, 3.0);
    let empty = overview_kpis(None, None, None);
    assert_eq!(empty.total_events, 0.0);
    assert_eq!(empty.estimated_revenue, 0.0);
}

fn cancelled_performance() -> hesocial_frontend::adminanalytics::EventsPerformance {
    hesocial_frontend::adminanalytics::EventsPerformance {
        period_days: 30,
        events: vec![PerformanceEvent {
            cancelled_registrations: 3.0,
            ..PerformanceEvent::default()
        }],
    }
}

#[test]
fn formatters_match_zh_tw_dashboard_copy() {
    assert_eq!(format_currency(0.0), "NT$0");
    assert_eq!(format_currency(1234.0), "NT$1,234");
    assert_eq!(format_currency(180000.4), "NT$180,000");
    assert_eq!(format_currency(-20.0), "-NT$20");
    assert_eq!(format_percentage(80.0), "80.0%");
    assert_eq!(format_percentage(33.33), "33.3%");
    assert_eq!(format_one_decimal(2.5), "2.5");
    assert_eq!(format_whole(48.2), "48");
    assert_eq!(format_year_month("2025-07"), "2025年7月");
    assert_eq!(format_year_month("2025-07-01"), "2025年7月");
    assert_eq!(format_year_month("bad"), "bad");
    assert_eq!(membership_tier_dot_class("Black Card"), "bg-luxury-gold");
    assert_eq!(membership_tier_dot_class("Diamond"), "bg-blue-400");
    assert_eq!(membership_tier_dot_class("Platinum"), "bg-gray-400");
}

#[test]
fn analytics_tab_keys_and_labels() {
    assert_eq!(AnalyticsTab::from_key("overview"), AnalyticsTab::Overview);
    assert_eq!(AnalyticsTab::from_key("revenue"), AnalyticsTab::Revenue);
    assert_eq!(
        AnalyticsTab::from_key("engagement"),
        AnalyticsTab::Engagement
    );
    assert_eq!(AnalyticsTab::from_key("nope"), AnalyticsTab::Overview);
    assert_eq!(AnalyticsTab::Overview.label(), "總覽");
    assert_eq!(AnalyticsTab::Revenue.label(), "營收分析");
    assert_eq!(AnalyticsTab::Engagement.label(), "會員參與");
    assert_eq!(AnalyticsTab::Overview.key(), "overview");
}

#[test]
fn analytics_phase_loading_error_ready() {
    let empty = DashboardData::default();
    assert_eq!(analytics_phase(true, &empty), AnalyticsPhase::Loading);
    assert_eq!(
        analytics_phase(
            false,
            &DashboardData {
                error: Some("boom".into()),
                ..DashboardData::default()
            }
        ),
        AnalyticsPhase::Error("boom".into())
    );
    let mut ready = DashboardData::default();
    ready.overview = Some(EventsOverview::default());
    ready.error = Some("partial".into());
    assert_eq!(analytics_phase(false, &ready), AnalyticsPhase::Ready);
    assert!(!dashboard_has_payload(&empty));
    assert!(dashboard_has_payload(&ready));
}

#[test]
fn chart_geometry_empty_single_and_all_zero() {
    let empty = bar_chart(&[], CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert!(empty.empty);
    assert!(empty.bars.is_empty());
    assert_eq!(scale_max(&[]), 1.0);
    assert_eq!(series_max(&[0.0, 0.0]), 0.0);

    let zeros = chart_points(["a".into(), "b".into(), "c".into()], [0.0, 0.0, 0.0]);
    let zero_bars = bar_chart(&zeros, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert!(!zero_bars.empty);
    assert_eq!(zero_bars.bars.len(), 3);
    assert!(zero_bars.bars.iter().all(|bar| bar.height == 0.0));

    let zero_line = line_chart(&zeros, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert!(zero_line.path.starts_with('M'));
    assert_eq!(zero_line.points.len(), 3);
    let baseline = zero_line.points[0].y;
    assert!(zero_line.points.iter().all(|point| point.y == baseline));

    let single = chart_points(["only".into()], [10.0]);
    let single_bars = bar_chart(&single, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert_eq!(single_bars.bars.len(), 1);
    assert!(single_bars.bars[0].height > 0.0);
    let single_line = line_chart(&single, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert_eq!(single_line.points.len(), 1);
    assert!(!single_line.path.contains('L'));

    let spark_empty = sparkline(&[], SPARKLINE_WIDTH, SPARKLINE_HEIGHT);
    assert!(spark_empty.empty);
    let spark_zero = sparkline(&[0.0], SPARKLINE_WIDTH, SPARKLINE_HEIGHT);
    assert!(!spark_zero.empty);
    let spark_single = sparkline(&[4.0], SPARKLINE_WIDTH, SPARKLINE_HEIGHT);
    assert!(spark_single.path.starts_with('M'));

    let donut_empty = donut_chart(&[], DONUT_SIZE / 2.0, DONUT_SIZE / 2.0, 48.0, 80.0);
    assert!(donut_empty.empty);
    let donut_zero = donut_chart(&zeros, 100.0, 100.0, 48.0, 80.0);
    assert!(donut_zero.empty);
    let donut_single = donut_chart(&single, 100.0, 100.0, 48.0, 80.0);
    assert_eq!(donut_single.slices.len(), 1);
    assert!((donut_single.slices[0].percent - 100.0).abs() < 1e-9);
    assert!(donut_single.slices[0].path.contains('A'));

    let populated = chart_points(["a".into(), "b".into()], [10.0, 20.0]);
    let bars = bar_chart(&populated, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert!(bars.bars[1].height > bars.bars[0].height);
    assert_eq!(slice_percent(10.0, 40.0), 25.0);
    assert_eq!(slice_percent(1.0, 0.0), 0.0);
    let ticks = y_ticks(20.0, 4, CHART_WIDTH, CHART_HEIGHT, DEFAULT_PAD);
    assert_eq!(ticks.len(), 4);
    assert_eq!(ticks[0].value, 0.0);
    assert_eq!(ticks[3].value, 20.0);
    assert_eq!(format_tick(20.0), "20");
    assert_eq!(
        chart_aria_label("月度營收趨勢", &[]),
        format!("月度營收趨勢：{EMPTY_CHART_LABEL}")
    );
    assert!(chart_aria_label("月度營收趨勢", &populated).contains("2 筆資料"));
}

#[test]
fn chart_point_helpers_preserve_api_order_and_labels() {
    let revenue = parse_revenue_response(200, revenue_body()).expect("ok");
    let points = monthly_revenue_points(&revenue.monthly_revenue);
    assert_eq!(points[0].label, "2025年7月");
    assert_eq!(points[0].value, 180000.0);
    let events = vec![PopularEvent {
        name: "松露季私宴".into(),
        occupancy_rate: 80.0,
        ..PopularEvent::default()
    }];
    assert_eq!(occupancy_points(&events)[0].value, 80.0);
    let daily = vec![
        DailyVisitor {
            date: "2026-08-31".into(),
            unique_visitors: 2.0,
            ..DailyVisitor::default()
        },
        DailyVisitor {
            date: "2026-08-30".into(),
            unique_visitors: 1.0,
            ..DailyVisitor::default()
        },
    ];
    let daily_points = daily_visitor_points(&daily);
    assert_eq!(daily_points[0].label, "2026-08-30");
    let conversion = ConversionStats {
        total_visitors: 10.0,
        event_viewers: 4.0,
        registered_users: 1.0,
        ..ConversionStats::default()
    };
    assert_eq!(conversion_points(&conversion).len(), 3);
}

#[tokio::test]
async fn native_fetch_does_not_hit_network() {
    let data = fetch_dashboard(Some("token"), 30).await;
    assert_eq!(data.error.as_deref(), Some(NETWORK_ERROR));
    assert!(!dashboard_has_payload(&data));
    let err = fetch_event_performance(Some("token"), "1").await;
    assert_eq!(err.unwrap_err(), NETWORK_ERROR);
}
