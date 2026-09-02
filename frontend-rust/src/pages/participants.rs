use std::cell::Cell;
use std::rc::Rc;

use crate::icons::{Icon, IconName};
use crate::participants::{
    ContactDraft, PAGE_SIZE, PRIVACY_UPDATE_SUCCESS, ParticipantAccessCheck, ParticipantFilters,
    ParticipantList, ParticipantsPhase, PrivacyPhase, PrivacySettings, can_send_contact,
    display_initial, fetch_participant_access, fetch_participants, fetch_privacy_settings,
    initiate_contact, membership_tier_badge_class, page_after_filter_change, page_in_range,
    participants_phase, privacy_level_card_class, privacy_level_description,
    privacy_level_dot_class, privacy_level_indicator_class, privacy_settings_phase, total_pages,
    update_privacy_settings, visible_interests,
};
use crate::permissions::{RouteGuard, Session, user_route_guard};
use crate::shell::{
    Presence, presence_after_animation_end, presence_class, presence_is_mounted, presence_toggle,
};
use dioxus::prelude::*;

#[component]
pub fn EventParticipants(id: String) -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match user_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            GuardStatus {
                id: "event-participants-guard-loading".to_string(),
                message: "驗證存取權限中...".to_string(),
                spinning: true,
            }
        },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "event-participants-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventParticipantsBody { id } },
    }
}

