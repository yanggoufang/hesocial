#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::permissions::{
    AuthSnapshot, AuthUser, Role, RouteGuard, Session, USER_ROUTE_FALLBACK,
};
use hesocial_frontend::sales::{
    ACCESS_TOKEN_REQUIRED, ACTIVITIES_API_PATH, ACTIVITIES_FETCH_FALLBACK, FUNNEL_VIEW_HEIGHT,
    FUNNEL_VIEW_WIDTH, LEADS_API_PATH, LEADS_FETCH_FALLBACK, LeadFilters, METRICS_API_PATH,
    METRICS_FETCH_FALLBACK, OPPORTUNITIES_API_PATH, OPPORTUNITIES_FETCH_FALLBACK,
    OpportunityFilters, PAGE_SIZE, PIPELINE_FETCH_FALLBACK, PIPELINE_STAGES_API_PATH,
    PipelineStage, SalesLead, SalesOpportunity, TEAM_API_PATH, TEAM_FETCH_FALLBACK,
    admin_route_guard, conversion_percent, format_currency, format_one_decimal, format_sales_date,
    funnel_bands, funnel_counts, funnel_polygon_points, lead_display_name, lead_status_class,
    lead_status_label, leads_query_string, membership_tier_badge_class, opportunities_query_string,
    opportunity_stage_label, parse_activities_response, parse_leads_response,
    parse_metrics_response, parse_opportunities_response, parse_pipeline_stages_response,
    parse_team_response, pipeline_stage_stats, score_bar_percent,
};

fn admin_snapshot() -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::Admin),
        ..AuthSnapshot::default()
    }
}

fn user_snapshot() -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        ..AuthSnapshot::default()
    }
}

fn stage(name: &str, order: i64) -> PipelineStage {
    PipelineStage {
        id: order.to_string(),
        name: name.to_string(),
        display_order: order,
        is_active: true,
        ..PipelineStage::default()
    }
}

fn opportunity(stage_name: &str, value: f64) -> SalesOpportunity {
    SalesOpportunity {
        id: format!("{stage_name}-{value}"),
        name: stage_name.to_string(),
        stage: stage_name.to_string(),
        value,
        ..SalesOpportunity::default()
    }
}

fn leads_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": 9001,
                "first_name": "Wei",
                "last_name": "Chen",
                "email": "wei@example.com",
                "company": "Hexagram",
                "job_title": "Managing Partner",
                "lead_score": 82,
                "annual_income": 5000000,
                "net_worth": 30000000,
                "source": "referral",
                "status": "qualified",
                "assigned_to": "admin-1",
                "interests": "[\"yacht\",\"art\"]",
                "next_follow_up_date": "2026-09-15T00:00:00.000Z",
                "created_at": "2026-08-01T00:00:00.000Z",
                "updated_at": "2026-08-02T00:00:00.000Z",
                "assigned_to_first_name": "Admin",
                "assigned_to_last_name": "User"
            }
        ],
        "pagination": { "page": 1, "limit": 20, "total": 1, "totalPages": 1 }
    }"#
    .to_string()
}

fn camel_lead_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": "7",
                "firstName": "Ada",
                "lastName": "Lovelace",
                "email": "ada@example.com",
                "position": "Analyst",
                "leadScore": 40,
                "annualIncome": 8000000,
                "netWorth": 12000000,
                "source": "event",
                "status": "new",
                "assignedTo": "u-9",
                "interests": ["math"],
                "nextFollowUpDate": "2026-10-01T00:00:00.000Z"
            }
        ],
        "pagination": { "page": 2, "limit": 20, "total": 21, "totalPages": 2 }
    }"#
    .to_string()
}

#[test]
fn page_size_matches_react() {
    assert_eq!(PAGE_SIZE, 20);
}

#[test]
fn api_paths_match_worker_routes() {
    assert_eq!(LEADS_API_PATH, "/api/sales/leads");
    assert_eq!(OPPORTUNITIES_API_PATH, "/api/sales/opportunities");
    assert_eq!(METRICS_API_PATH, "/api/sales/metrics");
    assert_eq!(PIPELINE_STAGES_API_PATH, "/api/sales/pipeline/stages");
    assert_eq!(ACTIVITIES_API_PATH, "/api/sales/activities");
    assert_eq!(TEAM_API_PATH, "/api/sales/team");
}

