//! Sales CRM domain ported from `backend/src/controllers/salesController.ts`.
//!
//! Everything here is pure so the Express quirks (lead scoring thresholds,
//! reporting-period math, the metrics coercion rules) stay unit-testable off
//! the wasm target.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::pagination::{js_parse_f64, number_json};

const MS_PER_DAY: f64 = 86_400_000.0;

/// `annualIncome >= 20000000` in the Express controller (NT$20M).
const ANNUAL_INCOME_SIGNAL: f64 = 20_000_000.0;
/// `netWorth >= 100000000` in the Express controller (NT$100M).
const NET_WORTH_SIGNAL: f64 = 100_000_000.0;
const WEALTH_SCORE: i64 = 40;
const BLACK_CARD_SCORE: i64 = 20;
const DIAMOND_SCORE: i64 = 15;
const PLATINUM_SCORE: i64 = 10;

/// D1 `sales_leads.status` CHECK vocabulary.
pub const LEAD_STATUSES: [&str; 8] = [
    "new",
    "qualified",
    "contacted",
    "nurturing",
    "proposal",
    "negotiation",
    "closed_won",
    "closed_lost",
];

/// D1 `sales_opportunities.stage` CHECK vocabulary.
pub const OPPORTUNITY_STAGES: [&str; 6] = [
    "qualification",
    "needs_analysis",
    "proposal",
    "negotiation",
    "closed_won",
    "closed_lost",
];

/// D1 CHECK vocabulary shared by `interested_membership_tier` and
/// `sales_opportunities.membership_tier`.
pub const MEMBERSHIP_TIERS: [&str; 3] = ["Platinum", "Diamond", "Black Card"];

/// D1 `sales_activities.activity_type` CHECK vocabulary.
pub const ACTIVITY_TYPES: [&str; 7] = [
    "call",
    "email",
    "meeting",
    "demo",
    "proposal",
    "follow_up",
    "note",
];

/// Body key (camelCase or the raw column name) to a writable `sales_leads`
/// column. Express interpolates `Object.keys(req.body)` straight into the `SET`
/// clause, so any key becomes a column reference; the worker resolves through
/// this map instead, which closes the injection and keeps `id`/timestamps out
/// of client reach.
const LEAD_UPDATE_FIELDS: &[(&str, &str)] = &[
    ("first_name", "first_name"),
    ("firstName", "first_name"),
    ("last_name", "last_name"),
    ("lastName", "last_name"),
    ("email", "email"),
    ("phone", "phone"),
    ("company", "company"),
    ("job_title", "job_title"),
    ("jobTitle", "job_title"),
    ("position", "job_title"),
    ("annual_income", "annual_income"),
    ("annualIncome", "annual_income"),
    ("net_worth", "net_worth"),
    ("netWorth", "net_worth"),
    ("source", "source"),
    ("referral_code", "referral_code"),
    ("referralCode", "referral_code"),
    ("lead_score", "lead_score"),
    ("leadScore", "lead_score"),
    ("status", "status"),
    ("interested_membership_tier", "interested_membership_tier"),
    ("interestedMembershipTier", "interested_membership_tier"),
    ("budget_range", "budget_range"),
    ("budgetRange", "budget_range"),
    ("timeline", "timeline"),
    ("pain_points", "pain_points"),
    ("painPoints", "pain_points"),
    ("interests", "interests"),
    ("notes", "notes"),
    ("last_contact_date", "last_contact_date"),
    ("lastContactDate", "last_contact_date"),
    ("next_follow_up_date", "next_follow_up_date"),
    ("nextFollowUpDate", "next_follow_up_date"),
    ("assigned_to", "assigned_to"),
    ("assignedTo", "assigned_to"),
];

/// Writable `sales_opportunities` columns. The table has no `notes` column even
/// though `salesService.ts` types one, so that key stays unmapped.
const OPPORTUNITY_UPDATE_FIELDS: &[(&str, &str)] = &[
    ("lead_id", "lead_id"),
    ("leadId", "lead_id"),
    ("name", "name"),
    ("description", "description"),
    ("stage", "stage"),
    ("probability", "probability"),
    ("value", "value"),
    ("expected_close_date", "expected_close_date"),
    ("expectedCloseDate", "expected_close_date"),
    ("actual_close_date", "actual_close_date"),
    ("actualCloseDate", "actual_close_date"),
    ("membership_tier", "membership_tier"),
    ("membershipTier", "membership_tier"),
    ("payment_terms", "payment_terms"),
    ("paymentTerms", "payment_terms"),
    ("close_reason", "close_reason"),
    ("closeReason", "close_reason"),
    ("assigned_to", "assigned_to"),
    ("assignedTo", "assigned_to"),
];

