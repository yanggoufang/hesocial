#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::pages::vvip::{VvipRecruitmentScreen, VvipScreen};
use hesocial_frontend::vvip::{
    PreviewAttendee, PreviewBundle, PreviewStatus, VvipEvent, preview_attendee,
};
use wasm_bindgen_test::wasm_bindgen_test;

fn sample_event() -> VvipEvent {
    VvipEvent {
        id: "11".to_string(),
        name: "松露季私宴".to_string(),
        description: "白松露當季，主廚八道式無菜單。".to_string(),
        date_time: "2026-10-04T12:00:00.000Z".to_string(),
        location: "Taipei Private Dining Room".to_string(),
        pricing_vvip: Some(15000.0),
        pricing_vip: Some(15000.0),
        current_attendees: 4,
        capacity: 12,
        category_id: "dining".to_string(),
        category_name: "私人晚宴".to_string(),
        exclusivity_level: Some("VVIP".to_string()),
        images: vec!["https://media.example/e11.webp".into()],
        tags: vec!["私宴".into(), "松露".into()],
    }
}

fn sample_preview() -> PreviewAttendee {
    preview_attendee(
        &serde_json::json!({
            "id": "p1",
            "displayName": "Ada Lovelace",
            "email": "ada@example.com",
            "contactInfo": {"email": "ada@example.com", "phone": "+886900000000"},
            "profession": "Technology",
            "company": "Technology Company",
            "membershipTier": "Diamond"
        }),
        "11",
    )
    .expect("preview")
}

#[component]
fn ContentAt(events: Vec<VvipEvent>, loading: bool, category: String) -> Element {
    rsx! {
        VvipScreen {
            loading,
            events,
            selected_category: category,
        }
    }
}