#[test]
fn leads_query_omits_empty_filters() {
    let filters = LeadFilters {
        page: 1,
        limit: PAGE_SIZE,
        ..LeadFilters::default()
    };
    assert_eq!(leads_query_string(&filters), "page=1&limit=20");
}

#[test]
fn leads_query_encodes_active_filters() {
    let filters = LeadFilters {
        search: "Wei Chen".to_string(),
        status: "new".to_string(),
        source: "event".to_string(),
        assigned_to: "admin 1".to_string(),
        page: 3,
        limit: 20,
    };
    let query = leads_query_string(&filters);
    assert!(query.contains("page=3"));
    assert!(query.contains("limit=20"));
    assert!(query.contains("search=Wei+Chen"));
    assert!(query.contains("status=new"));
    assert!(query.contains("source=event"));
    assert!(query.contains("assignedTo=admin+1"));
}

#[test]
fn opportunities_query_encodes_stage() {
    let filters = OpportunityFilters {
        search: "Black Card".to_string(),
        stage: "proposal".to_string(),
        assigned_to: String::new(),
        page: 1,
        limit: 20,
    };
    let query = opportunities_query_string(&filters);
    assert_eq!(query, "page=1&limit=20&search=Black+Card&stage=proposal");
}

#[test]
fn parse_leads_snake_case_worker_envelope() {
    let view = parse_leads_response(200, &leads_body()).expect("leads");
    assert_eq!(view.leads.len(), 1);
    let lead = &view.leads[0];
    assert_eq!(lead.id, "9001");
    assert_eq!(lead.first_name, "Wei");
    assert_eq!(lead.last_name, "Chen");
    assert_eq!(lead.email, "wei@example.com");
    assert_eq!(lead.company.as_deref(), Some("Hexagram"));
    assert_eq!(lead.position.as_deref(), Some("Managing Partner"));
    assert_eq!(lead.lead_score, 82.0);
    assert_eq!(lead.annual_income, 5_000_000.0);
    assert_eq!(lead.net_worth, 30_000_000.0);
    assert_eq!(lead.status, "qualified");
    assert_eq!(lead.interests, vec!["yacht", "art"]);
    assert_eq!(view.pagination.total, 1);
    assert_eq!(view.pagination.total_pages, 1);
    assert_eq!(lead_display_name(lead), "Wei Chen");
}

#[test]
fn parse_leads_camel_case_react_mapper_shape() {
    let view = parse_leads_response(200, &camel_lead_body()).expect("leads");
    let lead = &view.leads[0];
    assert_eq!(lead.id, "7");
    assert_eq!(lead.first_name, "Ada");
    assert_eq!(lead.position.as_deref(), Some("Analyst"));
    assert_eq!(lead.annual_income, 8_000_000.0);
    assert_eq!(view.pagination.page, 2);
    assert_eq!(view.pagination.total_pages, 2);
}

#[test]
fn parse_leads_empty_array() {
    let body =
        r#"{"success":true,"data":[],"pagination":{"page":1,"limit":20,"total":0,"totalPages":0}}"#;
    let view = parse_leads_response(200, body).expect("empty");
    assert!(view.leads.is_empty());
    assert_eq!(view.pagination.total, 0);
    assert_eq!(view.pagination.total_pages, 0);
}