#[derive(Clone, Debug, Deserialize)]
pub struct LeadRow {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub annual_income: Option<i64>,
    pub net_worth: Option<i64>,
    pub source: String,
    pub referral_code: Option<String>,
    pub lead_score: Option<i64>,
    pub status: Option<String>,
    pub interested_membership_tier: Option<String>,
    pub budget_range: Option<String>,
    pub timeline: Option<String>,
    pub pain_points: Option<String>,
    pub interests: Option<String>,
    pub notes: Option<String>,
    pub last_contact_date: Option<String>,
    pub next_follow_up_date: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub assigned_to_first_name: Option<String>,
    #[serde(default)]
    pub assigned_to_last_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpportunityRow {
    pub id: i64,
    pub lead_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub stage: String,
    pub probability: Option<i64>,
    pub value: f64,
    pub expected_close_date: Option<String>,
    pub actual_close_date: Option<String>,
    pub membership_tier: String,
    pub payment_terms: Option<String>,
    pub close_reason: Option<String>,
    pub assigned_to: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub lead_first_name: Option<String>,
    #[serde(default)]
    pub lead_last_name: Option<String>,
    #[serde(default)]
    pub lead_email: Option<String>,
    #[serde(default)]
    pub assigned_to_first_name: Option<String>,
    #[serde(default)]
    pub assigned_to_last_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActivityRow {
    pub id: i64,
    pub lead_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub outcome: Option<String>,
    pub duration_minutes: Option<i64>,
    pub scheduled_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub created_by_first_name: Option<String>,
    #[serde(default)]
    pub created_by_last_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PipelineStageRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i64,
    pub default_probability: Option<i64>,
    pub is_active: Option<i64>,
    pub color_code: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamMemberRow {
    pub id: i64,
    pub user_id: String,
    pub role: String,
    pub territory: Option<String>,
    pub commission_rate: Option<f64>,
    pub quota_amount: Option<i64>,
    pub is_active: Option<i64>,
    pub hire_date: Option<String>,
    pub manager_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub manager_first_name: Option<String>,
    #[serde(default)]
    pub manager_last_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct LeadMetricsRow {
    pub total_leads: i64,
    pub qualified_leads: i64,
    pub converted_leads: i64,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct OpportunityMetricsRow {
    pub total_opportunities: i64,
    pub total_pipeline_value: Option<f64>,
    pub average_deal_size: Option<f64>,
    pub won_opportunities: i64,
    pub won_revenue: Option<f64>,
}

fn row_json(fields: &[(&str, Value)]) -> Value {
    let mut map = Map::new();
    for (key, value) in fields {
        map.insert((*key).to_owned(), value.clone());
    }
    Value::Object(map)
}

/// D1 stores booleans as INTEGER 0/1; DuckDB returns a real JS boolean. The
/// response keeps the Express representation.
fn flag_json(value: Option<i64>) -> Value {
    match value {
        Some(raw) => json!(raw != 0),
        None => Value::Null,
    }
}

pub fn lead_json(row: &LeadRow) -> Value {
    row_json(&[
        ("id", json!(row.id)),
        ("first_name", json!(row.first_name)),
        ("last_name", json!(row.last_name)),
        ("email", json!(row.email)),
        ("phone", json!(row.phone)),
        ("company", json!(row.company)),
        ("job_title", json!(row.job_title)),
        ("annual_income", json!(row.annual_income)),
        ("net_worth", json!(row.net_worth)),
        ("source", json!(row.source)),
        ("referral_code", json!(row.referral_code)),
        ("lead_score", json!(row.lead_score)),
        ("status", json!(row.status)),
        (
            "interested_membership_tier",
            json!(row.interested_membership_tier),
        ),
        ("budget_range", json!(row.budget_range)),
        ("timeline", json!(row.timeline)),
        ("pain_points", json!(row.pain_points)),
        // Express hands the raw JSON column text to the client, which parses it
        // itself (`salesService.ts#parseInterests`), so the string is passed
        // through untouched.
        ("interests", json!(row.interests)),
        ("notes", json!(row.notes)),
        ("last_contact_date", json!(row.last_contact_date)),
        ("next_follow_up_date", json!(row.next_follow_up_date)),
        ("assigned_to", json!(row.assigned_to)),
        ("created_at", json!(row.created_at)),
        ("updated_at", json!(row.updated_at)),
        ("assigned_to_first_name", json!(row.assigned_to_first_name)),
        ("assigned_to_last_name", json!(row.assigned_to_last_name)),
    ])
}

pub fn opportunity_json(row: &OpportunityRow) -> Value {
    row_json(&[
        ("id", json!(row.id)),
        ("lead_id", json!(row.lead_id)),
        ("name", json!(row.name)),
        ("description", json!(row.description)),
        ("stage", json!(row.stage)),
        ("probability", json!(row.probability)),
        ("value", number_json(row.value)),
        ("expected_close_date", json!(row.expected_close_date)),
        ("actual_close_date", json!(row.actual_close_date)),
        ("membership_tier", json!(row.membership_tier)),
        ("payment_terms", json!(row.payment_terms)),
        ("close_reason", json!(row.close_reason)),
        ("assigned_to", json!(row.assigned_to)),
        ("created_at", json!(row.created_at)),
        ("updated_at", json!(row.updated_at)),
        ("lead_first_name", json!(row.lead_first_name)),
        ("lead_last_name", json!(row.lead_last_name)),
        ("lead_email", json!(row.lead_email)),
        ("assigned_to_first_name", json!(row.assigned_to_first_name)),
        ("assigned_to_last_name", json!(row.assigned_to_last_name)),
    ])
}

pub fn activity_json(row: &ActivityRow) -> Value {
    row_json(&[
        ("id", json!(row.id)),
        ("lead_id", json!(row.lead_id)),
        ("opportunity_id", json!(row.opportunity_id)),
        ("activity_type", json!(row.activity_type)),
        ("subject", json!(row.subject)),
        ("description", json!(row.description)),
        ("outcome", json!(row.outcome)),
        ("duration_minutes", json!(row.duration_minutes)),
        ("scheduled_at", json!(row.scheduled_at)),
        ("completed_at", json!(row.completed_at)),
        ("created_by", json!(row.created_by)),
        ("created_at", json!(row.created_at)),
        ("updated_at", json!(row.updated_at)),
        ("created_by_first_name", json!(row.created_by_first_name)),
        ("created_by_last_name", json!(row.created_by_last_name)),
    ])
}

pub fn pipeline_stage_json(row: &PipelineStageRow) -> Value {
    row_json(&[
        ("id", json!(row.id)),
        ("name", json!(row.name)),
        ("description", json!(row.description)),
        ("display_order", json!(row.display_order)),
        ("default_probability", json!(row.default_probability)),
        ("is_active", flag_json(row.is_active)),
        ("color_code", json!(row.color_code)),
        ("created_at", json!(row.created_at)),
        ("updated_at", json!(row.updated_at)),
    ])
}

pub fn team_member_json(row: &TeamMemberRow) -> Value {
    row_json(&[
        ("id", json!(row.id)),
        ("user_id", json!(row.user_id)),
        ("role", json!(row.role)),
        ("territory", json!(row.territory)),
        (
            "commission_rate",
            row.commission_rate.map_or(Value::Null, number_json),
        ),
        ("quota_amount", json!(row.quota_amount)),
        ("is_active", flag_json(row.is_active)),
        ("hire_date", json!(row.hire_date)),
        ("manager_id", json!(row.manager_id)),
        ("created_at", json!(row.created_at)),
        ("updated_at", json!(row.updated_at)),
        ("first_name", json!(row.first_name)),
        ("last_name", json!(row.last_name)),
        ("email", json!(row.email)),
        ("manager_first_name", json!(row.manager_first_name)),
        ("manager_last_name", json!(row.manager_last_name)),
    ])
}

/// JS truthiness, limited to the JSON value kinds the CRM endpoints receive.
fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|value| value != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JS `value && Number(value)`, i.e. the numeric value only when the operand is
/// truthy and coercible. Arrays and objects intentionally resolve to `None`.
fn truthy_number(value: Option<&Value>) -> Option<f64> {
    let value = value?;
    if !js_truthy(value) {
        return None;
    }
    match value {
        Value::Number(number) => number.as_f64(),
        Value::Bool(flag) => Some(f64::from(*flag)),
        Value::String(text) => js_parse_f64(text),
        _ => None,
    }
}

/// Port of the lead-score block in `createLead`.
pub fn lead_score_for(body: &Value) -> i64 {
    let mut score = 0;
    if truthy_number(body.get("annualIncome")).is_some_and(|value| value >= ANNUAL_INCOME_SIGNAL) {
        score += WEALTH_SCORE;
    }
    if truthy_number(body.get("netWorth")).is_some_and(|value| value >= NET_WORTH_SIGNAL) {
        score += WEALTH_SCORE;
    }
    match body.get("interestedMembershipTier") {
        Some(Value::String(tier)) if tier == "Black Card" => score += BLACK_CARD_SCORE,
        Some(Value::String(tier)) if tier == "Diamond" => score += DIAMOND_SCORE,
        Some(Value::String(tier)) if tier == "Platinum" => score += PLATINUM_SCORE,
        _ => {}
    }
    score
}

/// JS `JSON.stringify(interests || [])` from `createLead`.
pub fn interests_json(body: &Value) -> String {
    match body.get("interests") {
        Some(value) if js_truthy(value) => value.to_string(),
        _ => "[]".to_owned(),
    }
}

fn vocabulary_is_allowed(value: &Value, allowed: &[&str]) -> bool {
    match value {
        // D1 CHECK constraints pass on NULL, and the nullable columns accept
        // it, so the database decides for these values.
        Value::Null => true,
        other => other.as_str().is_some_and(|text| allowed.contains(&text)),
    }
}

/// D1 CHECK bounds for the percent columns (`lead_score`, `probability`,
/// `default_probability`). DuckDB only checks `probability`, and SQLite passes a
/// CHECK on NULL, so only a present non-numeric or out-of-range value is
/// rejected here.
fn percent_is_allowed(value: &Value) -> bool {
    let in_range = |number: f64| (0.0..=100.0).contains(&number);
    match value {
        Value::Null => true,
        Value::Number(number) => number.as_f64().is_some_and(in_range),
        Value::String(text) => js_parse_f64(text).is_some_and(in_range),
        Value::Bool(flag) => in_range(f64::from(*flag)),
        Value::Array(_) | Value::Object(_) => false,
    }
}

/// Does the D1 CHECK constraint accept this value for this column? Express has
/// no such rule for lead status or the tier columns and lets the database
/// answer; the worker answers up front with the same 500 envelope the failed
/// write would have produced.
pub fn update_value_is_allowed(column: &str, value: &Value) -> bool {
    match column {
        "status" => vocabulary_is_allowed(value, &LEAD_STATUSES),
        "stage" => vocabulary_is_allowed(value, &OPPORTUNITY_STAGES),
        "activity_type" => vocabulary_is_allowed(value, &ACTIVITY_TYPES),
        "interested_membership_tier" | "membership_tier" => {
            vocabulary_is_allowed(value, &MEMBERSHIP_TIERS)
        }
        "lead_score" | "probability" | "default_probability" => percent_is_allowed(value),
        _ => true,
    }
}

/// The body fields `createLead` writes that D1 constrains. A value outside the
/// vocabulary is refused with the same 500 envelope Express uses when its own
/// bind fails — DuckDB has no such CHECK, so Express would store it.
pub fn lead_insert_is_within_constraints(body: &Value) -> bool {
    update_value_is_allowed(
        "interested_membership_tier",
        body.get("interestedMembershipTier").unwrap_or(&Value::Null),
    ) && update_value_is_allowed("lead_score", body.get("lead_score").unwrap_or(&Value::Null))
}

/// The constrained fields behind `createOpportunity`.
pub fn opportunity_insert_is_within_constraints(body: &Value) -> bool {
    update_value_is_allowed("stage", body.get("stage").unwrap_or(&Value::Null))
        && update_value_is_allowed(
            "probability",
            body.get("probability").unwrap_or(&Value::Null),
        )
        && update_value_is_allowed(
            "membership_tier",
            body.get("membershipTier").unwrap_or(&Value::Null),
        )
}

/// The constrained field behind `createActivity`.
pub fn activity_insert_is_within_constraints(body: &Value) -> bool {
    update_value_is_allowed(
        "activity_type",
        body.get("activityType").unwrap_or(&Value::Null),
    )
}

pub fn lead_update_column(key: &str) -> Option<&'static str> {
    LEAD_UPDATE_FIELDS
        .iter()
        .find(|(field, _)| *field == key)
        .map(|(_, column)| *column)
}

pub fn opportunity_update_column(key: &str) -> Option<&'static str> {
    OPPORTUNITY_UPDATE_FIELDS
        .iter()
        .find(|(field, _)| *field == key)
        .map(|(_, column)| *column)
}

/// The JSON-text columns Express writes with `JSON.stringify`.
pub fn is_json_column(column: &str) -> bool {
    column == "interests"
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1460 - day_of_era / 36_524 + day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_probe = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_probe + 2) / 5 + 1) as u32;
    let month = if month_probe < 10 {
        month_probe + 3
    } else {
        month_probe - 9
    } as u32;

    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// SQLite has no `DATE_TRUNC`, so the reporting window Express builds with
/// `DATE_TRUNC('month'|'quarter'|'year', CURRENT_DATE)` is computed here in UTC
/// and bound as an ISO-8601 lower bound. Unsupported period strings return
/// `None`, matching Express dropping the filter altogether.
pub fn period_start(period: &str, now_ms: f64) -> Option<String> {
    if !now_ms.is_finite() {
        return None;
    }
    let (year, month, _) = civil_from_days((now_ms / MS_PER_DAY).floor() as i64);
    let window_month = match period {
        "monthly" => month,
        "quarterly" => ((month - 1) / 3) * 3 + 1,
        "yearly" => 1,
        _ => return None,
    };

    Some(format!("{year:04}-{window_month:02}-01T00:00:00.000Z"))
}

/// `CASE WHEN total > 0 THEN won * 100.0 / total ELSE 0 END`.
pub fn conversion_rate(won: i64, total: i64) -> f64 {
    if total > 0 {
        (won as f64 * 100.0) / total as f64
    } else {
        0.0
    }
}

/// JS `Number(value) || 0` for the aggregate columns.
fn metric_number(value: Option<f64>) -> f64 {
    value.filter(|value| value.is_finite()).unwrap_or(0.0)
}

/// The metrics payload: eleven camelCase numbers in the Express key order,
/// including the hard-coded `salesCycleLength` and the three revenue buckets
/// that all echo the same won revenue.
pub fn sales_metrics_json(leads: &LeadMetricsRow, opportunities: &OpportunityMetricsRow) -> Value {
    let won_revenue = metric_number(opportunities.won_revenue);

    row_json(&[
        ("totalLeads", json!(leads.total_leads)),
        ("qualifiedLeads", json!(leads.qualified_leads)),
        (
            "totalOpportunities",
            json!(opportunities.total_opportunities),
        ),
        (
            "totalPipelineValue",
            number_json(metric_number(opportunities.total_pipeline_value)),
        ),
        (
            "conversionRate",
            number_json(conversion_rate(leads.converted_leads, leads.total_leads)),
        ),
        (
            "averageDealSize",
            number_json(metric_number(opportunities.average_deal_size)),
        ),
        ("salesCycleLength", json!(30)),
        (
            "winRate",
            number_json(conversion_rate(
                opportunities.won_opportunities,
                opportunities.total_opportunities,
            )),
        ),
        ("monthlyRevenue", number_json(won_revenue)),
        ("quarterlyRevenue", number_json(won_revenue)),
        ("yearlyRevenue", number_json(won_revenue)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const MID_2026_SUMMER: f64 = 1_788_168_000_000.0; // 2026-08-31T09:20:00Z

    fn body(fields: Map<String, Value>) -> Value {
        Value::Object(fields)
    }

    #[test]
    fn lead_score_adds_the_wealth_and_tier_signals() {
        let scored = lead_score_for(&body(Map::from_iter([
            ("annualIncome".to_owned(), json!(25_000_000)),
            ("netWorth".to_owned(), json!(120_000_000)),
            ("interestedMembershipTier".to_owned(), json!("Black Card")),
        ])));
        assert_eq!(scored, 100);

        let diamond = lead_score_for(&body(Map::from_iter([
            ("annualIncome".to_owned(), json!(6_000_000)),
            ("netWorth".to_owned(), json!(12_000_000)),
            ("interestedMembershipTier".to_owned(), json!("Diamond")),
        ])));
        assert_eq!(diamond, 15);

        let platinum = lead_score_for(&body(Map::from_iter([
            ("annualIncome".to_owned(), json!(20_000_000)),
            ("netWorth".to_owned(), json!(100_000_000)),
            ("interestedMembershipTier".to_owned(), json!("Platinum")),
        ])));
        assert_eq!(platinum, 90);
    }

    #[test]
    fn lead_score_respects_javascript_falsy_and_numeric_strings() {
        assert_eq!(lead_score_for(&body(Map::new())), 0);

        let zeros = lead_score_for(&body(Map::from_iter([
            ("annualIncome".to_owned(), json!(0)),
            ("netWorth".to_owned(), json!(0)),
        ])));
        assert_eq!(zeros, 0);

        let strings = lead_score_for(&body(Map::from_iter([
            ("annualIncome".to_owned(), json!("25000000")),
            ("netWorth".to_owned(), json!(" 120000000 ")),
        ])));
        assert_eq!(strings, 80);

        let junk = lead_score_for(&body(Map::from_iter([
            ("annualIncome".to_owned(), json!("twenty million")),
            ("netWorth".to_owned(), Value::Null),
            ("interestedMembershipTier".to_owned(), json!("Black Card!")),
        ])));
        assert_eq!(junk, 0);
    }

    #[test]
    fn interests_mirror_json_stringify_with_an_empty_default() {
        assert_eq!(
            interests_json(&body(Map::from_iter([(
                "interests".to_owned(),
                json!(["fine dining", "yachting"])
            )]))),
            r#"["fine dining","yachting"]"#
        );
        assert_eq!(interests_json(&body(Map::new())), "[]");
        assert_eq!(
            interests_json(&body(Map::from_iter([("interests".to_owned(), json!([]))]))),
            "[]"
        );
    }

    #[test]
    fn the_check_vocabulary_defers_null_to_the_column_constraints() {
        assert!(update_value_is_allowed("status", &json!("closed_won")));
        assert!(!update_value_is_allowed("status", &json!("hot")));
        assert!(!update_value_is_allowed("status", &json!(7)));
        assert!(update_value_is_allowed("status", &Value::Null));
        assert!(update_value_is_allowed("stage", &json!("needs_analysis")));
        assert!(!update_value_is_allowed("stage", &json!("prospecting")));
        assert!(update_value_is_allowed(
            "membership_tier",
            &json!("Black Card")
        ));
        assert!(!update_value_is_allowed("membership_tier", &json!("Gold")));
        assert!(update_value_is_allowed(
            "activity_type",
            &json!("follow_up")
        ));
        // Columns without a D1 CHECK pass straight through to the statement.
        assert!(update_value_is_allowed("name", &json!("anything at all")));
    }

    #[test]
    fn percent_columns_are_bounded_inclusive() {
        assert!(update_value_is_allowed("lead_score", &json!(0)));
        assert!(update_value_is_allowed("lead_score", &json!(100)));
        assert!(!update_value_is_allowed("lead_score", &json!(900)));
        assert!(!update_value_is_allowed("probability", &json!(-5)));
        assert!(update_value_is_allowed("probability", &Value::Null));
        assert!(update_value_is_allowed("probability", &json!("40")));
    }

    #[test]
    fn create_bodies_are_checked_against_the_same_d1_vocabulary() {
        let diamond = Map::from_iter([("interestedMembershipTier".to_owned(), json!("Diamond"))]);
        assert!(lead_insert_is_within_constraints(&body(diamond.clone())));
        assert!(lead_insert_is_within_constraints(&body(Map::new())));

        let gold = diamond
            .into_iter()
            .map(|(key, _)| (key, json!("Gold")))
            .collect();
        assert!(!lead_insert_is_within_constraints(&body(gold)));

        let deal = Map::from_iter([
            ("stage".to_owned(), json!("needs_analysis")),
            ("probability".to_owned(), json!(40)),
            ("membershipTier".to_owned(), json!("Platinum")),
        ]);
        assert!(opportunity_insert_is_within_constraints(&body(deal)));
        assert!(!opportunity_insert_is_within_constraints(&json!({
            "stage": "prospecting",
            "probability": 40,
            "membershipTier": "Platinum",
        })));
        assert!(!opportunity_insert_is_within_constraints(&json!({
            "stage": "proposal",
            "probability": 101,
        })));
        assert!(opportunity_insert_is_within_constraints(&json!({
            "name": "no constrained values at all",
        })));

        assert!(activity_insert_is_within_constraints(&json!({
            "activityType": "follow_up",
        })));
        assert!(activity_insert_is_within_constraints(&json!({})));
        assert!(!activity_insert_is_within_constraints(&json!({
            "activityType": "sms",
        })));
    }

    #[test]
    fn update_columns_accept_both_spellings_and_reject_unknowns() {
        assert_eq!(lead_update_column("lead_score"), Some("lead_score"));
        assert_eq!(lead_update_column("leadScore"), Some("lead_score"));
        assert_eq!(
            lead_update_column("interestedMembershipTier"),
            Some("interested_membership_tier")
        );
        assert_eq!(lead_update_column("nope = 1; --"), None);
        assert_eq!(
            opportunity_update_column("expectedCloseDate"),
            Some("expected_close_date")
        );
        assert_eq!(opportunity_update_column("notes"), None);
        assert_eq!(opportunity_update_column("id"), None);
        assert_eq!(lead_update_column("id"), None);
    }

    #[test]
    fn reporting_periods_floor_to_utc_bucket_starts() {
        assert_eq!(
            period_start("monthly", MID_2026_SUMMER).as_deref(),
            Some("2026-08-01T00:00:00.000Z")
        );
        assert_eq!(
            period_start("quarterly", MID_2026_SUMMER).as_deref(),
            Some("2026-07-01T00:00:00.000Z")
        );
        assert_eq!(
            period_start("yearly", MID_2026_SUMMER).as_deref(),
            Some("2026-01-01T00:00:00.000Z")
        );
        assert_eq!(period_start("weekly", MID_2026_SUMMER), None);
        assert_eq!(period_start("", MID_2026_SUMMER), None);
    }

    #[test]
    fn reporting_periods_handle_leap_days_and_pre_epoch_milliseconds() {
        assert_eq!(
            period_start("monthly", 1_709_164_800_000.0).as_deref(),
            Some("2024-02-01T00:00:00.000Z")
        );
        assert_eq!(
            period_start("quarterly", 1_709_164_800_000.0).as_deref(),
            Some("2024-01-01T00:00:00.000Z")
        );
        assert_eq!(
            period_start("monthly", -3_600_000.0).as_deref(),
            Some("1969-12-01T00:00:00.000Z")
        );
        assert_eq!(
            period_start("yearly", 0.0).as_deref(),
            Some("1970-01-01T00:00:00.000Z")
        );
        assert_eq!(period_start("monthly", f64::NAN), None);
    }

    #[test]
    fn funnel_rates_guard_against_empty_denominators() {
        assert_eq!(conversion_rate(1, 3), 100.0 / 3.0);
        assert_eq!(conversion_rate(0, 0), 0.0);
        assert_eq!(conversion_rate(3, 3), 100.0);
    }

    #[test]
    fn metrics_payload_keeps_the_express_keys_and_coercions() {
        let leads = LeadMetricsRow {
            total_leads: 3,
            qualified_leads: 1,
            converted_leads: 1,
        };
        let opportunities = OpportunityMetricsRow {
            total_opportunities: 2,
            total_pipeline_value: Some(850_000.0),
            average_deal_size: Some(425_000.0),
            won_opportunities: 1,
            won_revenue: Some(250_000.0),
        };

        assert_eq!(
            serde_json::to_string(&sales_metrics_json(&leads, &opportunities))
                .expect("metrics JSON"),
            r#"{"totalLeads":3,"qualifiedLeads":1,"totalOpportunities":2,"totalPipelineValue":850000,"conversionRate":33.333333333333336,"averageDealSize":425000,"salesCycleLength":30,"winRate":50,"monthlyRevenue":250000,"quarterlyRevenue":250000,"yearlyRevenue":250000}"#
        );
    }

    #[test]
    fn empty_aggregates_coerce_to_zero_like_number_x_or_zero() {
        let metrics = sales_metrics_json(
            &LeadMetricsRow::default(),
            &OpportunityMetricsRow::default(),
        );

        assert_eq!(metrics["totalPipelineValue"], json!(0));
        assert_eq!(metrics["averageDealSize"], json!(0));
        assert_eq!(metrics["conversionRate"], json!(0));
        assert_eq!(metrics["winRate"], json!(0));
        assert_eq!(metrics["monthlyRevenue"], json!(0));
        assert_eq!(metrics["yearlyRevenue"], json!(0));
    }

    #[test]
    fn aggregate_defaults_back_the_zero_row_queries() {
        let leads: LeadMetricsRow = serde_json::from_value(
            json!({"total_leads": 0, "qualified_leads": 0, "converted_leads": 0}),
        )
        .expect("lead aggregate row");
        let opportunities: OpportunityMetricsRow = serde_json::from_value(json!({
            "total_opportunities": 0,
            "total_pipeline_value": null,
            "average_deal_size": null,
            "won_opportunities": 0,
            "won_revenue": null,
        }))
        .expect("opportunity aggregate row");

        assert_eq!(leads.total_leads, 0);
        assert_eq!(opportunities.total_pipeline_value, None);
        assert_eq!(
            sales_metrics_json(&leads, &opportunities)["totalPipelineValue"],
            json!(0)
        );
    }

    fn lead_row() -> LeadRow {
        LeadRow {
            id: 9001,
            first_name: "Seeded".to_owned(),
            last_name: "Contract".to_owned(),
            email: "crm-active@hesocial.test".to_owned(),
            phone: Some("+886900000001".to_owned()),
            company: Some("Contract Holdings".to_owned()),
            job_title: Some("Principal".to_owned()),
            annual_income: Some(25_000_000),
            net_worth: Some(120_000_000),
            source: "referral".to_owned(),
            referral_code: Some("CRM2F".to_owned()),
            lead_score: Some(100),
            status: Some("new".to_owned()),
            interested_membership_tier: Some("Black Card".to_owned()),
            budget_range: None,
            timeline: None,
            pain_points: None,
            interests: Some(r#"["fine dining","yachting"]"#.to_owned()),
            notes: None,
            last_contact_date: None,
            next_follow_up_date: Some("2026-09-07".to_owned()),
            assigned_to: Some("f47ac10b".to_owned()),
            created_at: Some("2026-08-31T00:00:00.000Z".to_owned()),
            updated_at: Some("2026-08-31T00:00:00.000Z".to_owned()),
            assigned_to_first_name: Some("Admin".to_owned()),
            assigned_to_last_name: Some("User".to_owned()),
        }
    }

    #[test]
    fn lead_rows_are_rendered_with_the_raw_column_names_the_service_maps() {
        let json = lead_json(&lead_row());

        assert_eq!(json["id"], json!(9001));
        assert_eq!(json["first_name"], json!("Seeded"));
        assert_eq!(json["job_title"], json!("Principal"));
        assert_eq!(json["lead_score"], json!(100));
        assert_eq!(json["interests"], json!(r#"["fine dining","yachting"]"#));
        assert_eq!(json["assigned_to_first_name"], json!("Admin"));
        assert_eq!(json["budget_range"], Value::Null);
        assert_eq!(json.get("firstName"), None);
        assert_eq!(
            json.get("interests").and_then(Value::as_str),
            Some(r#"["fine dining","yachting"]"#)
        );
    }

    #[test]
    fn opportunity_rows_keep_decimal_values_integral_and_join_the_lead() {
        let row = OpportunityRow {
            id: 9102,
            lead_id: 9001,
            name: "Diamond Membership Renewal".to_owned(),
            description: None,
            stage: "proposal".to_owned(),
            probability: Some(60),
            value: 480_000.0,
            expected_close_date: Some("2026-12-01".to_owned()),
            actual_close_date: None,
            membership_tier: "Diamond".to_owned(),
            payment_terms: Some("semi-annual".to_owned()),
            close_reason: None,
            assigned_to: "f47ac10b".to_owned(),
            created_at: None,
            updated_at: None,
            lead_first_name: Some("Seeded".to_owned()),
            lead_last_name: None,
            lead_email: Some("crm-active@hesocial.test".to_owned()),
            assigned_to_first_name: Some("Admin".to_owned()),
            assigned_to_last_name: Some("User".to_owned()),
        };

        let json = opportunity_json(&row);
        assert_eq!(
            serde_json::to_string(&json["value"]).expect("value"),
            "480000"
        );
        assert_eq!(json["lead_first_name"], json!("Seeded"));
        assert_eq!(json["lead_last_name"], Value::Null);
        assert_eq!(json["stage"], json!("proposal"));
    }

    #[test]
    fn boolean_columns_are_returned_as_javascript_booleans() {
        let stage: PipelineStageRow = serde_json::from_value(json!({
            "id": 9401,
            "name": "qualification",
            "description": null,
            "display_order": 1,
            "default_probability": 25,
            "is_active": 0,
            "color_code": "#94A3B8",
            "created_at": null,
            "updated_at": null,
        }))
        .expect("pipeline stage row");
        assert_eq!(pipeline_stage_json(&stage)["is_active"], json!(false));

        let team: TeamMemberRow = serde_json::from_value(json!({
            "id": 9301,
            "user_id": "f47ac10b",
            "role": "sales_rep",
            "territory": null,
            "commission_rate": 8.5,
            "quota_amount": 3_000_000,
            "is_active": 1,
            "hire_date": null,
            "manager_id": null,
            "created_at": null,
            "updated_at": null,
            "first_name": "Admin",
            "last_name": "User",
            "email": "admin@hesocial.com",
            "manager_first_name": "Test",
            "manager_last_name": "Platinum",
        }))
        .expect("team member row");
        let json = team_member_json(&team);
        assert_eq!(json["is_active"], json!(true));
        assert_eq!(json["commission_rate"], json!(8.5));
        assert_eq!(json["quota_amount"], json!(3_000_000));
        assert_eq!(json["manager_first_name"], json!("Test"));

        let activity: ActivityRow = serde_json::from_value(json!({
            "id": 9201,
            "lead_id": 9002,
            "opportunity_id": null,
            "activity_type": "meeting",
            "subject": "Founding seat presentation",
            "description": null,
            "outcome": null,
            "duration_minutes": 60,
            "scheduled_at": null,
            "completed_at": null,
            "created_by": "f47ac10b",
            "created_at": null,
            "updated_at": null,
            "created_by_first_name": "Admin",
            "created_by_last_name": "User",
        }))
        .expect("activity row");
        let json = activity_json(&activity);
        assert_eq!(json["opportunity_id"], Value::Null);
        assert_eq!(json["duration_minutes"], json!(60));
        assert_eq!(json["created_by_first_name"], json!("Admin"));
    }
}
