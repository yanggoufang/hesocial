use std::cell::Cell;
use std::rc::Rc;

use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session};
use crate::sales::{
    FUNNEL_VIEW_HEIGHT, FUNNEL_VIEW_WIDTH, LeadFilters, OpportunityFilters, PAGE_SIZE,
    PipelineStageStat, SalesLead, SalesMetrics, SalesOpportunity, SalesTab, admin_route_guard,
    fetch_leads, fetch_metrics, fetch_opportunities, fetch_pipeline_stages, format_currency,
    format_one_decimal, format_sales_date, funnel_bands, funnel_counts, funnel_polygon_points,
    lead_display_name, lead_status_class, lead_status_label, membership_tier_badge_class,
    opportunity_lead_name, opportunity_stage_label, pipeline_stage_stats, score_bar_percent,
};
use dioxus::prelude::*;

#[component]
pub fn AdminSales() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match admin_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            GuardStatus {
                id: "admin-sales-guard-loading".to_string(),
                message: "驗證存取權限中...".to_string(),
                spinning: true,
            }
        },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "admin-sales-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { AdminSalesBody {} },
    }
}

#[component]
fn AdminSalesBody() -> Element {
    let mut active_tab = use_signal(SalesTab::default);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut leads = use_signal(Vec::<SalesLead>::new);
    let mut opportunities = use_signal(Vec::<SalesOpportunity>::new);
    let mut metrics = use_signal(|| None::<SalesMetrics>);
    let mut pipeline = use_signal(Vec::<PipelineStageStat>::new);
    let mut lead_filters = use_signal(LeadFilters::with_page_size);
    let mut opp_filters = use_signal(OpportunityFilters::with_page_size);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let tab = active_tab();
            let current_leads = lead_filters();
            let current_opps = opp_filters();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            error.set(None);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                match tab {
                    SalesTab::Leads => {
                        let result = fetch_leads(&current_leads).await;
                        if fetch_gen.get() != request_id {
                            return;
                        }
                        match result {
                            Ok(view) => {
                                leads.set(view.leads);
                                error.set(None);
                            }
                            Err(message) => {
                                leads.set(Vec::new());
                                error.set(Some(message));
                            }
                        }
                    }
                    SalesTab::Opportunities => {
                        let result = fetch_opportunities(&current_opps).await;
                        if fetch_gen.get() != request_id {
                            return;
                        }
                        match result {
                            Ok(view) => {
                                opportunities.set(view.opportunities);
                                error.set(None);
                            }
                            Err(message) => {
                                opportunities.set(Vec::new());
                                error.set(Some(message));
                            }
                        }
                    }
                    SalesTab::Metrics => {
                        let metrics_result = fetch_metrics().await;
                        let stages_result = fetch_pipeline_stages().await;
                        let funnel_filters = OpportunityFilters {
                            page: 1,
                            limit: PAGE_SIZE,
                            ..OpportunityFilters::default()
                        };
                        let opps_result = fetch_opportunities(&funnel_filters).await;
                        if fetch_gen.get() != request_id {
                            return;
                        }
                        match metrics_result {
                            Ok(fetched) => {
                                metrics.set(Some(fetched));
                                error.set(None);
                            }
                            Err(message) => {
                                metrics.set(None);
                                error.set(Some(message));
                            }
                        }
                        let stages = stages_result.unwrap_or_default();
                        let funnel_opps = opps_result
                            .map(|view| view.opportunities)
                            .unwrap_or_default();
                        pipeline.set(pipeline_stage_stats(&stages, &funnel_opps));
                    }
                }
                if fetch_gen.get() == request_id {
                    loading.set(false);
                }
            });
        }
    });

    rsx! {
        AdminSalesScreen {
            active_tab: active_tab(),
            loading: loading(),
            error: error(),
            leads: leads(),
            opportunities: opportunities(),
            metrics: metrics(),
            pipeline: pipeline(),
            lead_filters: lead_filters(),
            opp_filters: opp_filters(),
            on_tab: move |tab: SalesTab| active_tab.set(tab),
            on_lead_search: move |value: String| {
                lead_filters.write().search = value;
            },
            on_lead_status: move |value: String| {
                lead_filters.write().status = value;
            },
            on_opp_search: move |value: String| {
                opp_filters.write().search = value;
            },
            on_opp_stage: move |value: String| {
                opp_filters.write().stage = value;
            },
            on_dismiss_error: move |_| error.set(None),
            on_refresh: move |_| {
                let tab = active_tab();
                let current_leads = lead_filters();
                let current_opps = opp_filters();
                let request_id = fetch_gen.get() + 1;
                fetch_gen.set(request_id);
                loading.set(true);
                error.set(None);
                let fetch_gen = fetch_gen.clone();
                spawn(async move {
                    match tab {
                        SalesTab::Leads => match fetch_leads(&current_leads).await {
                            Ok(view) => {
                                if fetch_gen.get() == request_id {
                                    leads.set(view.leads);
                                    error.set(None);
                                }
                            }
                            Err(message) => {
                                if fetch_gen.get() == request_id {
                                    leads.set(Vec::new());
                                    error.set(Some(message));
                                }
                            }
                        },
                        SalesTab::Opportunities => match fetch_opportunities(&current_opps).await {
                            Ok(view) => {
                                if fetch_gen.get() == request_id {
                                    opportunities.set(view.opportunities);
                                    error.set(None);
                                }
                            }
                            Err(message) => {
                                if fetch_gen.get() == request_id {
                                    opportunities.set(Vec::new());
                                    error.set(Some(message));
                                }
                            }
                        },
                        SalesTab::Metrics => {
                            let metrics_result = fetch_metrics().await;
                            let stages_result = fetch_pipeline_stages().await;
                            let funnel_filters = OpportunityFilters {
                                page: 1,
                                limit: PAGE_SIZE,
                                ..OpportunityFilters::default()
                            };
                            let opps_result = fetch_opportunities(&funnel_filters).await;
                            if fetch_gen.get() != request_id {
                                return;
                            }
                            match metrics_result {
                                Ok(fetched) => {
                                    metrics.set(Some(fetched));
                                    error.set(None);
                                }
                                Err(message) => {
                                    metrics.set(None);
                                    error.set(Some(message));
                                }
                            }
                            let stages = stages_result.unwrap_or_default();
                            let funnel_opps =
                                opps_result.map(|view| view.opportunities).unwrap_or_default();
                            pipeline.set(pipeline_stage_stats(&stages, &funnel_opps));
                        }
                    }
                    if fetch_gen.get() == request_id {
                        loading.set(false);
                    }
                });
            },
        }
    }
}