#[test]
fn parse_leads_401_uses_backend_error() {
    let err = parse_leads_response(401, r#"{"success":false,"error":"Access token required"}"#)
        .expect_err("401");
    assert_eq!(err, ACCESS_TOKEN_REQUIRED);
}

#[test]
fn parse_leads_401_without_body_uses_token_fallback() {
    let err = parse_leads_response(401, "nope").expect_err("401");
    assert_eq!(err, ACCESS_TOKEN_REQUIRED);
}

#[test]
fn parse_leads_invalid_json() {
    let err = parse_leads_response(200, "{").expect_err("bad json");
    assert_eq!(err, LEADS_FETCH_FALLBACK);
}

#[test]
fn parse_leads_success_false_uses_error_field() {
    let err = parse_leads_response(
        200,
        r#"{"success":false,"error":"Failed to fetch sales leads"}"#,
    )
    .expect_err("fail");
    assert_eq!(err, "Failed to fetch sales leads");
}

#[test]
fn parse_opportunities_snake_case_with_joined_lead() {
    let body = r#"{
        "success": true,
        "data": [
            {
                "id": 9101,
                "lead_id": 9001,
                "name": "Black Card upgrade",
                "stage": "proposal",
                "probability": 60,
                "value": 3000000,
                "membership_tier": "Black Card",
                "expected_close_date": "2026-12-01T00:00:00.000Z",
                "lead_first_name": "Wei",
                "lead_last_name": "Chen",
                "lead_email": "wei@example.com"
            }
        ],
        "pagination": { "page": 1, "limit": 20, "total": 1, "totalPages": 1 }
    }"#;
    let view = parse_opportunities_response(200, body).expect("opps");
    assert_eq!(view.opportunities.len(), 1);
    let opp = &view.opportunities[0];
    assert_eq!(opp.id, "9101");
    assert_eq!(opp.lead_id, "9001");
    assert_eq!(opp.value, 3_000_000.0);
    assert_eq!(opp.membership_tier, "Black Card");
    assert_eq!(opp.lead.first_name, "Wei");
    assert_eq!(opp.lead.email, "wei@example.com");
}

#[test]
fn parse_opportunities_nested_lead_object() {
    let body = r#"{
        "success": true,
        "data": [
            {
                "id": "9",
                "leadId": "1",
                "name": "Diamond",
                "stage": "negotiation",
                "probability": 80,
                "value": 1500000,
                "membershipTier": "Diamond",
                "expectedCloseDate": "2026-11-01",
                "lead": { "firstName": "Ada", "lastName": "Lovelace", "email": "ada@example.com" }
            }
        ]
    }"#;
    let view = parse_opportunities_response(200, body).expect("opps");
    assert_eq!(view.opportunities[0].lead.first_name, "Ada");
    assert_eq!(view.opportunities[0].lead.last_name, "Lovelace");
}

