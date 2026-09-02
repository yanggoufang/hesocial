#![cfg(target_arch = "wasm32")]

use std::rc::Rc;

use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::*;
use dioxus::router::components::HistoryProvider;
use hesocial_frontend::events::{Event, Pagination, Pricing, Venue};
use hesocial_frontend::ui::{EventCard, EventsScreen, LoginScreen, Route};
use wasm_bindgen_test::wasm_bindgen_test;

#[component]
fn At(path: String) -> Element {
    rsx! {
        HistoryProvider {
            history: move |_| {
                Rc::new(MemoryHistory::with_initial_path(path.clone())) as Rc<dyn History>
            },
            Router::<Route> {}
        }
    }
}

fn render_path(path: &str) -> String {
    let mut vdom = VirtualDom::new_with_props(At, AtProps { path: path.into() });
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

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

#[wasm_bindgen_test]
fn home_renders_heading_and_toggle_button() {
    let html = render_path("/");
    assert!(
        html.contains("HeSocial"),
        "expected heading text in SSR markup, got: {html}"
    );
    assert!(
        html.contains("toggle-btn"),
        "expected toggle button id in SSR markup, got: {html}"
    );
}

#[wasm_bindgen_test]
fn login_renders_traditional_chinese_copy() {
    let html = render_path("/login");
    for needle in [
        "歡迎回來",
        "登入您的尊榮帳戶",
        "電子郵件",
        "請輸入您的電子郵件",
        "密碼",
        "請輸入您的密碼",
        "記住我",
        "忘記密碼？",
        "登入",
        "或使用",
        "Google",
        "LinkedIn (即將推出)",
        "還沒有帳戶？",
        "立即申請加入",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in login markup, got: {html}"
        );
    }
    assert!(
        html.contains("href=\"/forgot-password\""),
        "forgot-password link missing: {html}"
    );
    assert!(
        html.contains("href=\"/register\""),
        "register link missing: {html}"
    );
}

#[wasm_bindgen_test]
fn login_linkedin_is_disabled_and_submit_is_not() {
    let html = render_path("/login");
    let linkedin = opening_tag(&html, "linkedin-login");
    assert!(!linkedin.is_empty(), "linkedin button missing: {html}");
    assert!(
        linkedin.contains("disabled=") || linkedin.contains(" disabled>"),
        "linkedin must be permanently disabled, tag={linkedin} html={html}"
    );

    let submit = opening_tag(&html, "login-submit");
    assert!(
        !submit.contains("disabled=") && !submit.contains(" disabled>"),
        "submit must be enabled at rest, tag={submit} html={html}"
    );
}

#[component]
fn SubmittingLogin() -> Element {
    rsx! {
        LoginScreen {
            email: String::new(),
            password: String::new(),
            show_password: false,
            submitting: true,
            error: None,
        }
    }
}

#[wasm_bindgen_test]
fn login_submit_disabled_while_in_flight() {
    let mut vdom = VirtualDom::new(SubmittingLogin);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    let submit = opening_tag(&html, "login-submit");
    assert!(
        submit.contains("disabled=") || submit.contains(" disabled>"),
        "submit must be disabled while a request is in flight, tag={submit} html={html}"
    );
    assert!(
        html.contains("登入中..."),
        "expected in-flight label, got: {html}"
    );
}

#[wasm_bindgen_test]
fn login_password_field_starts_masked() {
    let html = render_path("/login");
    let field = opening_tag(&html, "password");
    assert!(
        field.contains("type=\"password\"") || field.contains("type='password'"),
        "password must start masked, tag={field} html={html}"
    );
}

fn sample_event() -> Event {
    Event {
        id: "11".to_string(),
        name: "松露季私宴".to_string(),
        description: "白松露當季，主廚八道式無菜單。".to_string(),
        date_time: "2026-10-04T12:00:00.000Z".to_string(),
        venue: Some(Venue {
            name: "Taipei Private Dining Room".to_string(),
            address: "Da'an".to_string(),
            rating: 5.0,
        }),
        exclusivity_level: None,
        pricing: Pricing {
            vvip: Some(15000.0),
            vip: Some(15000.0),
            currency: "TWD".to_string(),
        },
        current_attendees: 0,
        capacity: 12,
        images: Some(vec!["https://media.example/e11.webp".into()]),
    }
}

#[component]
fn CardAt(event: Event) -> Element {
    rsx! {
        EventCard { event, index: 0 }
    }
}