#[component]
pub fn AdminSalesScreen(
    active_tab: SalesTab,
    loading: bool,
    error: Option<String>,
    leads: Vec<SalesLead>,
    opportunities: Vec<SalesOpportunity>,
    metrics: Option<SalesMetrics>,
    pipeline: Vec<PipelineStageStat>,
    lead_filters: LeadFilters,
    opp_filters: OpportunityFilters,
    #[props(default)] on_tab: EventHandler<SalesTab>,
    #[props(default)] on_lead_search: EventHandler<String>,
    #[props(default)] on_lead_status: EventHandler<String>,
    #[props(default)] on_opp_search: EventHandler<String>,
    #[props(default)] on_opp_stage: EventHandler<String>,
    #[props(default)] on_dismiss_error: EventHandler<()>,
    #[props(default)] on_refresh: EventHandler<()>,
) -> Element {
    let leads_tab_class = tab_class(active_tab == SalesTab::Leads);
    let opps_tab_class = tab_class(active_tab == SalesTab::Opportunities);
    let metrics_tab_class = tab_class(active_tab == SalesTab::Metrics);

    rsx! {
        div { id: "admin-sales", class: "min-h-screen bg-luxury-midnight-black py-8 px-4",
            div { class: "max-w-7xl mx-auto",
                div {
                    class: "hs-enter",
                    style: "--hs-from: 30px",
                    div { class: "mb-8",
                        div { class: "flex items-center justify-between",
                            div {
                                h1 {
                                    id: "admin-sales-heading",
                                    class: "text-3xl font-bold text-luxury-platinum flex items-center gap-3",
                                    Icon {
                                        name: IconName::TrendingUp,
                                        class: "w-8 h-8 text-luxury-gold".to_string(),
                                    }
                                    "銷售管理系統"
                                }
                                p { class: "text-luxury-platinum/70 mt-2",
                                    "管理銷售線索、商機與績效分析"
                                }
                            }
                            button {
                                id: "admin-sales-refresh",
                                r#type: "button",
                                class: "luxury-button",
                                onclick: move |_| on_refresh.call(()),
                                Icon { name: IconName::RefreshCw, class: "w-4 h-4".to_string() }
                                "重新整理"
                            }
                        }
                    }

                    if let Some(message) = error.clone() {
                        div {
                            id: "admin-sales-error",
                            class: "mb-6 luxury-glass border border-red-500/20 rounded-lg p-4",
                            div { class: "flex items-center",
                                Icon {
                                    name: IconName::AlertCircle,
                                    class: "w-5 h-5 text-red-400 mr-2".to_string(),
                                }
                                p { class: "text-red-300", "{message}" }
                                button {
                                    id: "admin-sales-error-dismiss",
                                    r#type: "button",
                                    class: "ml-auto text-red-400 hover:text-red-300",
                                    onclick: move |_| on_dismiss_error.call(()),
                                    Icon { name: IconName::X, class: "w-4 h-4".to_string() }
                                }
                            }
                        }
                    }

                    div { class: "mb-6",
                        div { class: "border-b border-luxury-gold/20",
                            nav { class: "-mb-px flex space-x-8",
                                button {
                                    id: "admin-sales-tab-leads",
                                    r#type: "button",
                                    class: "{leads_tab_class}",
                                    onclick: move |_| on_tab.call(SalesTab::Leads),
                                    div { class: "flex items-center gap-2",
                                        Icon { name: IconName::Users, class: "w-4 h-4".to_string() }
                                        "銷售線索"
                                    }
                                }
                                button {
                                    id: "admin-sales-tab-opportunities",
                                    r#type: "button",
                                    class: "{opps_tab_class}",
                                    onclick: move |_| on_tab.call(SalesTab::Opportunities),
                                    div { class: "flex items-center gap-2",
                                        Icon { name: IconName::Award, class: "w-4 h-4".to_string() }
                                        "銷售商機"
                                    }
                                }
                                button {
                                    id: "admin-sales-tab-metrics",
                                    r#type: "button",
                                    class: "{metrics_tab_class}",
                                    onclick: move |_| on_tab.call(SalesTab::Metrics),
                                    div { class: "flex items-center gap-2",
                                        Icon { name: IconName::Activity, class: "w-4 h-4".to_string() }
                                        "績效分析"
                                    }
                                }
                            }
                        }
                    }

                    match active_tab {
                        SalesTab::Leads => rsx! {
                            LeadsPanel {
                                loading,
                                leads,
                                lead_filters,
                                on_lead_search,
                                on_lead_status,
                            }
                        },
                        SalesTab::Opportunities => rsx! {
                            OpportunitiesPanel {
                                loading,
                                opportunities,
                                opp_filters,
                                on_opp_search,
                                on_opp_stage,
                            }
                        },
                        SalesTab::Metrics => rsx! {
                            MetricsPanel {
                                loading,
                                metrics,
                                pipeline,
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn LeadsPanel(
    loading: bool,
    leads: Vec<SalesLead>,
    lead_filters: LeadFilters,
    on_lead_search: EventHandler<String>,
    on_lead_status: EventHandler<String>,
) -> Element {
    rsx! {
        div { id: "admin-sales-leads",
            div { class: "luxury-glass rounded-lg border border-luxury-gold/20 mb-6 p-4",
                div { class: "flex items-center gap-4 mb-4",
                    div { class: "relative flex-1",
                        Icon {
                            name: IconName::Search,
                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-luxury-platinum/50".to_string(),
                        }
                        input {
                            r#type: "text",
                            id: "admin-sales-lead-search",
                            placeholder: "搜尋線索...",
                            value: "{lead_filters.search}",
                            class: "w-full pl-10 pr-4 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/50 focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold",
                            oninput: move |evt| on_lead_search.call(evt.value()),
                        }
                    }
                    select {
                        id: "admin-sales-lead-status",
                        value: "{lead_filters.status}",
                        class: "px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold",
                        onchange: move |evt| on_lead_status.call(evt.value()),
                        option { value: "", selected: lead_filters.status.is_empty(), "所有狀態" }
                        option { value: "new", selected: lead_filters.status == "new", "新線索" }
                        option { value: "qualified", selected: lead_filters.status == "qualified", "已審核" }
                        option { value: "contacted", selected: lead_filters.status == "contacted", "已聯繫" }
                        option { value: "nurturing", selected: lead_filters.status == "nurturing", "培養中" }
                    }
                    button { r#type: "button", class: "luxury-button",
                        Icon { name: IconName::Plus, class: "w-4 h-4".to_string() }
                        "新增線索"
                    }
                }
            }
            div { class: "luxury-glass rounded-lg border border-luxury-gold/20 overflow-hidden",
                if loading {
                    div {
                        id: "admin-sales-leads-loading",
                        class: "flex items-center justify-center py-12",
                        Icon {
                            name: IconName::RefreshCw,
                            class: "w-6 h-6 text-luxury-gold animate-spin".to_string(),
                        }
                        span { class: "ml-2 text-luxury-platinum", "載入中..." }
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "w-full",
                            thead { class: "bg-luxury-midnight-black/30",
                                tr {
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-luxury-gold uppercase tracking-wider",
                                        "線索資訊"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-luxury-gold uppercase tracking-wider",
                                        "分數"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-luxury-gold uppercase tracking-wider",
                                        "狀態"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-luxury-gold uppercase tracking-wider",
                                        "財務資訊"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-luxury-gold uppercase tracking-wider",
                                        "下次跟進"
                                    }
                                    th { class: "px-6 py-3 text-right text-xs font-medium text-luxury-gold uppercase tracking-wider",
                                        "操作"
                                    }
                                }
                            }
                            tbody { class: "divide-y divide-luxury-gold/10",
                                if leads.is_empty() {
                                    tr {
                                        td {
                                            id: "admin-sales-leads-empty",
                                            colspan: 6,
                                            class: "px-6 py-12 text-center text-luxury-platinum/70",
                                            "目前沒有符合條件的銷售線索"
                                        }
                                    }
                                }
                                for lead in leads.iter().cloned() {
                                    LeadRow { lead }
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
fn LeadRow(lead: SalesLead) -> Element {
    let name = lead_display_name(&lead);
    let email = lead.email.clone();
    let company = lead.company.clone().unwrap_or_default();
    let position = lead.position.clone().unwrap_or_default();
    let company_line = if company.is_empty() && position.is_empty() {
        String::new()
    } else if company.is_empty() {
        position.clone()
    } else if position.is_empty() {
        company.clone()
    } else {
        format!("{company} - {position}")
    };
    let score = lead.lead_score;
    let score_width = score_bar_percent(score);
    let status = lead.status.clone();
    let status_label = lead_status_label(&status);
    let status_class = lead_status_class(&status);
    let income = format_currency(lead.annual_income);
    let net_worth = format_currency(lead.net_worth);
    let follow_up = lead
        .next_follow_up_date
        .as_deref()
        .map(format_sales_date)
        .unwrap_or_else(|| "-".to_string());
    let row_id = format!("admin-sales-lead-{}", lead.id);

    rsx! {
        tr { id: "{row_id}", class: "hover:bg-luxury-gold/5 transition-colors",
            td { class: "px-6 py-4 whitespace-nowrap",
                div {
                    div { class: "text-sm font-medium text-luxury-platinum", "{name}" }
                    div { class: "text-sm text-luxury-platinum/70", "{email}" }
                    if !company_line.is_empty() {
                        div { class: "text-xs text-luxury-platinum/50", "{company_line}" }
                    }
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap",
                div { class: "flex items-center",
                    div { class: "text-sm font-medium text-luxury-gold", "{score}" }
                    div { class: "ml-2 w-16 bg-luxury-midnight-black/50 rounded-full h-2",
                        div {
                            class: "bg-luxury-gold h-2 rounded-full",
                            style: "width: {score_width}%",
                        }
                    }
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap",
                span { class: "inline-flex px-2 py-1 rounded-full text-xs font-medium {status_class}",
                    "{status_label}"
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap text-sm text-luxury-platinum",
                div { "年收: {income}" }
                div { class: "text-luxury-platinum/70", "淨值: {net_worth}" }
            }
            td { class: "px-6 py-4 whitespace-nowrap text-sm text-luxury-platinum/70", "{follow_up}" }
            td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                div { class: "flex items-center justify-end gap-2",
                    button {
                        r#type: "button",
                        class: "text-luxury-platinum/70 hover:text-luxury-platinum transition-colors",
                        Icon { name: IconName::Eye, class: "w-4 h-4".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "text-luxury-gold/70 hover:text-luxury-gold transition-colors",
                        Icon { name: IconName::Edit, class: "w-4 h-4".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "text-green-400/70 hover:text-green-400 transition-colors",
                        Icon { name: IconName::Phone, class: "w-4 h-4".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "text-blue-400/70 hover:text-blue-400 transition-colors",
                        Icon { name: IconName::Mail, class: "w-4 h-4".to_string() }
                    }
                }
            }
        }
    }
}

#[component]
fn OpportunitiesPanel(
    loading: bool,
    opportunities: Vec<SalesOpportunity>,
    opp_filters: OpportunityFilters,
    on_opp_search: EventHandler<String>,
    on_opp_stage: EventHandler<String>,
) -> Element {
    rsx! {
        div { id: "admin-sales-opportunities",
            div { class: "luxury-glass rounded-lg border border-luxury-gold/20 mb-6 p-4",
                div { class: "flex items-center gap-4",
                    div { class: "relative flex-1",
                        Icon {
                            name: IconName::Search,
                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-luxury-platinum/50".to_string(),
                        }
                        input {
                            r#type: "text",
                            id: "admin-sales-opp-search",
                            placeholder: "搜尋商機...",
                            value: "{opp_filters.search}",
                            class: "w-full pl-10 pr-4 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/50 focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold",
                            oninput: move |evt| on_opp_search.call(evt.value()),
                        }
                    }
                    select {
                        id: "admin-sales-opp-stage",
                        value: "{opp_filters.stage}",
                        class: "px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold",
                        onchange: move |evt| on_opp_stage.call(evt.value()),
                        option { value: "", selected: opp_filters.stage.is_empty(), "所有階段" }
                        option { value: "qualification", selected: opp_filters.stage == "qualification", "資格審核" }
                        option { value: "needs_analysis", selected: opp_filters.stage == "needs_analysis", "需求分析" }
                        option { value: "proposal", selected: opp_filters.stage == "proposal", "提案階段" }
                        option { value: "negotiation", selected: opp_filters.stage == "negotiation", "談判中" }
                    }
                    button { r#type: "button", class: "luxury-button",
                        Icon { name: IconName::Plus, class: "w-4 h-4".to_string() }
                        "新增商機"
                    }
                }
            }
            div { class: "bg-white rounded-lg shadow-sm border overflow-hidden",
                if loading {
                    div {
                        id: "admin-sales-opps-loading",
                        class: "flex items-center justify-center py-12",
                        Icon {
                            name: IconName::RefreshCw,
                            class: "w-6 h-6 text-gray-400 animate-spin".to_string(),
                        }
                        span { class: "ml-2 text-gray-600", "載入中..." }
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "w-full",
                            thead { class: "bg-gray-50",
                                tr {
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "商機名稱"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "階段"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "會員等級"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "機率"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "價值"
                                    }
                                    th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "預期成交"
                                    }
                                    th { class: "px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider",
                                        "操作"
                                    }
                                }
                            }
                            tbody { class: "bg-white divide-y divide-gray-200",
                                if opportunities.is_empty() {
                                    tr {
                                        td {
                                            id: "admin-sales-opps-empty",
                                            colspan: 7,
                                            class: "px-6 py-12 text-center text-gray-500",
                                            "目前沒有符合條件的銷售商機"
                                        }
                                    }
                                }
                                for opportunity in opportunities.iter().cloned() {
                                    OpportunityRow { opportunity }
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
fn OpportunityRow(opportunity: SalesOpportunity) -> Element {
    let name = opportunity.name.clone();
    let lead_name = opportunity_lead_name(&opportunity.lead);
    let lead_email = opportunity.lead.email.clone();
    let stage_label = opportunity_stage_label(&opportunity.stage);
    let tier = opportunity.membership_tier.clone();
    let badge_class = membership_tier_badge_class(&tier);
    let tier_icon = match tier.as_str() {
        "Diamond" => IconName::Crown,
        "Black Card" => IconName::Shield,
        _ => IconName::Star,
    };
    let probability = opportunity.probability;
    let probability_width = score_bar_percent(probability);
    let value = format_currency(opportunity.value);
    let close_date = format_sales_date(&opportunity.expected_close_date);
    let row_id = format!("admin-sales-opp-{}", opportunity.id);

    rsx! {
        tr { id: "{row_id}", class: "hover:bg-gray-50",
            td { class: "px-6 py-4 whitespace-nowrap",
                div {
                    div { class: "text-sm font-medium text-gray-900", "{name}" }
                    div { class: "text-sm text-gray-500", "{lead_name}" }
                    div { class: "text-xs text-gray-400", "{lead_email}" }
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900", "{stage_label}" }
            td { class: "px-6 py-4 whitespace-nowrap",
                span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border {badge_class}",
                    Icon { name: tier_icon, class: "w-3 h-3".to_string() }
                    "{tier}"
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap",
                div { class: "flex items-center",
                    div { class: "text-sm font-medium text-gray-900", "{probability}%" }
                    div { class: "ml-2 w-16 bg-gray-200 rounded-full h-2",
                        div {
                            class: "bg-green-500 h-2 rounded-full",
                            style: "width: {probability_width}%",
                        }
                    }
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900", "{value}" }
            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{close_date}" }
            td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                div { class: "flex items-center justify-end gap-2",
                    button {
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-900",
                        Icon { name: IconName::Eye, class: "w-4 h-4".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "text-blue-600 hover:text-blue-900",
                        Icon { name: IconName::Edit, class: "w-4 h-4".to_string() }
                    }
                }
            }
        }
    }
}

#[component]
fn MetricsPanel(
    loading: bool,
    metrics: Option<SalesMetrics>,
    pipeline: Vec<PipelineStageStat>,
) -> Element {
    if loading && metrics.is_none() {
        return rsx! {
            div {
                id: "admin-sales-metrics-loading",
                class: "flex items-center justify-center py-12",
                Icon {
                    name: IconName::RefreshCw,
                    class: "w-6 h-6 text-luxury-gold animate-spin".to_string(),
                }
                span { class: "ml-2 text-luxury-platinum", "載入中..." }
            }
        };
    }
    let Some(metrics) = metrics else {
        return rsx! {
            div { id: "admin-sales-metrics-empty" }
        };
    };

    let total_leads = metrics.total_leads;
    let qualified = metrics.qualified_leads;
    let total_opps = metrics.total_opportunities;
    let pipeline_value = format_currency(metrics.total_pipeline_value);
    let win_rate = format_one_decimal(metrics.win_rate);
    let conversion = metrics.conversion_rate;
    let conversion_fixed = format_one_decimal(metrics.conversion_rate);
    let avg_deal = format_currency(metrics.average_deal_size);
    let cycle = metrics.sales_cycle_length;
    let monthly = format_currency(metrics.monthly_revenue);
    let quarterly = format_currency(metrics.quarterly_revenue);
    let yearly = format_currency(metrics.yearly_revenue);
    let show_funnel = !pipeline.is_empty();
    let counts = funnel_counts(&pipeline);
    let bands = funnel_bands(&counts, FUNNEL_VIEW_WIDTH, FUNNEL_VIEW_HEIGHT);

    rsx! {
        div { id: "admin-sales-metrics",
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8",
                MetricCard {
                    label: "總線索數".to_string(),
                    value: total_leads.to_string(),
                    icon: IconName::Users,
                    icon_class: "w-8 h-8 text-luxury-gold".to_string(),
                }
                MetricCard {
                    label: "合格線索".to_string(),
                    value: qualified.to_string(),
                    icon: IconName::Check,
                    icon_class: "w-8 h-8 text-green-400".to_string(),
                }
                MetricCard {
                    label: "總商機數".to_string(),
                    value: total_opps.to_string(),
                    icon: IconName::Award,
                    icon_class: "w-8 h-8 text-luxury-gold".to_string(),
                }
                MetricCard {
                    label: "管道總值".to_string(),
                    value: pipeline_value,
                    icon: IconName::DollarSign,
                    icon_class: "w-8 h-8 text-yellow-400".to_string(),
                }
                MetricCard {
                    label: "成交數".to_string(),
                    value: format!("{win_rate}%"),
                    icon: IconName::Check,
                    icon_class: "w-8 h-8 text-emerald-400".to_string(),
                }
                MetricCard {
                    label: "轉換率".to_string(),
                    value: format!("{conversion}%"),
                    icon: IconName::TrendingUp,
                    icon_class: "w-8 h-8 text-luxury-gold".to_string(),
                }
                MetricCard {
                    label: "平均成交金額".to_string(),
                    value: avg_deal,
                    icon: IconName::DollarSign,
                    icon_class: "w-8 h-8 text-orange-400".to_string(),
                }
                MetricCard {
                    label: "銷售週期".to_string(),
                    value: format!("{cycle} 天"),
                    icon: IconName::Calendar,
                    icon_class: "w-8 h-8 text-red-400".to_string(),
                }
            }
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                div { class: "luxury-glass rounded-lg p-6 border border-luxury-gold/20",
                    h3 { class: "text-lg font-medium text-luxury-platinum mb-4", "銷售漏斗" }
                    if show_funnel {
                        svg {
                            id: "admin-sales-funnel",
                            class: "w-full h-64 text-luxury-gold mb-4",
                            view_box: format!(
                                "0 0 {:.0} {:.0}",
                                FUNNEL_VIEW_WIDTH, FUNNEL_VIEW_HEIGHT
                            ),
                            for (index, band) in bands.iter().cloned().enumerate() {
                                {
                                    let fill = pipeline
                                        .get(index)
                                        .and_then(|stat| stat.stage.color_code.clone())
                                        .filter(|code| code.starts_with('#'))
                                        .unwrap_or_else(|| "currentColor".to_string());
                                    let points = funnel_polygon_points(&band);
                                    let opacity = 0.35 + (index as f64 * 0.12);
                                    rsx! {
                                        polygon {
                                            points: "{points}",
                                            fill: "{fill}",
                                            opacity: "{opacity}",
                                            stroke: "currentColor",
                                            stroke_width: "1",
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "space-y-4",
                        FunnelMetricRow { label: "新線索".to_string(), value: total_leads.to_string() }
                        FunnelMetricRow { label: "合格線索".to_string(), value: qualified.to_string() }
                        FunnelMetricRow {
                            label: "提案階段".to_string(),
                            value: total_opps.to_string(),
                        }
                        FunnelMetricRow {
                            label: "談判中".to_string(),
                            value: format!("{conversion_fixed}%"),
                        }
                        FunnelMetricRow { label: "成交".to_string(), value: format!("{win_rate}%") }
                    }
                }
                div { class: "luxury-glass rounded-lg p-6 border border-luxury-gold/20",
                    h3 { class: "text-lg font-medium text-luxury-platinum mb-4", "營收" }
                    div { class: "space-y-4",
                        FunnelMetricRow { label: "本月成交營收".to_string(), value: monthly }
                        FunnelMetricRow { label: "本季成交營收".to_string(), value: quarterly }
                        FunnelMetricRow { label: "今年成交營收".to_string(), value: yearly }
                    }
                }
            }
        }
    }
}

#[component]
fn MetricCard(label: String, value: String, icon: IconName, icon_class: String) -> Element {
    rsx! {
        div { class: "luxury-glass rounded-lg p-6 border border-luxury-gold/20",
            div { class: "flex items-center justify-between",
                div {
                    p { class: "text-sm font-medium text-luxury-platinum/70", "{label}" }
                    p { class: "text-3xl font-bold text-luxury-platinum", "{value}" }
                }
                Icon { name: icon, class: icon_class }
            }
        }
    }
}

#[component]
fn FunnelMetricRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between",
            span { class: "text-sm text-luxury-platinum/70", "{label}" }
            span { class: "text-sm font-medium text-luxury-gold", "{value}" }
        }
    }
}

#[component]
fn GuardStatus(id: String, message: String, spinning: bool) -> Element {
    rsx! {
        div {
            id: "{id}",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                if spinning {
                    div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                }
                p { class: "text-luxury-platinum", "{message}" }
            }
        }
    }
}

fn tab_class(active: bool) -> &'static str {
    if active {
        "py-2 px-1 border-b-2 font-medium text-sm transition-colors border-luxury-gold text-luxury-gold"
    } else {
        "py-2 px-1 border-b-2 font-medium text-sm transition-colors border-transparent text-luxury-platinum/70 hover:text-luxury-platinum hover:border-luxury-gold/30"
    }
}
