use serde_json::Value;

use crate::permissions::{AuthSnapshot, RouteGuard, permissions};

pub const EVENTS_OVERVIEW_PATH: &str = "/api/analytics/events/overview";
pub const EVENTS_PERFORMANCE_PATH: &str = "/api/analytics/events/performance";
pub const EVENTS_ENGAGEMENT_PATH: &str = "/api/analytics/events/engagement";
pub const REVENUE_EVENTS_PATH: &str = "/api/analytics/revenue/events";
pub const MEMBERS_ENGAGEMENT_PATH: &str = "/api/analytics/engagement/members";
pub const VISITORS_PATH: &str = "/api/analytics/visitors";
pub const VISITORS_DAILY_PATH: &str = "/api/analytics/visitors/daily";
pub const POPULAR_PAGES_PATH: &str = "/api/analytics/pages/popular";
pub const CONVERSION_PATH: &str = "/api/analytics/conversion";

pub const NETWORK_ERROR: &str = "Network error occurred";
pub const UNAUTHORIZED: &str = "Access token required";
pub const OVERVIEW_FALLBACK: &str = "Failed to retrieve event analytics overview";
pub const PERFORMANCE_FALLBACK: &str = "Failed to retrieve event performance analytics";
pub const EVENT_DETAIL_FALLBACK: &str = "Failed to fetch event performance data";
pub const REVENUE_FALLBACK: &str = "Failed to fetch revenue analytics";
pub const MEMBERS_FALLBACK: &str = "Failed to fetch member engagement data";
pub const VISITORS_FALLBACK: &str = "Failed to retrieve visitor analytics";
pub const VISITORS_DAILY_FALLBACK: &str = "Failed to retrieve daily analytics";
pub const POPULAR_PAGES_FALLBACK: &str = "Failed to retrieve page analytics";
pub const CONVERSION_FALLBACK: &str = "Failed to retrieve conversion analytics";
pub const EVENTS_ENGAGEMENT_FALLBACK: &str = "Failed to retrieve event engagement analytics";
pub const ADMIN_ROUTE_FALLBACK: &str = "/login";
pub const DEFAULT_DATE_WINDOW_DAYS: u32 = 30;
pub const POPULAR_PAGES_LIMIT: u32 = 20;
pub const CHART_WIDTH: f64 = 640.0;
pub const CHART_HEIGHT: f64 = 220.0;
pub const DONUT_SIZE: f64 = 200.0;
pub const SPARKLINE_WIDTH: f64 = 96.0;
pub const SPARKLINE_HEIGHT: f64 = 28.0;
pub const EMPTY_CHART_LABEL: &str = "目前沒有資料";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateWindow {
    pub days: u32,
    pub label: &'static str,
}

