#![cfg(not(target_arch = "wasm32"))]

use dioxus::prelude::*;
use hesocial_frontend::events::Pagination;
use hesocial_frontend::pages::registrations::{EventRegisterScreen, MyRegistrationsScreen};
use hesocial_frontend::permissions::{
    AuthSnapshot, AuthUser, MembershipTier, Role, RouteGuard, USER_ROUTE_FALLBACK, user_route_guard,
};
use hesocial_frontend::registrations::{
    EVENT_WHEN_FALLBACK, FETCH_EVENT_ERROR, FETCH_REGISTRATIONS_ERROR, PAGE_SIZE, PRICE_ON_REQUEST,
    REGISTER_SUCCESS_NAV, RegisterEvent, RegisterUser, Registration, RegistrationFilters,
    RegistrationPricing, VENUE_FALLBACK, can_cancel, can_edit, collapse_on_error, default_filters,
    event_title, event_when_label, format_event_price, format_list_datetime,
    membership_tier_badge_class, membership_tier_label, page_after_filter_change, page_in_range,
    pagination_range, parse_create_registration_response, parse_iso_ms, parse_mutation_response,
    parse_register_event_response, parse_register_user_from_auth, parse_register_user_from_profile,
    parse_registration_detail_response, parse_user_registrations_response, payment_class,
    payment_label, register_dress_code_text, register_exclusivity_class,
    registrations_query_string, status_class, status_label, success_message_from_query,
    venue_label,
};
use hesocial_frontend::shell::Presence;

fn sample_registration() -> Registration {
    Registration {
        id: "9".into(),
        user_id: "user-1".into(),
        event_id: "7".into(),
        status: "pending".into(),
        payment_status: "pending".into(),
        payment_intent_id: None,
        special_requests: Some("Vegetarian".into()),
        event_name: Some("松露季私宴".into()),
        event_description: Some("白松露當季。".into()),
        event_date_time: Some("2026-12-01T18:00:00.000Z".into()),
        registration_deadline: Some("2026-11-20T00:00:00.000Z".into()),
        venue_name: Some("Taipei Private Dining Room".into()),
        venue_address: Some("Da'an".into()),
        created_at: "2026-08-31T00:00:00.000Z".into(),
        updated_at: "2026-08-31T00:00:00.000Z".into(),
        exclusivity_level: Some("VIP".into()),
        dress_code_label: "Black Tie".into(),
        capacity: 20,
        current_attendees: 1,
        pricing: RegistrationPricing {
            vip: Some(15000.0),
            vvip: Some(18000.0),
            general: None,
            currency: "TWD".into(),
        },
        amenities: vec!["valet".into()],
        privacy_guarantees: vec!["NDA".into()],
        requirements: vec!["Verified membership".into()],
        event_images: vec!["https://media.example/e7.webp".into()],
        category_name: Some("私人晚宴".into()),
    }
}

fn sample_event() -> RegisterEvent {
    RegisterEvent {
        id: "7".into(),
        name: "Autumn Yacht Social".into(),
        description: "Sunset cruise around Keelung Harbor.".into(),
        date_time: "2026-10-10T09:00:00.000Z".into(),
        registration_deadline: "2026-10-05T23:59:59.000Z".into(),
        venue_name: "Keelung Luxury Yacht".into(),
        venue_address: "Keelung Harbor Pier 8".into(),
        category_name: "遊艇派對".into(),
        exclusivity_level: Some("VVIP".into()),
        dress_code_label: "Resort Casual".into(),
        capacity: 30,
        current_attendees: 1,
        pricing: RegistrationPricing {
            vip: Some(18000.0),
            vvip: Some(18000.0),
            general: Some(18000.0),
            currency: "TWD".into(),
        },
        images: vec!["https://media.example/yacht.webp".into()],
        amenities: vec!["parking".into()],
        privacy_guarantees: vec!["Private guest list".into()],
        requirements: vec!["Diamond membership".into()],
    }
}

fn sample_user() -> RegisterUser {
    RegisterUser {
        first_name: "Wei".into(),
        last_name: "Chen".into(),
        email: "wei@example.com".into(),
        profession: "投資人".into(),
        membership_tier: "Diamond".into(),
    }
}

fn list_pagination() -> Pagination {
    Pagination {
        page: 1,
        limit: PAGE_SIZE,
        total: 1,
        total_pages: 1,
    }
}

