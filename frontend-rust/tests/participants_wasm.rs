#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::pages::participants::{EventParticipantsScreen, EventPrivacySettingsScreen};
use hesocial_frontend::participants::{
    ContactDraft, FilteredParticipant, ParticipantAccessCheck, ParticipantFilters, ParticipantList,
    ParticipantViewAccess, PrivacySettings,
};
use hesocial_frontend::shell::Presence;
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

fn paid_access() -> ParticipantAccessCheck {
    ParticipantAccessCheck {
        has_access: true,
        payment_required: false,
        payment_status: "paid".to_string(),
        registration_status: Some("approved".to_string()),
        access_level: ParticipantViewAccess {
            can_view_participants: true,
            max_privacy_level_visible: 3,
            can_see_contact_info: false,
            can_initiate_contact: true,
            participant_count_visible: true,
            access_level: 3,
        },
    }
}

fn sample_participant() -> FilteredParticipant {
    FilteredParticipant {
        id: "7".to_string(),
        display_name: "Ada L.".to_string(),
        profession: Some("Technology".to_string()),
        company: Some("Technology Company".to_string()),
        membership_tier: "Diamond".to_string(),
        interests: vec!["math".into(), "computing".into(), "music".into()],
        profile_picture: Some("ada.jpg".to_string()),
        age_range: Some("35-39".to_string()),
        city: Some("London".to_string()),
        bio: None,
        privacy_level: 2,
        can_contact: true,
        contact_info: None,
    }
}

fn sample_list() -> ParticipantList {
    let mut by_tier = std::collections::HashMap::new();
    by_tier.insert("Diamond".to_string(), 4);
    by_tier.insert("Black Card".to_string(), 1);
    ParticipantList {
        participants: vec![sample_participant()],
        total_count: 13,
        paid_participant_count: 8,
        unpaid_participant_count: 2,
        viewer_access: paid_access().access_level,
        participant_count_by_tier: by_tier,
    }
}

#[component]
fn ParticipantsAt(
    loading: bool,
    error: Option<String>,
    access: Option<ParticipantAccessCheck>,
    list: Option<ParticipantList>,
    filters_presence: Presence,
    contact: Option<ContactDraft>,
) -> Element {
    rsx! {
        EventParticipantsScreen {
            event_id: "42".to_string(),
            loading,
            error,
            access,
            list,
            current_page: 1,
            filters: ParticipantFilters::default(),
            filters_presence,
            contact,
        }
    }
}