pub const DATE_WINDOWS: &[DateWindow] = &[
    DateWindow {
        days: 7,
        label: "7 天",
    },
    DateWindow {
        days: 30,
        label: "30 天",
    },
    DateWindow {
        days: 90,
        label: "90 天",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AnalyticsTab {
    #[default]
    Overview,
    Revenue,
    Engagement,
}

impl AnalyticsTab {
    pub fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Revenue => "revenue",
            Self::Engagement => "engagement",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "總覽",
            Self::Revenue => "營收分析",
            Self::Engagement => "會員參與",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "revenue" => Self::Revenue,
            "engagement" => Self::Engagement,
            _ => Self::Overview,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalyticsPhase {
    Loading,
    Error(String),
    Ready,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventStats {
    pub total_events: f64,
    pub recent_events: f64,
    pub upcoming_events: f64,
    pub past_events: f64,
    pub avg_occupancy_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RegistrationStats {
    pub total_registrations: f64,
    pub recent_registrations: f64,
    pub unique_attendees: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PopularEvent {
    pub id: String,
    pub name: String,
    pub date_time: String,
    pub capacity: f64,
    pub current_attendees: f64,
    pub occupancy_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventsOverview {
    pub period_days: u32,
    pub event_stats: EventStats,
    pub registration_stats: RegistrationStats,
    pub popular_events: Vec<PopularEvent>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PerformanceEvent {
    pub id: String,
    pub name: String,
    pub date_time: String,
    pub capacity: f64,
    pub current_attendees: f64,
    pub pricing_vip: f64,
    pub pricing_vvip: f64,
    pub occupancy_rate: f64,
    pub total_registrations: f64,
    pub confirmed_registrations: f64,
    pub pending_registrations: f64,
    pub cancelled_registrations: f64,
    pub avg_revenue_per_attendee: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventsPerformance {
    pub period_days: u32,
    pub events: Vec<PerformanceEvent>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PerformanceEventDetail {
    pub id: String,
    pub title: String,
    pub fill_rate: f64,
    pub current_revenue: f64,
    pub potential_revenue: f64,
    pub category_name: String,
    pub venue_name: String,
    pub current_registrations: f64,
    pub capacity_max: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TimelinePoint {
    pub date: String,
    pub registrations: f64,
    pub cumulative_registrations: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MembershipSlice {
    pub membership_tier: String,
    pub count: f64,
    pub percentage: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatusSlice {
    pub status: String,
    pub count: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventPerformanceDetail {
    pub event: PerformanceEventDetail,
    pub registration_timeline: Vec<TimelinePoint>,
    pub membership_breakdown: Vec<MembershipSlice>,
    pub status_breakdown: Vec<StatusSlice>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MonthlyRevenue {
    pub month: String,
    pub revenue: f64,
    pub event_count: f64,
    pub total_registrations: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CategoryRevenue {
    pub category: String,
    pub revenue: f64,
    pub event_count: f64,
    pub avg_revenue_per_event: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TierRevenue {
    pub membership_tier: String,
    pub registration_count: f64,
    pub total_revenue: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RevenueData {
    pub monthly_revenue: Vec<MonthlyRevenue>,
    pub category_revenue: Vec<CategoryRevenue>,
    pub tier_revenue: Vec<TierRevenue>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TierEngagement {
    pub membership_tier: String,
    pub total_members: f64,
    pub active_members: f64,
    pub engagement_rate: f64,
    pub avg_events_per_member: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TopMember {
    pub first_name: String,
    pub last_name: String,
    pub membership_tier: String,
    pub events_attended: f64,
    pub total_spent: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RetentionCohort {
    pub cohort_month: String,
    pub cohort_size: f64,
    pub active_this_month: f64,
    pub retention_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct MembersEngagement {
    pub engagement: Vec<TierEngagement>,
    pub top_members: Vec<TopMember>,
    pub retention: Vec<RetentionCohort>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct VisitorsOverview {
    pub period_days: u32,
    pub unique_visitors: f64,
    pub total_page_views: f64,
    pub converted_visitors: f64,
    pub avg_pages_per_visitor: f64,
    pub new_visitors: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DailyVisitor {
    pub date: String,
    pub unique_visitors: f64,
    pub total_page_views: f64,
    pub converted_visitors: f64,
    pub avg_pages_per_visitor: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PopularPage {
    pub path: String,
    pub views: f64,
    pub unique_visitors: f64,
    pub conversion_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ConversionStats {
    pub period_days: u32,
    pub total_visitors: f64,
    pub event_viewers: f64,
    pub registered_users: f64,
    pub conversion_rate: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventEngagementDay {
    pub date: String,
    pub unique_visitors: f64,
    pub total_page_views: f64,
    pub event_page_views: f64,
    pub registration_page_views: f64,
    pub avg_time_spent: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EventsEngagement {
    pub period_days: u32,
    pub engagement: Vec<EventEngagementDay>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DashboardData {
    pub overview: Option<EventsOverview>,
    pub performance: Option<EventsPerformance>,
    pub revenue: Option<RevenueData>,
    pub members: Option<MembersEngagement>,
    pub visitors: Option<VisitorsOverview>,
    pub visitors_daily: Vec<DailyVisitor>,
    pub popular_pages: Vec<PopularPage>,
    pub conversion: Option<ConversionStats>,
    pub events_engagement: Option<EventsEngagement>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OverviewKpis {
    pub total_events: f64,
    pub published_events: f64,
    pub total_registrations: f64,
    pub avg_registrations: f64,
    pub estimated_revenue: f64,
    pub completed_events: f64,
    pub cancelled_events: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartPad {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

pub const DEFAULT_PAD: ChartPad = ChartPad {
    top: 16.0,
    right: 16.0,
    bottom: 36.0,
    left: 48.0,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    pub label: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    pub y: f64,
    pub label: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub label: String,
    pub value: f64,
    pub label_x: f64,
    pub label_y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BarChartGeom {
    pub width: f64,
    pub height: f64,
    pub pad: ChartPad,
    pub bars: Vec<Bar>,
    pub ticks: Vec<Tick>,
    pub empty: bool,
    pub max: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinePoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineChartGeom {
    pub width: f64,
    pub height: f64,
    pub pad: ChartPad,
    pub path: String,
    pub points: Vec<LinePoint>,
    pub ticks: Vec<Tick>,
    pub empty: bool,
    pub max: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DonutSlice {
    pub path: String,
    pub percent: f64,
    pub label: String,
    pub value: f64,
    pub color_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DonutGeom {
    pub cx: f64,
    pub cy: f64,
    pub inner_radius: f64,
    pub outer_radius: f64,
    pub slices: Vec<DonutSlice>,
    pub empty: bool,
    pub total: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SparklineGeom {
    pub width: f64,
    pub height: f64,
    pub path: String,
    pub empty: bool,
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

pub fn select_date_window(raw: Option<&str>) -> u32 {
    period_days(raw, DEFAULT_DATE_WINDOW_DAYS)
}

pub fn period_days(raw: Option<&str>, fallback: u32) -> u32 {
    let parsed = raw
        .map(str::trim)
        .and_then(|text| {
            let (sign, digits) = match text.strip_prefix('-') {
                Some(rest) => (-1i64, rest),
                None => (1i64, text.strip_prefix('+').unwrap_or(text)),
            };
            let digits: String = digits
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect();
            digits.parse::<i64>().ok().map(|value| sign * value)
        })
        .filter(|value| *value != 0)
        .unwrap_or(i64::from(fallback));
    parsed.clamp(1, i64::from(u32::MAX)) as u32
}

pub fn is_active_date_window(selected: u32, window: u32) -> bool {
    selected == window
}

pub fn days_query(days: u32) -> String {
    format!("days={days}")
}

pub fn events_overview_url(days: u32) -> String {
    format!("{EVENTS_OVERVIEW_PATH}?{}", days_query(days))
}

pub fn events_performance_url(days: u32) -> String {
    format!("{EVENTS_PERFORMANCE_PATH}?{}", days_query(days))
}

pub fn events_engagement_url(days: u32) -> String {
    format!("{EVENTS_ENGAGEMENT_PATH}?{}", days_query(days))
}

pub fn event_performance_url(id: &str) -> String {
    format!("/api/analytics/events/{id}/performance")
}

pub fn visitors_url(days: u32) -> String {
    format!("{VISITORS_PATH}?{}", days_query(days))
}

pub fn visitors_daily_url(days: u32) -> String {
    format!("{VISITORS_DAILY_PATH}?{}", days_query(days))
}

pub fn popular_pages_url(limit: u32) -> String {
    format!("{POPULAR_PAGES_PATH}?limit={limit}")
}

pub fn conversion_url(days: u32) -> String {
    format!("{CONVERSION_PATH}?{}", days_query(days))
}

pub fn export_endpoint_available() -> bool {
    false
}

pub fn analytics_phase(loading: bool, data: &DashboardData) -> AnalyticsPhase {
    if loading {
        return AnalyticsPhase::Loading;
    }
    if !dashboard_has_payload(data) {
        if let Some(error) = data
            .error
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return AnalyticsPhase::Error(error.to_string());
        }
    }
    AnalyticsPhase::Ready
}

pub fn dashboard_has_payload(data: &DashboardData) -> bool {
    data.overview.is_some()
        || data.performance.is_some()
        || data.revenue.is_some()
        || data.members.is_some()
        || data.visitors.is_some()
        || !data.visitors_daily.is_empty()
        || !data.popular_pages.is_empty()
        || data.conversion.is_some()
        || data.events_engagement.is_some()
}

pub fn overview_kpis(
    overview: Option<&EventsOverview>,
    revenue: Option<&RevenueData>,
    performance: Option<&EventsPerformance>,
) -> OverviewKpis {
    let total_events = overview
        .map(|item| item.event_stats.total_events)
        .unwrap_or(0.0);
    let total_registrations = overview
        .map(|item| item.registration_stats.total_registrations)
        .unwrap_or(0.0);
    let avg_registrations = if total_events > 0.0 {
        total_registrations / total_events
    } else {
        0.0
    };
    OverviewKpis {
        total_events,
        published_events: overview
            .map(|item| item.event_stats.recent_events)
            .unwrap_or(0.0),
        total_registrations,
        avg_registrations,
        estimated_revenue: revenue
            .map(|item| item.monthly_revenue.iter().map(|row| row.revenue).sum())
            .unwrap_or(0.0),
        completed_events: overview
            .map(|item| item.event_stats.past_events)
            .unwrap_or(0.0),
        cancelled_events: performance
            .map(|item| {
                item.events
                    .iter()
                    .map(|row| row.cancelled_registrations)
                    .sum()
            })
            .unwrap_or(0.0),
    }
}

pub fn format_currency(amount: f64) -> String {
    let rounded = amount.round();
    let negative = rounded < 0.0;
    let digits = format_thousands(rounded.abs() as u64);
    if negative {
        format!("-NT${digits}")
    } else {
        format!("NT${digits}")
    }
}

pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

pub fn format_one_decimal(value: f64) -> String {
    format!("{value:.1}")
}

pub fn format_whole(value: f64) -> String {
    format_thousands(value.round().abs() as u64)
}

pub fn format_year_month(value: &str) -> String {
    let digits: String = value
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .take(6)
        .collect();
    if digits.len() < 6 {
        return value.to_string();
    }
    let year = &digits[..4];
    let month = digits[4..6].parse::<u32>().unwrap_or(0);
    if !(1..=12).contains(&month) {
        return value.to_string();
    }
    format!("{year}年{month}月")
}

pub fn membership_tier_dot_class(tier: &str) -> &'static str {
    match tier {
        "Black Card" => "bg-luxury-gold",
        "Diamond" => "bg-blue-400",
        _ => "bg-gray-400",
    }
}

pub fn chart_points<L, V>(labels: L, values: V) -> Vec<ChartPoint>
where
    L: IntoIterator<Item = String>,
    V: IntoIterator<Item = f64>,
{
    labels
        .into_iter()
        .zip(values)
        .map(|(label, value)| ChartPoint { label, value })
        .collect()
}

pub fn series_max(values: &[f64]) -> f64 {
    values.iter().copied().fold(0.0_f64, f64::max)
}

pub fn scale_max(values: &[f64]) -> f64 {
    let max = series_max(values);
    if max <= 0.0 { 1.0 } else { max }
}

pub fn inner_size(width: f64, height: f64, pad: ChartPad) -> (f64, f64) {
    (
        (width - pad.left - pad.right).max(0.0),
        (height - pad.top - pad.bottom).max(0.0),
    )
}

pub fn y_ticks(max: f64, count: usize, width: f64, height: f64, pad: ChartPad) -> Vec<Tick> {
    let count = count.max(2);
    let scale = if max <= 0.0 { 1.0 } else { max };
    let inner_h = inner_size(width, height, pad).1;
    (0..count)
        .map(|index| {
            let t = index as f64 / (count - 1) as f64;
            let value = scale * t;
            let y = pad.top + inner_h - t * inner_h;
            Tick {
                y,
                label: format_tick(value),
                value,
            }
        })
        .collect()
}

pub fn format_tick(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.1}")
    }
}

pub fn bar_chart(points: &[ChartPoint], width: f64, height: f64, pad: ChartPad) -> BarChartGeom {
    let values: Vec<f64> = points.iter().map(|point| point.value).collect();
    let max = series_max(&values);
    let scale = scale_max(&values);
    let (inner_w, inner_h) = inner_size(width, height, pad);
    let ticks = y_ticks(max, 4, width, height, pad);
    if points.is_empty() {
        return BarChartGeom {
            width,
            height,
            pad,
            bars: Vec::new(),
            ticks,
            empty: true,
            max,
        };
    }
    let n = points.len() as f64;
    let gap = 8.0;
    let bar_w = ((inner_w - gap * (n + 1.0)) / n).max(1.0);
    let bars = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let height_px = if max <= 0.0 {
                0.0
            } else {
                (point.value / scale) * inner_h
            };
            let x = pad.left + gap + index as f64 * (bar_w + gap);
            let y = pad.top + inner_h - height_px;
            Bar {
                x,
                y,
                width: bar_w,
                height: height_px,
                label: point.label.clone(),
                value: point.value,
                label_x: x + bar_w / 2.0,
                label_y: pad.top + inner_h + 16.0,
            }
        })
        .collect();
    BarChartGeom {
        width,
        height,
        pad,
        bars,
        ticks,
        empty: false,
        max,
    }
}

pub fn line_chart(points: &[ChartPoint], width: f64, height: f64, pad: ChartPad) -> LineChartGeom {
    let values: Vec<f64> = points.iter().map(|point| point.value).collect();
    let max = series_max(&values);
    let scale = scale_max(&values);
    let (inner_w, inner_h) = inner_size(width, height, pad);
    let ticks = y_ticks(max, 4, width, height, pad);
    if points.is_empty() {
        return LineChartGeom {
            width,
            height,
            pad,
            path: String::new(),
            points: Vec::new(),
            ticks,
            empty: true,
            max,
        };
    }
    let last = (points.len() - 1) as f64;
    let mapped: Vec<LinePoint> = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let x = if points.len() == 1 {
                pad.left + inner_w / 2.0
            } else {
                pad.left + (index as f64 / last) * inner_w
            };
            let y = pad.top + inner_h - (point.value / scale) * inner_h;
            LinePoint {
                x,
                y,
                label: point.label.clone(),
                value: point.value,
            }
        })
        .collect();
    let path = line_path(&mapped);
    LineChartGeom {
        width,
        height,
        pad,
        path,
        points: mapped,
        ticks,
        empty: false,
        max,
    }
}

pub fn sparkline(values: &[f64], width: f64, height: f64) -> SparklineGeom {
    if values.is_empty() {
        return SparklineGeom {
            width,
            height,
            path: String::new(),
            empty: true,
        };
    }
    let max = scale_max(values);
    let last = (values.len() - 1) as f64;
    let points: Vec<LinePoint> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = if values.len() == 1 {
                width / 2.0
            } else {
                (index as f64 / last) * width
            };
            let y = height - (*value / max) * height;
            LinePoint {
                x,
                y,
                label: String::new(),
                value: *value,
            }
        })
        .collect();
    SparklineGeom {
        width,
        height,
        path: line_path(&points),
        empty: false,
    }
}

pub fn donut_chart(
    points: &[ChartPoint],
    cx: f64,
    cy: f64,
    inner_radius: f64,
    outer_radius: f64,
) -> DonutGeom {
    let total: f64 = points.iter().map(|point| point.value.max(0.0)).sum();
    if points.is_empty() || total <= 0.0 {
        return DonutGeom {
            cx,
            cy,
            inner_radius,
            outer_radius,
            slices: Vec::new(),
            empty: true,
            total,
        };
    }
    let mut start = -std::f64::consts::FRAC_PI_2;
    let slices = points
        .iter()
        .enumerate()
        .filter(|(_, point)| point.value > 0.0)
        .map(|(index, point)| {
            let sweep = (point.value / total) * std::f64::consts::TAU;
            let end = start + sweep;
            let path = donut_arc(cx, cy, inner_radius, outer_radius, start, end);
            start = end;
            DonutSlice {
                path,
                percent: slice_percent(point.value, total),
                label: point.label.clone(),
                value: point.value,
                color_index: index,
            }
        })
        .collect();
    DonutGeom {
        cx,
        cy,
        inner_radius,
        outer_radius,
        slices,
        empty: false,
        total,
    }
}

pub fn slice_percent(value: f64, total: f64) -> f64 {
    if total <= 0.0 {
        0.0
    } else {
        value * 100.0 / total
    }
}

pub fn donut_color(index: usize) -> &'static str {
    const COLORS: &[&str] = &[
        "#d4af37", "#60a5fa", "#c084fc", "#4ade80", "#f97316", "#e5e4e2",
    ];
    COLORS[index % COLORS.len()]
}

pub fn chart_aria_label(title: &str, points: &[ChartPoint]) -> String {
    if points.is_empty() {
        format!("{title}：{EMPTY_CHART_LABEL}")
    } else {
        format!("{title}，{} 筆資料", points.len())
    }
}

pub fn monthly_revenue_points(rows: &[MonthlyRevenue]) -> Vec<ChartPoint> {
    rows.iter()
        .take(6)
        .map(|row| ChartPoint {
            label: format_year_month(&row.month),
            value: row.revenue,
        })
        .collect()
}

pub fn category_revenue_points(rows: &[CategoryRevenue]) -> Vec<ChartPoint> {
    rows.iter()
        .map(|row| ChartPoint {
            label: row.category.clone(),
            value: row.revenue,
        })
        .collect()
}

pub fn tier_revenue_points(rows: &[TierRevenue]) -> Vec<ChartPoint> {
    rows.iter()
        .map(|row| ChartPoint {
            label: row.membership_tier.clone(),
            value: row.total_revenue,
        })
        .collect()
}

pub fn occupancy_points(events: &[PopularEvent]) -> Vec<ChartPoint> {
    events
        .iter()
        .take(5)
        .map(|event| ChartPoint {
            label: event.name.clone(),
            value: event.occupancy_rate,
        })
        .collect()
}

pub fn performance_occupancy_points(events: &[PerformanceEvent]) -> Vec<ChartPoint> {
    events
        .iter()
        .take(8)
        .map(|event| ChartPoint {
            label: event.name.clone(),
            value: event.occupancy_rate,
        })
        .collect()
}

pub fn daily_visitor_points(rows: &[DailyVisitor]) -> Vec<ChartPoint> {
    let mut points: Vec<ChartPoint> = rows
        .iter()
        .map(|row| ChartPoint {
            label: row.date.clone(),
            value: row.unique_visitors,
        })
        .collect();
    points.reverse();
    points
}

pub fn popular_page_points(rows: &[PopularPage]) -> Vec<ChartPoint> {
    rows.iter()
        .take(8)
        .map(|row| ChartPoint {
            label: row.path.clone(),
            value: row.views,
        })
        .collect()
}

pub fn conversion_points(stats: &ConversionStats) -> Vec<ChartPoint> {
    vec![
        ChartPoint {
            label: "訪客".to_string(),
            value: stats.total_visitors,
        },
        ChartPoint {
            label: "活動瀏覽".to_string(),
            value: stats.event_viewers,
        },
        ChartPoint {
            label: "註冊".to_string(),
            value: stats.registered_users,
        },
    ]
}

pub fn engagement_rate_points(rows: &[TierEngagement]) -> Vec<ChartPoint> {
    rows.iter()
        .map(|row| ChartPoint {
            label: row.membership_tier.clone(),
            value: row.engagement_rate,
        })
        .collect()
}

pub fn retention_points(rows: &[RetentionCohort]) -> Vec<ChartPoint> {
    rows.iter()
        .take(6)
        .map(|row| ChartPoint {
            label: format_year_month(&row.cohort_month),
            value: row.retention_rate,
        })
        .collect()
}

pub fn events_engagement_points(rows: &[EventEngagementDay]) -> Vec<ChartPoint> {
    let mut points: Vec<ChartPoint> = rows
        .iter()
        .map(|row| ChartPoint {
            label: row.date.clone(),
            value: row.event_page_views,
        })
        .collect();
    points.reverse();
    points
}

pub fn parse_events_overview_response(status: u16, body: &str) -> Result<EventsOverview, String> {
    let data = parse_success_data(status, body, OVERVIEW_FALLBACK)?;
    Ok(EventsOverview {
        period_days: json_u32(data.get("period_days")).unwrap_or(DEFAULT_DATE_WINDOW_DAYS),
        event_stats: data
            .get("event_stats")
            .map(parse_event_stats)
            .unwrap_or_default(),
        registration_stats: data
            .get("registration_stats")
            .map(parse_registration_stats)
            .unwrap_or_default(),
        popular_events: data
            .get("popular_events")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().filter_map(parse_popular_event).collect())
            .unwrap_or_default(),
    })
}

pub fn parse_events_performance_response(
    status: u16,
    body: &str,
) -> Result<EventsPerformance, String> {
    let data = parse_success_data(status, body, PERFORMANCE_FALLBACK)?;
    Ok(EventsPerformance {
        period_days: json_u32(data.get("period_days")).unwrap_or(DEFAULT_DATE_WINDOW_DAYS),
        events: data
            .get("events")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().filter_map(parse_performance_event).collect())
            .unwrap_or_default(),
    })
}

pub fn parse_event_performance_detail_response(
    status: u16,
    body: &str,
) -> Result<EventPerformanceDetail, String> {
    if status == 404 {
        return Err(json_error_or(body, "Event not found"));
    }
    let data = parse_success_data(status, body, EVENT_DETAIL_FALLBACK)?;
    let event = data.get("event").ok_or(EVENT_DETAIL_FALLBACK)?;
    Ok(EventPerformanceDetail {
        event: parse_performance_event_detail(event),
        registration_timeline: data
            .get("registrationTimeline")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_timeline_point).collect())
            .unwrap_or_default(),
        membership_breakdown: data
            .get("membershipBreakdown")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_membership_slice).collect())
            .unwrap_or_default(),
        status_breakdown: data
            .get("statusBreakdown")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_status_slice).collect())
            .unwrap_or_default(),
    })
}

pub fn parse_revenue_response(status: u16, body: &str) -> Result<RevenueData, String> {
    let data = parse_success_data(status, body, REVENUE_FALLBACK)?;
    Ok(RevenueData {
        monthly_revenue: data
            .get("monthlyRevenue")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_monthly_revenue).collect())
            .unwrap_or_default(),
        category_revenue: data
            .get("categoryRevenue")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_category_revenue).collect())
            .unwrap_or_default(),
        tier_revenue: data
            .get("tierRevenue")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_tier_revenue).collect())
            .unwrap_or_default(),
    })
}

pub fn parse_members_engagement_response(
    status: u16,
    body: &str,
) -> Result<MembersEngagement, String> {
    let data = parse_success_data(status, body, MEMBERS_FALLBACK)?;
    Ok(MembersEngagement {
        engagement: data
            .get("engagement")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_tier_engagement).collect())
            .unwrap_or_default(),
        top_members: data
            .get("topMembers")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_top_member).collect())
            .unwrap_or_default(),
        retention: data
            .get("retention")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_retention).collect())
            .unwrap_or_default(),
    })
}

pub fn parse_visitors_overview_response(
    status: u16,
    body: &str,
) -> Result<VisitorsOverview, String> {
    let data = parse_success_data(status, body, VISITORS_FALLBACK)?;
    Ok(VisitorsOverview {
        period_days: json_u32(data.get("period_days")).unwrap_or(DEFAULT_DATE_WINDOW_DAYS),
        unique_visitors: json_f64(data.get("unique_visitors")).unwrap_or(0.0),
        total_page_views: json_f64(data.get("total_page_views")).unwrap_or(0.0),
        converted_visitors: json_f64(data.get("converted_visitors")).unwrap_or(0.0),
        avg_pages_per_visitor: json_f64(data.get("avg_pages_per_visitor")).unwrap_or(0.0),
        new_visitors: json_f64(data.get("new_visitors")).unwrap_or(0.0),
    })
}

pub fn parse_visitors_daily_response(status: u16, body: &str) -> Result<Vec<DailyVisitor>, String> {
    let data = parse_success_array(status, body, VISITORS_DAILY_FALLBACK)?;
    Ok(data.iter().map(parse_daily_visitor).collect())
}

pub fn parse_popular_pages_response(status: u16, body: &str) -> Result<Vec<PopularPage>, String> {
    let data = parse_success_array(status, body, POPULAR_PAGES_FALLBACK)?;
    Ok(data.iter().map(parse_popular_page).collect())
}

pub fn parse_conversion_response(status: u16, body: &str) -> Result<ConversionStats, String> {
    let data = parse_success_data(status, body, CONVERSION_FALLBACK)?;
    Ok(ConversionStats {
        period_days: json_u32(data.get("period_days")).unwrap_or(DEFAULT_DATE_WINDOW_DAYS),
        total_visitors: json_f64(data.get("total_visitors")).unwrap_or(0.0),
        event_viewers: json_f64(data.get("event_viewers")).unwrap_or(0.0),
        registered_users: json_f64(data.get("registered_users")).unwrap_or(0.0),
        conversion_rate: json_f64(data.get("conversion_rate")).unwrap_or(0.0),
    })
}

pub fn parse_events_engagement_response(
    status: u16,
    body: &str,
) -> Result<EventsEngagement, String> {
    let data = parse_success_data(status, body, EVENTS_ENGAGEMENT_FALLBACK)?;
    Ok(EventsEngagement {
        period_days: json_u32(data.get("period_days")).unwrap_or(DEFAULT_DATE_WINDOW_DAYS),
        engagement: data
            .get("engagement")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().map(parse_event_engagement_day).collect())
            .unwrap_or_default(),
    })
}

pub async fn fetch_dashboard(token: Option<&str>, days: u32) -> DashboardData {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(token) = token.filter(|value| !value.is_empty()) else {
            return DashboardData {
                error: Some(UNAUTHORIZED.to_string()),
                ..DashboardData::default()
            };
        };
        let mut data = DashboardData::default();
        let mut errors: Vec<String> = Vec::new();
        match send_authorized(token, &events_overview_url(days)).await {
            Ok((status, body)) => match parse_events_overview_response(status, &body) {
                Ok(value) => data.overview = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, &events_performance_url(days)).await {
            Ok((status, body)) => match parse_events_performance_response(status, &body) {
                Ok(value) => data.performance = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, REVENUE_EVENTS_PATH).await {
            Ok((status, body)) => match parse_revenue_response(status, &body) {
                Ok(value) => data.revenue = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, MEMBERS_ENGAGEMENT_PATH).await {
            Ok((status, body)) => match parse_members_engagement_response(status, &body) {
                Ok(value) => data.members = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, &visitors_url(days)).await {
            Ok((status, body)) => match parse_visitors_overview_response(status, &body) {
                Ok(value) => data.visitors = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, &visitors_daily_url(days)).await {
            Ok((status, body)) => match parse_visitors_daily_response(status, &body) {
                Ok(value) => data.visitors_daily = value,
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, &popular_pages_url(POPULAR_PAGES_LIMIT)).await {
            Ok((status, body)) => match parse_popular_pages_response(status, &body) {
                Ok(value) => data.popular_pages = value,
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, &conversion_url(days)).await {
            Ok((status, body)) => match parse_conversion_response(status, &body) {
                Ok(value) => data.conversion = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        match send_authorized(token, &events_engagement_url(days)).await {
            Ok((status, body)) => match parse_events_engagement_response(status, &body) {
                Ok(value) => data.events_engagement = Some(value),
                Err(error) => errors.push(error),
            },
            Err(error) => errors.push(error),
        }
        if !errors.is_empty() && !dashboard_has_payload(&data) {
            data.error = errors.into_iter().next();
        } else if !errors.is_empty() {
            data.error = errors.into_iter().next();
        }
        return data;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (token, days);
        DashboardData {
            error: Some(NETWORK_ERROR.to_string()),
            ..DashboardData::default()
        }
    }
}

pub async fn fetch_event_performance(
    token: Option<&str>,
    event_id: &str,
) -> Result<EventPerformanceDetail, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let token = token
            .filter(|value| !value.is_empty())
            .ok_or(UNAUTHORIZED)?;
        let (status, body) = send_authorized(token, &event_performance_url(event_id)).await?;
        return parse_event_performance_detail_response(status, &body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (token, event_id);
        Err(NETWORK_ERROR.to_string())
    }
}

fn line_path(points: &[LinePoint]) -> String {
    let mut path = String::new();
    for (index, point) in points.iter().enumerate() {
        if index == 0 {
            path.push_str(&format!("M{:.3} {:.3}", point.x, point.y));
        } else {
            path.push_str(&format!(" L{:.3} {:.3}", point.x, point.y));
        }
    }
    path
}

fn donut_arc(
    cx: f64,
    cy: f64,
    inner_radius: f64,
    outer_radius: f64,
    start: f64,
    end: f64,
) -> String {
    let sweep = end - start;
    if sweep.abs() >= std::f64::consts::TAU - 1e-6 {
        let mid = start + std::f64::consts::PI;
        return format!(
            "{} {}",
            donut_arc(cx, cy, inner_radius, outer_radius, start, mid),
            donut_arc(cx, cy, inner_radius, outer_radius, mid, end)
        );
    }
    let (sx, sy) = polar(cx, cy, outer_radius, start);
    let (ex, ey) = polar(cx, cy, outer_radius, end);
    let (ix, iy) = polar(cx, cy, inner_radius, end);
    let (jx, jy) = polar(cx, cy, inner_radius, start);
    let large = if sweep.abs() > std::f64::consts::PI {
        1
    } else {
        0
    };
    format!(
        "M{sx:.3} {sy:.3} A{outer_radius:.3} {outer_radius:.3} 0 {large} 1 {ex:.3} {ey:.3} L{ix:.3} {iy:.3} A{inner_radius:.3} {inner_radius:.3} 0 {large} 0 {jx:.3} {jy:.3} Z"
    )
}

fn polar(cx: f64, cy: f64, radius: f64, angle: f64) -> (f64, f64) {
    (cx + radius * angle.cos(), cy + radius * angle.sin())
}

fn parse_success_data(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    let root = parse_success_root(status, body, fallback)?;
    match root.get("data") {
        Some(Value::Null) | None => Err(fallback.to_string()),
        Some(data) => Ok(data.clone()),
    }
}

fn parse_success_array(
    status: u16,
    body: &str,
    fallback: &'static str,
) -> Result<Vec<Value>, String> {
    let data = parse_success_data(status, body, fallback)?;
    match data {
        Value::Array(items) => Ok(items),
        _ => Err(fallback.to_string()),
    }
}

fn parse_success_root(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    if status == 401 {
        return Err(json_error_or(body, UNAUTHORIZED));
    }
    if body.trim().is_empty() {
        return Err(fallback.to_string());
    }
    let value: Value = serde_json::from_str(body).map_err(|_| fallback.to_string())?;
    if !(200..300).contains(&status) {
        return Err(json_error_message(&value, fallback));
    }
    if value.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(value)
    } else {
        Err(json_error_message(&value, fallback))
    }
}

fn json_error_or(body: &str, fallback: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .map(|value| json_error_message(&value, fallback))
        .unwrap_or_else(|| fallback.to_string())
}

fn json_error_message(value: &Value, fallback: &str) -> String {
    value
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|error| !error.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn parse_event_stats(value: &Value) -> EventStats {
    EventStats {
        total_events: json_f64(value.get("total_events")).unwrap_or(0.0),
        recent_events: json_f64(value.get("recent_events")).unwrap_or(0.0),
        upcoming_events: json_f64(value.get("upcoming_events")).unwrap_or(0.0),
        past_events: json_f64(value.get("past_events")).unwrap_or(0.0),
        avg_occupancy_rate: json_f64(value.get("avg_occupancy_rate")).unwrap_or(0.0),
    }
}

fn parse_registration_stats(value: &Value) -> RegistrationStats {
    RegistrationStats {
        total_registrations: json_f64(value.get("total_registrations")).unwrap_or(0.0),
        recent_registrations: json_f64(value.get("recent_registrations")).unwrap_or(0.0),
        unique_attendees: json_f64(value.get("unique_attendees")).unwrap_or(0.0),
    }
}

fn parse_popular_event(value: &Value) -> Option<PopularEvent> {
    Some(PopularEvent {
        id: json_id(value.get("id")).unwrap_or_default(),
        name: json_string(value.get("name")).unwrap_or_default(),
        date_time: json_string(value.get("date_time")).unwrap_or_default(),
        capacity: json_f64(value.get("capacity")).unwrap_or(0.0),
        current_attendees: json_f64(value.get("current_attendees")).unwrap_or(0.0),
        occupancy_rate: json_f64(value.get("occupancy_rate")).unwrap_or(0.0),
    })
}

fn parse_performance_event(value: &Value) -> Option<PerformanceEvent> {
    Some(PerformanceEvent {
        id: json_id(value.get("id")).unwrap_or_default(),
        name: json_string(value.get("name")).unwrap_or_default(),
        date_time: json_string(value.get("date_time")).unwrap_or_default(),
        capacity: json_f64(value.get("capacity")).unwrap_or(0.0),
        current_attendees: json_f64(value.get("current_attendees")).unwrap_or(0.0),
        pricing_vip: json_f64(value.get("pricing_vip")).unwrap_or(0.0),
        pricing_vvip: json_f64(value.get("pricing_vvip")).unwrap_or(0.0),
        occupancy_rate: json_f64(value.get("occupancy_rate")).unwrap_or(0.0),
        total_registrations: json_f64(value.get("total_registrations")).unwrap_or(0.0),
        confirmed_registrations: json_f64(value.get("confirmed_registrations")).unwrap_or(0.0),
        pending_registrations: json_f64(value.get("pending_registrations")).unwrap_or(0.0),
        cancelled_registrations: json_f64(value.get("cancelled_registrations")).unwrap_or(0.0),
        avg_revenue_per_attendee: json_f64(value.get("avg_revenue_per_attendee")).unwrap_or(0.0),
    })
}

fn parse_performance_event_detail(value: &Value) -> PerformanceEventDetail {
    PerformanceEventDetail {
        id: json_id(value.get("id")).unwrap_or_default(),
        title: json_string(value.get("title"))
            .or_else(|| json_string(value.get("name")))
            .unwrap_or_default(),
        fill_rate: json_f64(value.get("fill_rate")).unwrap_or(0.0),
        current_revenue: json_f64(value.get("current_revenue")).unwrap_or(0.0),
        potential_revenue: json_f64(value.get("potential_revenue")).unwrap_or(0.0),
        category_name: json_string(value.get("category_name")).unwrap_or_default(),
        venue_name: json_string(value.get("venue_name")).unwrap_or_default(),
        current_registrations: json_f64(value.get("current_registrations")).unwrap_or(0.0),
        capacity_max: json_f64(value.get("capacity_max")).unwrap_or(0.0),
    }
}

fn parse_timeline_point(value: &Value) -> TimelinePoint {
    TimelinePoint {
        date: json_string(value.get("date")).unwrap_or_default(),
        registrations: json_f64(value.get("registrations")).unwrap_or(0.0),
        cumulative_registrations: json_f64(value.get("cumulative_registrations")).unwrap_or(0.0),
    }
}

fn parse_membership_slice(value: &Value) -> MembershipSlice {
    MembershipSlice {
        membership_tier: json_string(value.get("membership_tier")).unwrap_or_default(),
        count: json_f64(value.get("count")).unwrap_or(0.0),
        percentage: json_f64(value.get("percentage")).unwrap_or(0.0),
    }
}

fn parse_status_slice(value: &Value) -> StatusSlice {
    StatusSlice {
        status: json_string(value.get("status")).unwrap_or_default(),
        count: json_f64(value.get("count")).unwrap_or(0.0),
    }
}

fn parse_monthly_revenue(value: &Value) -> MonthlyRevenue {
    MonthlyRevenue {
        month: json_string(value.get("month")).unwrap_or_default(),
        revenue: json_f64(value.get("revenue")).unwrap_or(0.0),
        event_count: json_f64(value.get("event_count")).unwrap_or(0.0),
        total_registrations: json_f64(value.get("total_registrations")).unwrap_or(0.0),
    }
}

fn parse_category_revenue(value: &Value) -> CategoryRevenue {
    CategoryRevenue {
        category: json_string(value.get("category")).unwrap_or_default(),
        revenue: json_f64(value.get("revenue")).unwrap_or(0.0),
        event_count: json_f64(value.get("event_count")).unwrap_or(0.0),
        avg_revenue_per_event: json_f64(value.get("avg_revenue_per_event")).unwrap_or(0.0),
    }
}

fn parse_tier_revenue(value: &Value) -> TierRevenue {
    TierRevenue {
        membership_tier: json_string(value.get("membership_tier")).unwrap_or_default(),
        registration_count: json_f64(value.get("registration_count")).unwrap_or(0.0),
        total_revenue: json_f64(value.get("total_revenue")).unwrap_or(0.0),
    }
}

fn parse_tier_engagement(value: &Value) -> TierEngagement {
    TierEngagement {
        membership_tier: json_string(value.get("membership_tier")).unwrap_or_default(),
        total_members: json_f64(value.get("total_members")).unwrap_or(0.0),
        active_members: json_f64(value.get("active_members")).unwrap_or(0.0),
        engagement_rate: json_f64(value.get("engagement_rate")).unwrap_or(0.0),
        avg_events_per_member: json_f64(value.get("avg_events_per_member")).unwrap_or(0.0),
    }
}

fn parse_top_member(value: &Value) -> TopMember {
    TopMember {
        first_name: json_string(value.get("first_name")).unwrap_or_default(),
        last_name: json_string(value.get("last_name")).unwrap_or_default(),
        membership_tier: json_string(value.get("membership_tier")).unwrap_or_default(),
        events_attended: json_f64(value.get("events_attended")).unwrap_or(0.0),
        total_spent: json_f64(value.get("total_spent")).unwrap_or(0.0),
    }
}

fn parse_retention(value: &Value) -> RetentionCohort {
    RetentionCohort {
        cohort_month: json_string(value.get("cohort_month")).unwrap_or_default(),
        cohort_size: json_f64(value.get("cohort_size")).unwrap_or(0.0),
        active_this_month: json_f64(value.get("active_this_month")).unwrap_or(0.0),
        retention_rate: json_f64(value.get("retention_rate")).unwrap_or(0.0),
    }
}

fn parse_daily_visitor(value: &Value) -> DailyVisitor {
    DailyVisitor {
        date: json_display(value.get("date")),
        unique_visitors: json_f64(value.get("unique_visitors")).unwrap_or(0.0),
        total_page_views: json_f64(value.get("total_page_views")).unwrap_or(0.0),
        converted_visitors: json_f64(value.get("converted_visitors")).unwrap_or(0.0),
        avg_pages_per_visitor: json_f64(value.get("avg_pages_per_visitor")).unwrap_or(0.0),
    }
}

fn parse_popular_page(value: &Value) -> PopularPage {
    PopularPage {
        path: json_string(value.get("path")).unwrap_or_default(),
        views: json_f64(value.get("views")).unwrap_or(0.0),
        unique_visitors: json_f64(value.get("unique_visitors")).unwrap_or(0.0),
        conversion_rate: json_f64(value.get("conversion_rate")).unwrap_or(0.0),
    }
}

fn parse_event_engagement_day(value: &Value) -> EventEngagementDay {
    EventEngagementDay {
        date: json_display(value.get("date")),
        unique_visitors: json_f64(value.get("unique_visitors")).unwrap_or(0.0),
        total_page_views: json_f64(value.get("total_page_views")).unwrap_or(0.0),
        event_page_views: json_f64(value.get("event_page_views")).unwrap_or(0.0),
        registration_page_views: json_f64(value.get("registration_page_views")).unwrap_or(0.0),
        avg_time_spent: json_f64(value.get("avg_time_spent")).unwrap_or(0.0),
    }
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

fn json_display(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn json_u32(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(number) => number.as_u64().map(|n| n as u32),
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

fn format_thousands(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();
    let mut rest = digits.as_str();
    while rest.len() > 3 {
        let split = rest.len() - 3;
        grouped.insert_str(0, &format!(",{}", &rest[split..]));
        rest = &rest[..split];
    }
    grouped.insert_str(0, rest);
    grouped
}

#[cfg(target_arch = "wasm32")]
async fn send_authorized(token: &str, url: &str) -> Result<(u16, String), String> {
    let response = gloo_net::http::Request::get(url)
        .header("Authorization", &crate::auth::bearer_authorization(token))
        .send()
        .await
        .map_err(|_| NETWORK_ERROR.to_string())?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Ok((status, body))
}