fn render_content(events: Vec<VvipEvent>, loading: bool, category: &str) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ContentAt,
        ContentAtProps {
            events,
            loading,
            category: category.into(),
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[component]
fn RecruitmentAt(
    events: Vec<VvipEvent>,
    preview: PreviewBundle,
    loading: bool,
    is_authenticated: bool,
) -> Element {
    rsx! {
        VvipRecruitmentScreen {
            loading,
            events,
            preview,
            is_authenticated,
        }
    }
}

fn render_recruitment(
    events: Vec<VvipEvent>,
    preview: PreviewBundle,
    loading: bool,
    is_authenticated: bool,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        RecruitmentAt,
        RecruitmentAtProps {
            events,
            preview,
            loading,
            is_authenticated,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn recruitment_markup_explains_benefits_threshold_and_join_cta() {
    let html = render_recruitment(
        vec![],
        PreviewBundle {
            status: PreviewStatus::SignedOut,
            attendees: Vec::new(),
        },
        false,
        false,
    );
    for needle in [
        "id=\"vvip-recruitment\"",
        "VVIP 專區",
        "專為最頂級會員打造的獨家體驗空間",
        "享受前所未有的奢華與尊榮",
        "VVIP 專屬特權",
        "超越一般會員的頂級服務體驗",
        "專屬禮賓服務",
        "24/7 專人服務，滿足您的每一個需求",
        "獨家活動優先權",
        "最高隱私保障",
        "客製化體驗",
        "僅限 Diamond 以上且已完成身份驗證的會員進入",
        "申請 VVIP 會員資格",
        "VVIP 會員採邀請制，需經過嚴格審核。",
        "申請邀請函",
        "聯繫專屬顧問",
        "年費 NT$ 500,000 起，享受全年無限制專屬服務",
        "href=\"/register\"",
        "id=\"vvip-join-cta\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in recruitment markup, got: {html}"
        );
    }
    assert!(
        !html.contains("id=\"vvip-content\""),
        "recruitment must not render the member-only surface: {html}"
    );
}

#[wasm_bindgen_test]
fn recruitment_never_falls_back_to_events_list() {
    let html = render_recruitment(
        vec![],
        PreviewBundle {
            status: PreviewStatus::SignedOut,
            attendees: Vec::new(),
        },
        false,
        false,
    );
    assert!(
        !html.contains("href=\"/events\""),
        "signed-out recruitment must not redirect or link to /events: {html}"
    );
}

#[wasm_bindgen_test]
fn signed_out_preview_degrades_without_blanking_the_page() {
    let html = render_recruitment(
        vec![sample_event()],
        PreviewBundle {
            status: PreviewStatus::SignedOut,
            attendees: Vec::new(),
        },
        false,
        false,
    );
    assert!(
        html.contains("id=\"vvip-preview-degraded\""),
        "missing degraded preview: {html}"
    );
    assert!(
        html.contains("登入後可預覽正在參加活動的優質會員（身分已遮蔽）"),
        "missing signed-out copy: {html}"
    );
    assert!(html.contains("VVIP 專區"), "page must still render: {html}");
}

#[wasm_bindgen_test]
fn unauthorized_preview_degrades_like_signed_out() {
    let html = render_recruitment(
        vec![sample_event()],
        PreviewBundle {
            status: PreviewStatus::Unauthorized,
            attendees: Vec::new(),
        },
        false,
        true,
    );
    assert!(html.contains("id=\"vvip-preview-degraded\""), "{html}");
    assert!(
        html.contains("登入後可預覽正在參加活動的優質會員（身分已遮蔽）"),
        "missing 401 copy: {html}"
    );
    assert!(
        html.contains("href=\"/profile\""),
        "signed-in join CTA should go to profile: {html}"
    );
}

#[wasm_bindgen_test]
fn masked_preview_shows_profession_industry_tier_not_identity() {
    let html = render_recruitment(
        vec![sample_event()],
        PreviewBundle {
            status: PreviewStatus::Ready,
            attendees: vec![sample_preview()],
        },
        false,
        true,
    );
    assert!(html.contains("id=\"vvip-preview\""), "{html}");
    assert!(html.contains("Technology"), "profession missing: {html}");
    assert!(
        html.contains("Technology Company"),
        "industry missing: {html}"
    );
    assert!(html.contains("Diamond"), "tier missing: {html}");
    assert!(!html.contains("Ada Lovelace"), "identity leaked: {html}");
    assert!(!html.contains("ada@example.com"), "email leaked: {html}");
    assert!(!html.contains("+886900000000"), "phone leaked: {html}");
}

#[wasm_bindgen_test]
fn content_markup_ports_vvip_copy_and_api_events() {
    let html = render_content(vec![sample_event()], false, "all");
    for needle in [
        "id=\"vvip-content\"",
        "VVIP 專區",
        "獨家活動",
        "僅限 VVIP 會員參與的頂級社交體驗",
        "全部活動",
        "頂級餐飲",
        "奢華旅遊",
        "藝術收藏",
        "商務社交",
        "松露季私宴",
        "白松露當季，主廚八道式無菜單。",
        "4/12 人",
        "NT$ 15,000",
        "查看詳情",
        "href=\"/events/11\"",
        "申請 VVIP 會員資格",
        "暫無此類別活動",
    ] {
        if needle == "暫無此類別活動" {
            assert!(
                !html.contains(needle),
                "empty category must not show when events exist: {html}"
            );
            continue;
        }
        assert!(
            html.contains(needle),
            "expected {needle:?} in content markup, got: {html}"
        );
    }
    assert!(
        !html.contains("id=\"vvip-recruitment\""),
        "member surface must not render recruitment: {html}"
    );
}

#[wasm_bindgen_test]
fn content_empty_category_copy() {
    let html = render_content(vec![sample_event()], false, "business");
    assert!(html.contains("暫無此類別活動"), "{html}");
    assert!(html.contains("請選擇其他類別或稍後再來查看"), "{html}");
    assert!(
        !html.contains("松露季私宴"),
        "filtered-out event must not render: {html}"
    );
}

#[wasm_bindgen_test]
fn content_loading_hides_grid() {
    let html = render_content(vec![sample_event()], true, "all");
    assert!(html.contains("載入中..."), "{html}");
    assert!(
        !html.contains("id=\"vvip-event-11\""),
        "cards must not render while loading: {html}"
    );
}
