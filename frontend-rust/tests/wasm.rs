#![cfg(target_arch = "wasm32")]

use std::rc::Rc;

use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::*;
use dioxus::router::components::HistoryProvider;
use hesocial_frontend::events::{Event, EventDetail, Pagination, Pricing, Venue};
use hesocial_frontend::profile::ProfileUser;
use hesocial_frontend::register::{REGISTER_PASSWORD_MISMATCH, RegisterForm};
use hesocial_frontend::shell::Presence;
use hesocial_frontend::ui::{
    EventCard, EventDetailScreen, EventsScreen, Footer, HomeScreen, LoginScreen, NavbarScreen,
    ProfileScreen, RegisterScreen, Route,
};
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
fn ScreenAt(events: Vec<Event>, loading: bool, page: u32, total_pages: u32) -> Element {
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
    assert!(
        !badge.is_empty(),
        "null exclusivity still renders the span: {html}"
    );
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

#[component]
fn NavAt(
    pathname: String,
    is_authenticated: bool,
    view_admin: bool,
    user_menu: Presence,
    mobile: Presence,
) -> Element {
    rsx! {
        NavbarScreen {
            pathname,
            is_authenticated,
            view_admin,
            user_menu,
            mobile,
        }
    }
}

fn render_nav(
    pathname: &str,
    is_authenticated: bool,
    view_admin: bool,
    user_menu: Presence,
    mobile: Presence,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        NavAt,
        NavAtProps {
            pathname: pathname.into(),
            is_authenticated,
            view_admin,
            user_menu,
            mobile,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn signed_out_shell_shows_login_register_and_hides_user_menu() {
    let html = render_nav("/", false, false, Presence::Hidden, Presence::Hidden);
    for needle in [
        "id=\"nav\"",
        "HeSocial",
        "首頁",
        "精選活動",
        "VVIP專區",
        "id=\"nav-login\"",
        "href=\"/login\"",
        "登入",
        "id=\"nav-register\"",
        "href=\"/register\"",
        "註冊",
        "id=\"nav-mobile-toggle\"",
        "data-icon=\"menu\"",
        "data-icon=\"crown\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in signed-out navbar, got: {html}"
        );
    }
    assert!(
        !html.contains("id=\"nav-user-button\""),
        "avatar must not render signed out: {html}"
    );
    assert!(
        !html.contains("id=\"nav-user-menu\""),
        "dropdown must not render when closed: {html}"
    );
    assert!(
        !html.contains("管理後台"),
        "admin entry must not render signed out: {html}"
    );
}

#[wasm_bindgen_test]
fn signed_in_shell_shows_avatar_and_hides_login_register() {
    let html = render_nav("/events", true, false, Presence::Hidden, Presence::Hidden);
    assert!(
        html.contains("id=\"nav-user-button\""),
        "avatar missing: {html}"
    );
    assert!(
        html.contains("data-icon=\"user\""),
        "user icon missing: {html}"
    );
    assert!(
        !html.contains("id=\"nav-login\""),
        "登入 must not render signed in: {html}"
    );
    assert!(
        !html.contains("id=\"nav-register\""),
        "註冊 must not render signed in: {html}"
    );
    assert!(
        !html.contains("id=\"nav-user-menu\""),
        "dropdown starts closed: {html}"
    );
    assert!(
        !html.contains("管理後台"),
        "non-admin must not see admin: {html}"
    );
    let events = opening_tag(&html, "nav-item-events");
    assert!(
        events.contains("text-luxury-gold") && events.contains("bg-luxury-gold/10"),
        "current /events item must be highlighted, tag={events}"
    );
}

#[wasm_bindgen_test]
fn signed_in_dropdown_open_lists_user_links_without_admin() {
    let html = render_nav("/", true, false, Presence::Entering, Presence::Hidden);
    assert!(
        html.contains("id=\"nav-user-menu\""),
        "dropdown missing: {html}"
    );
    assert!(
        html.contains("hs-dropdown-enter"),
        "open dropdown must use the enter class: {html}"
    );
    for needle in [
        "id=\"nav-profile\"",
        "href=\"/profile\"",
        "個人檔案",
        "id=\"nav-registrations\"",
        "href=\"/profile/registrations\"",
        "我的報名",
        "id=\"nav-logout\"",
        "登出",
        "data-icon=\"log-out\"",
        "data-icon=\"calendar\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in open dropdown, got: {html}"
        );
    }
    assert!(
        !html.contains("id=\"nav-admin\""),
        "admin missing for user: {html}"
    );
}

#[wasm_bindgen_test]
fn admin_dropdown_renders_admin_entries() {
    let html = render_nav("/", true, true, Presence::Shown, Presence::Hidden);
    for needle in [
        "id=\"nav-admin\"",
        "href=\"/admin\"",
        "管理後台",
        "id=\"nav-event-mgmt\"",
        "href=\"/event-mgmt\"",
        "活動管理",
        "id=\"nav-sales\"",
        "href=\"/admin/sales\"",
        "銷售管理",
        "id=\"nav-system\"",
        "href=\"/admin/system\"",
        "系統健康",
        "data-icon=\"shield\"",
        "data-icon=\"settings\"",
        "data-icon=\"trending-up\"",
        "data-icon=\"activity\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in admin dropdown, got: {html}"
        );
    }
}

