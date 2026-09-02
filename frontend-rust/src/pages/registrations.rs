use std::cell::Cell;
use std::rc::Rc;

use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session, user_route_guard};
use crate::registrations::{
    CANCEL_SUCCESS, EVENT_TITLE_FALLBACK, PAGE_SIZE, RegisterEvent, RegisterUser, Registration,
    RegistrationFilters, UPDATE_SUCCESS, boot_success_message, can_cancel, can_edit,
    cancel_registration, confirm_cancel, create_registration, default_filters, event_title,
    event_when_label, fetch_register_event, fetch_user_registrations, format_event_price,
    format_list_datetime, format_register_datetime, membership_tier_badge_class, now_ms,
    page_after_filter_change, page_in_range, pagination_range, parse_register_user_from_auth,
    payment_class, payment_label, register_exclusivity_class, register_user_from_profile,
    status_class, status_label, update_registration, venue_label,
};
use crate::shell::{Presence, presence_after_animation_end, presence_class, presence_is_mounted};
use dioxus::prelude::*;

#[component]
pub fn MyRegistrations() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match user_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { RegistrationsGuardLoading {} },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "registrations-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { MyRegistrationsBody {} },
    }
}

#[component]
pub fn EventRegister(id: String) -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match user_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { RegistrationsGuardLoading {} },
        RouteGuard::Redirect(_) => {
            navigator.replace("/login");
            rsx! {
                p { id: "event-register-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventRegisterBody { id, session } },
    }
}

#[component]
fn RegistrationsGuardLoading() -> Element {
    rsx! {
        div {
            id: "registrations-guard-loading",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                p { class: "text-luxury-platinum", "驗證存取權限中..." }
            }
        }
    }
}

fn start_list_fetch(
    fetch_gen: Rc<Cell<u32>>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut registrations: Signal<Vec<Registration>>,
    mut pagination: Signal<crate::events::Pagination>,
    next: RegistrationFilters,
) {
    let request_id = fetch_gen.get() + 1;
    fetch_gen.set(request_id);
    loading.set(true);
    error.set(None);
    spawn(async move {
        let result = fetch_user_registrations(&next).await;
        if fetch_gen.get() != request_id {
            return;
        }
        match result {
            Ok(view) => {
                registrations.set(view.registrations);
                pagination.set(view.pagination);
                error.set(None);
            }
            Err(message) => {
                registrations.set(Vec::new());
                error.set(Some(message));
            }
        }
        loading.set(false);
    });
}

#[component]
fn MyRegistrationsBody() -> Element {
    let registrations = use_signal(Vec::<Registration>::new);
    let loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(boot_success_message);
    let mut filters = use_signal(default_filters);
    let pagination = use_signal(|| crate::events::Pagination {
        page: 1,
        limit: PAGE_SIZE,
        total: 0,
        total_pages: 1,
    });
    let mut edit_modal = use_signal(|| Presence::Hidden);
    let mut edit_registration = use_signal(|| None::<Registration>);
    let mut edit_requests = use_signal(String::new);
    let mut action_loading = use_signal(|| None::<String>);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            start_list_fetch(
                fetch_gen.clone(),
                loading,
                error,
                registrations,
                pagination,
                filters(),
            );
        }
    });

    rsx! {
        MyRegistrationsScreen {
            registrations: registrations(),
            loading: loading(),
            error: error(),
            success_message: success(),
            filters: filters(),
            pagination: pagination(),
            edit_modal: edit_modal(),
            edit_registration: edit_registration(),
            edit_requests: edit_requests(),
            action_loading: action_loading(),
            now_ms: now_ms(),
            on_search: move |value: String| {
                let mut next = filters();
                next.search = value;
                next.page = page_after_filter_change(next.page);
                filters.set(next);
            },
            on_status: move |value: String| {
                let mut next = filters();
                next.status = value;
                next.page = page_after_filter_change(next.page);
                filters.set(next);
            },
            on_payment_status: move |value: String| {
                let mut next = filters();
                next.payment_status = value;
                next.page = page_after_filter_change(next.page);
                filters.set(next);
            },
            on_clear_filters: move |_| {
                filters.set(default_filters());
            },
            on_refresh: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    start_list_fetch(
                        fetch_gen.clone(),
                        loading,
                        error,
                        registrations,
                        pagination,
                        filters(),
                    );
                }
            },
            on_page: move |new_page: u32| {
                if loading() || !page_in_range(new_page, pagination().total_pages) {
                    return;
                }
                let mut next = filters();
                next.page = new_page;
                filters.set(next);
            },
            on_dismiss_error: move |_| error.set(None),
            on_dismiss_success: move |_| success.set(None),
            on_open_edit: move |registration: Registration| {
                edit_requests.set(registration.special_requests.clone().unwrap_or_default());
                edit_registration.set(Some(registration));
                edit_modal.set(Presence::Entering);
            },
            on_close_edit: move |_| edit_modal.set(Presence::Exiting),
            on_edit_animation_end: move |_| {
                let next = presence_after_animation_end(edit_modal());
                edit_modal.set(next);
                if next == Presence::Hidden {
                    edit_registration.set(None);
                }
            },
            on_edit_requests: move |value: String| edit_requests.set(value),
            on_save_edit: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    let Some(selected) = edit_registration() else {
                        return;
                    };
                    let id = selected.id.clone();
                    let requests = edit_requests();
                    action_loading.set(Some("edit".to_string()));
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        match update_registration(&id, &requests).await {
                            Ok(_) => {
                                edit_modal.set(Presence::Exiting);
                                success.set(Some(UPDATE_SUCCESS.to_string()));
                                error.set(None);
                                start_list_fetch(
                                    fetch_gen,
                                    loading,
                                    error,
                                    registrations,
                                    pagination,
                                    filters(),
                                );
                            }
                            Err(message) => error.set(Some(message)),
                        }
                        action_loading.set(None);
                    });
                }
            },
            on_cancel: {
                let fetch_gen = fetch_gen.clone();
                move |id: String| {
                    if !confirm_cancel() {
                        return;
                    }
                    action_loading.set(Some(format!("cancel-{id}")));
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        match cancel_registration(&id).await {
                            Ok(_) => {
                                success.set(Some(CANCEL_SUCCESS.to_string()));
                                error.set(None);
                                start_list_fetch(
                                    fetch_gen,
                                    loading,
                                    error,
                                    registrations,
                                    pagination,
                                    filters(),
                                );
                            }
                            Err(message) => error.set(Some(message)),
                        }
                        action_loading.set(None);
                    });
                }
            },
        }
    }
}

