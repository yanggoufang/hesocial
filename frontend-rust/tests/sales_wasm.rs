#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::pages::sales::AdminSalesScreen;
use hesocial_frontend::sales::{
    LeadFilters, OpportunityFilters, OpportunityLead, PipelineStage, PipelineStageStat, SalesLead,
    SalesMetrics, SalesOpportunity, SalesTab, pipeline_stage_stats,
};
use wasm_bindgen_test::wasm_bindgen_test;

fn opening_tag<'a>(html: &'a str, id: &str) -> &'a str {
    let needle = format!("id=\"{id}\"");
    let Some(id_at) = html.find(&needle) else {
        return "";
    };
    let start = html[..id_at].rfind('<').unwrap_or(id_at);
    let end = html[id_at..]
        .find('>')
        .map(|rel| id_at + rel + 1)
        .unwrap_or(html.len());
    &html[start..end]
}

fn sample_lead() -> SalesLead {
    SalesLead {
        id: "9001".to_string(),
        first_name: "Wei".to_string(),
        last_name: "Chen".to_string(),
        email: "wei@example.com".to_string(),
        company: Some("Hexagram".to_string()),
        position: Some("Managing Partner".to_string()),
        lead_score: 82.0,
        annual_income: 5_000_000.0,
        net_worth: 30_000_000.0,
        status: "qualified".to_string(),
        next_follow_up_date: Some("2026-09-15T00:00:00.000Z".to_string()),
        ..SalesLead::default()
    }
}

fn sample_opportunity() -> SalesOpportunity {
    SalesOpportunity {
        id: "9101".to_string(),
        lead_id: "9001".to_string(),
        name: "Black Card upgrade".to_string(),
        stage: "proposal".to_string(),
        probability: 60.0,
        value: 3_000_000.0,
        membership_tier: "Black Card".to_string(),
        expected_close_date: "2026-12-01T00:00:00.000Z".to_string(),
        lead: OpportunityLead {
            first_name: "Wei".to_string(),
            last_name: "Chen".to_string(),
            email: "wei@example.com".to_string(),
        },
        ..SalesOpportunity::default()
    }
}

fn sample_metrics() -> SalesMetrics {
    SalesMetrics {
        total_leads: 4.0,
        qualified_leads: 2.0,
        total_opportunities: 3.0,
        total_pipeline_value: 730_000.0,
        conversion_rate: 25.0,
        average_deal_size: 243_333.0,
        sales_cycle_length: 30.0,
        win_rate: 33.3,
        monthly_revenue: 500_000.0,
        quarterly_revenue: 500_000.0,
        yearly_revenue: 500_000.0,
    }
}

fn sample_pipeline() -> Vec<PipelineStageStat> {
    let stages = vec![
        PipelineStage {
            id: "9401".into(),
            name: "qualification".into(),
            display_order: 1,
            color_code: Some("#3b82f6".into()),
            is_active: true,
            ..PipelineStage::default()
        },
        PipelineStage {
            id: "9402".into(),
            name: "proposal".into(),
            display_order: 2,
            is_active: true,
            ..PipelineStage::default()
        },
    ];
    pipeline_stage_stats(&stages, &[sample_opportunity()])
}

#[component]
fn SalesAt(
    active_tab: SalesTab,
    loading: bool,
    error: Option<String>,
    leads: Vec<SalesLead>,
    opportunities: Vec<SalesOpportunity>,
    metrics: Option<SalesMetrics>,
    pipeline: Vec<PipelineStageStat>,
) -> Element {
    rsx! {
        AdminSalesScreen {
            active_tab,
            loading,
            error,
            leads,
            opportunities,
            metrics,
            pipeline,
            lead_filters: LeadFilters::with_page_size(),
            opp_filters: OpportunityFilters::with_page_size(),
        }
    }
}

