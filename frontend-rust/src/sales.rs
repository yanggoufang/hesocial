use serde_json::Value;

use crate::permissions::{AuthSnapshot, RouteGuard, USER_ROUTE_FALLBACK, permissions};

pub const PAGE_SIZE: u32 = 20;
pub const NETWORK_ERROR: &str = "Network error occurred";
pub const LEADS_FETCH_FALLBACK: &str = "Failed to fetch leads";
pub const OPPORTUNITIES_FETCH_FALLBACK: &str = "Failed to fetch opportunities";
pub const METRICS_FETCH_FALLBACK: &str = "Failed to fetch metrics";
pub const PIPELINE_FETCH_FALLBACK: &str = "Failed to fetch pipeline stages";
pub const ACTIVITIES_FETCH_FALLBACK: &str = "Failed to fetch sales activities";
pub const TEAM_FETCH_FALLBACK: &str = "Failed to fetch sales team";
pub const ACCESS_TOKEN_REQUIRED: &str = "Access token required";

pub const LEADS_API_PATH: &str = "/api/sales/leads";
pub const OPPORTUNITIES_API_PATH: &str = "/api/sales/opportunities";
pub const METRICS_API_PATH: &str = "/api/sales/metrics";
pub const PIPELINE_STAGES_API_PATH: &str = "/api/sales/pipeline/stages";
pub const ACTIVITIES_API_PATH: &str = "/api/sales/activities";
pub const TEAM_API_PATH: &str = "/api/sales/team";