fn render_participants(
    loading: bool,
    error: Option<String>,
    access: Option<ParticipantAccessCheck>,
    list: Option<ParticipantList>,
    filters_presence: Presence,
    contact: Option<ContactDraft>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ParticipantsAt,
        ParticipantsAtProps {
            loading,
            error,
            access,
            list,
            filters_presence,
            contact,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[component]
fn PrivacyAt(
    loading: bool,
    saving: bool,
    error: Option<String>,
    success: Option<String>,
    success_presence: Presence,
    settings: Option<PrivacySettings>,
) -> Element {
    rsx! {
        EventPrivacySettingsScreen {
            event_id: "42".to_string(),
            loading,
            saving,
            error,
            success,
            success_presence,
            settings,
        }
    }
}

fn render_privacy(
    loading: bool,
    saving: bool,
    error: Option<String>,
    success: Option<String>,
    success_presence: Presence,
    settings: Option<PrivacySettings>,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        PrivacyAt,
        PrivacyAtProps {
            loading,
            saving,
            error,
            success,
            success_presence,
            settings,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn participants_loading_copy() {
    let html = render_participants(true, None, None, None, Presence::Hidden, None);
    assert!(
        html.contains("id=\"event-participants-loading\""),
        "loading id missing: {html}"
    );
    assert!(
        html.contains("載入參與者資訊中..."),
        "loading copy missing: {html}"
    );
    assert!(
        !html.contains("活動參與者"),
        "heading must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn participants_paywall_and_pending_banner() {
    let denied = ParticipantAccessCheck {
        has_access: false,
        payment_required: true,
        payment_status: "none".to_string(),
        registration_status: Some("none".to_string()),
        access_level: ParticipantViewAccess::denied(),
    };
    let html = render_participants(false, None, Some(denied), None, Presence::Hidden, None);
    for needle in [
        "id=\"event-participants-paywall\"",
        "需要付費才能查看參與者",
        "只有已付費參與此活動的會員才能查看其他參與者資訊",
        "立即報名",
        "查看活動詳情",
        "href=\"/events/42/register\"",
        "data-icon=\"lock\"",
    ] {
        assert!(html.contains(needle), "expected {needle:?} in {html}");
    }
    assert!(!html.contains("您的報名付款正在處理中"));

    let pending = ParticipantAccessCheck {
        has_access: false,
        payment_required: true,
        payment_status: "pending".to_string(),
        registration_status: Some("pending".to_string()),
        access_level: ParticipantViewAccess::denied(),
    };
    let html = render_participants(false, None, Some(pending), None, Presence::Hidden, None);
    assert!(html.contains("您的報名付款正在處理中，完成付款後即可查看參與者"));
    assert!(html.contains("id=\"event-participants-pending\""));
}

#[wasm_bindgen_test]
fn participants_error_state() {
    let html = render_participants(
        false,
        Some("Failed to fetch participants".into()),
        Some(paid_access()),
        None,
        Presence::Hidden,
        None,
    );
    assert!(html.contains("id=\"event-participants-error\""));
    assert!(html.contains("載入失敗"));
    assert!(html.contains("Failed to fetch participants"));
    assert!(html.contains("重新載入"));
}

#[wasm_bindgen_test]
fn participants_empty_state() {
    let mut list = sample_list();
    list.participants.clear();
    list.total_count = 0;
    let html = render_participants(
        false,
        None,
        Some(paid_access()),
        Some(list),
        Presence::Hidden,
        None,
    );
    assert!(html.contains("id=\"event-participants-empty\""));
    assert!(html.contains("目前沒有符合條件的參與者"));
    assert!(html.contains("請調整篩選條件或稍後再試"));
    assert!(html.contains("活動參與者"));
    assert!(html.contains("與其他尊貴會員建立聯繫"));
}

#[wasm_bindgen_test]
fn participants_populated_grid_and_filters() {
    let html = render_participants(
        false,
        None,
        Some(paid_access()),
        Some(sample_list()),
        Presence::Shown,
        None,
    );
    for needle in [
        "id=\"event-participants\"",
        "活動參與者",
        "與其他尊貴會員建立聯繫",
        "已付費參與者",
        ">8<",
        "Diamond 會員",
        ">4<",
        "Black Card 會員",
        "可查看資料",
        "Ada L.",
        "Technology",
        "Technology Company",
        "📍 London",
        "35-39",
        "math",
        "id=\"participant-card-7\"",
        "href=\"/events/42/privacy-settings\"",
        "隱私設定",
        "篩選",
        "搜尋參與者...",
        "所有等級",
        "清除篩選",
        "第 1 頁，共 2 頁",
        "上一頁",
        "下一頁",
        "data-icon=\"crown\"",
        "data-icon=\"mail\"",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in populated markup, got: {html}"
        );
    }
    let card = opening_tag(&html, "participant-card-7");
    assert!(!card.is_empty(), "card missing: {html}");
}

#[wasm_bindgen_test]
fn participants_contact_modal_copy() {
    let html = render_participants(
        false,
        None,
        Some(paid_access()),
        Some(sample_list()),
        Presence::Hidden,
        Some(ContactDraft {
            participant: sample_participant(),
            message: String::new(),
            sending: false,
            sent: false,
        }),
    );
    for needle in [
        "id=\"event-participants-contact-modal\"",
        "聯繫 Ada L.",
        "訊息內容",
        "請輸入您想要傳達的訊息...",
        "取消",
        "發送訊息",
    ] {
        assert!(html.contains(needle), "expected {needle:?} in {html}");
    }
    let send = opening_tag(&html, "event-participants-contact-send");
    assert!(
        send.contains("disabled=") || send.contains(" disabled>"),
        "empty message must disable send, tag={send}"
    );

    let sent = render_participants(
        false,
        None,
        Some(paid_access()),
        Some(sample_list()),
        Presence::Hidden,
        Some(ContactDraft {
            participant: sample_participant(),
            message: "你好".into(),
            sending: false,
            sent: true,
        }),
    );
    assert!(sent.contains("訊息已發送"));
    assert!(sent.contains("您的聯繫請求已送達 Ada L."));
}

#[wasm_bindgen_test]
fn privacy_loading_and_error() {
    let loading = render_privacy(true, false, None, None, Presence::Hidden, None);
    assert!(loading.contains("id=\"event-privacy-loading\""));
    assert!(loading.contains("載入隱私設定中..."));

    let error = render_privacy(
        false,
        false,
        Some("Failed to fetch privacy settings".into()),
        None,
        Presence::Hidden,
        None,
    );
    assert!(error.contains("id=\"event-privacy-error\""));
    assert!(error.contains("載入失敗"));
    assert!(error.contains("Failed to fetch privacy settings"));
    assert!(error.contains("重新載入"));
}

#[wasm_bindgen_test]
fn privacy_populated_copy_and_levels() {
    let settings = PrivacySettings {
        privacy_level: 3,
        allow_contact: true,
        show_in_list: false,
    };
    let html = render_privacy(
        false,
        false,
        None,
        Some("隱私設定已成功更新".into()),
        Presence::Shown,
        Some(settings),
    );
    for needle in [
        "id=\"event-privacy-settings\"",
        "隱私設定",
        "控制您在此活動中的資訊顯示程度和聯繫偏好",
        "返回參與者列表",
        "href=\"/events/42/participants\"",
        "資訊公開程度",
        "等級 1 - 公開資料",
        "等級 2 - 半私人資料",
        "等級 3 - 選擇性分享",
        "等級 4 - 增強資料",
        "等級 5 - 完全公開",
        "顯示基本資訊：姓名縮寫、年齡範圍、職業類別、會員等級",
        "顯示完整名字、公司行業、經驗範圍、城市",
        "顯示全名、公司名稱、具體興趣、專業成就",
        "顯示聯絡資訊、社交連結、詳細履歷",
        "顯示直接聯絡方式、個人興趣、網絡連接",
        "所有付費參與者可見",
        "Diamond 和 Black Card 會員可見",
        "聯繫偏好",
        "允許其他會員聯繫我",
        "其他付費參與者可以向您發送聯繫請求",
        "顯示在參與者列表中",
        "在活動參與者列表中顯示您的資訊",
        "隱私保護說明",
        "只有已付費參與此活動的會員才能查看參與者資訊",
        "您可以隨時調整隱私等級，變更將立即生效",
        "聯繫請求會通過平台系統發送，不會直接暴露個人聯絡方式",
        "所有參與者查看記錄都會被記錄，確保安全性",
        "Diamond 和 Black Card 會員享有更高等級的資訊查看權限",
        "取消",
        "儲存設定",
        "隱私設定已成功更新",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in privacy markup, got: {html}"
        );
    }
}

#[wasm_bindgen_test]
fn privacy_saving_disables_submit() {
    let settings = PrivacySettings {
        privacy_level: 1,
        allow_contact: false,
        show_in_list: true,
    };
    let html = render_privacy(false, true, None, None, Presence::Hidden, Some(settings));
    let save = opening_tag(&html, "event-privacy-save");
    assert!(
        save.contains("disabled=") || save.contains(" disabled>"),
        "save must be disabled while saving, tag={save}"
    );
    assert!(html.contains("儲存中..."));
}