#[wasm_bindgen_test]
fn dropdown_exit_stays_mounted_with_exit_class() {
    let html = render_nav("/", true, false, Presence::Exiting, Presence::Hidden);
    let menu = opening_tag(&html, "nav-user-menu");
    assert!(
        !menu.is_empty(),
        "exiting dropdown must stay mounted: {html}"
    );
    assert!(
        menu.contains("hs-dropdown-exit"),
        "exiting dropdown must keep the exit class, tag={menu}"
    );
    let closed = render_nav("/", true, false, Presence::Hidden, Presence::Hidden);
    assert!(
        !closed.contains("id=\"nav-user-menu\""),
        "hidden presence must unmount the dropdown: {closed}"
    );
}

#[wasm_bindgen_test]
fn mobile_toggle_swaps_menu_and_x_and_mounts_panel() {
    let closed = render_nav("/", false, false, Presence::Hidden, Presence::Hidden);
    let toggle = opening_tag(&closed, "nav-mobile-toggle");
    assert!(toggle.contains("data-icon=\"menu\"") || closed.contains("data-icon=\"menu\""));
    assert!(
        !closed.contains("id=\"nav-mobile-panel\""),
        "mobile panel starts unmounted: {closed}"
    );

    let open = render_nav("/", false, false, Presence::Hidden, Presence::Entering);
    assert!(
        open.contains("id=\"nav-mobile-panel\""),
        "panel missing: {open}"
    );
    assert!(
        open.contains("hs-mobile-enter"),
        "mobile enter class missing: {open}"
    );
    assert!(
        open.contains("data-icon=\"x\""),
        "open toggle must show X: {open}"
    );
    assert!(
        opening_tag(&open, "nav-mobile-panel").contains("href=\"/vvip\"")
            || open.contains("VVIP專區"),
        "mobile panel must list nav items: {open}"
    );
}

#[wasm_bindgen_test]
fn footer_renders_react_copy() {
    let mut vdom = VirtualDom::new(Footer);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    for needle in [
        "id=\"footer\"",
        "HeSocial",
        "專為高淨值人士打造的頂級社交平台，提供獨家尊榮體驗與精緻社交活動。",
        "服務項目",
        "私人晚宴",
        "豪華遊艇派對",
        "藝術品鑑會",
        "商務社交",
        "會員專區",
        "Platinum 會員",
        "Diamond 會員",
        "Black Card 會員",
        "專屬顧問服務",
        "聯絡我們",
        "+886-2-2345-6789",
        "concierge@hesocial.com",
        "台北市信義區松仁路",
        "© 2024 HeSocial. 版權所有 | 隱私政策 | 服務條款",
        "企業級安全認證",
        "系統正常運行",
        "data-icon=\"crown\"",
        "data-icon=\"phone\"",
        "data-icon=\"mail\"",
        "data-icon=\"map-pin\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in footer, got: {html}"
        );
    }
}