fn render_card(event: Event) -> String {
    let mut vdom = VirtualDom::new_with_props(CardAt, CardAtProps { event });
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[component]
fn ScreenAt(
    events: Vec<Event>,
    loading: bool,
    page: u32,
    total_pages: u32,
) -> Element {
    let total = if events.is_empty() { 0 } else { 12 };
    rsx! {
        EventsScreen {
            search: String::new(),
            category: "all".to_string(),
            level: "all".to_string(),
            events,
            loading,
            pagination: Pagination {
                page,
                limit: 9,
                total,
                total_pages,
            },
        }
    }
}

fn render_screen(events: Vec<Event>, loading: bool, page: u32, total_pages: u32) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ScreenAt,
        ScreenAtProps {
            events,
            loading,
            page,
            total_pages,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn events_route_starts_in_loading_state() {
    let html = render_path("/events");
    assert!(
        html.contains("載入中..."),
        "expected loading copy on first paint, got: {html}"
    );
    assert!(
        html.contains("精選活動"),
        "expected page heading, got: {html}"
    );
}

#[wasm_bindgen_test]
fn events_loading_state_hides_grid_and_empty() {
    let html = render_screen(vec![sample_event()], true, 1, 2);
    assert!(html.contains("載入中..."), "missing loading copy: {html}");
    assert!(
        !html.contains("event-card-11"),
        "cards must not render while loading: {html}"
    );
    assert!(
        !html.contains("找不到符合條件的活動"),
        "empty state must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn events_empty_state_copy() {
    let html = render_screen(vec![], false, 1, 1);
    assert!(
        html.contains("找不到符合條件的活動。"),
        "missing empty title: {html}"
    );
    assert!(
        html.contains("請嘗試調整您的篩選條件，或稍後再試。"),
        "missing empty hint: {html}"
    );
    assert!(
        !html.contains("載入中..."),
        "loading copy must not show in empty state: {html}"
    );
}

#[wasm_bindgen_test]
fn event_card_renders_fields_and_placeholder_rules() {
    let html = render_card(sample_event());
    for needle in [
        "松露季私宴",
        "白松露當季，主廚八道式無菜單。",
        "Taipei Private Dining Room",
        "0/12 人",
        "NT$ 15,000",
        "查看詳情",
        "https://media.example/e11.webp",
        "href=\"/events/11\"",
        "data-icon=\"calendar\"",
        "data-icon=\"map-pin\"",
        "data-icon=\"users\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in card markup, got: {html}"
        );
    }
}

#[wasm_bindgen_test]
fn null_exclusivity_renders_empty_gray_badge_without_stars() {
    let html = render_card(sample_event());
    let badge = opening_tag(&html, "event-badge-11");
    assert!(!badge.is_empty(), "null exclusivity still renders the span: {html}");
    assert!(
        badge.contains("bg-gray-500/20") && badge.contains("text-gray-400"),
        "null level must use the default gray classes, tag={badge}"
    );
    assert!(
        !html.contains("data-icon=\"star\""),
        "null level must not render stars: {html}"
    );
    assert!(
        !html.contains("data-icon=\"diamond\""),
        "null level must not render diamond: {html}"
    );
}

#[wasm_bindgen_test]
fn vip_badge_uses_blue_and_two_stars() {
    let mut event = sample_event();
    event.exclusivity_level = Some("VIP".into());
    let html = render_card(event);
    let badge = opening_tag(&html, "event-badge-11");
    assert!(
        badge.contains("bg-blue-500/20") && badge.contains("text-blue-400"),
        "VIP color missing, tag={badge}"
    );
    assert!(
        html.contains(">VIP<") || html.contains(">VIP</span>"),
        "VIP label missing: {html}"
    );
    let stars = html.matches("data-icon=\"star\"").count();
    assert_eq!(stars, 2, "VIP must render two stars, got {stars} in {html}");
    assert!(
        !html.contains("data-icon=\"diamond\""),
        "VIP must not render diamond: {html}"
    );
}

#[wasm_bindgen_test]
fn vvip_and_invitation_only_select_stars_and_diamond() {
    let mut vvip = sample_event();
    vvip.exclusivity_level = Some("VVIP".into());
    let html = render_card(vvip);
    let badge = opening_tag(&html, "event-badge-11");
    assert!(
        badge.contains("text-luxury-gold"),
        "VVIP gold color missing, tag={badge}"
    );
    assert_eq!(html.matches("data-icon=\"star\"").count(), 3);
    assert!(!html.contains("data-icon=\"diamond\""));

    let mut invitation = sample_event();
    invitation.id = "99".into();
    invitation.exclusivity_level = Some("Invitation Only".into());
    let html = render_card(invitation);
    assert_eq!(html.matches("data-icon=\"star\"").count(), 3);
    assert!(
        html.contains("data-icon=\"diamond\""),
        "Invitation Only must render diamond: {html}"
    );
}