#[test]
fn parse_opportunities_empty_and_401() {
    let empty = parse_opportunities_response(200, r#"{"success":true,"data":[]}"#).expect("empty");
    assert!(empty.opportunities.is_empty());
    let err =
        parse_opportunities_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .expect_err("401");
    assert_eq!(err, ACCESS_TOKEN_REQUIRED);
    let fallback = parse_opportunities_response(500, r#"{"success":false}"#).expect_err("500");
    assert_eq!(fallback, OPPORTUNITIES_FETCH_FALLBACK);
}

#[test]
fn parse_metrics_camel_case_worker_shape() {
    let body = r#"{
        "success": true,
        "data": {
            "totalLeads": 4,
            "qualifiedLeads": 2,
            "totalOpportunities": 3,
            "totalPipelineValue": 730000,
            "conversionRate": 25,
            "averageDealSize": 243333.333,
            "salesCycleLength": 30,
            "winRate": 33.333333333,
            "monthlyRevenue": 500000,
            "quarterlyRevenue": 500000,
            "yearlyRevenue": 500000
        }
    }"#;
    let metrics = parse_metrics_response(200, body).expect("metrics");
    assert_eq!(metrics.total_leads, 4.0);
    assert_eq!(metrics.qualified_leads, 2.0);
    assert_eq!(metrics.total_pipeline_value, 730_000.0);
    assert_eq!(metrics.sales_cycle_length, 30.0);
    assert_eq!(metrics.monthly_revenue, 500_000.0);
    assert_eq!(metrics.yearly_revenue, metrics.quarterly_revenue);
}

#[test]
fn parse_metrics_missing_data_and_401() {
    let missing = parse_metrics_response(200, r#"{"success":true}"#).expect_err("no data");
    assert_eq!(missing, METRICS_FETCH_FALLBACK);
    let null_data =
        parse_metrics_response(200, r#"{"success":true,"data":null}"#).expect_err("null");
    assert_eq!(null_data, METRICS_FETCH_FALLBACK);
    let err = parse_metrics_response(401, r#"{"success":false,"error":"Access token required"}"#)
        .expect_err("401");
    assert_eq!(err, ACCESS_TOKEN_REQUIRED);
}

#[test]
fn parse_pipeline_stages_worker_shape() {
    let body = r##"{
        "success": true,
        "data": [
            {
                "id": 9401,
                "name": "qualification",
                "display_order": 1,
                "default_probability": 25,
                "is_active": true,
                "color_code": "#3b82f6"
            },
            {
                "id": 9402,
                "name": "needs_analysis",
                "display_order": 2,
                "default_probability": 40,
                "is_active": true
            }
        ]
    }"##;
    let stages = parse_pipeline_stages_response(200, body).expect("stages");
    assert_eq!(stages.len(), 2);
    assert_eq!(stages[0].id, "9401");
    assert_eq!(stages[0].name, "qualification");
    assert_eq!(stages[0].display_order, 1);
    assert_eq!(stages[0].default_probability, 25.0);
    assert_eq!(stages[0].color_code.as_deref(), Some("#3b82f6"));
    assert!(stages[0].is_active);
}

#[test]
fn parse_pipeline_empty_and_401() {
    let empty =
        parse_pipeline_stages_response(200, r#"{"success":true,"data":[]}"#).expect("empty");
    assert!(empty.is_empty());
    let err = parse_pipeline_stages_response(401, "").expect_err("401");
    assert_eq!(err, ACCESS_TOKEN_REQUIRED);
    let fail = parse_pipeline_stages_response(500, r#"{"success":false}"#).expect_err("500");
    assert_eq!(fail, PIPELINE_FETCH_FALLBACK);
}

#[test]
fn parse_activities_and_team() {
    let activities = parse_activities_response(
        200,
        r#"{
            "success": true,
            "data": [
                {
                    "id": 1,
                    "lead_id": 9002,
                    "activity_type": "call",
                    "subject": "Intro",
                    "created_by_first_name": "Admin",
                    "created_by_last_name": "User",
                    "created_at": "2026-08-01T00:00:00.000Z"
                }
            ]
        }"#,
    )
    .expect("activities");
    assert_eq!(activities.len(), 1);
    assert_eq!(activities[0].activity_type, "call");
    assert_eq!(activities[0].lead_id.as_deref(), Some("9002"));

    let empty_activities =
        parse_activities_response(200, r#"{"success":true,"data":[]}"#).expect("empty");
    assert!(empty_activities.is_empty());
    let act_401 =
        parse_activities_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .expect_err("401");
    assert_eq!(act_401, ACCESS_TOKEN_REQUIRED);
    let act_fail = parse_activities_response(500, r#"{"success":false}"#).expect_err("500");
    assert_eq!(act_fail, ACTIVITIES_FETCH_FALLBACK);

    let team = parse_team_response(
        200,
        r#"{
            "success": true,
            "data": [
                {
                    "id": 9301,
                    "user_id": "admin-1",
                    "role": "sales_rep",
                    "territory": "Taipei",
                    "quota_amount": 3000000,
                    "is_active": true,
                    "first_name": "Admin",
                    "last_name": "User",
                    "email": "admin@hesocial.com",
                    "manager_first_name": "Test",
                    "manager_last_name": "Platinum"
                }
            ]
        }"#,
    )
    .expect("team");
    assert_eq!(team.len(), 1);
    assert_eq!(team[0].quota_amount, 3_000_000.0);
    assert_eq!(team[0].territory.as_deref(), Some("Taipei"));
    assert_eq!(team[0].email.as_deref(), Some("admin@hesocial.com"));
    let team_401 = parse_team_response(401, "").expect_err("401");
    assert_eq!(team_401, ACCESS_TOKEN_REQUIRED);
    let team_fail = parse_team_response(500, r#"{"success":false}"#).expect_err("500");
    assert_eq!(team_fail, TEAM_FETCH_FALLBACK);
}

#[test]
fn conversion_percent_avoids_divide_by_zero() {
    assert_eq!(conversion_percent(0, 0), 0.0);
    assert_eq!(conversion_percent(5, 0), 0.0);
    assert_eq!(conversion_percent(0, 10), 0.0);
    assert_eq!(conversion_percent(5, 10), 50.0);
    assert_eq!(conversion_percent(3, 4), 75.0);
}

#[test]
fn pipeline_all_zero_does_not_divide_by_zero() {
    let stages = vec![
        stage("qualification", 1),
        stage("needs_analysis", 2),
        stage("proposal", 3),
        stage("negotiation", 4),
    ];
    let stats = pipeline_stage_stats(&stages, &[]);
    assert_eq!(stats.len(), 4);
    for stat in &stats {
        assert_eq!(stat.count, 0);
        assert_eq!(stat.value, 0.0);
        assert_eq!(stat.conversion_from_previous, 0.0);
        assert_eq!(stat.share_of_first, 0.0);
        assert!(stat.conversion_from_previous.is_finite());
        assert!(stat.share_of_first.is_finite());
    }
}

#[test]
fn pipeline_stage_totals_and_conversion() {
    let stages = vec![
        stage("qualification", 1),
        stage("needs_analysis", 2),
        stage("proposal", 3),
        stage("negotiation", 4),
    ];
    let opps = vec![
        opportunity("qualification", 1_000_000.0),
        opportunity("qualification", 2_000_000.0),
        opportunity("qualification", 500_000.0),
        opportunity("qualification", 500_000.0),
        opportunity("needs_analysis", 3_000_000.0),
        opportunity("needs_analysis", 1_000_000.0),
        opportunity("proposal", 4_000_000.0),
        opportunity("negotiation", 5_000_000.0),
    ];
    let stats = pipeline_stage_stats(&stages, &opps);
    assert_eq!(stats[0].count, 4);
    assert_eq!(stats[0].value, 4_000_000.0);
    assert_eq!(stats[0].conversion_from_previous, 100.0);
    assert_eq!(stats[0].share_of_first, 100.0);
    assert_eq!(stats[1].count, 2);
    assert_eq!(stats[1].conversion_from_previous, 50.0);
    assert_eq!(stats[1].share_of_first, 50.0);
    assert_eq!(stats[2].count, 1);
    assert_eq!(stats[2].conversion_from_previous, 50.0);
    assert_eq!(stats[2].share_of_first, 25.0);
    assert_eq!(stats[3].count, 1);
    assert_eq!(stats[3].conversion_from_previous, 100.0);
    assert_eq!(stats[3].share_of_first, 25.0);
}

#[test]
fn funnel_bands_empty_and_all_zero_are_finite() {
    assert!(funnel_bands(&[], FUNNEL_VIEW_WIDTH, FUNNEL_VIEW_HEIGHT).is_empty());
    let bands = funnel_bands(&[0, 0, 0, 0], FUNNEL_VIEW_WIDTH, FUNNEL_VIEW_HEIGHT);
    assert_eq!(bands.len(), 4);
    for band in &bands {
        assert!(band.top_left_x.is_finite());
        assert!(band.top_right_x.is_finite());
        assert!(band.bottom_left_x.is_finite());
        assert!(band.bottom_right_x.is_finite());
        assert!(band.height.is_finite());
        assert!(band.top_right_x > band.top_left_x);
        assert!(band.bottom_right_x > band.bottom_left_x);
        let points = funnel_polygon_points(band);
        assert!(!points.contains("NaN"));
        assert!(!points.contains("inf"));
    }
    assert!(
        bands[0].top_right_x - bands[0].top_left_x > bands[3].top_right_x - bands[3].top_left_x
    );
}

#[test]
fn funnel_bands_scale_with_counts() {
    let bands = funnel_bands(&[8, 4, 2, 1], 400.0, 200.0);
    assert_eq!(bands.len(), 4);
    let top0 = bands[0].top_right_x - bands[0].top_left_x;
    let top3 = bands[3].top_right_x - bands[3].top_left_x;
    assert!((top0 - 400.0).abs() < 0.001);
    assert!(top3 < top0);
    assert_eq!(bands[0].height, 50.0);
    let points = funnel_polygon_points(&bands[0]);
    assert!(points.starts_with("0,0 400,0"));
}

#[test]
fn funnel_counts_extracts_stage_totals() {
    let stages = vec![stage("qualification", 1), stage("proposal", 2)];
    let stats = pipeline_stage_stats(&stages, &[opportunity("qualification", 1.0)]);
    assert_eq!(funnel_counts(&stats), vec![1, 0]);
}

#[test]
fn format_currency_boundaries() {
    assert_eq!(format_currency(0.0), "NT$ 0");
    assert_eq!(format_currency(999.0), "NT$ 999");
    assert_eq!(format_currency(1000.0), "NT$ 1,000");
    assert_eq!(format_currency(1_000_000.0), "NT$ 1,000,000");
    assert_eq!(format_currency(5_000_000.0), "NT$ 5,000,000");
    assert_eq!(format_currency(30_000_000.0), "NT$ 30,000,000");
    assert_eq!(format_currency(-5_000_000.0), "NT$ -5,000,000");
    assert_eq!(format_currency(1_000.4), "NT$ 1,000");
    assert_eq!(format_currency(1_000.6), "NT$ 1,001");
    assert_eq!(format_currency(f64::NAN), "NT$ 0");
    assert_eq!(format_currency(f64::INFINITY), "NT$ 0");
}

#[test]
fn format_one_decimal_and_score_bar() {
    assert_eq!(format_one_decimal(33.333333), "33.3");
    assert_eq!(format_one_decimal(50.0), "50.0");
    assert_eq!(format_one_decimal(f64::NAN), "0.0");
    assert_eq!(score_bar_percent(82.0), 82.0);
    assert_eq!(score_bar_percent(-4.0), 0.0);
    assert_eq!(score_bar_percent(140.0), 100.0);
    assert_eq!(score_bar_percent(f64::NAN), 0.0);
}

#[test]
fn format_sales_date_iso_and_empty() {
    assert_eq!(format_sales_date(""), "-");
    assert_eq!(format_sales_date("   "), "-");
    assert_eq!(format_sales_date("2026-09-15T00:00:00.000Z"), "2026/9/15");
    assert_eq!(format_sales_date("2026-12-01"), "2026/12/1");
}

#[test]
fn labels_and_badge_classes_match_react() {
    assert_eq!(lead_status_label("new"), "新線索");
    assert_eq!(lead_status_label("qualified"), "已審核");
    assert_eq!(lead_status_label("contacted"), "已聯繫");
    assert_eq!(lead_status_label("nurturing"), "培養中");
    assert_eq!(lead_status_label("proposal"), "提案階段");
    assert_eq!(lead_status_label("negotiation"), "談判中");
    assert_eq!(lead_status_label("closed_won"), "成交");
    assert_eq!(lead_status_label("closed_lost"), "失單");
    assert_eq!(lead_status_label("mystery"), "mystery");
    assert_eq!(lead_status_class("new"), "bg-blue-100 text-blue-800");
    assert_eq!(opportunity_stage_label("qualification"), "資格審核");
    assert_eq!(opportunity_stage_label("needs_analysis"), "需求分析");
    assert_eq!(opportunity_stage_label("unknown_stage"), "unknown_stage");
    assert_eq!(
        membership_tier_badge_class("Black Card"),
        "bg-black text-white border-black"
    );
    assert_eq!(
        membership_tier_badge_class("Diamond"),
        "bg-blue-100 text-blue-800 border-blue-300"
    );
}

#[test]
fn admin_route_guard_three_states() {
    assert_eq!(
        admin_route_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading
    );
    assert_eq!(
        admin_route_guard(true, &admin_snapshot()),
        RouteGuard::Loading
    );
    assert_eq!(
        admin_route_guard(false, &AuthSnapshot::default()),
        RouteGuard::Redirect(USER_ROUTE_FALLBACK)
    );
    assert_eq!(
        admin_route_guard(false, &user_snapshot()),
        RouteGuard::Redirect(USER_ROUTE_FALLBACK)
    );
    let signed_out_admin = AuthSnapshot {
        is_authenticated: false,
        role: Some(Role::Admin),
        ..AuthSnapshot::default()
    };
    assert_eq!(
        admin_route_guard(false, &signed_out_admin),
        RouteGuard::Redirect(USER_ROUTE_FALLBACK)
    );
    assert_eq!(
        admin_route_guard(false, &admin_snapshot()),
        RouteGuard::Allow
    );
    let super_admin = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::SuperAdmin),
        ..AuthSnapshot::default()
    };
    assert_eq!(admin_route_guard(false, &super_admin), RouteGuard::Allow);
}

#[test]
fn session_admin_user_is_allowed() {
    let session = Session {
        token: Some("tok".into()),
        user: Some(AuthUser {
            role: Some(Role::Admin),
            ..AuthUser::default()
        }),
        restoring: false,
    };
    assert_eq!(
        admin_route_guard(session.restoring, &session.snapshot()),
        RouteGuard::Allow
    );
}

#[test]
fn unused_lead_struct_fields_stay_available() {
    let lead = SalesLead {
        phone: Some("09".into()),
        notes: Some("n".into()),
        ..SalesLead::default()
    };
    assert_eq!(lead.phone.as_deref(), Some("09"));
}