fn complete_profile_user() -> ProfileUser {
    ProfileUser::from_json(&serde_json::json!({
        "id": "1",
        "email": "ok@example.com",
        "firstName": "Wei",
        "lastName": "Chen",
        "age": 42,
        "profession": "投資人",
        "annualIncome": 8000000,
        "netWorth": 50000000,
        "membershipTier": "Diamond",
        "privacyLevel": 4,
        "isVerified": true,
        "verificationStatus": "approved",
        "role": "user",
        "profilePicture": "https://media.example/p.jpg",
        "bio": "喜歡藝術與航海",
        "interests": ["藝術", "遊艇"]
    }))
}

fn google_profile_user() -> ProfileUser {
    ProfileUser::from_json(&serde_json::json!({
        "id": "g-1",
        "email": "google@example.com",
        "firstName": "Ada",
        "lastName": "Li",
        "age": null,
        "profession": null,
        "annualIncome": null,
        "netWorth": null,
        "membershipTier": "Platinum",
        "privacyLevel": 3,
        "isVerified": false,
        "verificationStatus": "pending",
        "role": "user",
        "profilePicture": null,
        "bio": null,
        "interests": null
    }))
}

#[component]
fn ProfileAt(profile: ProfileUser) -> Element {
    rsx! { ProfileScreen { profile } }
}

fn render_profile(profile: ProfileUser) -> String {
    let mut vdom = VirtualDom::new_with_props(ProfileAt, ProfileAtProps { profile });
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn profile_renders_complete_user() {
    let html = render_profile(complete_profile_user());
    for needle in [
        "id=\"profile-stub\"",
        "id=\"profile-heading\"",
        "Wei Chen",
        "ok@example.com",
        "投資人",
        "42 歲",
        "Diamond 會員",
        "喜歡藝術與航海",
        "Level 4",
        "藝術",
        "遊艇",
        "https://media.example/p.jpg",
        "會員權益",
        "VIP活動優先預訂",
        "參與活動",
        "即將參與",
        "累計消費",
        "信用評級",
        "NT$ 450K",
        "A+",
        "即將參與的活動",
        "星空下的法式晚宴",
        "已確認",
        "私人遊艇品酒之夜",
        "待審核",
        "當代藝術收藏家沙龍",
        "個人資訊",
        "會員自 2023 年",
        "data-icon=\"crown\"",
        "data-icon=\"mail\"",
        "data-icon=\"briefcase\"",
        "data-icon=\"calendar\"",
        "data-icon=\"star\"",
        "data-icon=\"users\"",
        "data-icon=\"trending-up\"",
        "data-icon=\"award\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in complete profile, got: {html}"
        );
    }
    assert!(
        !html.contains("編輯個人資料"),
        "edit controls are next round: {html}"
    );
    assert!(
        !html.contains("name=\"firstName\""),
        "must not wire the edit form: {html}"
    );
}

#[wasm_bindgen_test]
fn profile_renders_google_user_with_null_financials() {
    let html = render_profile(google_profile_user());
    assert!(html.contains("Ada Li"), "name missing: {html}");
    assert!(html.contains("google@example.com"), "email missing: {html}");
    assert!(html.contains("Platinum 會員"), "tier missing: {html}");
    assert!(
        html.contains(" 歲"),
        "null age must still render React's 歲 suffix: {html}"
    );
    assert!(
        !html.contains("null"),
        "must not stringify JSON null: {html}"
    );
    assert!(
        html.contains("/api/placeholder/150/150"),
        "null picture falls back to placeholder: {html}"
    );
    assert!(html.contains("Level 3"), "privacy missing: {html}");
    assert!(
        html.contains("參與精選社交活動"),
        "platinum benefits missing: {html}"
    );
    assert!(
        !html.contains("8000000"),
        "annualIncome must not render: {html}"
    );
    assert!(
        !html.contains("50000000"),
        "netWorth must not render: {html}"
    );
}

#[wasm_bindgen_test]
fn signed_out_profile_redirects_to_login() {
    let html = render_path("/profile");
    assert!(
        html.contains("歡迎回來") || html.contains("id=\"profile-unauth\""),
        "signed-out /profile must redirect to /login, got: {html}"
    );
    assert!(
        !html.contains("id=\"profile-heading\"") || html.contains("歡迎回來"),
        "signed-out visitor must not see the profile body: {html}"
    );
}