#[component]
fn EventParticipantsBody(id: String) -> Element {
    let mut event_id = use_signal(|| id.clone());
    if event_id.peek().as_str() != id {
        event_id.set(id.clone());
    }
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut access = use_signal(|| None::<ParticipantAccessCheck>);
    let mut list = use_signal(|| None::<ParticipantList>);
    let mut current_page = use_signal(|| 1u32);
    let mut filters = use_signal(ParticipantFilters::default);
    let mut filters_presence = use_signal(|| Presence::Hidden);
    let mut contact = use_signal(|| None::<ContactDraft>);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let id = event_id();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            error.set(None);
            access.set(None);
            list.set(None);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let result = fetch_participant_access(&id).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                match result {
                    Ok(check) => {
                        let granted = check.has_access;
                        access.set(Some(check));
                        if !granted {
                            loading.set(false);
                        }
                    }
                    Err(message) => {
                        error.set(Some(message));
                        loading.set(false);
                    }
                }
            });
        }
    });

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let id = event_id();
            let page = current_page();
            let active_filters = filters();
            let Some(check) = access() else {
                return;
            };
            if !check.has_access {
                return;
            }
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let result = fetch_participants(&id, page, &active_filters).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                match result {
                    Ok(fetched) => {
                        list.set(Some(fetched));
                        error.set(None);
                    }
                    Err(message) => {
                        error.set(Some(message));
                    }
                }
                loading.set(false);
            });
        }
    });

    rsx! {
        EventParticipantsScreen {
            event_id: event_id(),
            loading: loading(),
            error: error(),
            access: access(),
            list: list(),
            current_page: current_page(),
            filters: filters(),
            filters_presence: filters_presence(),
            contact: contact(),
            on_toggle_filters: move |_| filters_presence.set(presence_toggle(filters_presence())),
            on_filters_animation_end: move |_| {
                filters_presence.set(presence_after_animation_end(filters_presence()));
            },
            on_search: move |value: String| {
                filters.write().search = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_membership_tier: move |value: String| {
                filters.write().membership_tier = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_profession: move |value: String| {
                filters.write().profession = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_clear_filters: move |_| {
                filters.set(ParticipantFilters::default());
                current_page.set(page_after_filter_change(current_page()));
            },
            on_page: move |new_page: u32| {
                let pages = list()
                    .as_ref()
                    .map(|items| total_pages(items.total_count, PAGE_SIZE))
                    .unwrap_or(1);
                if loading() || !page_in_range(new_page, pages) {
                    return;
                }
                current_page.set(new_page);
            },
            on_reload: move |_| reload_window(),
            on_open_contact: move |participant| {
                contact.set(Some(ContactDraft {
                    participant,
                    message: String::new(),
                    sending: false,
                    sent: false,
                }));
            },
            on_close_contact: move |_| contact.set(None),
            on_contact_message: move |value: String| {
                if let Some(draft) = contact.write().as_mut() {
                    draft.message = value;
                }
            },
            on_send_contact: move |_| {
                let Some(draft) = contact() else {
                    return;
                };
                if !can_send_contact(&draft.message) || draft.sending {
                    return;
                }
                if let Some(current) = contact.write().as_mut() {
                    current.sending = true;
                }
                let event_id = event_id();
                spawn(async move {
                    let result =
                        initiate_contact(&event_id, &draft.participant.id, &draft.message).await;
                    if let Some(current) = contact.write().as_mut() {
                        current.sending = false;
                        if result.is_ok() {
                            current.sent = true;
                        }
                    }
                });
            },
        }
    }
}

#[component]
pub fn EventParticipantsScreen(
    event_id: String,
    loading: bool,
    error: Option<String>,
    access: Option<ParticipantAccessCheck>,
    list: Option<ParticipantList>,
    current_page: u32,
    filters: ParticipantFilters,
    filters_presence: Presence,
    contact: Option<ContactDraft>,
    #[props(default)] on_toggle_filters: EventHandler<()>,
    #[props(default)] on_filters_animation_end: EventHandler<()>,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_membership_tier: EventHandler<String>,
    #[props(default)] on_profession: EventHandler<String>,
    #[props(default)] on_clear_filters: EventHandler<()>,
    #[props(default)] on_page: EventHandler<u32>,
    #[props(default)] on_reload: EventHandler<()>,
    #[props(default)] on_open_contact: EventHandler<crate::participants::FilteredParticipant>,
    #[props(default)] on_close_contact: EventHandler<()>,
    #[props(default)] on_contact_message: EventHandler<String>,
    #[props(default)] on_send_contact: EventHandler<()>,
) -> Element {
    match participants_phase(loading, access.as_ref(), list.as_ref(), error.as_deref()) {
        ParticipantsPhase::Loading => {
            return rsx! {
                GuardStatus {
                    id: "event-participants-loading".to_string(),
                    message: "載入參與者資訊中...".to_string(),
                    spinning: true,
                }
            };
        }
        ParticipantsPhase::Paywall { payment_pending } => {
            return rsx! {
                ParticipantsPaywall {
                    event_id,
                    payment_pending,
                }
            };
        }
        ParticipantsPhase::Error(message) => {
            return rsx! {
                ErrorStatus {
                    id: "event-participants-error".to_string(),
                    message,
                    on_reload,
                }
            };
        }
        ParticipantsPhase::Empty | ParticipantsPhase::Ready => {}
    }

    let pages = list
        .as_ref()
        .map(|items| total_pages(items.total_count, PAGE_SIZE))
        .unwrap_or(0);
    let detail_href = format!("/events/{event_id}");
    let privacy_href = format!("/events/{event_id}/privacy-settings");
    let participants = list
        .as_ref()
        .map(|items| items.participants.clone())
        .unwrap_or_default();
    let paid = list
        .as_ref()
        .map(|items| items.paid_participant_count)
        .unwrap_or(0);
    let diamond = list
        .as_ref()
        .map(|items| items.tier_count("Diamond"))
        .unwrap_or(0);
    let black_card = list
        .as_ref()
        .map(|items| items.tier_count("Black Card"))
        .unwrap_or(0);
    let visible_count = participants.len();
    let can_initiate = list
        .as_ref()
        .map(|items| items.viewer_access.can_initiate_contact)
        .unwrap_or(false);
    let show_grid = !participants.is_empty();
    let filter_class = format!(
        "luxury-glass p-6 rounded-2xl mb-8 {}",
        presence_class(filters_presence, "hs-enter", "hs-exit")
    );

    rsx! {
        div { id: "event-participants", class: "min-h-screen bg-luxury-midnight-black",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div { class: "mb-8",
                    a {
                        id: "event-participants-back",
                        href: "{detail_href}",
                        class: "inline-flex items-center gap-2 text-luxury-gold hover:text-luxury-gold/80 mb-4",
                        Icon { name: IconName::ArrowLeft, class: "w-4 h-4".to_string() }
                        "返回活動詳情"
                    }
                    div { class: "flex items-center justify-between",
                        div {
                            h1 { class: "text-3xl font-luxury font-bold text-luxury-gold mb-2",
                                "活動參與者"
                            }
                            p { class: "text-luxury-platinum/80", "與其他尊貴會員建立聯繫" }
                        }
                        div { class: "flex items-center gap-4",
                            button {
                                id: "event-participants-filter-toggle",
                                r#type: "button",
                                class: "inline-flex items-center gap-2 px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors",
                                onclick: move |_| on_toggle_filters.call(()),
                                Icon { name: IconName::Filter, class: "w-4 h-4".to_string() }
                                "篩選"
                            }
                            a {
                                id: "event-participants-privacy-link",
                                href: "{privacy_href}",
                                class: "inline-flex items-center gap-2 px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors",
                                Icon { name: IconName::Settings, class: "w-4 h-4".to_string() }
                                "隱私設定"
                            }
                        }
                    }
                }

                if list.is_some() {
                    div { class: "grid grid-cols-1 md:grid-cols-4 gap-6 mb-8",
                        StatCard {
                            icon: IconName::Users,
                            icon_class: "w-8 h-8 text-luxury-gold".to_string(),
                            value: paid.to_string(),
                            label: "已付費參與者".to_string(),
                        }
                        StatCard {
                            icon: IconName::Crown,
                            icon_class: "w-8 h-8 text-blue-400".to_string(),
                            value: diamond.to_string(),
                            label: "Diamond 會員".to_string(),
                        }
                        StatCard {
                            icon: IconName::Shield,
                            icon_class: "w-8 h-8 text-luxury-gold".to_string(),
                            value: black_card.to_string(),
                            label: "Black Card 會員".to_string(),
                        }
                        StatCard {
                            icon: IconName::Eye,
                            icon_class: "w-8 h-8 text-green-400".to_string(),
                            value: visible_count.to_string(),
                            label: "可查看資料".to_string(),
                        }
                    }
                }

                if presence_is_mounted(filters_presence) {
                    div {
                        id: "event-participants-filters",
                        class: "{filter_class}",
                        onanimationend: move |_| on_filters_animation_end.call(()),
                        div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                            div {
                                label { class: "block text-sm font-medium text-luxury-platinum/80 mb-2",
                                    "搜尋"
                                }
                                div { class: "relative",
                                    Icon {
                                        name: IconName::Search,
                                        class: "absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-luxury-platinum/60".to_string(),
                                    }
                                    input {
                                        r#type: "text",
                                        id: "event-participants-search",
                                        placeholder: "搜尋參與者...",
                                        value: "{filters.search}",
                                        class: "w-full pl-10 pr-4 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/60 focus:border-luxury-gold focus:outline-none",
                                        oninput: move |evt| on_search.call(evt.value()),
                                    }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-luxury-platinum/80 mb-2",
                                    "會員等級"
                                }
                                select {
                                    id: "event-participants-tier",
                                    value: "{filters.membership_tier}",
                                    class: "w-full px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum focus:border-luxury-gold focus:outline-none",
                                    onchange: move |evt| on_membership_tier.call(evt.value()),
                                    option { value: "", "所有等級" }
                                    option { value: "Platinum", "Platinum" }
                                    option { value: "Diamond", "Diamond" }
                                    option { value: "Black Card", "Black Card" }
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-luxury-platinum/80 mb-2",
                                    "職業領域"
                                }
                                input {
                                    r#type: "text",
                                    id: "event-participants-profession",
                                    placeholder: "如：科技、金融、醫療...",
                                    value: "{filters.profession}",
                                    class: "w-full px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/60 focus:border-luxury-gold focus:outline-none",
                                    oninput: move |evt| on_profession.call(evt.value()),
                                }
                            }
                            div { class: "flex items-end",
                                button {
                                    id: "event-participants-clear-filters",
                                    r#type: "button",
                                    class: "w-full px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors",
                                    onclick: move |_| on_clear_filters.call(()),
                                    "清除篩選"
                                }
                            }
                        }
                    }
                }

                if show_grid {
                    div { id: "event-participants-grid", class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 mb-8",
                        for participant in participants.iter().cloned() {
                            ParticipantCard {
                                participant,
                                can_initiate,
                                on_open_contact,
                            }
                        }
                    }
                    if pages > 1 {
                        div { class: "flex justify-center items-center gap-2",
                            button {
                                id: "event-participants-prev",
                                r#type: "button",
                                class: "px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 disabled:opacity-50 disabled:cursor-not-allowed transition-colors",
                                disabled: current_page == 1,
                                onclick: move |_| on_page.call(current_page.saturating_sub(1)),
                                "上一頁"
                            }
                            span { id: "event-participants-page-label", class: "px-4 py-2 text-luxury-platinum",
                                "第 {current_page} 頁，共 {pages} 頁"
                            }
                            button {
                                id: "event-participants-next",
                                r#type: "button",
                                class: "px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 disabled:opacity-50 disabled:cursor-not-allowed transition-colors",
                                disabled: current_page == pages,
                                onclick: move |_| on_page.call(current_page.saturating_add(1)),
                                "下一頁"
                            }
                        }
                    }
                } else {
                    div {
                        id: "event-participants-empty",
                        class: "luxury-glass p-12 rounded-2xl text-center",
                        Icon {
                            name: IconName::Users,
                            class: "w-16 h-16 text-luxury-platinum/60 mx-auto mb-4".to_string(),
                        }
                        h3 { class: "text-xl font-medium text-luxury-platinum mb-2",
                            "目前沒有符合條件的參與者"
                        }
                        p { class: "text-luxury-platinum/60", "請調整篩選條件或稍後再試" }
                    }
                }
            }

            if let Some(draft) = contact {
                ContactModal {
                    draft,
                    on_close: on_close_contact,
                    on_message: on_contact_message,
                    on_send: on_send_contact,
                }
            }
        }
    }
}