#[component]
fn EventRegisterBody(id: String, session: Signal<Session>) -> Element {
    let navigator = use_navigator();
    let mut event_id = use_signal(|| id.clone());
    if event_id.peek().as_str() != id {
        event_id.set(id.clone());
    }
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut event = use_signal(|| None::<RegisterEvent>);
    let mut user = use_signal(|| None::<RegisterUser>);
    let mut special_requests = use_signal(String::new);
    let mut registering = use_signal(|| false);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let id = event_id();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            error.set(None);
            event.set(None);
            let snapshot_user = session().user.clone();
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                if let Some(auth) = snapshot_user.as_ref() {
                    user.set(Some(parse_register_user_from_auth(auth)));
                }
                if let Ok(profile) = crate::profile::fetch_profile().await {
                    user.set(Some(register_user_from_profile(&profile)));
                }
                let fetched = fetch_register_event(&id).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                match fetched {
                    Ok(value) => {
                        event.set(Some(value));
                        error.set(None);
                    }
                    Err(_) => {
                        event.set(None);
                    }
                }
                loading.set(false);
            });
        }
    });

    rsx! {
        EventRegisterScreen {
            loading: loading(),
            error: error(),
            event: event(),
            user: user(),
            special_requests: special_requests(),
            registering: registering(),
            on_special_requests: move |value: String| special_requests.set(value),
            on_submit: move |_| {
                let Some(current) = event() else {
                    return;
                };
                registering.set(true);
                error.set(None);
                let requests = special_requests();
                spawn(async move {
                    match create_registration(&current.id, &requests).await {
                        Ok(_) => {
                            navigator.push("/profile/registrations?registered=1");
                        }
                        Err(message) => {
                            error.set(Some(message));
                            registering.set(false);
                        }
                    }
                });
            },
        }
    }
}