#[component]
fn ListScreen(
    registrations: Vec<Registration>,
    loading: bool,
    error: Option<String>,
    success: Option<String>,
    page: u32,
    total_pages: u32,
    total: u32,
    edit_modal: Presence,
) -> Element {
    rsx! {
        MyRegistrationsScreen {
            registrations,
            loading,
            error,
            success_message: success,
            filters: default_filters(),
            pagination: Pagination {
                page,
                limit: PAGE_SIZE,
                total,
                total_pages,
            },
            edit_modal,
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
    success: Option<String>,
    page: u32,
    total_pages: u32,
    total: u32,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ListScreen,
        ListScreenProps {
            registrations,
            loading,
            error,
            success,
            page,
            total_pages,
            total,
            edit_modal: Presence::Hidden,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[component]
fn RegisterScreenAt(
    loading: bool,
    error: Option<String>,
    event: Option<RegisterEvent>,
    user: Option<RegisterUser>,
    registering: bool,
) -> Element {
    rsx! {
        EventRegisterScreen {
            loading,
            error,
            event,
            user,
            special_requests: String::new(),
            registering,
        }
    }
}

fn render_register(
    loading: bool,
    error: Option<String>,
    event: Option<RegisterEvent>,
    user: Option<RegisterUser>,
    registering: bool,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        RegisterScreenAt,
        RegisterScreenAtProps {
            loading,
            error,
            event,
            user,
            registering,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[test]
fn default_query_sends_page_and_limit() {
    assert_eq!(
        registrations_query_string(&default_filters()),
        "page=1&limit=10"
    );
}

#[test]
fn query_appends_status_payment_and_search() {
    let filters = RegistrationFilters {
        page: 2,
        limit: 10,
        status: "pending".into(),
        payment_status: "paid".into(),
        search: "松露 私宴".into(),
    };
    assert_eq!(
        registrations_query_string(&filters),
        "page=2&limit=10&status=pending&paymentStatus=paid&search=%E6%9D%BE%E9%9C%B2+%E7%A7%81%E5%AE%B4"
    );
}

#[test]
fn changing_filter_resets_to_page_one() {
    assert_eq!(page_after_filter_change(4), 1);
    assert_eq!(page_after_filter_change(1), 1);
}

#[test]
fn pagination_range_matches_react() {
    assert!(page_in_range(1, 3));
    assert!(!page_in_range(0, 3));
    assert!(!page_in_range(4, 3));
    assert_eq!(pagination_range(1, 10, 23), (1, 10));
    assert_eq!(pagination_range(3, 10, 23), (21, 23));
}

#[test]
fn parse_user_registrations_success_stringifies_ids() {
    let body = r#"{
        "success": true,
        "data": [{
            "id": 9,
            "userId": "user-1",
            "eventId": 7,
            "status": "pending",
            "paymentStatus": "pending",
            "paymentIntentId": null,
            "specialRequests": "Vegetarian",
            "createdAt": "2026-08-31T00:00:00.000Z",
            "updatedAt": "2026-08-31T00:00:00.000Z",
            "eventName": "Gala",
            "eventDescription": null,
            "eventDateTime": "2026-12-01T18:00:00.000Z",
            "registrationDeadline": null,
            "exclusivityLevel": null,
            "dressCode": "Black Tie",
            "capacity": 20,
            "currentAttendees": 1,
            "pricing": {"vip": 100, "vvip": 90, "general": 80, "currency": "TWD"},
            "amenities": null,
            "privacyGuarantees": null,
            "requirements": [],
            "eventImages": null,
            "venueName": "Room",
            "venueAddress": "Taipei",
            "categoryName": "Dinner"
        }],
        "pagination": {"page": 1, "limit": 10, "total": 1, "totalPages": 1}
    }"#;
    let view = parse_user_registrations_response(200, body, 1, 10).expect("ok");
    assert_eq!(view.registrations.len(), 1);
    assert_eq!(view.registrations[0].id, "9");
    assert_eq!(view.registrations[0].event_id, "7");
    assert_eq!(view.registrations[0].event_name.as_deref(), Some("Gala"));
    assert_eq!(view.registrations[0].dress_code_label, "Black Tie");
    assert_eq!(view.registrations[0].pricing.general, Some(80.0));
    assert_eq!(view.pagination.total, 1);
}

#[test]
fn parse_user_registrations_empty_success() {
    let body =
        r#"{"success":true,"data":[],"pagination":{"page":1,"limit":10,"total":0,"totalPages":0}}"#;
    let view = parse_user_registrations_response(200, body, 1, 10).expect("empty ok");
    assert!(view.registrations.is_empty());
    assert_eq!(view.pagination.total, 0);
    assert_eq!(view.pagination.total_pages, 0);
}

#[test]
fn parse_user_registrations_success_false_uses_api_error() {
    let err = parse_user_registrations_response(200, r#"{"success":false,"error":"nope"}"#, 2, 10)
        .expect_err("false");
    assert_eq!(err, "nope");
}

#[test]
fn parse_user_registrations_http_error_and_malformed() {
    let missing =
        parse_user_registrations_response(500, r#"{"success":false}"#, 1, 10).expect_err("500");
    assert_eq!(missing, FETCH_REGISTRATIONS_ERROR);
    let malformed = parse_user_registrations_response(200, "not-json", 3, 10).expect_err("bad");
    assert_eq!(malformed, FETCH_REGISTRATIONS_ERROR);
    let collapsed = collapse_on_error(3, 10);
    assert_eq!(collapsed.page, 3);
    assert_eq!(collapsed.total, 0);
}

#[test]
fn parse_registration_detail_and_create() {
    let detail = parse_registration_detail_response(
        200,
        r#"{"success":true,"data":{"id":9,"userId":"u","eventId":7,"status":"pending","paymentStatus":"paid","createdAt":"t","updatedAt":"t","eventName":"Gala"}}"#,
    )
    .expect("detail");
    assert_eq!(detail.id, "9");
    assert_eq!(detail.payment_status, "paid");

    let created = parse_create_registration_response(
        201,
        r#"{"success":true,"data":{"registrationId":12,"status":"pending","message":"Registration submitted successfully. Pending approval."}}"#,
    )
    .expect("created");
    assert_eq!(created.registration_id, "12");
    assert_eq!(created.status, "pending");

    let rejected = parse_create_registration_response(
        400,
        r#"{"success":false,"error":"You are already registered for this event"}"#,
    )
    .expect_err("dup");
    assert_eq!(rejected, "You are already registered for this event");
}

#[test]
fn parse_mutation_and_register_event() {
    let ok = parse_mutation_response(
        200,
        r#"{"success":true,"message":"Registration cancelled successfully"}"#,
        "取消活動報名失敗",
    )
    .expect("cancel");
    assert_eq!(ok, "Registration cancelled successfully");
    let err = parse_mutation_response(
        400,
        r#"{"success":false,"error":"Cannot cancel registration within 24 hours of event"}"#,
        "取消活動報名失敗",
    )
    .expect_err("24h");
    assert_eq!(err, "Cannot cancel registration within 24 hours of event");

    let event = parse_register_event_response(
        200,
        r#"{"success":true,"data":{"id":2,"name":"Autumn Yacht Social","description":"Sunset.","dateTime":"2026-10-10T09:00:00.000Z","registrationDeadline":"2026-10-05T23:59:59.000Z","pricing":{"vip":18000,"vvip":18000,"general":18000,"currency":"TWD"},"exclusivityLevel":null,"dressCode":"Resort Casual","capacity":30,"currentAttendees":1,"amenities":null,"privacyGuarantees":null,"images":null,"requirements":[{"description":"ID check"}],"venue":{"name":"Keelung Luxury Yacht","address":"Pier 8"},"category":{"name":"遊艇派對"}}}"#,
    )
    .expect("event");
    assert_eq!(event.name, "Autumn Yacht Social");
    assert_eq!(event.venue_name, "Keelung Luxury Yacht");
    assert_eq!(event.category_name, "遊艇派對");
    assert_eq!(event.dress_code_label, "Resort Casual");
    assert_eq!(event.requirements, vec!["ID check".to_string()]);

    let missing =
        parse_register_event_response(404, r#"{"success":false,"error":"Event not found"}"#)
            .expect_err("404");
    assert_eq!(missing, "Event not found");
    let malformed = parse_register_event_response(200, "nope").expect_err("bad");
    assert_eq!(malformed, FETCH_EVENT_ERROR);
}

#[test]
fn status_and_payment_labels_match_react() {
    assert_eq!(status_label("pending"), "審核中");
    assert_eq!(status_label("approved"), "已核准");
    assert_eq!(status_label("confirmed"), "已核准");
    assert_eq!(status_label("rejected"), "已婉拒");
    assert_eq!(status_label("cancelled"), "已取消");
    assert_eq!(status_label("waitlisted"), "候補中");
    assert_eq!(status_label("mystery"), "mystery");
    assert!(status_class("pending").contains("text-yellow-300"));
    assert!(status_class("approved").contains("text-green-300"));
    assert_eq!(payment_label("pending"), "待付款");
    assert_eq!(payment_label("paid"), "已付款");
    assert_eq!(payment_label("refunded"), "已退款");
    assert!(payment_class("paid").contains("text-green-300"));
}

#[test]
fn fallbacks_and_native_date_formatting() {
    assert_eq!(event_title(None), "活動");
    assert_eq!(event_title(Some("")), "活動");
    assert_eq!(event_title(Some("Gala")), "Gala");
    assert_eq!(venue_label(None), VENUE_FALLBACK);
    assert_eq!(event_when_label(None), EVENT_WHEN_FALLBACK);
    assert_eq!(
        format_list_datetime("2026-12-01T18:00:00.000Z"),
        "2026-12-01T18:00:00.000Z"
    );
}

#[test]
fn price_and_dress_and_exclusivity() {
    assert_eq!(
        format_event_price(&RegistrationPricing {
            general: Some(18000.0),
            currency: "TWD".into(),
            ..RegistrationPricing::default()
        }),
        "NT$18,000"
    );
    assert_eq!(
        format_event_price(&RegistrationPricing {
            vip: Some(10000.0),
            vvip: Some(20000.0),
            currency: "TWD".into(),
            ..RegistrationPricing::default()
        }),
        "NT$10,000 - NT$20,000"
    );
    assert_eq!(
        format_event_price(&RegistrationPricing {
            vip: Some(8000.0),
            currency: "TWD".into(),
            ..RegistrationPricing::default()
        }),
        "NT$8,000"
    );
    assert_eq!(
        format_event_price(&RegistrationPricing {
            vvip: Some(9000.0),
            currency: "TWD".into(),
            ..RegistrationPricing::default()
        }),
        PRICE_ON_REQUEST
    );
    assert_eq!(
        format_event_price(&RegistrationPricing::default()),
        PRICE_ON_REQUEST
    );
    assert_eq!(register_dress_code_text(1), "Casual");
    assert_eq!(register_dress_code_text(5), "Black Tie");
    assert_eq!(register_dress_code_text(0), "Not specified");
    assert!(register_exclusivity_class(Some("VIP")).contains("purple"));
    assert!(register_exclusivity_class(Some("Invitation Only")).contains("red"));
}

#[test]
fn can_edit_and_can_cancel_use_status_and_event_time() {
    let now = parse_iso_ms("2026-11-01T00:00:00.000Z").expect("now");
    let future = "2026-12-01T18:00:00.000Z";
    let soon = {
        let ms = now + 10.0 * 3_600_000.0;
        let _ = ms;
        "2026-11-01T10:00:00.000Z"
    };
    assert!(can_edit("pending", Some(future), now));
    assert!(!can_edit("approved", Some(future), now));
    assert!(!can_edit("pending", None, now));
    assert!(can_cancel("pending", Some(future), now));
    assert!(!can_cancel("cancelled", Some(future), now));
    assert!(!can_cancel("rejected", Some(future), now));
    assert!(!can_cancel("pending", Some(soon), now));
    assert!(!can_cancel("pending", None, now));
}

#[test]
fn registered_query_surfaces_success_copy() {
    assert_eq!(
        success_message_from_query("?registered=1"),
        Some(REGISTER_SUCCESS_NAV.to_string())
    );
    assert_eq!(success_message_from_query(""), None);
    assert_eq!(success_message_from_query("page=2"), None);
}

#[test]
fn user_route_guard_three_states() {
    assert_eq!(USER_ROUTE_FALLBACK, "/login");
    assert_eq!(
        user_route_guard(false, &AuthSnapshot::default()),
        RouteGuard::Redirect("/login")
    );
    assert_eq!(
        user_route_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading
    );
    let signed_in = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        ..AuthSnapshot::default()
    };
    assert_eq!(user_route_guard(false, &signed_in), RouteGuard::Allow);
}

#[test]
fn register_user_from_auth_and_profile_json() {
    let auth = AuthUser {
        email: Some("a@b.c".into()),
        membership_tier: Some(MembershipTier::Diamond),
        ..AuthUser::default()
    };
    let user = parse_register_user_from_auth(&auth);
    assert_eq!(user.email, "a@b.c");
    assert_eq!(user.membership_tier, "Diamond");
    assert_eq!(
        membership_tier_label(Some(MembershipTier::BlackCard)),
        Some("Black Card")
    );
    assert!(membership_tier_badge_class("Diamond").contains("blue"));
    let parsed = parse_register_user_from_profile(&serde_json::json!({
        "firstName": "Wei",
        "lastName": "Chen",
        "email": "wei@example.com",
        "profession": "投資人",
        "membershipTier": "Platinum"
    }));
    assert_eq!(parsed.first_name, "Wei");
    assert_eq!(parsed.profession, "投資人");
}

#[test]
fn list_ssr_loading_hides_empty_and_cards() {
    let html = render_list(vec![sample_registration()], true, None, None, 1, 1, 1);
    assert!(html.contains("讀取報名記錄中..."), "loading copy: {html}");
    assert!(html.contains("id=\"my-registrations-loading\""), "{html}");
    assert!(
        !html.contains("registration-card-9"),
        "cards hidden while loading: {html}"
    );
    assert!(
        !html.contains("尚無報名記錄"),
        "empty hidden while loading: {html}"
    );
}

#[test]
fn list_ssr_empty_copy() {
    let html = render_list(vec![], false, None, None, 1, 1, 0);
    assert!(html.contains("尚無報名記錄"), "{html}");
    assert!(
        html.contains("您尚未報名任何活動，立即探索我們的精選活動吧！"),
        "{html}"
    );
    assert!(html.contains("探索活動"), "{html}");
    assert!(html.contains("href=\"/events\""), "{html}");
    assert!(!html.contains("讀取報名記錄中..."), "{html}");
}

#[test]
fn list_ssr_populated_and_error_and_success() {
    let html = render_list(
        vec![sample_registration()],
        false,
        Some("無法獲取活動報名記錄".into()),
        Some("活動報名已成功取消".into()),
        1,
        3,
        23,
    );
    for needle in [
        "我的活動報名",
        "管理您的活動報名與申請狀態",
        "松露季私宴",
        "Taipei Private Dining Room",
        "特別要求:",
        "Vegetarian",
        "審核中",
        "待付款",
        "無法獲取活動報名記錄",
        "活動報名已成功取消",
        "href=\"/events/7\"",
        "registration-card-9",
        "顯示第 1 至 10 項，共 23 項結果",
        "第 1 / 3 頁",
        "篩選器",
        "報名總覽",
    ] {
        assert!(html.contains(needle), "missing {needle:?} in {html}");
    }
}

#[test]
fn register_ssr_loading_empty_populated_error() {
    let loading = render_register(true, None, None, None, false);
    assert!(loading.contains("Loading event details..."), "{loading}");
    assert!(
        loading.contains("id=\"event-register-loading\""),
        "{loading}"
    );

    let missing = render_register(false, None, None, None, false);
    assert!(missing.contains("Event Not Found"), "{missing}");
    assert!(
        missing.contains("looking for") && missing.contains("no longer available."),
        "{missing}"
    );
    assert!(missing.contains("Back to Events"), "{missing}");

    let html = render_register(
        false,
        Some("Failed to register for event".into()),
        Some(sample_event()),
        Some(sample_user()),
        true,
    );
    for needle in [
        "Event Registration",
        "Complete your registration for this exclusive event",
        "Autumn Yacht Social",
        "Sunset cruise around Keelung Harbor.",
        "Keelung Luxury Yacht",
        "Keelung Harbor Pier 8",
        "遊艇派對",
        "NT$18,000",
        "Per person",
        "Date &#38; Time",
        "Dress Code",
        "Resort Casual",
        "Registration Deadline",
        "Requirements",
        "Diamond membership",
        "Amenities",
        "parking",
        "Privacy &#38; Security",
        "Private guest list",
        "Your Information",
        "Wei Chen",
        "wei@example.com",
        "投資人",
        "Diamond",
        "Special Requests (Optional)",
        "Submit registration request",
        "Submitting...",
        "Failed to register for event",
        "https://media.example/yacht.webp",
    ] {
        assert!(html.contains(needle), "missing {needle:?} in {html}");
    }
    assert!(
        !html.contains("Submit Registration") || html.contains("Submitting..."),
        "submit label while in flight: {html}"
    );
}

#[test]
fn list_pagination_hidden_on_single_page() {
    let html = render_list(vec![sample_registration()], false, None, None, 1, 1, 1);
    assert!(!html.contains("上一頁"), "pagination hidden: {html}");
    let _ = list_pagination();
}