pub const FUNNEL_VIEW_WIDTH: f64 = 400.0;
pub const FUNNEL_VIEW_HEIGHT: f64 = 280.0;
const FUNNEL_MIN_RATIO: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SalesTab {
    #[default]
    Leads,
    Opportunities,
    Metrics,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LeadFilters {
    pub search: String,
    pub status: String,
    pub source: String,
    pub assigned_to: String,
    pub page: u32,
    pub limit: u32,
}

impl LeadFilters {
    pub fn with_page_size() -> Self {
        Self {
            page: 1,
            limit: PAGE_SIZE,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct OpportunityFilters {
    pub search: String,
    pub stage: String,
    pub assigned_to: String,
    pub page: u32,
    pub limit: u32,
}

impl OpportunityFilters {
    pub fn with_page_size() -> Self {
        Self {
            page: 1,
            limit: PAGE_SIZE,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SalesPagination {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SalesLead {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub lead_score: f64,
    pub annual_income: f64,
    pub net_worth: f64,
    pub source: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub interests: Vec<String>,
    pub last_contact_date: Option<String>,
    pub next_follow_up_date: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OpportunityLead {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SalesOpportunity {
    pub id: String,
    pub lead_id: String,
    pub name: String,
    pub description: Option<String>,
    pub stage: String,
    pub probability: f64,
    pub value: f64,
    pub membership_tier: String,
    pub expected_close_date: String,
    pub actual_close_date: Option<String>,
    pub assigned_to: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub lead: OpportunityLead,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SalesMetrics {
    pub total_leads: f64,
    pub qualified_leads: f64,
    pub total_opportunities: f64,
    pub total_pipeline_value: f64,
    pub conversion_rate: f64,
    pub average_deal_size: f64,
    pub sales_cycle_length: f64,
    pub win_rate: f64,
    pub monthly_revenue: f64,
    pub quarterly_revenue: f64,
    pub yearly_revenue: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PipelineStage {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i64,
    pub default_probability: f64,
    pub is_active: bool,
    pub color_code: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct PipelineStageStat {
    pub stage: PipelineStage,
    pub count: u32,
    pub value: f64,
    pub conversion_from_previous: f64,
    pub share_of_first: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct FunnelBand {
    pub index: usize,
    pub y: f64,
    pub height: f64,
    pub top_left_x: f64,
    pub top_right_x: f64,
    pub bottom_left_x: f64,
    pub bottom_right_x: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SalesActivity {
    pub id: String,
    pub lead_id: Option<String>,
    pub opportunity_id: Option<String>,
    pub activity_type: String,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub outcome: Option<String>,
    pub duration_minutes: Option<f64>,
    pub scheduled_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub created_by_first_name: Option<String>,
    pub created_by_last_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SalesTeamMember {
    pub id: String,
    pub user_id: Option<String>,
    pub role: String,
    pub territory: Option<String>,
    pub commission_rate: Option<f64>,
    pub quota_amount: f64,
    pub is_active: bool,
    pub hire_date: Option<String>,
    pub manager_id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub manager_first_name: Option<String>,
    pub manager_last_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct LeadsView {
    pub leads: Vec<SalesLead>,
    pub pagination: SalesPagination,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OpportunitiesView {
    pub opportunities: Vec<SalesOpportunity>,
    pub pagination: SalesPagination,
}

pub fn admin_route_guard(restoring: bool, snapshot: &AuthSnapshot) -> RouteGuard {
    if restoring {
        RouteGuard::Loading
    } else {
        let can = permissions(snapshot);
        if can.access && can.view_admin {
            RouteGuard::Allow
        } else {
            RouteGuard::Redirect(USER_ROUTE_FALLBACK)
        }
    }
}

pub fn leads_query_string(filters: &LeadFilters) -> String {
    let mut out = String::new();
    let page = if filters.page == 0 { 1 } else { filters.page };
    let limit = if filters.limit == 0 {
        PAGE_SIZE
    } else {
        filters.limit
    };
    push_param(&mut out, "page", &page.to_string());
    push_param(&mut out, "limit", &limit.to_string());
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    if !filters.status.is_empty() {
        push_param(&mut out, "status", &filters.status);
    }
    if !filters.source.is_empty() {
        push_param(&mut out, "source", &filters.source);
    }
    if !filters.assigned_to.is_empty() {
        push_param(&mut out, "assignedTo", &filters.assigned_to);
    }
    out
}

pub fn opportunities_query_string(filters: &OpportunityFilters) -> String {
    let mut out = String::new();
    let page = if filters.page == 0 { 1 } else { filters.page };
    let limit = if filters.limit == 0 {
        PAGE_SIZE
    } else {
        filters.limit
    };
    push_param(&mut out, "page", &page.to_string());
    push_param(&mut out, "limit", &limit.to_string());
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    if !filters.stage.is_empty() {
        push_param(&mut out, "stage", &filters.stage);
    }
    if !filters.assigned_to.is_empty() {
        push_param(&mut out, "assignedTo", &filters.assigned_to);
    }
    out
}

pub fn conversion_percent(current: u32, previous: u32) -> f64 {
    if previous == 0 {
        0.0
    } else {
        (current as f64 * 100.0) / previous as f64
    }
}

pub fn pipeline_stage_stats(
    stages: &[PipelineStage],
    opportunities: &[SalesOpportunity],
) -> Vec<PipelineStageStat> {
    let mut stats = Vec::with_capacity(stages.len());
    let mut previous_count = 0u32;
    let mut first_count = 0u32;
    for (index, stage) in stages.iter().enumerate() {
        let mut count = 0u32;
        let mut value = 0.0;
        for opportunity in opportunities {
            if opportunity.stage == stage.name {
                count = count.saturating_add(1);
                value += opportunity.value;
            }
        }
        if index == 0 {
            first_count = count;
        }
        let conversion_from_previous = if index == 0 {
            conversion_percent(count, count)
        } else {
            conversion_percent(count, previous_count)
        };
        let share_of_first = conversion_percent(count, first_count);
        stats.push(PipelineStageStat {
            stage: stage.clone(),
            count,
            value,
            conversion_from_previous,
            share_of_first,
        });
        previous_count = count;
    }
    stats
}

pub fn funnel_bands(counts: &[u32], width: f64, height: f64) -> Vec<FunnelBand> {
    if counts.is_empty() || width <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let n = counts.len();
    let band_height = height / n as f64;
    let max = counts.iter().copied().max().unwrap_or(0);
    let mut bands = Vec::with_capacity(n);
    for index in 0..n {
        let top = band_width(counts[index], max, width, index, n);
        let bottom = if index + 1 < n {
            band_width(counts[index + 1], max, width, index + 1, n)
        } else {
            top.min(width * FUNNEL_MIN_RATIO)
        };
        let y = index as f64 * band_height;
        let top_inset = ((width - top) / 2.0).max(0.0);
        let bottom_inset = ((width - bottom) / 2.0).max(0.0);
        bands.push(FunnelBand {
            index,
            y,
            height: band_height,
            top_left_x: top_inset,
            top_right_x: top_inset + top,
            bottom_left_x: bottom_inset,
            bottom_right_x: bottom_inset + bottom,
        });
    }
    bands
}

pub fn funnel_polygon_points(band: &FunnelBand) -> String {
    format!(
        "{},{} {},{} {},{} {},{}",
        band.top_left_x,
        band.y,
        band.top_right_x,
        band.y,
        band.bottom_right_x,
        band.y + band.height,
        band.bottom_left_x,
        band.y + band.height
    )
}

pub fn funnel_counts(stats: &[PipelineStageStat]) -> Vec<u32> {
    stats.iter().map(|stat| stat.count).collect()
}

pub fn format_currency(amount: f64) -> String {
    if !amount.is_finite() {
        return "NT$ 0".to_string();
    }
    let rounded = amount.round();
    format!("NT$ {}", format_thousands(rounded))
}

pub fn format_one_decimal(value: f64) -> String {
    if !value.is_finite() {
        return "0.0".to_string();
    }
    format!("{value:.1}")
}

pub fn score_bar_percent(score: f64) -> f64 {
    if !score.is_finite() {
        return 0.0;
    }
    score.clamp(0.0, 100.0)
}

pub fn format_sales_date(iso: &str) -> String {
    let trimmed = iso.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }
    #[cfg(target_arch = "wasm32")]
    {
        return format_sales_date_js(trimmed);
    }
    #[cfg(not(target_arch = "wasm32"))]
    format_sales_date_native(trimmed)
}

pub fn lead_status_label(status: &str) -> String {
    match status {
        "new" => "新線索".to_string(),
        "qualified" => "已審核".to_string(),
        "contacted" => "已聯繫".to_string(),
        "nurturing" => "培養中".to_string(),
        "proposal" => "提案階段".to_string(),
        "negotiation" => "談判中".to_string(),
        "closed_won" => "成交".to_string(),
        "closed_lost" => "失單".to_string(),
        other => other.to_string(),
    }
}

pub fn lead_status_class(status: &str) -> &'static str {
    match status {
        "new" => "bg-blue-100 text-blue-800",
        "qualified" => "bg-green-100 text-green-800",
        "contacted" => "bg-yellow-100 text-yellow-800",
        "nurturing" => "bg-purple-100 text-purple-800",
        "proposal" => "bg-orange-100 text-orange-800",
        "negotiation" => "bg-indigo-100 text-indigo-800",
        "closed_won" => "bg-emerald-100 text-emerald-800",
        "closed_lost" => "bg-red-100 text-red-800",
        _ => "bg-gray-100 text-gray-800",
    }
}

pub fn opportunity_stage_label(stage: &str) -> String {
    match stage {
        "qualification" => "資格審核".to_string(),
        "needs_analysis" => "需求分析".to_string(),
        "proposal" => "提案階段".to_string(),
        "negotiation" => "談判中".to_string(),
        "closed_won" => "成交".to_string(),
        "closed_lost" => "失單".to_string(),
        other => other.to_string(),
    }
}

pub fn membership_tier_badge_class(tier: &str) -> &'static str {
    match tier {
        "Platinum" => "bg-gray-100 text-gray-800 border-gray-300",
        "Diamond" => "bg-blue-100 text-blue-800 border-blue-300",
        "Black Card" => "bg-black text-white border-black",
        _ => "bg-gray-100 text-gray-800 border-gray-300",
    }
}

pub fn lead_display_name(lead: &SalesLead) -> String {
    format!("{} {}", lead.first_name, lead.last_name)
        .trim()
        .to_string()
}

pub fn opportunity_lead_name(lead: &OpportunityLead) -> String {
    format!("{} {}", lead.first_name, lead.last_name)
        .trim()
        .to_string()
}

pub fn parse_leads_response(status: u16, body: &str) -> Result<LeadsView, String> {
    let value = parse_success_root(status, body, LEADS_FETCH_FALLBACK)?;
    let leads = value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_lead).collect())
        .ok_or_else(|| LEADS_FETCH_FALLBACK.to_string())?;
    let pagination = parse_pagination(value.get("pagination"), PAGE_SIZE);
    Ok(LeadsView { leads, pagination })
}

pub fn parse_opportunities_response(status: u16, body: &str) -> Result<OpportunitiesView, String> {
    let value = parse_success_root(status, body, OPPORTUNITIES_FETCH_FALLBACK)?;
    let opportunities = value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_opportunity).collect())
        .ok_or_else(|| OPPORTUNITIES_FETCH_FALLBACK.to_string())?;
    let pagination = parse_pagination(value.get("pagination"), PAGE_SIZE);
    Ok(OpportunitiesView {
        opportunities,
        pagination,
    })
}

pub fn parse_metrics_response(status: u16, body: &str) -> Result<SalesMetrics, String> {
    let value = parse_success_data(status, body, METRICS_FETCH_FALLBACK)?;
    Ok(SalesMetrics {
        total_leads: json_f64(value.get("totalLeads")).unwrap_or(0.0),
        qualified_leads: json_f64(value.get("qualifiedLeads")).unwrap_or(0.0),
        total_opportunities: json_f64(value.get("totalOpportunities")).unwrap_or(0.0),
        total_pipeline_value: json_f64(value.get("totalPipelineValue")).unwrap_or(0.0),
        conversion_rate: json_f64(value.get("conversionRate")).unwrap_or(0.0),
        average_deal_size: json_f64(value.get("averageDealSize")).unwrap_or(0.0),
        sales_cycle_length: json_f64(value.get("salesCycleLength")).unwrap_or(0.0),
        win_rate: json_f64(value.get("winRate")).unwrap_or(0.0),
        monthly_revenue: json_f64(value.get("monthlyRevenue")).unwrap_or(0.0),
        quarterly_revenue: json_f64(value.get("quarterlyRevenue")).unwrap_or(0.0),
        yearly_revenue: json_f64(value.get("yearlyRevenue")).unwrap_or(0.0),
    })
}

pub fn parse_pipeline_stages_response(
    status: u16,
    body: &str,
) -> Result<Vec<PipelineStage>, String> {
    let value = parse_success_root(status, body, PIPELINE_FETCH_FALLBACK)?;
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_pipeline_stage).collect())
        .ok_or_else(|| PIPELINE_FETCH_FALLBACK.to_string())
}

pub fn parse_activities_response(status: u16, body: &str) -> Result<Vec<SalesActivity>, String> {
    let value = parse_success_root(status, body, ACTIVITIES_FETCH_FALLBACK)?;
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_activity).collect())
        .ok_or_else(|| ACTIVITIES_FETCH_FALLBACK.to_string())
}

pub fn parse_team_response(status: u16, body: &str) -> Result<Vec<SalesTeamMember>, String> {
    let value = parse_success_root(status, body, TEAM_FETCH_FALLBACK)?;
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_team_member).collect())
        .ok_or_else(|| TEAM_FETCH_FALLBACK.to_string())
}

pub async fn fetch_leads(filters: &LeadFilters) -> Result<LeadsView, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{LEADS_API_PATH}?{}", leads_query_string(filters));
        return authorized_get(&url, LEADS_FETCH_FALLBACK, parse_leads_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filters;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_opportunities(
    filters: &OpportunityFilters,
) -> Result<OpportunitiesView, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!(
            "{OPPORTUNITIES_API_PATH}?{}",
            opportunities_query_string(filters)
        );
        return authorized_get(
            &url,
            OPPORTUNITIES_FETCH_FALLBACK,
            parse_opportunities_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filters;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_metrics() -> Result<SalesMetrics, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            METRICS_API_PATH,
            METRICS_FETCH_FALLBACK,
            parse_metrics_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn fetch_pipeline_stages() -> Result<Vec<PipelineStage>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            PIPELINE_STAGES_API_PATH,
            PIPELINE_FETCH_FALLBACK,
            parse_pipeline_stages_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn fetch_activities() -> Result<Vec<SalesActivity>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            ACTIVITIES_API_PATH,
            ACTIVITIES_FETCH_FALLBACK,
            parse_activities_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn fetch_team() -> Result<Vec<SalesTeamMember>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(TEAM_API_PATH, TEAM_FETCH_FALLBACK, parse_team_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn authorized_get<T>(
    url: &str,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let token = crate::auth::read_stored_token().ok_or_else(|| NETWORK_ERROR.to_string())?;
    let builder = gloo_net::http::Request::get(url)
        .header("Authorization", &crate::auth::bearer_authorization(&token));
    let response = builder
        .send()
        .await
        .map_err(|_| NETWORK_ERROR.to_string())?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    parse(status, &text).map_err(|err| {
        if err.is_empty() {
            fallback.to_string()
        } else {
            err
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn format_sales_date_js(iso: &str) -> String {
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_str(iso));
    if date.get_time().is_nan() {
        return format_sales_date_native(iso);
    }
    date.to_locale_date_string("zh-TW", &js_sys::Object::new())
        .into()
}

fn format_sales_date_native(iso: &str) -> String {
    let date = iso.get(..10).unwrap_or(iso);
    let mut parts = date.split('-');
    let Some(year) = parts.next() else {
        return iso.to_string();
    };
    let Some(month) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
        return iso.to_string();
    };
    let Some(day) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
        return iso.to_string();
    };
    if year.len() != 4 {
        return iso.to_string();
    }
    format!("{year}/{month}/{day}")
}

fn band_width(count: u32, max: u32, width: f64, index: usize, n: usize) -> f64 {
    if max == 0 {
        let t = if n <= 1 {
            0.0
        } else {
            index as f64 / (n as f64 - 1.0)
        };
        return width * (1.0 - (1.0 - FUNNEL_MIN_RATIO) * t);
    }
    let ratio = count as f64 / max as f64;
    width * (FUNNEL_MIN_RATIO + (1.0 - FUNNEL_MIN_RATIO) * ratio)
}

fn parse_success_root(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    if status == 401 {
        return Err(error_from_body(body, ACCESS_TOKEN_REQUIRED));
    }
    let value: Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => {
            if !(200..300).contains(&status) {
                return Err(fallback.to_string());
            }
            return Err(fallback.to_string());
        }
    };
    if !(200..300).contains(&status) {
        return Err(json_error_message(&value, fallback));
    }
    if value.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(value)
    } else {
        Err(json_error_message(&value, fallback))
    }
}

fn parse_success_data(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    let value = parse_success_root(status, body, fallback)?;
    match value.get("data") {
        Some(Value::Null) | None => Err(fallback.to_string()),
        Some(data) => Ok(data.clone()),
    }
}

fn error_from_body(body: &str, fallback: &'static str) -> String {
    match serde_json::from_str::<Value>(body) {
        Ok(value) => json_error_message(&value, fallback),
        Err(_) => fallback.to_string(),
    }
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

fn parse_pagination(value: Option<&Value>, default_limit: u32) -> SalesPagination {
    let Some(value) = value else {
        return SalesPagination {
            page: 1,
            limit: default_limit,
            total: 0,
            total_pages: 0,
        };
    };
    SalesPagination {
        page: json_u32(value.get("page")).unwrap_or(1),
        limit: json_u32(value.get("limit")).unwrap_or(default_limit),
        total: json_u32(value.get("total")).unwrap_or(0),
        total_pages: json_u32(value.get("totalPages"))
            .or_else(|| json_u32(value.get("total_pages")))
            .unwrap_or(0),
    }
}

fn parse_lead(value: &Value) -> Option<SalesLead> {
    let id = json_id(value.get("id"))?;
    Some(SalesLead {
        id,
        first_name: json_string(value.get("firstName"))
            .or_else(|| json_string(value.get("first_name")))
            .unwrap_or_default(),
        last_name: json_string(value.get("lastName"))
            .or_else(|| json_string(value.get("last_name")))
            .unwrap_or_default(),
        email: json_string(value.get("email")).unwrap_or_default(),
        phone: json_string(value.get("phone")).filter(|value| !value.is_empty()),
        company: json_string(value.get("company")).filter(|value| !value.is_empty()),
        position: json_string(value.get("position"))
            .or_else(|| json_string(value.get("jobTitle")))
            .or_else(|| json_string(value.get("job_title")))
            .filter(|value| !value.is_empty()),
        lead_score: json_f64(value.get("leadScore"))
            .or_else(|| json_f64(value.get("lead_score")))
            .unwrap_or(0.0),
        annual_income: json_f64(value.get("annualIncome"))
            .or_else(|| json_f64(value.get("annual_income")))
            .unwrap_or(0.0),
        net_worth: json_f64(value.get("netWorth"))
            .or_else(|| json_f64(value.get("net_worth")))
            .unwrap_or(0.0),
        source: json_string(value.get("source")).unwrap_or_else(|| "website".to_string()),
        status: json_string(value.get("status")).unwrap_or_else(|| "new".to_string()),
        assigned_to: json_id(value.get("assignedTo")).or_else(|| json_id(value.get("assigned_to"))),
        interests: parse_interests(value.get("interests")),
        last_contact_date: json_string(value.get("lastContactDate"))
            .or_else(|| json_string(value.get("last_contact_date")))
            .filter(|value| !value.is_empty()),
        next_follow_up_date: json_string(value.get("nextFollowUpDate"))
            .or_else(|| json_string(value.get("next_follow_up_date")))
            .filter(|value| !value.is_empty()),
        notes: json_string(value.get("notes")).filter(|value| !value.is_empty()),
        created_at: json_string(value.get("createdAt"))
            .or_else(|| json_string(value.get("created_at")))
            .unwrap_or_default(),
        updated_at: json_string(value.get("updatedAt"))
            .or_else(|| json_string(value.get("updated_at")))
            .unwrap_or_default(),
    })
}

fn parse_opportunity(value: &Value) -> Option<SalesOpportunity> {
    let id = json_id(value.get("id"))?;
    Some(SalesOpportunity {
        id,
        lead_id: json_id(value.get("leadId"))
            .or_else(|| json_id(value.get("lead_id")))
            .unwrap_or_default(),
        name: json_string(value.get("name")).unwrap_or_default(),
        description: json_string(value.get("description")).filter(|value| !value.is_empty()),
        stage: json_string(value.get("stage")).unwrap_or_else(|| "qualification".to_string()),
        probability: json_f64(value.get("probability")).unwrap_or(0.0),
        value: json_f64(value.get("value")).unwrap_or(0.0),
        membership_tier: json_string(value.get("membershipTier"))
            .or_else(|| json_string(value.get("membership_tier")))
            .unwrap_or_else(|| "Platinum".to_string()),
        expected_close_date: json_string(value.get("expectedCloseDate"))
            .or_else(|| json_string(value.get("expected_close_date")))
            .unwrap_or_default(),
        actual_close_date: json_string(value.get("actualCloseDate"))
            .or_else(|| json_string(value.get("actual_close_date")))
            .filter(|value| !value.is_empty()),
        assigned_to: json_id(value.get("assignedTo")).or_else(|| json_id(value.get("assigned_to"))),
        notes: json_string(value.get("notes")).filter(|value| !value.is_empty()),
        created_at: json_string(value.get("createdAt"))
            .or_else(|| json_string(value.get("created_at")))
            .unwrap_or_default(),
        updated_at: json_string(value.get("updatedAt"))
            .or_else(|| json_string(value.get("updated_at")))
            .unwrap_or_default(),
        lead: parse_opportunity_lead(value),
    })
}

fn parse_opportunity_lead(value: &Value) -> OpportunityLead {
    if let Some(lead) = value.get("lead").filter(|lead| lead.is_object()) {
        return OpportunityLead {
            first_name: json_string(lead.get("firstName"))
                .or_else(|| json_string(lead.get("first_name")))
                .unwrap_or_default(),
            last_name: json_string(lead.get("lastName"))
                .or_else(|| json_string(lead.get("last_name")))
                .unwrap_or_default(),
            email: json_string(lead.get("email")).unwrap_or_default(),
        };
    }
    OpportunityLead {
        first_name: json_string(value.get("lead_first_name")).unwrap_or_default(),
        last_name: json_string(value.get("lead_last_name")).unwrap_or_default(),
        email: json_string(value.get("lead_email")).unwrap_or_default(),
    }
}

fn parse_pipeline_stage(value: &Value) -> Option<PipelineStage> {
    let id = json_id(value.get("id"))?;
    let name = json_string(value.get("name")).filter(|value| !value.is_empty())?;
    Some(PipelineStage {
        id,
        name,
        description: json_string(value.get("description")).filter(|value| !value.is_empty()),
        display_order: json_i64(value.get("display_order"))
            .or_else(|| json_i64(value.get("displayOrder")))
            .unwrap_or(0),
        default_probability: json_f64(value.get("default_probability"))
            .or_else(|| json_f64(value.get("defaultProbability")))
            .unwrap_or(0.0),
        is_active: json_bool(value.get("is_active"))
            .or_else(|| json_bool(value.get("isActive")))
            .unwrap_or(true),
        color_code: json_string(value.get("color_code"))
            .or_else(|| json_string(value.get("colorCode")))
            .filter(|value| !value.is_empty()),
    })
}

fn parse_activity(value: &Value) -> Option<SalesActivity> {
    let id = json_id(value.get("id"))?;
    Some(SalesActivity {
        id,
        lead_id: json_id(value.get("lead_id")).or_else(|| json_id(value.get("leadId"))),
        opportunity_id: json_id(value.get("opportunity_id"))
            .or_else(|| json_id(value.get("opportunityId"))),
        activity_type: json_string(value.get("activity_type"))
            .or_else(|| json_string(value.get("activityType")))
            .unwrap_or_default(),
        subject: json_string(value.get("subject")).filter(|value| !value.is_empty()),
        description: json_string(value.get("description")).filter(|value| !value.is_empty()),
        outcome: json_string(value.get("outcome")).filter(|value| !value.is_empty()),
        duration_minutes: json_f64(value.get("duration_minutes"))
            .or_else(|| json_f64(value.get("durationMinutes"))),
        scheduled_at: json_string(value.get("scheduled_at"))
            .or_else(|| json_string(value.get("scheduledAt")))
            .filter(|value| !value.is_empty()),
        completed_at: json_string(value.get("completed_at"))
            .or_else(|| json_string(value.get("completedAt")))
            .filter(|value| !value.is_empty()),
        created_by: json_id(value.get("created_by")).or_else(|| json_id(value.get("createdBy"))),
        created_at: json_string(value.get("created_at"))
            .or_else(|| json_string(value.get("createdAt")))
            .unwrap_or_default(),
        created_by_first_name: json_string(value.get("created_by_first_name"))
            .or_else(|| json_string(value.get("createdByFirstName"))),
        created_by_last_name: json_string(value.get("created_by_last_name"))
            .or_else(|| json_string(value.get("createdByLastName"))),
    })
}

fn parse_team_member(value: &Value) -> Option<SalesTeamMember> {
    let id = json_id(value.get("id"))?;
    Some(SalesTeamMember {
        id,
        user_id: json_id(value.get("user_id")).or_else(|| json_id(value.get("userId"))),
        role: json_string(value.get("role")).unwrap_or_default(),
        territory: json_string(value.get("territory")).filter(|value| !value.is_empty()),
        commission_rate: json_f64(value.get("commission_rate"))
            .or_else(|| json_f64(value.get("commissionRate"))),
        quota_amount: json_f64(value.get("quota_amount"))
            .or_else(|| json_f64(value.get("quotaAmount")))
            .unwrap_or(0.0),
        is_active: json_bool(value.get("is_active"))
            .or_else(|| json_bool(value.get("isActive")))
            .unwrap_or(false),
        hire_date: json_string(value.get("hire_date"))
            .or_else(|| json_string(value.get("hireDate")))
            .filter(|value| !value.is_empty()),
        manager_id: json_id(value.get("manager_id")).or_else(|| json_id(value.get("managerId"))),
        first_name: json_string(value.get("first_name"))
            .or_else(|| json_string(value.get("firstName"))),
        last_name: json_string(value.get("last_name"))
            .or_else(|| json_string(value.get("lastName"))),
        email: json_string(value.get("email")),
        manager_first_name: json_string(value.get("manager_first_name"))
            .or_else(|| json_string(value.get("managerFirstName"))),
        manager_last_name: json_string(value.get("manager_last_name"))
            .or_else(|| json_string(value.get("managerLastName"))),
    })
}

fn parse_interests(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(raw)) if raw.is_empty() => Vec::new(),
        Some(Value::String(raw)) => serde_json::from_str::<Value>(raw)
            .ok()
            .map(|parsed| parse_interests(Some(&parsed)))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn format_thousands(amount: f64) -> String {
    let negative = amount < 0.0;
    let abs = amount.abs();
    let whole = abs as u64;
    let mut digits = whole.to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        grouped.insert_str(0, &format!(",{}", &digits[split..]));
        digits.truncate(split);
    }
    grouped.insert_str(0, &digits);
    if negative {
        grouped.insert(0, '-');
    }
    grouped
}

fn push_param(out: &mut String, key: &str, value: &str) {
    if !out.is_empty() {
        out.push('&');
    }
    out.push_str(key);
    out.push('=');
    out.push_str(&form_encode(value));
}

fn form_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn json_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn json_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn json_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_u64().map(|v| v as i64))
            .or_else(|| n.as_f64().map(|v| v as i64)),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_u32(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(n) => n
            .as_u64()
            .map(|v| v as u32)
            .or_else(|| n.as_i64().and_then(|v| u32::try_from(v).ok()))
            .or_else(|| n.as_f64().and_then(|v| u32::try_from(v as i64).ok())),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_bool(value: Option<&Value>) -> Option<bool> {
    match value? {
        Value::Bool(flag) => Some(*flag),
        Value::Number(n) => n
            .as_i64()
            .map(|v| v != 0)
            .or_else(|| n.as_f64().map(|v| v != 0.0)),
        Value::String(s) => match s.as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}