fn render_sales(
    active_tab: SalesTab,
    loading: bool,
    error: Option<String>,
    leads: Vec<SalesLead>,
    opportunities: Vec<SalesOpportunity>,
    metrics: Option<SalesMetrics>,
    pipeline: Vec<PipelineStageStat>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        SalesAt,
        SalesAtProps {
            active_tab,
            loading,
            error,
            leads,
            opportunities,
            metrics,
            pipeline,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn sales_loading_copy() {
    let html = render_sales(
        SalesTab::Leads,
        true,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(
        html.contains("id=\"admin-sales-leads-loading\""),
        "loading id missing: {html}"
    );
    assert!(html.contains("載入中..."), "loading copy missing: {html}");
    assert!(html.contains("銷售管理系統"));
    assert!(html.contains("管理銷售線索、商機與績效分析"));
    assert!(!html.contains("目前沒有符合條件的銷售線索"));
}

#[wasm_bindgen_test]
fn sales_leads_empty_state() {
    let html = render_sales(
        SalesTab::Leads,
        false,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(html.contains("id=\"admin-sales-leads-empty\""));
    assert!(html.contains("目前沒有符合條件的銷售線索"));
    assert!(html.contains("搜尋線索..."));
    assert!(html.contains("所有狀態"));
    assert!(html.contains("新增線索"));
    assert!(html.contains("線索資訊"));
    assert!(html.contains("下次跟進"));
}

#[wasm_bindgen_test]
fn sales_leads_populated() {
    let html = render_sales(
        SalesTab::Leads,
        false,
        None,
        vec![sample_lead()],
        Vec::new(),
        None,
        Vec::new(),
    );
    for needle in [
        "id=\"admin-sales-lead-9001\"",
        "Wei Chen",
        "wei@example.com",
        "Hexagram - Managing Partner",
        "已審核",
        "年收: NT$ 5,000,000",
        "淨值: NT$ 30,000,000",
        "銷售線索",
        "重新整理",
        "data-icon=\"phone\"",
        "data-icon=\"mail\"",
        "data-icon=\"eye\"",
        "data-icon=\"edit\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in populated leads: {html}"
        );
    }
    let row = opening_tag(&html, "admin-sales-lead-9001");
    assert!(!row.is_empty(), "lead row missing: {html}");
}

#[wasm_bindgen_test]
fn sales_error_banner() {
    let html = render_sales(
        SalesTab::Leads,
        false,
        Some("Failed to fetch leads".into()),
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(html.contains("id=\"admin-sales-error\""));
    assert!(html.contains("Failed to fetch leads"));
    assert!(html.contains("id=\"admin-sales-error-dismiss\""));
}

#[wasm_bindgen_test]
fn sales_opportunities_empty_and_populated() {
    let empty = render_sales(
        SalesTab::Opportunities,
        false,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(empty.contains("id=\"admin-sales-opps-empty\""));
    assert!(empty.contains("目前沒有符合條件的銷售商機"));
    assert!(empty.contains("搜尋商機..."));
    assert!(empty.contains("新增商機"));
    assert!(empty.contains("所有階段"));

    let loading = render_sales(
        SalesTab::Opportunities,
        true,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(loading.contains("id=\"admin-sales-opps-loading\""));
    assert!(loading.contains("載入中..."));

    let html = render_sales(
        SalesTab::Opportunities,
        false,
        None,
        Vec::new(),
        vec![sample_opportunity()],
        None,
        Vec::new(),
    );
    for needle in [
        "id=\"admin-sales-opp-9101\"",
        "Black Card upgrade",
        "Wei Chen",
        "wei@example.com",
        "提案階段",
        "Black Card",
        "NT$ 3,000,000",
        "data-icon=\"shield\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in populated opps: {html}"
        );
    }
}

#[wasm_bindgen_test]
fn sales_metrics_loading_empty_populated() {
    let loading = render_sales(
        SalesTab::Metrics,
        true,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(loading.contains("id=\"admin-sales-metrics-loading\""));
    assert!(loading.contains("載入中..."));
    assert!(loading.contains("績效分析"));

    let empty = render_sales(
        SalesTab::Metrics,
        false,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(empty.contains("id=\"admin-sales-metrics-empty\""));
    assert!(!empty.contains("總線索數"));

    let html = render_sales(
        SalesTab::Metrics,
        false,
        None,
        Vec::new(),
        Vec::new(),
        Some(sample_metrics()),
        Vec::new(),
    );
    for needle in [
        "id=\"admin-sales-metrics\"",
        "總線索數",
        "合格線索",
        "總商機數",
        "管道總值",
        "NT$ 730,000",
        "成交數",
        "33.3%",
        "轉換率",
        "平均成交金額",
        "銷售週期",
        "30 天",
        "銷售漏斗",
        "新線索",
        "合格線索",
        "提案階段",
        "談判中",
        "成交",
        "營收",
        "本月成交營收",
        "本季成交營收",
        "今年成交營收",
        "NT$ 500,000",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in populated metrics: {html}"
        );
    }
    assert!(!html.contains("id=\"admin-sales-funnel\""));
}

#[wasm_bindgen_test]
fn sales_metrics_funnel_svg_from_pipeline_stages() {
    let html = render_sales(
        SalesTab::Metrics,
        false,
        None,
        Vec::new(),
        Vec::new(),
        Some(sample_metrics()),
        sample_pipeline(),
    );
    assert!(html.contains("id=\"admin-sales-funnel\""));
    assert!(html.contains("<svg"));
    assert!(html.contains("<polygon"));
    assert!(html.contains("新線索"));
    assert!(html.contains("合格線索"));
    assert!(html.contains("提案階段"));
    assert!(html.contains("談判中"));
    assert!(html.contains("成交"));
    assert!(html.contains("viewBox=\"0 0 400 280\"") || html.contains("viewbox=\"0 0 400 280\""));
}

#[wasm_bindgen_test]
fn sales_tabs_and_heading_always_present() {
    let html = render_sales(
        SalesTab::Leads,
        false,
        None,
        Vec::new(),
        Vec::new(),
        None,
        Vec::new(),
    );
    assert!(html.contains("id=\"admin-sales\""));
    assert!(html.contains("id=\"admin-sales-heading\""));
    assert!(html.contains("id=\"admin-sales-tab-leads\""));
    assert!(html.contains("id=\"admin-sales-tab-opportunities\""));
    assert!(html.contains("id=\"admin-sales-tab-metrics\""));
    assert!(html.contains("data-icon=\"trending-up\""));
    assert!(html.contains("data-icon=\"users\""));
    assert!(html.contains("data-icon=\"activity\""));
}