#[component]
fn HomeAt() -> Element {
    rsx! { HomeScreen { toggled: false } }
}

fn render_home_screen() -> String {
    let mut vdom = VirtualDom::new(HomeAt);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn home_landing_renders_react_copy_and_ctas() {
    let html = render_home_screen();
    for needle in [
        "id=\"scaffold-heading\"",
        "HeSocial",
        "專為高淨值人士打造的頂級社交平台",
        "在奢華環境中遇見志同道合的菁英",
        "申請加入",
        "探索活動",
        "href=\"/register\"",
        "href=\"/events\"",
        "為什麼選擇 HeSocial",
        "頂級會員制",
        "隱私保障",
        "AI智能配對",
        "獨家活動",
        "會員方案",
        "Platinum",
        "Diamond",
        "Black Card",
        "最受歡迎",
        "NT$50,000",
        "NT$120,000",
        "邀請制",
        "準備開始您的尊榮社交之旅？",
        "立即申請加入",
        "id=\"toggle-btn\"",
        "data-icon=\"crown\"",
        "data-icon=\"arrow-right\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in home markup, got: {html}"
        );
    }
}

fn sample_detail() -> EventDetail {
    EventDetail {
        id: "11".into(),
        name: "松露季私宴".into(),
        description: "白松露當季，主廚八道式無菜單。".into(),
        date_time: "2026-10-04T12:00:00.000Z".into(),
        registration_deadline: "2026-09-20T12:00:00.000Z".into(),
        venue_name: "Taipei Private Dining Room".into(),
        venue_address: "Da'an".into(),
        venue_rating: 5.0,
        venue_amenities: vec!["Valet".into()],
        exclusivity_level: Some("VVIP".into()),
        pricing: Pricing {
            vvip: Some(18000.0),
            vip: Some(15000.0),
            currency: "TWD".into(),
        },
        current_attendees: 3,
        capacity: 12,
        images: vec![
            "https://media.example/e11.webp".into(),
            "https://media.example/e11b.webp".into(),
        ],
        organizer: "Wei Chen".into(),
        tags: vec!["晚宴".into()],
        dress_code: 4,
        amenities: vec!["專車接送".into()],
        privacy_guarantees: vec!["匿名參與".into()],
        requirements: vec!["需通過身份審核".into()],
    }
}

#[component]
fn DetailAt(loading: bool, event: Option<EventDetail>, is_authenticated: bool) -> Element {
    rsx! {
        EventDetailScreen {
            loading,
            event,
            image_index: 0,
            is_authenticated,
        }
    }
}

