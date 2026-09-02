#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::events::Pagination;
use hesocial_frontend::pages::registrations::{EventRegisterScreen, MyRegistrationsScreen};
use hesocial_frontend::registrations::{
    PAGE_SIZE, RegisterEvent, RegisterUser, Registration, RegistrationPricing, default_filters,
};
use hesocial_frontend::shell::Presence;
use wasm_bindgen_test::wasm_bindgen_test;

fn sample_registration() -> Registration {
    Registration {
        id: "9".into(),
        user_id: "user-1".into(),
        event_id: "7".into(),
        status: "pending".into(),
        payment_status: "paid".into(),
        special_requests: Some("Vegetarian".into()),
        event_name: Some("松露季私宴".into()),
        event_date_time: Some("2026-12-01T18:00:00.000Z".into()),
        venue_name: Some("Taipei Private Dining Room".into()),
        created_at: "2026-08-31T00:00:00.000Z".into(),
        ..Registration::default()
    }
}

fn sample_event() -> RegisterEvent {
    RegisterEvent {
        id: "7".into(),
        name: "Autumn Yacht Social".into(),
        description: "Sunset cruise.".into(),
        date_time: "2026-10-10T09:00:00.000Z".into(),
        registration_deadline: "2026-10-05T23:59:59.000Z".into(),
        venue_name: "Keelung Luxury Yacht".into(),
        venue_address: "Pier 8".into(),
        category_name: "遊艇派對".into(),
        exclusivity_level: Some("VIP".into()),
        dress_code_label: "Casual".into(),
        capacity: 30,
        current_attendees: 1,
        pricing: RegistrationPricing {
            general: Some(18000.0),
            currency: "TWD".into(),
            ..RegistrationPricing::default()
        },
        images: vec!["https://media.example/yacht.webp".into()],
        amenities: vec!["parking".into()],
        privacy_guarantees: Vec::new(),
        requirements: Vec::new(),
    }
}

#[component]
fn ListAt(
    registrations: Vec<Registration>,
    loading: bool,
    error: Option<String>,
    total_pages: u32,
) -> Element {
    rsx! {
        MyRegistrationsScreen {
            registrations,
            loading,
            error,
            success_message: None,
            filters: default_filters(),
            pagination: Pagination {
                page: 1,
                limit: PAGE_SIZE,
                total: if registrations.is_empty() { 0 } else { 1 },
                total_pages,
            },
            edit_modal: Presence::Hidden,
            edit_registration: None,
            edit_requests: String::new(),
            action_loading: None,
            now_ms: 0.0,
        }
    }
}

fn render_list(
    registrations: Vec<Registration>,
    loading: bool,
    error: Option<String>,
    total_pages: u32,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ListAt,
        ListAtProps {
            registrations,
            loading,
            error,
            total_pages,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[component]
fn RegisterAt(
    loading: bool,
    error: Option<String>,
    event: Option<RegisterEvent>,
    user: Option<RegisterUser>,
) -> Element {
    rsx! {
        EventRegisterScreen {
            loading,
            error,
            event,
            user,
            special_requests: String::new(),
            registering: false,
        }
    }
}

fn render_register(
    loading: bool,
    error: Option<String>,
    event: Option<RegisterEvent>,
    user: Option<RegisterUser>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        RegisterAt,
        RegisterAtProps {
            loading,
            error,
            event,
            user,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn list_loading_state() {
    let html = render_list(vec![sample_registration()], true, None, 1);
    assert!(
        html.contains("讀取報名記錄中..."),
        "expected loading copy, got: {html}"
    );
    assert!(
        !html.contains("registration-card-9"),
        "cards must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn list_empty_state() {
    let html = render_list(vec![], false, None, 1);
    assert!(html.contains("尚無報名記錄"), "missing empty title: {html}");
    assert!(
        html.contains("您尚未報名任何活動，立即探索我們的精選活動吧！"),
        "missing empty hint: {html}"
    );
}

#[wasm_bindgen_test]
fn list_populated_and_error() {
    let html = render_list(
        vec![sample_registration()],
        false,
        Some("無法獲取活動報名記錄".into()),
        2,
    );
    for needle in [
        "我的活動報名",
        "松露季私宴",
        "Taipei Private Dining Room",
        "審核中",
        "已付款",
        "無法獲取活動報名記錄",
        "href=\"/events/7\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in list markup, got: {html}"
        );
    }
}

#[wasm_bindgen_test]
fn register_loading_and_not_found() {
    let loading = render_register(true, None, None, None);
    assert!(
        loading.contains("Loading event details..."),
        "expected loading copy, got: {loading}"
    );
    let missing = render_register(false, None, None, None);
    assert!(
        missing.contains("Event Not Found"),
        "expected not-found heading, got: {missing}"
    );
}

#[wasm_bindgen_test]
fn register_populated_error_and_copy() {
    let html = render_register(
        false,
        Some("Failed to register for event".into()),
        Some(sample_event()),
        Some(RegisterUser {
            first_name: "Wei".into(),
            last_name: "Chen".into(),
            email: "wei@example.com".into(),
            profession: "投資人".into(),
            membership_tier: "Diamond".into(),
        }),
    );
    for needle in [
        "Event Registration",
        "Autumn Yacht Social",
        "Keelung Luxury Yacht",
        "Submit Registration",
        "Failed to register for event",
        "Wei Chen",
        "NT$18,000",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in register markup, got: {html}"
        );
    }
}