#[component]
pub fn MyRegistrationsScreen(
    registrations: Vec<Registration>,
    loading: bool,
    error: Option<String>,
    success_message: Option<String>,
    filters: RegistrationFilters,
    pagination: crate::events::Pagination,
    edit_modal: Presence,
    edit_registration: Option<Registration>,
    edit_requests: String,
    action_loading: Option<String>,
    now_ms: f64,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_status: EventHandler<String>,
    #[props(default)] on_payment_status: EventHandler<String>,
    #[props(default)] on_clear_filters: EventHandler<()>,
    #[props(default)] on_refresh: EventHandler<()>,
    #[props(default)] on_page: EventHandler<u32>,
    #[props(default)] on_dismiss_error: EventHandler<()>,
    #[props(default)] on_dismiss_success: EventHandler<()>,
    #[props(default)] on_open_edit: EventHandler<Registration>,
    #[props(default)] on_close_edit: EventHandler<()>,
    #[props(default)] on_edit_animation_end: EventHandler<()>,
    #[props(default)] on_edit_requests: EventHandler<String>,
    #[props(default)] on_save_edit: EventHandler<()>,
    #[props(default)] on_cancel: EventHandler<String>,
) -> Element {
    let (range_start, range_end) =
        pagination_range(pagination.page, pagination.limit, pagination.total);
    let prev_disabled = pagination.page <= 1 || loading;
    let next_disabled = pagination.page >= pagination.total_pages || loading;
    let show_pagination = pagination.total_pages > 1;
    let edit_mounted = presence_is_mounted(edit_modal);
    let edit_name = edit_registration
        .as_ref()
        .map(|row| event_title(row.event_name.as_deref()))
        .unwrap_or_else(|| EVENT_TITLE_FALLBACK.to_string());
    let saving = action_loading.as_deref() == Some("edit");

    rsx! {
        div {
            id: "my-registrations",
            class: "min-h-screen bg-luxury-midnight-black text-luxury-platinum font-sans",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12",
                div { class: "mb-10 hs-enter",
                    div { class: "flex items-center justify-between",
                        div {
                            h1 {
                                id: "my-registrations-heading",
                                class: "text-4xl font-luxury font-bold text-luxury-gold tracking-wider",
                                "我的活動報名"
                            }
                            p { class: "text-luxury-platinum/80 mt-3 text-lg",
                                "管理您的活動報名與申請狀態"
                            }
                        }
                        a {
                            id: "my-registrations-explore",
                            href: "/events",
                            class: "luxury-button-outline inline-flex items-center gap-2",
                            Icon { name: IconName::ArrowRight, class: "w-4 h-4".to_string() }
                            "探索更多活動"
                        }
                    }
                }

                if let Some(message) = success_message.as_ref() {
                    div {
                        id: "my-registrations-success",
                        class: "mb-6 bg-green-900/30 border border-green-600/50 rounded-lg p-4 flex items-center justify-between backdrop-blur-sm",
                        div { class: "flex items-center gap-3",
                            Icon { name: IconName::Check, class: "w-5 h-5 text-green-400".to_string() }
                            p { class: "text-green-200", "{message}" }
                        }
                        button {
                            r#type: "button",
                            id: "my-registrations-dismiss-success",
                            class: "text-green-400 hover:text-green-200",
                            onclick: move |_| on_dismiss_success.call(()),
                            Icon { name: IconName::X, class: "w-4 h-4".to_string() }
                        }
                    }
                }

                if let Some(message) = error.as_ref() {
                    div {
                        id: "my-registrations-error",
                        class: "mb-6 bg-red-900/30 border border-red-600/50 rounded-lg p-4 flex items-center justify-between backdrop-blur-sm",
                        div { class: "flex items-center gap-3",
                            Icon { name: IconName::AlertCircle, class: "w-5 h-5 text-red-400".to_string() }
                            p { class: "text-red-200", "{message}" }
                        }
                        button {
                            r#type: "button",
                            id: "my-registrations-dismiss-error",
                            class: "text-red-400 hover:text-red-200",
                            onclick: move |_| on_dismiss_error.call(()),
                            Icon { name: IconName::X, class: "w-4 h-4".to_string() }
                        }
                    }
                }

                div { class: "luxury-glass rounded-xl shadow-2xl mb-8 p-6 hs-enter-filters",
                    div { class: "flex items-center justify-between mb-5",
                        h3 { class: "text-xl font-luxury font-semibold text-luxury-gold flex items-center gap-2",
                            Icon { name: IconName::Filter, class: "w-5 h-5".to_string() }
                            "篩選器"
                        }
                        button {
                            r#type: "button",
                            id: "my-registrations-refresh",
                            class: "inline-flex items-center gap-2 px-3 py-1.5 text-sm text-luxury-platinum/80 hover:text-luxury-gold hover:bg-white/10 rounded-md transition-colors",
                            onclick: move |_| on_refresh.call(()),
                            Icon { name: IconName::RefreshCw, class: "w-4 h-4".to_string() }
                            "刷新"
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-4 gap-6",
                        div { class: "relative",
                            label { class: "block text-sm font-medium text-luxury-platinum mb-2",
                                "關鍵字搜尋"
                            }
                            Icon { name: IconName::Search, class: "absolute left-3 top-10 w-4 h-4 text-luxury-gold/60".to_string() }
                            input {
                                r#type: "text",
                                id: "my-registrations-search",
                                placeholder: "搜尋活動名稱...",
                                value: "{filters.search}",
                                class: "w-full pl-10 pr-4 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum placeholder-luxury-platinum/50",
                                oninput: move |evt| on_search.call(evt.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-luxury-platinum mb-2",
                                "報名狀態"
                            }
                            select {
                                id: "my-registrations-status",
                                value: "{filters.status}",
                                class: "w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum",
                                onchange: move |evt| on_status.call(evt.value()),
                                option { value: "", selected: filters.status.is_empty(), "所有狀態" }
                                option { value: "pending", selected: filters.status == "pending", "審核中" }
                                option { value: "approved", selected: filters.status == "approved", "已核准" }
                                option { value: "rejected", selected: filters.status == "rejected", "已婉拒" }
                                option { value: "cancelled", selected: filters.status == "cancelled", "已取消" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-luxury-platinum mb-2",
                                "付款狀態"
                            }
                            select {
                                id: "my-registrations-payment",
                                value: "{filters.payment_status}",
                                class: "w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum",
                                onchange: move |evt| on_payment_status.call(evt.value()),
                                option { value: "", selected: filters.payment_status.is_empty(), "所有狀態" }
                                option { value: "pending", selected: filters.payment_status == "pending", "待付款" }
                                option { value: "paid", selected: filters.payment_status == "paid", "已付款" }
                                option { value: "refunded", selected: filters.payment_status == "refunded", "已退款" }
                            }
                        }
                        div { class: "flex items-end",
                            button {
                                r#type: "button",
                                id: "my-registrations-clear",
                                class: "w-full px-4 py-2 text-luxury-platinum bg-white/10 border border-white/20 rounded-lg hover:bg-white/20 transition-colors",
                                onclick: move |_| on_clear_filters.call(()),
                                "清除篩選"
                            }
                        }
                    }
                }

                div { class: "luxury-glass rounded-xl shadow-2xl overflow-hidden",
                    div { class: "px-6 py-5 border-b border-white/20 bg-white/5",
                        h3 { class: "text-xl font-luxury font-semibold text-luxury-gold",
                            "報名總覽 "
                            span { class: "text-luxury-platinum/80",
                                "({pagination.total} 筆記錄)"
                            }
                        }
                    }

                    if loading {
                        div {
                            id: "my-registrations-loading",
                            class: "flex items-center justify-center py-20 gap-3",
                            div { class: "w-8 h-8 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin" }
                            span { class: "text-lg text-luxury-platinum", "讀取報名記錄中..." }
                        }
                    } else if registrations.is_empty() {
                        div {
                            id: "my-registrations-empty",
                            class: "text-center py-20",
                            Icon { name: IconName::Ticket, class: "w-20 h-20 text-luxury-gold/50 mx-auto mb-6".to_string() }
                            h3 { class: "text-2xl font-luxury font-semibold text-luxury-gold mb-3",
                                "尚無報名記錄"
                            }
                            p { class: "text-luxury-platinum/80 mb-6",
                                "您尚未報名任何活動，立即探索我們的精選活動吧！"
                            }
                            a {
                                id: "my-registrations-empty-explore",
                                href: "/events",
                                class: "luxury-button inline-flex items-center gap-2",
                                Icon { name: IconName::ArrowRight, class: "w-5 h-5".to_string() }
                                "探索活動"
                            }
                        }
                    } else {
                        div { id: "my-registrations-list", class: "divide-y divide-white/20",
                            for registration in registrations.iter().cloned() {
                                RegistrationRow {
                                    registration,
                                    now_ms,
                                    action_loading: action_loading.clone(),
                                    on_open_edit,
                                    on_cancel,
                                }
                            }
                        }
                    }

                    if show_pagination {
                        div { class: "px-6 py-4 border-t border-white/20 bg-white/5 flex items-center justify-between",
                            div { id: "my-registrations-range", class: "text-sm text-luxury-platinum/80",
                                "顯示第 {range_start} 至 {range_end} 項，共 {pagination.total} 項結果"
                            }
                            div { class: "flex items-center gap-2",
                                button {
                                    r#type: "button",
                                    id: "my-registrations-prev",
                                    class: "inline-flex items-center px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-sm font-medium text-luxury-platinum hover:bg-white/20 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: prev_disabled,
                                    onclick: move |_| on_page.call(pagination.page.saturating_sub(1)),
                                    Icon { name: IconName::ChevronLeft, class: "w-4 h-4".to_string() }
                                    "上一頁"
                                }
                                span { id: "my-registrations-page-label", class: "text-sm text-luxury-platinum",
                                    "第 {pagination.page} / {pagination.total_pages} 頁"
                                }
                                button {
                                    r#type: "button",
                                    id: "my-registrations-next",
                                    class: "inline-flex items-center px-3 py-2 bg-white/10 border border-white/20 rounded-lg text-sm font-medium text-luxury-platinum hover:bg-white/20 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: next_disabled,
                                    onclick: move |_| on_page.call(pagination.page + 1),
                                    "下一頁"
                                    Icon { name: IconName::ChevronRight, class: "w-4 h-4".to_string() }
                                }
                            }
                        }
                    }
                }
            }

            if edit_mounted {
                div {
                    id: "my-registrations-edit-modal",
                    class: "fixed inset-0 bg-luxury-midnight-black/90 backdrop-blur-luxury flex items-center justify-center z-50 {presence_class(edit_modal, \"hs-dropdown-enter\", \"hs-dropdown-exit\")}",
                    onanimationend: move |_| on_edit_animation_end.call(()),
                    div { class: "luxury-glass rounded-xl p-7 w-full max-w-md shadow-2xl",
                        div { class: "flex items-center justify-between mb-5",
                            h3 { class: "text-2xl font-luxury font-bold text-luxury-gold",
                                "編輯報名資訊"
                            }
                            button {
                                r#type: "button",
                                id: "my-registrations-edit-close",
                                class: "text-luxury-platinum/60 hover:text-luxury-gold transition-colors",
                                onclick: move |_| on_close_edit.call(()),
                                Icon { name: IconName::X, class: "w-7 h-7".to_string() }
                            }
                        }
                        div { class: "mb-6",
                            p { class: "text-luxury-platinum mb-4",
                                "活動: "
                                strong { class: "text-luxury-gold", "{edit_name}" }
                            }
                            label { class: "block text-sm font-medium text-luxury-platinum mb-2",
                                "特別要求"
                            }
                            textarea {
                                id: "my-registrations-edit-requests",
                                rows: 5,
                                value: "{edit_requests}",
                                class: "w-full px-4 py-3 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold transition-colors text-luxury-platinum placeholder-luxury-platinum/50",
                                placeholder: "請輸入您的飲食限制、過敏資訊或任何特殊需求...",
                                oninput: move |evt| on_edit_requests.call(evt.value()),
                            }
                        }
                        div { class: "flex justify-end gap-4",
                            button {
                                r#type: "button",
                                id: "my-registrations-edit-cancel",
                                class: "px-5 py-2.5 text-luxury-platinum bg-white/10 border border-white/20 rounded-lg hover:bg-white/20 transition-colors",
                                onclick: move |_| on_close_edit.call(()),
                                "取消"
                            }
                            button {
                                r#type: "button",
                                id: "my-registrations-edit-save",
                                class: "luxury-button px-5 py-2.5 flex items-center gap-2 disabled:opacity-50",
                                disabled: saving,
                                onclick: move |_| on_save_edit.call(()),
                                if saving {
                                    div { class: "w-5 h-5 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin" }
                                } else {
                                    "儲存變更"
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
fn RegistrationRow(
    registration: Registration,
    now_ms: f64,
    action_loading: Option<String>,
    on_open_edit: EventHandler<Registration>,
    on_cancel: EventHandler<String>,
) -> Element {
    let title = event_title(registration.event_name.as_deref());
    let when = event_when_label(registration.event_date_time.as_deref());
    let venue = venue_label(registration.venue_name.as_deref());
    let created = format_list_datetime(&registration.created_at);
    let status_text = status_label(&registration.status).to_string();
    let status_styles = status_class(&registration.status);
    let payment_text = payment_label(&registration.payment_status).to_string();
    let payment_styles = payment_class(&registration.payment_status);
    let detail_href = format!("/events/{}", registration.event_id);
    let row_id = format!("registration-card-{}", registration.id);
    let editable = can_edit(
        &registration.status,
        registration.event_date_time.as_deref(),
        now_ms,
    );
    let cancellable = can_cancel(
        &registration.status,
        registration.event_date_time.as_deref(),
        now_ms,
    );
    let cancel_key = format!("cancel-{}", registration.id);
    let cancel_busy = action_loading.as_deref() == Some(cancel_key.as_str());
    let registration_id = registration.id.clone();
    let registration_edit = registration.clone();
    let special = registration.special_requests.clone();
    let status_icon = match registration.status.as_str() {
        "pending" => IconName::Clock,
        "approved" | "confirmed" => IconName::Check,
        "rejected" => IconName::X,
        _ => IconName::AlertCircle,
    };

    rsx! {
        div { id: "{row_id}", class: "p-6 hover:bg-white/5 transition-colors duration-300",
            div { class: "flex flex-col md:flex-row items-start justify-between gap-4",
                div { class: "flex-1",
                    h4 { class: "text-xl font-luxury font-bold text-luxury-gold mb-3", "{title}" }
                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 mb-4 text-luxury-platinum",
                        div { class: "flex items-center gap-2",
                            Icon { name: IconName::Calendar, class: "w-4 h-4 text-luxury-gold".to_string() }
                            span { "{when}" }
                        }
                        div { class: "flex items-center gap-2",
                            Icon { name: IconName::MapPin, class: "w-4 h-4 text-luxury-gold".to_string() }
                            span { "{venue}" }
                        }
                        div { class: "flex items-center gap-2",
                            Icon { name: IconName::Clock, class: "w-4 h-4 text-luxury-gold".to_string() }
                            span { "報名於 {created}" }
                        }
                    }
                    div { class: "flex items-center gap-3 mb-4",
                        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border {status_styles}",
                            Icon { name: status_icon, class: "w-3 h-3".to_string() }
                            "{status_text}"
                        }
                        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium {payment_styles}",
                            Icon { name: IconName::CreditCard, class: "w-3 h-3".to_string() }
                            "{payment_text}"
                        }
                    }
                    if let Some(requests) = special.filter(|text| !text.is_empty()) {
                        div { class: "luxury-glass rounded-lg p-3",
                            p { class: "text-sm font-medium text-luxury-platinum mb-1", "特別要求:" }
                            p { class: "text-sm text-luxury-platinum/80 whitespace-pre-wrap", "{requests}" }
                        }
                    }
                }
                div { class: "flex items-center gap-2 self-start md:self-center mt-4 md:mt-0",
                    a {
                        href: "{detail_href}",
                        class: "p-2 text-luxury-platinum/60 hover:text-luxury-gold hover:bg-white/10 rounded-full transition-colors",
                        title: "查看活動詳情",
                        Icon { name: IconName::Eye, class: "w-5 h-5".to_string() }
                    }
                    if editable {
                        button {
                            r#type: "button",
                            class: "p-2 text-blue-400 hover:text-blue-300 hover:bg-blue-900/50 rounded-full transition-colors",
                            title: "編輯報名",
                            onclick: move |_| on_open_edit.call(registration_edit.clone()),
                            Icon { name: IconName::Edit, class: "w-5 h-5".to_string() }
                        }
                    }
                    if cancellable {
                        button {
                            r#type: "button",
                            class: "p-2 text-red-400 hover:text-red-300 hover:bg-red-900/50 rounded-full transition-colors disabled:opacity-50",
                            title: "取消報名",
                            disabled: cancel_busy,
                            onclick: move |_| on_cancel.call(registration_id.clone()),
                            Icon { name: IconName::X, class: "w-5 h-5".to_string() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn EventRegisterScreen(
    loading: bool,
    error: Option<String>,
    event: Option<RegisterEvent>,
    user: Option<RegisterUser>,
    special_requests: String,
    registering: bool,
    #[props(default)] on_special_requests: EventHandler<String>,
    #[props(default)] on_submit: EventHandler<()>,
) -> Element {
    if loading {
        return rsx! {
            div {
                id: "event-register-loading",
                class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
                div { class: "text-center",
                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-luxury-gold mx-auto mb-4" }
                    p { class: "text-luxury-platinum/80", "Loading event details..." }
                }
            }
        };
    }

    let Some(event) = event else {
        return rsx! {
            div {
                id: "event-register-not-found",
                class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
                div { class: "text-center",
                    Icon { name: IconName::AlertCircle, class: "w-16 h-16 text-luxury-platinum/40 mx-auto mb-4".to_string() }
                    h1 { class: "text-2xl font-bold text-luxury-gold mb-2", "Event Not Found" }
                    p { class: "text-luxury-platinum/80 mb-4",
                        "The event you're looking for doesn't exist or is no longer available."
                    }
                    a {
                        id: "event-register-back-missing",
                        href: "/events",
                        class: "inline-flex items-center gap-2 px-4 py-2 luxury-button",
                        Icon { name: IconName::ArrowLeft, class: "w-4 h-4".to_string() }
                        "Back to Events"
                    }
                }
            }
        };
    };

    let price = format_event_price(&event.pricing);
    let when = format_register_datetime(&event.date_time);
    let deadline = format_register_datetime(&event.registration_deadline);
    let badge_class = register_exclusivity_class(event.exclusivity_level.as_deref());
    let badge_label = event.exclusivity_level.clone().unwrap_or_default();
    let hero = event.images.first().cloned();
    let capacity = format!(
        "{} / {} registered",
        event.current_attendees, event.capacity
    );
    let user_name = user
        .as_ref()
        .map(|u| format!("{} {}", u.first_name, u.last_name));
    let submit_disabled = registering;

    rsx! {
        div { id: "event-register", class: "min-h-screen bg-luxury-midnight-black",
            div { class: "max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div { class: "mb-6 hs-enter",
                    a {
                        id: "event-register-back",
                        href: "/events",
                        class: "inline-flex items-center gap-2 text-luxury-platinum/80 hover:text-luxury-gold mb-4",
                        Icon { name: IconName::ArrowLeft, class: "w-4 h-4".to_string() }
                        "Back to Events"
                    }
                    h1 {
                        id: "event-register-heading",
                        class: "text-3xl font-bold text-luxury-gold mb-2",
                        "Event Registration"
                    }
                    p { class: "text-luxury-platinum/80",
                        "Complete your registration for this exclusive event"
                    }
                }

                if let Some(message) = error.as_ref() {
                    div {
                        id: "event-register-error",
                        class: "mb-6 bg-red-900/30 border border-red-600/50 rounded-lg p-4",
                        div { class: "flex items-center",
                            Icon { name: IconName::X, class: "w-5 h-5 text-red-400 mr-2".to_string() }
                            p { class: "text-red-200", "{message}" }
                        }
                    }
                }

                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                    div { class: "lg:col-span-2",
                        div { class: "luxury-glass rounded-lg shadow-sm border border-white/10 overflow-hidden",
                            if let Some(src) = hero {
                                div { class: "aspect-video bg-luxury-midnight-black/50",
                                    img {
                                        src: "{src}",
                                        alt: "{event.name}",
                                        class: "w-full h-full object-cover",
                                    }
                                }
                            }
                            div { class: "p-6",
                                div { class: "flex items-start justify-between mb-4",
                                    div {
                                        h2 { class: "text-2xl font-bold text-luxury-gold mb-2", "{event.name}" }
                                        div { class: "flex items-center gap-3 mb-2",
                                            span { class: "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium border {badge_class}",
                                                "{badge_label}"
                                            }
                                            span { class: "text-luxury-platinum/80", "{event.category_name}" }
                                        }
                                    }
                                    div { class: "text-right",
                                        p { id: "event-register-price", class: "text-2xl font-bold text-luxury-gold", "{price}" }
                                        p { class: "text-sm text-luxury-platinum/60", "Per person" }
                                    }
                                }
                                p { class: "text-luxury-platinum/80 mb-6", "{event.description}" }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mb-6",
                                    div { class: "flex items-center gap-3",
                                        Icon { name: IconName::Calendar, class: "w-5 h-5 text-luxury-gold".to_string() }
                                        div {
                                            p { class: "font-medium text-luxury-platinum", "Date & Time" }
                                            p { class: "text-luxury-platinum/80", "{when}" }
                                        }
                                    }
                                    div { class: "flex items-center gap-3",
                                        Icon { name: IconName::MapPin, class: "w-5 h-5 text-luxury-gold".to_string() }
                                        div {
                                            p { class: "font-medium text-luxury-platinum", "Venue" }
                                            p { class: "text-luxury-platinum/80", "{event.venue_name}" }
                                            p { class: "text-sm text-luxury-platinum/60", "{event.venue_address}" }
                                        }
                                    }
                                    div { class: "flex items-center gap-3",
                                        Icon { name: IconName::Users, class: "w-5 h-5 text-luxury-gold".to_string() }
                                        div {
                                            p { class: "font-medium text-luxury-platinum", "Capacity" }
                                            p { class: "text-luxury-platinum/80", "{capacity}" }
                                        }
                                    }
                                    div { class: "flex items-center gap-3",
                                        Icon { name: IconName::Clock, class: "w-5 h-5 text-luxury-gold".to_string() }
                                        div {
                                            p { class: "font-medium text-luxury-platinum", "Dress Code" }
                                            p { class: "text-luxury-platinum/80", "{event.dress_code_label}" }
                                        }
                                    }
                                }
                                div { class: "bg-yellow-900/20 border border-yellow-600/40 rounded-lg p-4 mb-6",
                                    div { class: "flex items-center gap-2",
                                        Icon { name: IconName::AlertCircle, class: "w-5 h-5 text-yellow-400".to_string() }
                                        p { class: "font-medium text-yellow-200", "Registration Deadline" }
                                    }
                                    p { class: "text-yellow-100/80 mt-1", "{deadline}" }
                                }
                                if !event.requirements.is_empty() {
                                    div { class: "mb-6",
                                        h3 { class: "text-lg font-medium text-luxury-gold mb-3", "Requirements" }
                                        div { class: "space-y-2",
                                            for req in event.requirements.iter() {
                                                div { class: "flex items-center gap-2",
                                                    Icon { name: IconName::Shield, class: "w-4 h-4 text-luxury-platinum/60".to_string() }
                                                    span { class: "text-luxury-platinum/80", "{req}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if !event.amenities.is_empty() {
                                    div { class: "mb-6",
                                        h3 { class: "text-lg font-medium text-luxury-gold mb-3", "Amenities" }
                                        div { class: "flex flex-wrap gap-2",
                                            for amenity in event.amenities.iter() {
                                                span { class: "px-3 py-1 bg-white/10 text-luxury-platinum rounded-full text-sm",
                                                    "{amenity}"
                                                }
                                            }
                                        }
                                    }
                                }
                                if !event.privacy_guarantees.is_empty() {
                                    div {
                                        h3 { class: "text-lg font-medium text-luxury-gold mb-3", "Privacy & Security" }
                                        div { class: "space-y-2",
                                            for guarantee in event.privacy_guarantees.iter() {
                                                div { class: "flex items-center gap-2",
                                                    Icon { name: IconName::Check, class: "w-4 h-4 text-green-400".to_string() }
                                                    span { class: "text-luxury-platinum/80", "{guarantee}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "lg:col-span-1",
                        div { class: "luxury-glass rounded-lg shadow-sm border border-white/10 p-6 sticky top-8",
                            h3 { class: "text-xl font-bold text-luxury-gold mb-4", "Registration" }
                            if let Some(user) = user.as_ref() {
                                div { class: "mb-6 p-4 bg-white/5 rounded-lg",
                                    h4 { class: "font-medium text-luxury-platinum mb-3", "Your Information" }
                                    div { class: "space-y-2 text-sm",
                                        div { class: "flex items-center gap-2",
                                            Icon { name: IconName::User, class: "w-4 h-4 text-luxury-platinum/60".to_string() }
                                            span { "{user_name.clone().unwrap_or_default()}" }
                                        }
                                        div { class: "flex items-center gap-2",
                                            Icon { name: IconName::Mail, class: "w-4 h-4 text-luxury-platinum/60".to_string() }
                                            span { "{user.email}" }
                                        }
                                        div { class: "flex items-center gap-2",
                                            Icon { name: IconName::Briefcase, class: "w-4 h-4 text-luxury-platinum/60".to_string() }
                                            span { "{user.profession}" }
                                        }
                                        div { class: "flex items-center gap-2",
                                            Icon { name: IconName::Crown, class: "w-4 h-4 text-luxury-platinum/60".to_string() }
                                            span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border {membership_tier_badge_class(&user.membership_tier)}",
                                                "{user.membership_tier}"
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "mb-4",
                                label { class: "block text-sm font-medium text-luxury-platinum mb-2",
                                    "Special Requests (Optional)"
                                }
                                textarea {
                                    id: "event-register-requests",
                                    value: "{special_requests}",
                                    rows: 4,
                                    class: "w-full px-3 py-2 bg-white/10 border border-white/20 rounded-lg focus:ring-2 focus:ring-luxury-gold focus:border-luxury-gold text-luxury-platinum",
                                    placeholder: "Any dietary restrictions, accessibility needs, or special requests...",
                                    oninput: move |evt| on_special_requests.call(evt.value()),
                                }
                            }
                            div { class: "mb-6 p-4 bg-blue-900/20 border border-blue-500/30 rounded-lg",
                                h4 { class: "font-medium text-blue-100 mb-2", "Registration Process" }
                                ol { class: "list-decimal list-inside text-sm text-blue-100/80 space-y-1",
                                    li { "Submit registration request" }
                                    li { "Admin review and approval" }
                                    li { "Payment processing" }
                                    li { "Confirmation and event details" }
                                }
                            }
                            button {
                                r#type: "button",
                                id: "event-register-submit",
                                class: "w-full px-4 py-3 luxury-button disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium flex items-center justify-center gap-2",
                                disabled: submit_disabled,
                                onclick: move |_| on_submit.call(()),
                                if registering {
                                    div { class: "animate-spin rounded-full h-4 w-4 border-b-2 border-luxury-midnight-black" }
                                    "Submitting..."
                                } else {
                                    Icon { name: IconName::DollarSign, class: "w-4 h-4".to_string() }
                                    "Submit Registration"
                                }
                            }
                            p { class: "text-xs text-luxury-platinum/60 mt-3 text-center",
                                "Registration is subject to approval. You will be notified via email once your application is reviewed."
                            }
                        }
                    }
                }
            }
        }
    }
}