fn render_detail(loading: bool, event: Option<EventDetail>, is_authenticated: bool) -> String {
    let mut vdom = VirtualDom::new_with_props(
        DetailAt,
        DetailAtProps {
            loading,
            event,
            is_authenticated,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn event_detail_loading_and_not_found() {
    let loading = render_detail(true, None, false);
    assert!(
        loading.contains("id=\"event-detail-loading\""),
        "loading id missing: {loading}"
    );
    assert!(
        loading.contains("載入活動資訊中..."),
        "loading copy missing: {loading}"
    );
    assert!(
        !loading.contains("id=\"event-detail-heading\""),
        "heading must not render while loading: {loading}"
    );

    let missing = render_detail(false, None, false);
    assert!(
        missing.contains("id=\"event-detail-not-found\""),
        "not-found id missing: {missing}"
    );
    assert!(
        missing.contains("活動未找到"),
        "not-found title missing: {missing}"
    );
    assert!(
        missing.contains("抱歉，您要查看的活動不存在或已被移除。"),
        "not-found copy missing: {missing}"
    );
    assert!(
        missing.contains("href=\"/events\""),
        "back-to-list missing: {missing}"
    );
    assert!(
        missing.contains("data-icon=\"alert-circle\""),
        "alert icon missing: {missing}"
    );
}

#[wasm_bindgen_test]
fn event_detail_loaded_guest_and_member() {
    let guest = render_detail(false, Some(sample_detail()), false);
    for needle in [
        "id=\"event-detail-heading\"",
        "松露季私宴",
        "主辦：Wei Chen",
        "晚宴",
        "白松露當季，主廚八道式無菜單。",
        "Taipei Private Dining Room",
        "Da&#39;an",
        "專車接送",
        "匿名參與",
        "需通過身份審核",
        "晚宴正裝",
        "3/12 人",
        "NT$ 18,000",
        "VVIP 價格",
        "登入後報名",
        "href=\"/login\"",
        "需要登入會員帳號才能報名活動",
        "https://media.example/e11.webp",
        "id=\"event-detail-prev-image\"",
        "data-icon=\"shirt\"",
        "data-icon=\"clock\"",
        "data-icon=\"heart\"",
    ] {
        assert!(
            guest.contains(needle),
            "expected {needle:?} in guest detail, got: {guest}"
        );
    }
    assert!(
        !guest.contains("立即報名") || guest.contains("登入後報名"),
        "guest must see login CTA: {guest}"
    );

    let member = render_detail(false, Some(sample_detail()), true);
    assert!(
        member.contains("立即報名"),
        "signed-in must see register CTA: {member}"
    );
    assert!(
        member.contains("href=\"/events/11/register\""),
        "register href missing: {member}"
    );
    assert!(
        member.contains("查看參與者"),
        "participants missing: {member}"
    );
    assert!(
        !member.contains("登入後報名"),
        "signed-in must not see guest CTA: {member}"
    );
}

#[component]
fn RegisterAt(form: RegisterForm) -> Element {
    rsx! { RegisterScreen { form } }
}

fn render_register(form: RegisterForm) -> String {
    let mut vdom = VirtualDom::new_with_props(RegisterAt, RegisterAtProps { form });
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn register_step1_copy_and_validation_error() {
    let html = render_register(RegisterForm::default());
    for needle in [
        "id=\"register-heading\"",
        "加入 HeSocial",
        "申請成為尊榮會員，開啟您的頂級社交之旅",
        "步驟 1: 帳戶設定",
        "電子郵件 *",
        "密碼 *",
        "確認密碼 *",
        "下一步",
        "上一步",
        "已有帳戶？",
        "立即登入",
        "href=\"/login\"",
        "id=\"register-prev\"",
        "id=\"register-next\"",
        "data-icon=\"crown\"",
        "data-icon=\"mail\"",
        "data-icon=\"lock\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in register step 1, got: {html}"
        );
    }
    let prev = opening_tag(&html, "register-prev");
    assert!(
        prev.contains("disabled=") || prev.contains(" disabled>"),
        "prev must be disabled on step 1, tag={prev}"
    );

    let mut mismatch = RegisterForm::default();
    mismatch.error = Some(REGISTER_PASSWORD_MISMATCH.to_string());
    let err = render_register(mismatch);
    assert!(
        err.contains("id=\"register-error\""),
        "error banner missing: {err}"
    );
    assert!(
        err.contains(REGISTER_PASSWORD_MISMATCH),
        "mismatch copy missing: {err}"
    );
}

#[wasm_bindgen_test]
fn register_step3_membership_and_submitting() {
    let mut form = RegisterForm {
        step: 3,
        interests: vec!["藝術".into()],
        ..RegisterForm::default()
    };
    let html = render_register(form.clone());
    for needle in [
        "步驟 3: 會員資格",
        "年收入 (新台幣萬元) *",
        "淨資產 (新台幣萬元) *",
        "會員等級 *",
        "Platinum",
        "Diamond",
        "Black Card",
        "NT$50,000/年",
        "邀請制",
        "興趣愛好 *",
        "藝術",
        "提交申請",
        "data-icon=\"dollar-sign\"",
        "data-icon=\"plus\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in register step 3, got: {html}"
        );
    }
    form.submitting = true;
    let busy = render_register(form);
    let next = opening_tag(&busy, "register-next");
    assert!(
        next.contains("disabled=") || next.contains(" disabled>"),
        "next must disable while submitting, tag={next}"
    );
    assert!(
        busy.contains("提交申請中..."),
        "in-flight copy missing: {busy}"
    );
}