#[component]
fn ParticipantsPaywall(event_id: String, payment_pending: bool) -> Element {
    let detail_href = format!("/events/{event_id}");
    let register_href = format!("/events/{event_id}/register");
    rsx! {
        div { id: "event-participants-paywall", class: "min-h-screen bg-luxury-midnight-black",
            div { class: "max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div { class: "mb-6",
                    a {
                        href: "{detail_href}",
                        class: "inline-flex items-center gap-2 text-luxury-gold hover:text-luxury-gold/80 mb-4",
                        Icon { name: IconName::ArrowLeft, class: "w-4 h-4".to_string() }
                        "返回活動詳情"
                    }
                }
                div { class: "luxury-glass p-8 rounded-2xl text-center",
                    Icon {
                        name: IconName::Lock,
                        class: "w-16 h-16 text-luxury-gold mx-auto mb-4".to_string(),
                    }
                    h2 { class: "text-2xl font-luxury font-bold text-luxury-gold mb-4",
                        "需要付費才能查看參與者"
                    }
                    p { class: "text-luxury-platinum/80 mb-6",
                        "只有已付費參與此活動的會員才能查看其他參與者資訊"
                    }
                    if payment_pending {
                        div { id: "event-participants-pending", class: "bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4 mb-6",
                            div { class: "flex items-center justify-center gap-2 text-yellow-400",
                                Icon { name: IconName::CreditCard, class: "w-5 h-5".to_string() }
                                p { "您的報名付款正在處理中，完成付款後即可查看參與者" }
                            }
                        }
                    }
                    div { class: "flex justify-center gap-4",
                        a {
                            id: "event-participants-register",
                            href: "{register_href}",
                            class: "luxury-button",
                            "立即報名"
                        }
                        a {
                            href: "{detail_href}",
                            class: "px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors",
                            "查看活動詳情"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ParticipantCard(
    participant: crate::participants::FilteredParticipant,
    can_initiate: bool,
    on_open_contact: EventHandler<crate::participants::FilteredParticipant>,
) -> Element {
    let card_id = format!("participant-card-{}", participant.id);
    let initial = display_initial(&participant.display_name);
    let badge_class = membership_tier_badge_class(&participant.membership_tier);
    let indicator_class = privacy_level_indicator_class(participant.privacy_level);
    let tier_icon = match participant.membership_tier.as_str() {
        "Diamond" => IconName::Crown,
        "Black Card" => IconName::Shield,
        _ => IconName::Star,
    };
    let show_contact = participant.can_contact && can_initiate;
    let interests = visible_interests(&participant.interests).to_vec();
    let display_name = participant.display_name.clone();
    let profession = participant.profession.clone();
    let company = participant.company.clone();
    let city = participant.city.clone();
    let age_range = participant.age_range.clone();
    let membership_tier = participant.membership_tier.clone();
    let contact_target = participant.clone();

    rsx! {
        div {
            id: "{card_id}",
            class: "luxury-glass p-6 rounded-2xl hover:bg-luxury-midnight-black/60 transition-colors hs-enter",
            div { class: "flex items-start justify-between mb-4",
                div { class: "flex items-center gap-3",
                    div { class: "w-12 h-12 bg-luxury-gold/20 rounded-full flex items-center justify-center",
                        span { class: "text-luxury-gold font-bold text-lg", "{initial}" }
                    }
                    div {
                        h3 { class: "font-medium text-luxury-platinum", "{display_name}" }
                        if let Some(age_range) = age_range {
                            p { class: "text-sm text-luxury-platinum/60", "{age_range}" }
                        }
                    }
                }
                div { class: "w-2 h-2 rounded-full {indicator_class}" }
            }
            div { class: "space-y-2 mb-4",
                if let Some(profession) = profession {
                    p { class: "text-sm text-luxury-platinum/80", "{profession}" }
                }
                if let Some(company) = company {
                    p { class: "text-sm text-luxury-platinum/60", "{company}" }
                }
                if let Some(city) = city {
                    p { class: "text-sm text-luxury-platinum/60", "📍 {city}" }
                }
            }
            div { class: "flex items-center justify-between",
                span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border {badge_class}",
                    Icon { name: tier_icon, class: "w-3 h-3".to_string() }
                    "{membership_tier}"
                }
                if show_contact {
                    button {
                        r#type: "button",
                        class: "p-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors",
                        onclick: move |_| on_open_contact.call(contact_target.clone()),
                        Icon { name: IconName::MessageCircle, class: "w-4 h-4".to_string() }
                    }
                }
            }
            if !interests.is_empty() {
                div { class: "mt-4 pt-4 border-t border-luxury-gold/20",
                    div { class: "flex flex-wrap gap-1",
                        for interest in interests.iter() {
                            span { class: "px-2 py-1 bg-luxury-gold/10 text-luxury-gold text-xs rounded-full",
                                "{interest}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContactModal(
    draft: ContactDraft,
    on_close: EventHandler<()>,
    on_message: EventHandler<String>,
    on_send: EventHandler<()>,
) -> Element {
    let send_disabled = !can_send_contact(&draft.message) || draft.sending;
    let name = draft.participant.display_name.clone();
    rsx! {
        div {
            id: "event-participants-contact-modal",
            class: "fixed inset-0 bg-luxury-midnight-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "luxury-glass p-6 rounded-2xl max-w-md w-full hs-enter",
                if draft.sent {
                    div { class: "text-center",
                        div { class: "w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-4",
                            Icon { name: IconName::MessageCircle, class: "w-8 h-8 text-green-400".to_string() }
                        }
                        h3 { class: "text-xl font-bold text-luxury-gold mb-2", "訊息已發送" }
                        p { class: "text-luxury-platinum/80", "您的聯繫請求已送達 {name}" }
                    }
                } else {
                    div { class: "flex items-center justify-between mb-4",
                        h3 { class: "text-xl font-bold text-luxury-gold", "聯繫 {name}" }
                        button {
                            r#type: "button",
                            class: "p-2 text-luxury-platinum/60 hover:text-luxury-platinum",
                            onclick: move |_| on_close.call(()),
                            Icon { name: IconName::X, class: "w-4 h-4".to_string() }
                        }
                    }
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-luxury-platinum/80 mb-2",
                            "訊息內容"
                        }
                        textarea {
                            id: "event-participants-contact-message",
                            rows: 4,
                            value: "{draft.message}",
                            class: "w-full px-3 py-2 bg-luxury-midnight-black/50 border border-luxury-gold/20 rounded-lg text-luxury-platinum placeholder-luxury-platinum/60 focus:border-luxury-gold focus:outline-none resize-none",
                            placeholder: "請輸入您想要傳達的訊息...",
                            oninput: move |evt| on_message.call(evt.value()),
                        }
                    }
                    div { class: "flex justify-end gap-3",
                        button {
                            r#type: "button",
                            class: "px-4 py-2 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors",
                            onclick: move |_| on_close.call(()),
                            "取消"
                        }
                        button {
                            id: "event-participants-contact-send",
                            r#type: "button",
                            class: "px-4 py-2 bg-luxury-gold text-luxury-midnight-black rounded-lg hover:bg-luxury-gold/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium",
                            disabled: send_disabled,
                            onclick: move |_| on_send.call(()),
                            if draft.sending {
                                "發送中..."
                            } else {
                                "發送訊息"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn EventPrivacySettings(id: String) -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match user_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            GuardStatus {
                id: "event-privacy-guard-loading".to_string(),
                message: "驗證存取權限中...".to_string(),
                spinning: true,
            }
        },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "event-privacy-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventPrivacySettingsBody { id } },
    }
}

#[component]
fn EventPrivacySettingsBody(id: String) -> Element {
    let mut event_id = use_signal(|| id.clone());
    if event_id.peek().as_str() != id {
        event_id.set(id.clone());
    }
    let mut loading = use_signal(|| true);
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(|| None::<String>);
    let mut success_presence = use_signal(|| Presence::Hidden);
    let mut settings = use_signal(|| None::<PrivacySettings>);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let id = event_id();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            error.set(None);
            settings.set(None);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let result = fetch_privacy_settings(&id).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                match result {
                    Ok(fetched) => {
                        settings.set(Some(fetched));
                        error.set(None);
                    }
                    Err(message) => {
                        error.set(Some(message));
                    }
                }
                loading.set(false);
            });
        }
    });

    rsx! {
        EventPrivacySettingsScreen {
            event_id: event_id(),
            loading: loading(),
            saving: saving(),
            error: error(),
            success: success(),
            success_presence: success_presence(),
            settings: settings(),
            on_privacy_level: move |level: i64| {
                if let Some(current) = settings.write().as_mut() {
                    current.privacy_level = level;
                }
            },
            on_toggle_allow_contact: move |_| {
                if let Some(current) = settings.write().as_mut() {
                    current.allow_contact = !current.allow_contact;
                }
            },
            on_toggle_show_in_list: move |_| {
                if let Some(current) = settings.write().as_mut() {
                    current.show_in_list = !current.show_in_list;
                }
            },
            on_save: move |_| {
                let Some(current) = settings() else {
                    return;
                };
                saving.set(true);
                error.set(None);
                let id = event_id();
                spawn(async move {
                    match update_privacy_settings(&id, &current).await {
                        Ok(()) => {
                            success.set(Some(PRIVACY_UPDATE_SUCCESS.to_string()));
                            success_presence.set(Presence::Entering);
                        }
                        Err(message) => {
                            error.set(Some(message));
                            success.set(None);
                            success_presence.set(Presence::Hidden);
                        }
                    }
                    saving.set(false);
                });
            },
            on_success_animation_end: move |_| {
                success_presence.set(presence_after_animation_end(success_presence()));
            },
            on_reload: move |_| reload_window(),
        }
    }
}

#[component]
pub fn EventPrivacySettingsScreen(
    event_id: String,
    loading: bool,
    saving: bool,
    error: Option<String>,
    success: Option<String>,
    success_presence: Presence,
    settings: Option<PrivacySettings>,
    #[props(default)] on_privacy_level: EventHandler<i64>,
    #[props(default)] on_toggle_allow_contact: EventHandler<()>,
    #[props(default)] on_toggle_show_in_list: EventHandler<()>,
    #[props(default)] on_save: EventHandler<()>,
    #[props(default)] on_success_animation_end: EventHandler<()>,
    #[props(default)] on_reload: EventHandler<()>,
) -> Element {
    match privacy_settings_phase(loading, settings.as_ref(), error.as_deref()) {
        PrivacyPhase::Loading => {
            return rsx! {
                GuardStatus {
                    id: "event-privacy-loading".to_string(),
                    message: "載入隱私設定中...".to_string(),
                    spinning: true,
                }
            };
        }
        PrivacyPhase::Error(message) => {
            return rsx! {
                ErrorStatus {
                    id: "event-privacy-error".to_string(),
                    message,
                    on_reload,
                }
            };
        }
        PrivacyPhase::Ready => {}
    }

    let settings = settings.expect("ready");
    let participants_href = format!("/events/{event_id}/participants");
    let detail_href = format!("/events/{event_id}");
    let success_class = format!(
        "mb-6 bg-green-500/10 border border-green-500/20 rounded-lg p-4 {}",
        presence_class(success_presence, "hs-enter", "hs-exit")
    );
    let allow_track = if settings.allow_contact {
        "bg-luxury-gold"
    } else {
        "bg-luxury-platinum/20"
    };
    let allow_thumb = if settings.allow_contact {
        "translate-x-6"
    } else {
        "translate-x-0.5"
    };
    let list_track = if settings.show_in_list {
        "bg-luxury-gold"
    } else {
        "bg-luxury-platinum/20"
    };
    let list_thumb = if settings.show_in_list {
        "translate-x-6"
    } else {
        "translate-x-0.5"
    };

    rsx! {
        div { id: "event-privacy-settings", class: "min-h-screen bg-luxury-midnight-black",
            div { class: "max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div { class: "mb-8",
                    div { class: "flex items-center gap-4 mb-4",
                        a {
                            href: "{participants_href}",
                            class: "inline-flex items-center gap-2 text-luxury-gold hover:text-luxury-gold/80",
                            Icon { name: IconName::ArrowLeft, class: "w-4 h-4".to_string() }
                            "返回參與者列表"
                        }
                        a {
                            href: "{detail_href}",
                            class: "text-luxury-platinum/60 hover:text-luxury-platinum",
                            "活動詳情"
                        }
                    }
                    div { class: "flex items-center gap-3 mb-2",
                        Icon { name: IconName::Shield, class: "w-8 h-8 text-luxury-gold".to_string() }
                        h1 { class: "text-3xl font-luxury font-bold text-luxury-gold", "隱私設定" }
                    }
                    p { class: "text-luxury-platinum/80",
                        "控制您在此活動中的資訊顯示程度和聯繫偏好"
                    }
                }

                if let Some(success) = success.filter(|_| presence_is_mounted(success_presence)) {
                    div {
                        id: "event-privacy-success",
                        class: "{success_class}",
                        onanimationend: move |_| on_success_animation_end.call(()),
                        div { class: "flex items-center gap-2 text-green-400",
                            Icon { name: IconName::Check, class: "w-5 h-5".to_string() }
                            p { "{success}" }
                        }
                    }
                }

                if let Some(error) = error {
                    div { id: "event-privacy-inline-error", class: "mb-6 bg-red-500/10 border border-red-500/20 rounded-lg p-4",
                        div { class: "flex items-center gap-2 text-red-400",
                            Icon { name: IconName::AlertCircle, class: "w-5 h-5".to_string() }
                            p { "{error}" }
                        }
                    }
                }

                div { class: "space-y-8",
                    div { class: "luxury-glass p-6 rounded-2xl",
                        div { class: "flex items-center gap-3 mb-6",
                            Icon { name: IconName::Eye, class: "w-6 h-6 text-luxury-gold".to_string() }
                            h2 { class: "text-xl font-luxury font-semibold text-luxury-gold",
                                "資訊公開程度"
                            }
                        }
                        div { class: "space-y-4",
                            for level in 1..=5i64 {
                                {
                                    let selected = settings.privacy_level == level;
                                    let copy = privacy_level_description(level).expect("1-5");
                                    let card_class = if selected {
                                        privacy_level_card_class(level)
                                    } else {
                                        "border-luxury-gold/20 bg-luxury-midnight-black/30 hover:border-luxury-gold/30"
                                    };
                                    let dot = privacy_level_dot_class(level);
                                    rsx! {
                                        button {
                                            id: "event-privacy-level-{level}",
                                            r#type: "button",
                                            class: "w-full text-left p-4 rounded-xl border-2 cursor-pointer transition-all {card_class}",
                                            onclick: move |_| on_privacy_level.call(level),
                                            div { class: "flex items-start justify-between",
                                                div { class: "flex-1",
                                                    div { class: "flex items-center gap-3 mb-2",
                                                        div { class: "w-3 h-3 rounded-full {dot}" }
                                                        h3 { class: "font-medium text-luxury-platinum",
                                                            "等級 {level} - {copy.title}"
                                                        }
                                                    }
                                                    p { class: "text-sm text-luxury-platinum/80 mb-2",
                                                        "{copy.description}"
                                                    }
                                                    p { class: "text-xs text-luxury-platinum/60",
                                                        "可見範圍：{copy.visibility}"
                                                    }
                                                }
                                                if selected {
                                                    div { class: "ml-4",
                                                        div { class: "w-5 h-5 bg-luxury-gold rounded-full flex items-center justify-center",
                                                            div { class: "w-2 h-2 bg-luxury-midnight-black rounded-full" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "luxury-glass p-6 rounded-2xl",
                        div { class: "flex items-center gap-3 mb-6",
                            Icon { name: IconName::MessageCircle, class: "w-6 h-6 text-luxury-gold".to_string() }
                            h2 { class: "text-xl font-luxury font-semibold text-luxury-gold",
                                "聯繫偏好"
                            }
                        }
                        div { class: "space-y-4",
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-xl",
                                div { class: "flex items-center gap-3",
                                    Icon { name: IconName::MessageCircle, class: "w-5 h-5 text-luxury-platinum/60".to_string() }
                                    div {
                                        h3 { class: "font-medium text-luxury-platinum",
                                            "允許其他會員聯繫我"
                                        }
                                        p { class: "text-sm text-luxury-platinum/60",
                                            "其他付費參與者可以向您發送聯繫請求"
                                        }
                                    }
                                }
                                button {
                                    id: "event-privacy-allow-contact",
                                    r#type: "button",
                                    class: "relative w-12 h-6 rounded-full transition-colors {allow_track}",
                                    onclick: move |_| on_toggle_allow_contact.call(()),
                                    div { class: "absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform {allow_thumb}" }
                                }
                            }
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-xl",
                                div { class: "flex items-center gap-3",
                                    Icon { name: IconName::Users, class: "w-5 h-5 text-luxury-platinum/60".to_string() }
                                    div {
                                        h3 { class: "font-medium text-luxury-platinum",
                                            "顯示在參與者列表中"
                                        }
                                        p { class: "text-sm text-luxury-platinum/60",
                                            "在活動參與者列表中顯示您的資訊"
                                        }
                                    }
                                }
                                button {
                                    id: "event-privacy-show-in-list",
                                    r#type: "button",
                                    class: "relative w-12 h-6 rounded-full transition-colors {list_track}",
                                    onclick: move |_| on_toggle_show_in_list.call(()),
                                    div { class: "absolute top-0.5 w-5 h-5 bg-white rounded-full transition-transform {list_thumb}" }
                                }
                            }
                        }
                    }

                    div { class: "luxury-glass p-6 rounded-2xl border border-blue-500/20",
                        div { class: "flex items-start gap-3",
                            Icon { name: IconName::Info, class: "w-6 h-6 text-blue-400 mt-0.5".to_string() }
                            div {
                                h3 { class: "font-medium text-blue-400 mb-2", "隱私保護說明" }
                                div { class: "space-y-2 text-sm text-luxury-platinum/80",
                                    p { "• 只有已付費參與此活動的會員才能查看參與者資訊" }
                                    p { "• 您可以隨時調整隱私等級，變更將立即生效" }
                                    p { "• 聯繫請求會通過平台系統發送，不會直接暴露個人聯絡方式" }
                                    p { "• 所有參與者查看記錄都會被記錄，確保安全性" }
                                    p { "• Diamond 和 Black Card 會員享有更高等級的資訊查看權限" }
                                }
                            }
                        }
                    }

                    div { class: "flex justify-end gap-4",
                        a {
                            href: "{participants_href}",
                            class: "px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors",
                            "取消"
                        }
                        button {
                            id: "event-privacy-save",
                            r#type: "button",
                            class: "luxury-button px-8 py-3 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2",
                            disabled: saving,
                            onclick: move |_| on_save.call(()),
                            if saving {
                                div { class: "w-4 h-4 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin" }
                                "儲存中..."
                            } else {
                                Icon { name: IconName::Save, class: "w-4 h-4".to_string() }
                                "儲存設定"
                            }
                        }
                    }
                }
            }
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

#[component]
fn ErrorStatus(id: String, message: String, on_reload: EventHandler<()>) -> Element {
    rsx! {
        div { class: "min-h-screen bg-luxury-midnight-black",
            div { class: "max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div {
                    id: "{id}",
                    class: "luxury-glass p-8 rounded-2xl text-center",
                    Icon {
                        name: IconName::AlertCircle,
                        class: "w-16 h-16 text-red-400 mx-auto mb-4".to_string(),
                    }
                    h2 { class: "text-2xl font-luxury font-bold text-luxury-gold mb-4", "載入失敗" }
                    p { class: "text-luxury-platinum/80 mb-6", "{message}" }
                    button {
                        r#type: "button",
                        class: "luxury-button",
                        onclick: move |_| on_reload.call(()),
                        "重新載入"
                    }
                }
            }
        }
    }
}

#[component]
fn StatCard(icon: IconName, icon_class: String, value: String, label: String) -> Element {
    rsx! {
        div { class: "luxury-glass p-6 rounded-2xl",
            div { class: "flex items-center gap-3",
                Icon { name: icon, class: icon_class }
                div {
                    p { class: "text-2xl font-bold text-luxury-gold", "{value}" }
                    p { class: "text-luxury-platinum/80 text-sm", "{label}" }
                }
            }
        }
    }
}

fn reload_window() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
    }
}
