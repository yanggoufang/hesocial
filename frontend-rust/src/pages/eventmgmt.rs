use std::cell::Cell;
use std::rc::Rc;

use crate::eventmgmt::{
    CREATE_ACTION, DELETE_ACTION, EDIT_ACTION, EVENT_FETCH_FALLBACK, EVENT_LOAD_ERROR,
    EVENT_NOT_FOUND, EventCategory, EventFormData, EventMgmtFilters, EventMgmtModal, EventVenue,
    EventsPage, EventsPagination, FileCandidate, MAX_DOCUMENT_FILES, MAX_IMAGE_FILES, MAX_SIZE_MB,
    ManagedEvent, MediaAsset, MediaKind, MediaTab, PendingFile, UPLOAD_ACTION, UploadBytes,
    action_is, approve_action_key, approve_event, can_approve, can_publish, create_event,
    delete_event, delete_media, empty_form, event_management_guard, fetch_categories,
    fetch_event_media, fetch_managed_event, fetch_managed_events, fetch_venues, form_from_event,
    format_event_datetime, format_file_size, format_media_date, format_price, lines_to_list,
    list_to_lines, media_page_path, media_status_badge_class, page_after_filter_change,
    page_in_range, parse_f64_input, parse_u32_input, publish_action_key, publish_event,
    select_valid_files, status_badge_class, status_label, tab_media_kind, toggle_membership_tier,
    update_event, upload_event_media, validate_event_form, venue_option_label,
};
use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session, permissions};
use crate::shell::{Presence, presence_after_animation_end, presence_class, presence_is_mounted};
use dioxus::prelude::*;

#[component]
pub fn EventMgmt() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match event_management_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { EventMgmtGuardLoading {} },
        RouteGuard::Redirect(path) => {
            navigator.replace(path);
            rsx! {
                p { id: "event-mgmt-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventMgmtBody { session } },
    }
}

#[component]
fn EventMgmtGuardLoading() -> Element {
    rsx! {
        div {
            id: "event-mgmt-guard-loading",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
            }
        }
    }
}

#[component]
fn EventMgmtBody(session: Signal<Session>) -> Element {
    let navigator = use_navigator();
    let loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let events = use_signal(Vec::<ManagedEvent>::new);
    let categories = use_signal(Vec::<EventCategory>::new);
    let venues = use_signal(Vec::<EventVenue>::new);
    let pagination = use_signal(EventsPagination::default);
    let mut filters = use_signal(EventMgmtFilters::list_default);
    let mut selected_event = use_signal(|| None::<ManagedEvent>);
    let mut form_data = use_signal(empty_form);
    let mut modal = use_signal(|| EventMgmtModal::None);
    let mut form_presence = use_signal(|| Presence::Hidden);
    let mut delete_presence = use_signal(|| Presence::Hidden);
    let mut action_loading = use_signal(|| None::<String>);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let active = filters();
            start_events_fetch(
                fetch_gen.clone(),
                loading,
                error,
                events,
                pagination,
                active,
            );
            start_taxonomy_fetch(categories, venues, error);
        }
    });

    let snapshot = session().snapshot();
    let can_manage = permissions(&snapshot).event_management;
    let can_delete = permissions(&snapshot).manage_super_admin;

    rsx! {
        EventMgmtScreen {
            can_manage,
            can_delete,
            loading: loading(),
            error: error(),
            events: events(),
            categories: categories(),
            venues: venues(),
            filters: filters(),
            pagination: pagination(),
            selected_event: selected_event(),
            form_data: form_data(),
            modal: modal(),
            form_presence: form_presence(),
            delete_presence: delete_presence(),
            action_loading: action_loading(),
            on_search: move |value: String| {
                filters.write().search = value;
                filters.write().page = page_after_filter_change(filters().page);
            },
            on_status: move |value: String| {
                filters.write().status = value;
                filters.write().page = page_after_filter_change(filters().page);
            },
            on_category: move |value: String| {
                filters.write().category = value;
                filters.write().page = page_after_filter_change(filters().page);
            },
            on_venue: move |value: String| {
                filters.write().venue = value;
                filters.write().page = page_after_filter_change(filters().page);
            },
            on_page: move |new_page: u32| {
                if loading() || !page_in_range(new_page, pagination().total_pages) {
                    return;
                }
                filters.write().page = new_page;
            },
            on_open_create: move |_| {
                selected_event.set(None);
                form_data.set(empty_form());
                modal.set(EventMgmtModal::Form);
                form_presence.set(Presence::Entering);
            },
            on_open_edit: move |event: ManagedEvent| {
                form_data.set(form_from_event(&event));
                selected_event.set(Some(event));
                modal.set(EventMgmtModal::Form);
                form_presence.set(Presence::Entering);
            },
            on_open_delete: move |event: ManagedEvent| {
                selected_event.set(Some(event));
                modal.set(EventMgmtModal::Delete);
                delete_presence.set(Presence::Entering);
            },
            on_open_media: move |event_id: String| {
                navigator.push(media_page_path(&event_id));
            },
            on_close_form: move |_| {
                form_presence.set(Presence::Exiting);
                modal.set(EventMgmtModal::None);
                selected_event.set(None);
                form_data.set(empty_form());
            },
            on_close_delete: move |_| {
                delete_presence.set(Presence::Exiting);
                modal.set(EventMgmtModal::None);
                selected_event.set(None);
            },
            on_form_presence_end: move |_| {
                form_presence.set(presence_after_animation_end(form_presence()));
            },
            on_delete_presence_end: move |_| {
                delete_presence.set(presence_after_animation_end(delete_presence()));
            },
            on_form_data: move |data: EventFormData| form_data.set(data),
            on_save_form: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    if let Err(message) = validate_event_form(&form_data()) {
                        error.set(Some(message));
                        return;
                    }
                    let editing = selected_event();
                    let key = if editing.is_some() {
                        EDIT_ACTION
                    } else {
                        CREATE_ACTION
                    };
                    if action_is(action_loading().as_deref(), key) {
                        return;
                    }
                    action_loading.set(Some(key.to_string()));
                    let data = form_data();
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        let result = if let Some(event) = editing {
                            update_event(&event.id, &data).await
                        } else {
                            create_event(&data).await
                        };
                        action_loading.set(None);
                        match result {
                            Ok(()) => {
                                modal.set(EventMgmtModal::None);
                                selected_event.set(None);
                                form_data.set(empty_form());
                                form_presence.set(Presence::Hidden);
                                start_events_fetch(
                                    fetch_gen,
                                    loading,
                                    error,
                                    events,
                                    pagination,
                                    filters(),
                                );
                            }
                            Err(message) => error.set(Some(message)),
                        }
                    });
                }
            },
            on_confirm_delete: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    let Some(event) = selected_event() else {
                        return;
                    };
                    if action_is(action_loading().as_deref(), DELETE_ACTION) {
                        return;
                    }
                    action_loading.set(Some(DELETE_ACTION.to_string()));
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        let result = delete_event(&event.id).await;
                        action_loading.set(None);
                        match result {
                            Ok(()) => {
                                modal.set(EventMgmtModal::None);
                                selected_event.set(None);
                                delete_presence.set(Presence::Hidden);
                                start_events_fetch(
                                    fetch_gen,
                                    loading,
                                    error,
                                    events,
                                    pagination,
                                    filters(),
                                );
                            }
                            Err(message) => error.set(Some(message)),
                        }
                    });
                }
            },
            on_publish: {
                let fetch_gen = fetch_gen.clone();
                move |event_id: String| {
                    start_status_action(
                        fetch_gen.clone(),
                        loading,
                        error,
                        events,
                        pagination,
                        action_loading,
                        filters(),
                        publish_action_key(&event_id),
                        async move { publish_event(&event_id).await },
                    );
                }
            },
            on_approve: {
                let fetch_gen = fetch_gen.clone();
                move |event_id: String| {
                    start_status_action(
                        fetch_gen.clone(),
                        loading,
                        error,
                        events,
                        pagination,
                        action_loading,
                        filters(),
                        approve_action_key(&event_id),
                        async move { approve_event(&event_id, true).await },
                    );
                }
            },
            on_reject: {
                let fetch_gen = fetch_gen.clone();
                move |event_id: String| {
                    start_status_action(
                        fetch_gen.clone(),
                        loading,
                        error,
                        events,
                        pagination,
                        action_loading,
                        filters(),
                        approve_action_key(&event_id),
                        async move { approve_event(&event_id, false).await },
                    );
                }
            },
        }
    }
}

fn start_events_fetch(
    fetch_gen: Rc<Cell<u32>>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut events: Signal<Vec<ManagedEvent>>,
    mut pagination: Signal<EventsPagination>,
    filters: EventMgmtFilters,
) {
    let request_id = fetch_gen.get() + 1;
    fetch_gen.set(request_id);
    loading.set(true);
    error.set(None);
    spawn(async move {
        let result = fetch_managed_events(&filters).await;
        if fetch_gen.get() != request_id {
            return;
        }
        match result {
            Ok(EventsPage {
                events: fetched,
                pagination: next,
            }) => {
                events.set(fetched);
                pagination.set(next);
            }
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn start_taxonomy_fetch(
    mut categories: Signal<Vec<EventCategory>>,
    mut venues: Signal<Vec<EventVenue>>,
    mut error: Signal<Option<String>>,
) {
    spawn(async move {
        match fetch_categories().await {
            Ok(fetched) => categories.set(fetched),
            Err(message) if error().is_none() => error.set(Some(message)),
            Err(_) => {}
        }
    });
    spawn(async move {
        match fetch_venues().await {
            Ok(fetched) => venues.set(fetched),
            Err(message) if error().is_none() => error.set(Some(message)),
            Err(_) => {}
        }
    });
}

fn start_status_action<F>(
    fetch_gen: Rc<Cell<u32>>,
    loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    events: Signal<Vec<ManagedEvent>>,
    pagination: Signal<EventsPagination>,
    mut action_loading: Signal<Option<String>>,
    filters: EventMgmtFilters,
    key: String,
    work: F,
) where
    F: std::future::Future<Output = Result<(), String>> + 'static,
{
    if action_is(action_loading().as_deref(), &key) {
        return;
    }
    action_loading.set(Some(key));
    spawn(async move {
        let result = work.await;
        action_loading.set(None);
        match result {
            Ok(()) => start_events_fetch(fetch_gen, loading, error, events, pagination, filters),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[component]
pub fn EventMgmtScreen(
    can_manage: bool,
    can_delete: bool,
    loading: bool,
    error: Option<String>,
    events: Vec<ManagedEvent>,
    categories: Vec<EventCategory>,
    venues: Vec<EventVenue>,
    filters: EventMgmtFilters,
    pagination: EventsPagination,
    selected_event: Option<ManagedEvent>,
    form_data: EventFormData,
    modal: EventMgmtModal,
    form_presence: Presence,
    delete_presence: Presence,
    action_loading: Option<String>,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_status: EventHandler<String>,
    #[props(default)] on_category: EventHandler<String>,
    #[props(default)] on_venue: EventHandler<String>,
    #[props(default)] on_page: EventHandler<u32>,
    #[props(default)] on_open_create: EventHandler<()>,
    #[props(default)] on_open_edit: EventHandler<ManagedEvent>,
    #[props(default)] on_open_delete: EventHandler<ManagedEvent>,
    #[props(default)] on_open_media: EventHandler<String>,
    #[props(default)] on_close_form: EventHandler<()>,
    #[props(default)] on_close_delete: EventHandler<()>,
    #[props(default)] on_form_presence_end: EventHandler<()>,
    #[props(default)] on_delete_presence_end: EventHandler<()>,
    #[props(default)] on_form_data: EventHandler<EventFormData>,
    #[props(default)] on_save_form: EventHandler<()>,
    #[props(default)] on_confirm_delete: EventHandler<()>,
    #[props(default)] on_publish: EventHandler<String>,
    #[props(default)] on_approve: EventHandler<String>,
    #[props(default)] on_reject: EventHandler<String>,
) -> Element {
    if !can_manage {
        return rsx! {
            div {
                id: "event-mgmt-denied",
                class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center text-luxury-platinum",
                div { class: "text-center",
                    h1 { class: "text-2xl font-bold mb-4", "Access Denied" }
                    p { "You need admin privileges to access this page." }
                }
            }
        };
    }

    let show_pagination = pagination.total_pages > 1;
    let empty = !loading && events.is_empty();
    let page_copy = format!("Page {} of {}", pagination.page, pagination.total_pages);
    let prev_disabled = pagination.page <= 1;
    let next_disabled = pagination.page >= pagination.total_pages;
    let form_open = presence_is_mounted(form_presence) || modal == EventMgmtModal::Form;
    let delete_open = (presence_is_mounted(delete_presence) || modal == EventMgmtModal::Delete)
        && selected_event.is_some();
    let form_class = format!(
        "fixed inset-0 bg-black/50 overflow-y-auto h-full w-full z-50 {}",
        presence_class(form_presence, "hs-enter", "hs-exit")
    );
    let delete_class = format!(
        "fixed inset-0 bg-black/70 flex items-center justify-center z-50 {}",
        presence_class(delete_presence, "hs-enter", "hs-exit")
    );
    let save_busy = action_is(action_loading.as_deref(), CREATE_ACTION)
        || action_is(action_loading.as_deref(), EDIT_ACTION);
    let delete_busy = action_is(action_loading.as_deref(), DELETE_ACTION);
    let editing = selected_event.is_some() && modal == EventMgmtModal::Form;
    let form_title = if editing {
        "Edit Event"
    } else {
        "Create New Event"
    };
    let save_label = if save_busy {
        "Saving..."
    } else if editing {
        "Update Event"
    } else {
        "Create Event"
    };

    rsx! {
        div { id: "event-mgmt-page", class: "p-4 sm:p-6 lg:p-8 bg-luxury-midnight-black min-h-screen text-luxury-platinum",
            div { class: "mb-8",
                h1 { class: "text-3xl font-bold text-luxury-gold font-luxury tracking-wider", "Event Dashboard" }
                p { class: "text-luxury-platinum/60 mt-1", "Manage all aspects of your events, categories, and venues." }
            }
            div { class: "luxury-glass p-2 rounded-xl mb-8",
                nav { class: "flex items-center space-x-2 sm:space-x-4",
                    a {
                        href: "/event-mgmt",
                        class: "flex-1 sm:flex-initial flex items-center justify-center px-3 py-2 rounded-lg transition-all duration-300 text-sm sm:text-base font-medium bg-luxury-gold/20 text-luxury-gold shadow-inner",
                        Icon { name: IconName::Briefcase, class: "w-4 h-4 mr-2".to_string() }
                        span { "Events" }
                    }
                    a {
                        href: "/event-mgmt/categories",
                        class: "flex-1 sm:flex-initial flex items-center justify-center px-3 py-2 rounded-lg transition-all duration-300 text-sm sm:text-base font-medium text-luxury-platinum/70 hover:bg-luxury-gold/10 hover:text-luxury-platinum",
                        Icon { name: IconName::Filter, class: "w-4 h-4 mr-2".to_string() }
                        span { "Categories" }
                    }
                    a {
                        href: "/event-mgmt/venues",
                        class: "flex-1 sm:flex-initial flex items-center justify-center px-3 py-2 rounded-lg transition-all duration-300 text-sm sm:text-base font-medium text-luxury-platinum/70 hover:bg-luxury-gold/10 hover:text-luxury-platinum",
                        Icon { name: IconName::MapPin, class: "w-4 h-4 mr-2".to_string() }
                        span { "Venues" }
                    }
                }
            }
            div { class: "space-y-6",
                div { class: "luxury-glass p-4 rounded-lg",
                    div { class: "flex flex-col md:flex-row justify-between items-center mb-4",
                        h2 { class: "text-xl font-bold text-luxury-platinum mb-4 md:mb-0", "Event List" }
                        button {
                            id: "event-mgmt-create",
                            r#type: "button",
                            class: "luxury-button-primary w-full md:w-auto",
                            onclick: move |_| on_open_create.call(()),
                            Icon { name: IconName::Plus, class: "h-4 w-4 mr-2".to_string() }
                            "Create Event"
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                        div { class: "relative",
                            Icon { name: IconName::Search, class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string() }
                            input {
                                r#type: "text",
                                placeholder: "Search by title...",
                                value: "{filters.search}",
                                class: "luxury-input w-full pl-10",
                                oninput: move |evt| on_search.call(evt.value()),
                            }
                        }
                        select {
                            class: "luxury-input",
                            value: "{filters.status}",
                            onchange: move |evt| on_status.call(evt.value()),
                            option { value: "", "All Statuses" }
                            option { value: "draft", "Draft" }
                            option { value: "pending_review", "Pending Review" }
                            option { value: "approved", "Approved" }
                            option { value: "published", "Published" }
                        }
                        select {
                            class: "luxury-input",
                            value: "{filters.category}",
                            onchange: move |evt| on_category.call(evt.value()),
                            option { value: "", "All Categories" }
                            for category in categories.iter() {
                                option { value: "{category.filter_value()}", "{category.name}" }
                            }
                        }
                        select {
                            class: "luxury-input",
                            value: "{filters.venue}",
                            onchange: move |evt| on_venue.call(evt.value()),
                            option { value: "", "All Venues" }
                            for venue in venues.iter() {
                                option { value: "{venue.id}", "{venue.name}" }
                            }
                        }
                    }
                }
                if let Some(message) = error.clone() {
                    div {
                        id: "event-mgmt-error",
                        class: "bg-red-900/50 text-red-200 border border-red-700 p-4 rounded-lg",
                        "{message}"
                    }
                }
                div { class: "luxury-glass overflow-x-auto rounded-lg",
                    if loading {
                        div {
                            id: "event-mgmt-loading",
                            class: "p-6 text-center text-luxury-platinum/70",
                            "Loading events..."
                        }
                    } else if empty {
                        div {
                            id: "event-mgmt-empty",
                            class: "p-6 text-center text-luxury-platinum/70",
                            "No events found."
                        }
                    } else {
                        table { id: "event-mgmt-table", class: "w-full text-sm text-left text-luxury-platinum/80",
                            thead { class: "text-xs text-luxury-gold uppercase bg-luxury-gold/10",
                                tr {
                                    th { class: "px-6 py-3", "Event" }
                                    th { class: "px-6 py-3", "Status" }
                                    th { class: "px-6 py-3", "Details" }
                                    th { class: "px-6 py-3", "Pricing" }
                                    th { class: "px-6 py-3 text-right", "Actions" }
                                }
                            }
                            tbody {
                                for event in events.iter() {
                                    {
                                        let event = event.clone();
                                        let plat = format!("{} (Plat)", format_price(event.price_platinum, &event.currency));
                                        let dia = format!("{} (Dia)", format_price(event.price_diamond, &event.currency));
                                        let when = format_event_datetime(&event.start_datetime);
                                        let badge = status_badge_class(&event.status);
                                        let label = status_label(&event.status);
                                        let show_publish = can_publish(&event.status, &event.approval_status);
                                        let show_approve = can_approve(&event.status, &event.approval_status);
                                        let event_for_edit = event.clone();
                                        let event_for_delete = event.clone();
                                        let publish_id = event.id.clone();
                                        let approve_id = event.id.clone();
                                        let reject_id = event.id.clone();
                                        let media_id = event.id.clone();
                                        rsx! {
                                            tr { class: "border-b border-luxury-gold/10 hover:bg-luxury-gold/5",
                                                td { class: "px-6 py-4 font-medium text-white", "{event.title}" }
                                                td { class: "px-6 py-4",
                                                    span { class: "px-2 py-1 rounded-full text-xs font-medium {badge}", "{label}" }
                                                }
                                                td { class: "px-6 py-4",
                                                    div { "{event.venue_name}" }
                                                    div { class: "text-xs text-luxury-platinum/60", "{when}" }
                                                }
                                                td { class: "px-6 py-4",
                                                    div { "{plat}" }
                                                    div { class: "text-xs text-luxury-platinum/60", "{dia}" }
                                                }
                                                td { class: "px-6 py-4 text-right space-x-2",
                                                    button {
                                                        r#type: "button",
                                                        class: "luxury-button-icon",
                                                        onclick: move |_| on_open_edit.call(event_for_edit.clone()),
                                                        Icon { name: IconName::Edit, class: "h-4 w-4".to_string() }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "luxury-button-icon",
                                                        onclick: move |_| on_open_media.call(media_id.clone()),
                                                        Icon { name: IconName::Eye, class: "h-4 w-4".to_string() }
                                                    }
                                                    if show_publish {
                                                        button {
                                                            id: "event-mgmt-publish-{event.id}",
                                                            r#type: "button",
                                                            class: "luxury-button-icon",
                                                            onclick: move |_| on_publish.call(publish_id.clone()),
                                                            Icon { name: IconName::CheckCircle, class: "h-4 w-4".to_string() }
                                                        }
                                                    }
                                                    if show_approve {
                                                        button {
                                                            id: "event-mgmt-approve-{event.id}",
                                                            r#type: "button",
                                                            class: "luxury-button-icon",
                                                            onclick: move |_| on_approve.call(approve_id.clone()),
                                                            Icon { name: IconName::Check, class: "h-4 w-4".to_string() }
                                                        }
                                                        button {
                                                            id: "event-mgmt-reject-{event.id}",
                                                            r#type: "button",
                                                            class: "luxury-button-icon",
                                                            onclick: move |_| on_reject.call(reject_id.clone()),
                                                            Icon { name: IconName::XCircle, class: "h-4 w-4".to_string() }
                                                        }
                                                    }
                                                    if can_delete {
                                                        button {
                                                            r#type: "button",
                                                            class: "luxury-button-icon-danger",
                                                            onclick: move |_| on_open_delete.call(event_for_delete.clone()),
                                                            Icon { name: IconName::Trash2, class: "h-4 w-4".to_string() }
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
                }
                if show_pagination {
                    div { class: "flex items-center justify-between text-luxury-platinum/80",
                        span { "{page_copy}" }
                        div { class: "space-x-2",
                            button {
                                r#type: "button",
                                class: "luxury-button-secondary",
                                disabled: prev_disabled,
                                onclick: move |_| on_page.call(pagination.page.saturating_sub(1)),
                                "Previous"
                            }
                            button {
                                r#type: "button",
                                class: "luxury-button-secondary",
                                disabled: next_disabled,
                                onclick: move |_| on_page.call(pagination.page.saturating_add(1)),
                                "Next"
                            }
                        }
                    }
                }
            }
            if form_open {
                div {
                    class: "{form_class}",
                    onanimationend: move |_| on_form_presence_end.call(()),
                    EventFormPanel {
                        title: form_title.to_string(),
                        save_label: save_label.to_string(),
                        save_busy,
                        form_data,
                        categories,
                        venues,
                        error: error.clone(),
                        on_close: on_close_form,
                        on_form_data,
                        on_save: on_save_form,
                    }
                }
            }
            if delete_open {
                if let Some(event) = selected_event.clone() {
                    div {
                        class: "{delete_class}",
                        onanimationend: move |_| on_delete_presence_end.call(()),
                        div { class: "luxury-glass p-6 rounded-lg text-center",
                            h3 { class: "text-lg font-bold text-luxury-gold mb-4", "Delete Event" }
                            p { "Are you sure you want to delete \"{event.title}\"?" }
                            div { class: "mt-6 space-x-4",
                                button {
                                    r#type: "button",
                                    class: "luxury-button-secondary",
                                    onclick: move |_| on_close_delete.call(()),
                                    "Cancel"
                                }
                                button {
                                    r#type: "button",
                                    class: "luxury-button-danger",
                                    disabled: delete_busy,
                                    onclick: move |_| on_confirm_delete.call(()),
                                    "Delete"
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
fn EventFormPanel(
    title: String,
    save_label: String,
    save_busy: bool,
    form_data: EventFormData,
    categories: Vec<EventCategory>,
    venues: Vec<EventVenue>,
    error: Option<String>,
    on_close: EventHandler<()>,
    on_form_data: EventHandler<EventFormData>,
    on_save: EventHandler<()>,
) -> Element {
    let inclusions = list_to_lines(&form_data.inclusions);
    let exclusions = list_to_lines(&form_data.exclusions);
    let form = std::rc::Rc::new(form_data.clone());
    rsx! {
        div { class: "relative top-4 mx-auto p-5 border border-gray-200 w-full max-w-4xl shadow-lg rounded-md bg-white mb-4",
            div { class: "flex items-center justify-between border-b pb-4 mb-6",
                h3 { class: "text-2xl font-semibold text-gray-900", "{title}" }
                button {
                    r#type: "button",
                    class: "text-gray-400 hover:text-gray-600 transition-colors",
                    onclick: move |_| on_close.call(()),
                    Icon { name: IconName::X, class: "h-6 w-6".to_string() }
                }
            }
            if let Some(message) = error.clone() {
                div { class: "mb-6 p-4 bg-red-50 border border-red-200 rounded-md",
                    div { class: "flex",
                        Icon { name: IconName::AlertCircle, class: "h-5 w-5 text-red-400".to_string() }
                        div { class: "ml-3",
                            h3 { class: "text-sm font-medium text-red-800", "Error" }
                            p { class: "mt-1 text-sm text-red-700", "{message}" }
                        }
                    }
                }
            }
            form {
                class: "space-y-6",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    on_save.call(());
                },
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    div { class: "lg:col-span-2",
                        h4 { class: "text-lg font-medium text-gray-900 flex items-center mb-4",
                            Icon { name: IconName::Info, class: "h-5 w-5 mr-2".to_string() }
                            "Basic Information"
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "title", "Event Title *" }
                        input {
                            r#type: "text",
                            id: "title",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "Enter event title",
                            value: "{form_data.title}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.title = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "categoryId", "Category *" }
                        select {
                            id: "categoryId",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            value: "{form_data.category_id}",
                            onchange: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.category_id = evt.value();
                                on_form_data.call(next);
                            } },
                            option { value: "", "Select a category" }
                            for category in categories.iter() {
                                option { value: "{category.id}", "{category.name}" }
                            }
                        }
                    }
                    div { class: "lg:col-span-2",
                        label { class: "block text-sm font-medium text-gray-700", r#for: "description", "Description *" }
                        textarea {
                            id: "description",
                            rows: 3,
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "Brief description of the event",
                            value: "{form_data.description}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.description = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                    div { class: "lg:col-span-2",
                        label { class: "block text-sm font-medium text-gray-700", r#for: "detailedDescription", "Detailed Description" }
                        textarea {
                            id: "detailedDescription",
                            rows: 5,
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "Detailed event description, agenda, and special notes",
                            value: "{form_data.detailed_description}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.detailed_description = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    div { class: "lg:col-span-2",
                        h4 { class: "text-lg font-medium text-gray-900 flex items-center mb-4",
                            Icon { name: IconName::Calendar, class: "h-5 w-5 mr-2".to_string() }
                            "Schedule & Venue"
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "venueId", "Venue *" }
                        select {
                            id: "venueId",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            value: "{form_data.venue_id}",
                            onchange: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.venue_id = evt.value();
                                on_form_data.call(next);
                            } },
                            option { value: "", "Select a venue" }
                            for venue in venues.iter() {
                                option { value: "{venue.id}", "{venue_option_label(venue)}" }
                            }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "timezone", "Timezone" }
                        select {
                            id: "timezone",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            value: "{form_data.timezone}",
                            onchange: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.timezone = evt.value();
                                on_form_data.call(next);
                            } },
                            option { value: "Asia/Taipei", "Asia/Taipei" }
                            option { value: "Asia/Hong_Kong", "Asia/Hong Kong" }
                            option { value: "Asia/Singapore", "Asia/Singapore" }
                            option { value: "UTC", "UTC" }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "startDatetime", "Start Date & Time *" }
                        input {
                            r#type: "datetime-local",
                            id: "startDatetime",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            value: "{form_data.start_datetime}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.start_datetime = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "endDatetime", "End Date & Time *" }
                        input {
                            r#type: "datetime-local",
                            id: "endDatetime",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            value: "{form_data.end_datetime}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.end_datetime = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    div { class: "lg:col-span-2",
                        h4 { class: "text-lg font-medium text-gray-900 flex items-center mb-4",
                            Icon { name: IconName::Users, class: "h-5 w-5 mr-2".to_string() }
                            "Capacity & Pricing"
                        }
                    }
                    NumberField {
                        id: "capacityMin",
                        label: "Minimum Capacity",
                        value: form_data.capacity_min.to_string(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.capacity_min = parse_u32_input(&value);
                            on_form_data.call(next);
                        } },
                    }
                    NumberField {
                        id: "capacityMax",
                        label: "Maximum Capacity *",
                        value: form_data.capacity_max.to_string(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.capacity_max = parse_u32_input(&value);
                            on_form_data.call(next);
                        } },
                    }
                    NumberField {
                        id: "pricePlatinum",
                        label: "Platinum Price (TWD)",
                        value: form_data.price_platinum.to_string(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.price_platinum = parse_f64_input(&value);
                            on_form_data.call(next);
                        } },
                    }
                    NumberField {
                        id: "priceDiamond",
                        label: "Diamond Price (TWD)",
                        value: form_data.price_diamond.to_string(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.price_diamond = parse_f64_input(&value);
                            on_form_data.call(next);
                        } },
                    }
                    NumberField {
                        id: "priceBlackCard",
                        label: "Black Card Price (TWD)",
                        value: form_data.price_black_card.to_string(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.price_black_card = parse_f64_input(&value);
                            on_form_data.call(next);
                        } },
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "Required Membership Tiers" }
                        div { class: "space-y-2",
                            for tier in ["Platinum", "Diamond", "Black Card"] {
                                label { class: "inline-flex items-center mr-4",
                                    input {
                                        r#type: "checkbox",
                                        checked: form_data.required_membership_tiers.iter().any(|item| item == tier),
                                        onchange: { let form = form.clone(); move |evt| {
                                            let mut next = (*form).clone();
                                            let current = next.required_membership_tiers.clone();
                                            next.required_membership_tiers = toggle_membership_tier(
                                                &current,
                                                tier,
                                                evt.checked(),
                                            );
                                            on_form_data.call(next);
                                        } },
                                    }
                                    span { class: "ml-2 text-sm text-gray-700", "{tier}" }
                                }
                            }
                        }
                    }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    div { class: "lg:col-span-2",
                        h4 { class: "text-lg font-medium text-gray-900 flex items-center mb-4",
                            Icon { name: IconName::Clock, class: "h-5 w-5 mr-2".to_string() }
                            "Event Details"
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "dressCode", "Dress Code" }
                        input {
                            r#type: "text",
                            id: "dressCode",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "e.g., Cocktail, Business formal",
                            value: "{form_data.dress_code}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.dress_code = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700", r#for: "language", "Language" }
                        select {
                            id: "language",
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            value: "{form_data.language}",
                            onchange: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.language = evt.value();
                                on_form_data.call(next);
                            } },
                            option { value: "Traditional Chinese", "Traditional Chinese" }
                            option { value: "English", "English" }
                            option { value: "Bilingual", "Bilingual" }
                        }
                    }
                    div { class: "lg:col-span-2",
                        label { class: "block text-sm font-medium text-gray-700", r#for: "inclusions", "Inclusions (one per line)" }
                        textarea {
                            id: "inclusions",
                            rows: 3,
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "Welcome drink\nMulti-course dinner\nPremium wine pairing",
                            value: "{inclusions}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.inclusions = lines_to_list(&evt.value());
                                on_form_data.call(next);
                            } },
                        }
                    }
                    div { class: "lg:col-span-2",
                        label { class: "block text-sm font-medium text-gray-700", r#for: "exclusions", "Exclusions (one per line)" }
                        textarea {
                            id: "exclusions",
                            rows: 3,
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "Transportation\nAdditional beverages\nGratuity",
                            value: "{exclusions}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.exclusions = lines_to_list(&evt.value());
                                on_form_data.call(next);
                            } },
                        }
                    }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    div { class: "lg:col-span-2",
                        h4 { class: "text-lg font-medium text-gray-900 flex items-center mb-4",
                            Icon { name: IconName::DollarSign, class: "h-5 w-5 mr-2".to_string() }
                            "Registration & Settings"
                        }
                    }
                    DatetimeField {
                        id: "registrationOpensAt",
                        label: "Registration Opens At",
                        value: form_data.registration_opens_at.clone(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.registration_opens_at = value;
                            on_form_data.call(next);
                        } },
                    }
                    DatetimeField {
                        id: "registrationClosesAt",
                        label: "Registration Closes At",
                        value: form_data.registration_closes_at.clone(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.registration_closes_at = value;
                            on_form_data.call(next);
                        } },
                    }
                    DatetimeField {
                        id: "cancellationDeadline",
                        label: "Cancellation Deadline",
                        value: form_data.cancellation_deadline.clone(),
                        on_input: { let form = form.clone(); move |value: String| {
                            let mut next = (*form).clone();
                            next.cancellation_deadline = value;
                            on_form_data.call(next);
                        } },
                    }
                    div { class: "space-y-4",
                        CheckboxField {
                            label: "Enable Waitlist",
                            checked: form_data.waitlist_enabled,
                            on_change: { let form = form.clone(); move |checked: bool| {
                                let mut next = (*form).clone();
                                next.waitlist_enabled = checked;
                                on_form_data.call(next);
                            } },
                        }
                        CheckboxField {
                            label: "Auto-approve Registrations",
                            checked: form_data.auto_approval,
                            on_change: { let form = form.clone(); move |checked: bool| {
                                let mut next = (*form).clone();
                                next.auto_approval = checked;
                                on_form_data.call(next);
                            } },
                        }
                        CheckboxField {
                            label: "Require User Verification",
                            checked: form_data.required_verification,
                            on_change: { let form = form.clone(); move |checked: bool| {
                                let mut next = (*form).clone();
                                next.required_verification = checked;
                                on_form_data.call(next);
                            } },
                        }
                    }
                    div { class: "lg:col-span-2",
                        label { class: "block text-sm font-medium text-gray-700", r#for: "internalNotes", "Internal Notes (Admin Only)" }
                        textarea {
                            id: "internalNotes",
                            rows: 3,
                            class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                            placeholder: "Internal notes for admin reference",
                            value: "{form_data.internal_notes}",
                            oninput: { let form = form.clone(); move |evt| {
                                let mut next = (*form).clone();
                                next.internal_notes = evt.value();
                                on_form_data.call(next);
                            } },
                        }
                    }
                }
                div { class: "flex items-center justify-end space-x-4 pt-6 border-t",
                    button {
                        r#type: "button",
                        class: "px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        r#type: "submit",
                        class: "inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed",
                        disabled: save_busy,
                        Icon { name: IconName::Save, class: "h-4 w-4 mr-2".to_string() }
                        "{save_label}"
                    }
                }
            }
        }
    }
}

#[component]
fn NumberField(
    id: String,
    label: String,
    value: String,
    on_input: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-medium text-gray-700", r#for: "{id}", "{label}" }
            input {
                r#type: "number",
                id: "{id}",
                class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                value: "{value}",
                oninput: move |evt| on_input.call(evt.value()),
            }
        }
    }
}

#[component]
fn DatetimeField(
    id: String,
    label: String,
    value: String,
    on_input: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-medium text-gray-700", r#for: "{id}", "{label}" }
            input {
                r#type: "datetime-local",
                id: "{id}",
                class: "mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                value: "{value}",
                oninput: move |evt| on_input.call(evt.value()),
            }
        }
    }
}

#[component]
fn CheckboxField(label: String, checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        label { class: "inline-flex items-center",
            input {
                r#type: "checkbox",
                checked,
                onchange: move |evt| on_change.call(evt.checked()),
            }
            span { class: "ml-2 text-sm text-gray-700", "{label}" }
        }
    }
}

#[component]
pub fn EventMedia(event_id: String) -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match event_management_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { EventMgmtGuardLoading {} },
        RouteGuard::Redirect(path) => {
            navigator.replace(path);
            rsx! {
                p { id: "event-media-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventMediaBody { session, event_id } },
    }
}

#[component]
fn EventMediaBody(session: Signal<Session>, event_id: String) -> Element {
    let navigator = use_navigator();
    let loading = use_signal(|| true);
    let error = use_signal(|| None::<String>);
    let event = use_signal(|| None::<ManagedEvent>);
    let media = use_signal(Vec::<MediaAsset>::new);
    let media_loading = use_signal(|| true);
    let mut media_error = use_signal(|| None::<String>);
    let mut tab = use_signal(|| MediaTab::All);
    let mut pending_images = use_signal(Vec::<PendingFile>::new);
    let mut pending_docs = use_signal(Vec::<PendingFile>::new);
    let mut upload_errors = use_signal(Vec::<String>::new);
    let action_loading = use_signal(|| None::<String>);
    let mut selected_media = use_signal(|| None::<MediaAsset>);
    let mut lightbox_presence = use_signal(|| Presence::Hidden);
    let image_files = use_hook(|| Rc::new(Cell::new(Vec::<UploadBytes>::new())));
    let doc_files = use_hook(|| Rc::new(Cell::new(Vec::<UploadBytes>::new())));
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let event_id = event_id.clone();
        move || {
            start_event_fetch(loading, error, event, event_id.clone());
        }
    });

    use_effect({
        let event_id = event_id.clone();
        let fetch_gen = fetch_gen.clone();
        move || {
            start_media_fetch(
                fetch_gen.clone(),
                media_loading,
                media_error,
                media,
                event_id.clone(),
                tab_media_kind(tab()),
            );
        }
    });

    let can_edit = permissions(&session().snapshot()).event_management;

    rsx! {
        EventMediaScreen {
            loading: loading(),
            error: error(),
            event: event(),
            tab: tab(),
            media: media(),
            media_loading: media_loading(),
            media_error: media_error(),
            pending_images: pending_images(),
            pending_docs: pending_docs(),
            upload_errors: upload_errors(),
            action_loading: action_loading(),
            selected_media: selected_media(),
            lightbox_presence: lightbox_presence(),
            can_edit,
            on_back: move |_| {
                navigator.push("/event-mgmt");
            },
            on_tab: move |next: MediaTab| tab.set(next),
            on_select_images: {
                let image_files = image_files.clone();
                move |evt: FormEvent| {
                    handle_file_pick(
                        evt,
                        MediaKind::Image,
                        pending_images().len() as u32,
                        MAX_IMAGE_FILES,
                        pending_images,
                        upload_errors,
                        image_files.clone(),
                    );
                }
            },
            on_select_docs: {
                let doc_files = doc_files.clone();
                move |evt: FormEvent| {
                    handle_file_pick(
                        evt,
                        MediaKind::Document,
                        pending_docs().len() as u32,
                        MAX_DOCUMENT_FILES,
                        pending_docs,
                        upload_errors,
                        doc_files.clone(),
                    );
                }
            },
            on_clear_images: {
                let image_files = image_files.clone();
                move |_| {
                    pending_images.set(Vec::new());
                    image_files.set(Vec::new());
                    upload_errors.set(Vec::new());
                }
            },
            on_clear_docs: {
                let doc_files = doc_files.clone();
                move |_| {
                    pending_docs.set(Vec::new());
                    doc_files.set(Vec::new());
                    upload_errors.set(Vec::new());
                }
            },
            on_upload_images: {
                let event_id = event_id.clone();
                let fetch_gen = fetch_gen.clone();
                let image_files = image_files.clone();
                move |_| {
                    start_upload(
                        event_id.clone(),
                        MediaKind::Image,
                        image_files.take(),
                        pending_images,
                        upload_errors,
                        action_loading,
                        fetch_gen.clone(),
                        media_loading,
                        media_error,
                        media,
                        tab(),
                    );
                }
            },
            on_upload_docs: {
                let event_id = event_id.clone();
                let fetch_gen = fetch_gen.clone();
                let doc_files = doc_files.clone();
                move |_| {
                    start_upload(
                        event_id.clone(),
                        MediaKind::Document,
                        doc_files.take(),
                        pending_docs,
                        upload_errors,
                        action_loading,
                        fetch_gen.clone(),
                        media_loading,
                        media_error,
                        media,
                        tab(),
                    );
                }
            },
            on_delete_media: {
                let fetch_gen = fetch_gen.clone();
                let event_id = event_id.clone();
                move |media_id: String| {
                    spawn({
                        let fetch_gen = fetch_gen.clone();
                        let event_id = event_id.clone();
                        async move {
                            match delete_media(&media_id).await {
                                Ok(()) => {
                                    selected_media.set(None);
                                    lightbox_presence.set(Presence::Hidden);
                                    start_media_fetch(
                                        fetch_gen,
                                        media_loading,
                                        media_error,
                                        media,
                                        event_id,
                                        tab_media_kind(tab()),
                                    );
                                }
                                Err(message) => media_error.set(Some(message)),
                            }
                        }
                    });
                }
            },
            on_open_lightbox: move |asset: MediaAsset| {
                selected_media.set(Some(asset));
                lightbox_presence.set(Presence::Entering);
            },
            on_close_lightbox: move |_| {
                selected_media.set(None);
                lightbox_presence.set(Presence::Exiting);
            },
            on_lightbox_end: move |_| {
                lightbox_presence.set(presence_after_animation_end(lightbox_presence()));
            },
        }
    }
}

fn start_event_fetch(
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut event: Signal<Option<ManagedEvent>>,
    event_id: String,
) {
    loading.set(true);
    error.set(None);
    spawn(async move {
        match fetch_managed_event(&event_id).await {
            Ok(fetched) => {
                event.set(Some(fetched));
                error.set(None);
            }
            Err(message) => {
                event.set(None);
                if message == EVENT_FETCH_FALLBACK {
                    error.set(Some(EVENT_LOAD_ERROR.to_string()));
                } else {
                    error.set(Some(message));
                }
            }
        }
        loading.set(false);
    });
}

fn start_media_fetch(
    fetch_gen: Rc<Cell<u32>>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut media: Signal<Vec<MediaAsset>>,
    event_id: String,
    kind: Option<MediaKind>,
) {
    let request_id = fetch_gen.get() + 1;
    fetch_gen.set(request_id);
    loading.set(true);
    error.set(None);
    spawn(async move {
        let result = fetch_event_media(&event_id, kind).await;
        if fetch_gen.get() != request_id {
            return;
        }
        match result {
            Ok(fetched) => media.set(fetched),
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn handle_file_pick(
    evt: FormEvent,
    kind: MediaKind,
    existing: u32,
    max_files: u32,
    mut pending: Signal<Vec<PendingFile>>,
    mut upload_errors: Signal<Vec<String>>,
    store: Rc<Cell<Vec<UploadBytes>>>,
) {
    let files = evt.files();
    let candidates: Vec<FileCandidate> = files
        .iter()
        .map(|file| FileCandidate {
            name: file.name(),
            mime: file.content_type().unwrap_or_default(),
            size: file.size(),
        })
        .collect();
    let (accepted, errors) =
        select_valid_files(&candidates, existing, max_files, kind, MAX_SIZE_MB);
    upload_errors.set(errors);
    if accepted.is_empty() {
        return;
    }
    let accepted_names: Vec<String> = accepted.iter().map(|file| file.name.clone()).collect();
    let mut next_pending = pending();
    for file in accepted {
        next_pending.push(PendingFile {
            name: file.name,
            size: file.size,
            mime: file.mime,
            kind,
        });
    }
    pending.set(next_pending);
    spawn(async move {
        let mut stored = store.take();
        for file in files {
            if !accepted_names.iter().any(|name| name == &file.name()) {
                continue;
            }
            if let Ok(bytes) = file.read_bytes().await {
                stored.push(UploadBytes {
                    filename: file.name(),
                    mime_type: file.content_type().unwrap_or_default(),
                    bytes: bytes.to_vec(),
                });
            }
        }
        store.set(stored);
    });
}

fn start_upload(
    event_id: String,
    kind: MediaKind,
    files: Vec<UploadBytes>,
    mut pending: Signal<Vec<PendingFile>>,
    mut upload_errors: Signal<Vec<String>>,
    mut action_loading: Signal<Option<String>>,
    fetch_gen: Rc<Cell<u32>>,
    media_loading: Signal<bool>,
    media_error: Signal<Option<String>>,
    media: Signal<Vec<MediaAsset>>,
    tab: MediaTab,
) {
    if files.is_empty() || action_is(action_loading().as_deref(), UPLOAD_ACTION) {
        return;
    }
    action_loading.set(Some(UPLOAD_ACTION.to_string()));
    spawn(async move {
        let result = upload_event_media(&event_id, kind, &files).await;
        action_loading.set(None);
        match result {
            Ok(_) => {
                pending.set(Vec::new());
                upload_errors.set(Vec::new());
                start_media_fetch(
                    fetch_gen,
                    media_loading,
                    media_error,
                    media,
                    event_id,
                    tab_media_kind(tab),
                );
            }
            Err(message) => {
                let mut errors = upload_errors();
                errors.push(message);
                upload_errors.set(errors);
            }
        }
    });
}

#[component]
pub fn EventMediaScreen(
    loading: bool,
    error: Option<String>,
    event: Option<ManagedEvent>,
    tab: MediaTab,
    media: Vec<MediaAsset>,
    media_loading: bool,
    media_error: Option<String>,
    pending_images: Vec<PendingFile>,
    pending_docs: Vec<PendingFile>,
    upload_errors: Vec<String>,
    action_loading: Option<String>,
    selected_media: Option<MediaAsset>,
    lightbox_presence: Presence,
    can_edit: bool,
    #[props(default)] on_back: EventHandler<()>,
    #[props(default)] on_tab: EventHandler<MediaTab>,
    #[props(default)] on_select_images: EventHandler<FormEvent>,
    #[props(default)] on_select_docs: EventHandler<FormEvent>,
    #[props(default)] on_clear_images: EventHandler<()>,
    #[props(default)] on_clear_docs: EventHandler<()>,
    #[props(default)] on_upload_images: EventHandler<()>,
    #[props(default)] on_upload_docs: EventHandler<()>,
    #[props(default)] on_delete_media: EventHandler<String>,
    #[props(default)] on_open_lightbox: EventHandler<MediaAsset>,
    #[props(default)] on_close_lightbox: EventHandler<()>,
    #[props(default)] on_lightbox_end: EventHandler<()>,
) -> Element {
    if loading {
        return rsx! {
            div { id: "event-media-loading", class: "min-h-screen bg-luxury-midnight-black py-12",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div { class: "flex items-center justify-center h-64",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-luxury-gold" }
                    }
                }
            }
        };
    }

    let Some(event) = event else {
        let message = error.unwrap_or_else(|| EVENT_NOT_FOUND.to_string());
        return rsx! {
            div { id: "event-media-error", class: "min-h-screen bg-luxury-midnight-black py-12",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div { class: "luxury-glass p-8 rounded-2xl text-center",
                        h2 { class: "text-2xl font-luxury font-bold text-luxury-gold mb-4", "Event Not Found" }
                        p { class: "text-luxury-platinum/80 mb-6", "{message}" }
                        button {
                            r#type: "button",
                            class: "luxury-button-primary",
                            onclick: move |_| on_back.call(()),
                            Icon { name: IconName::ArrowLeft, class: "h-4 w-4 mr-2".to_string() }
                            "Back to Events"
                        }
                    }
                }
            }
        };
    };

    let uploading = action_is(action_loading.as_deref(), UPLOAD_ACTION);
    let when = format_event_datetime(&event.start_datetime);
    let venue = if event.venue_name.is_empty() {
        "TBD".to_string()
    } else {
        event.venue_name.clone()
    };
    let status_class = media_status_badge_class(&event.status);
    let status_text = status_label(&event.status);
    let lightbox_open = selected_media.is_some() && presence_is_mounted(lightbox_presence);
    let lightbox_class = format!(
        "fixed inset-0 bg-luxury-midnight-black/90 backdrop-blur-sm z-50 flex items-center justify-center p-4 {}",
        presence_class(lightbox_presence, "hs-enter", "hs-exit")
    );

    rsx! {
        div { id: "event-media-page", class: "min-h-screen bg-luxury-midnight-black py-12",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "flex items-center justify-between mb-8",
                    div { class: "flex items-center space-x-4",
                        button {
                            r#type: "button",
                            class: "luxury-button-secondary flex items-center space-x-2",
                            onclick: move |_| on_back.call(()),
                            Icon { name: IconName::ArrowLeft, class: "h-4 w-4".to_string() }
                            span { "Back to Events" }
                        }
                        div {
                            h1 { class: "text-3xl font-luxury font-bold text-luxury-gold", "Media Management" }
                            p { class: "text-luxury-platinum/80 text-lg", "{event.title}" }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl mb-8",
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        div {
                            h3 { class: "text-sm font-medium text-luxury-platinum/60", "Event Date" }
                            p { class: "text-luxury-platinum", "{when}" }
                        }
                        div {
                            h3 { class: "text-sm font-medium text-luxury-platinum/60", "Venue" }
                            p { class: "text-luxury-platinum", "{venue}" }
                        }
                        div {
                            h3 { class: "text-sm font-medium text-luxury-platinum/60", "Status" }
                            span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {status_class}", "{status_text}" }
                        }
                    }
                }
                div { class: "flex space-x-1 mb-8",
                    for next in [MediaTab::All, MediaTab::Images, MediaTab::Documents] {
                        {
                            let active = tab == next;
                            let class = if active {
                                "flex items-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors bg-luxury-gold text-luxury-midnight-black"
                            } else {
                                "flex items-center space-x-2 px-6 py-3 rounded-lg font-medium transition-colors text-luxury-platinum hover:bg-luxury-gold/10"
                            };
                            let icon = match next {
                                MediaTab::All => IconName::Eye,
                                MediaTab::Images => IconName::Star,
                                MediaTab::Documents => IconName::Info,
                            };
                            rsx! {
                                button {
                                    r#type: "button",
                                    class: "{class}",
                                    onclick: move |_| on_tab.call(next),
                                    Icon { name: icon, class: "h-4 w-4".to_string() }
                                    span { "{next.label()}" }
                                }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl mb-8",
                    h2 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6 flex items-center",
                        Icon { name: IconName::Plus, class: "h-5 w-5 mr-2".to_string() }
                        "Upload New Media"
                    }
                    div { class: "grid grid-cols-1 lg:grid-cols-2 gap-8",
                        UploaderPanel {
                            title: "Event Images".to_string(),
                            hint: "Upload high-quality images for event galleries and marketing. Supported formats: JPEG, PNG, WebP, GIF. Max 10MB per file.".to_string(),
                            accept: "image/jpeg,image/png,image/webp,image/gif".to_string(),
                            kind_label: "Images".to_string(),
                            pending: pending_images.clone(),
                            uploading,
                            on_select: on_select_images,
                            on_clear: on_clear_images,
                            on_upload: on_upload_images,
                        }
                        UploaderPanel {
                            title: "Event Documents".to_string(),
                            hint: "Upload event-related documents like schedules, menus, or information packets. Supported formats: PDF, DOC, XLS. Max 10MB per file.".to_string(),
                            accept: ".pdf,.doc,.docx,.xls,.xlsx".to_string(),
                            kind_label: "Documents".to_string(),
                            pending: pending_docs.clone(),
                            uploading,
                            on_select: on_select_docs,
                            on_clear: on_clear_docs,
                            on_upload: on_upload_docs,
                        }
                    }
                    if !upload_errors.is_empty() {
                        div { class: "bg-red-500/10 border border-red-500/20 rounded-lg p-4 mt-4",
                            h4 { class: "text-red-400 font-medium mb-1", "Upload Errors" }
                            ul { class: "text-red-300 text-sm space-y-1",
                                for message in upload_errors.iter() {
                                    li { "• {message}" }
                                }
                            }
                        }
                    }
                }
                div { class: "luxury-glass p-6 rounded-2xl",
                    h2 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-6", "Current Media" }
                    if media_loading {
                        div { class: "flex items-center justify-center py-12",
                            p { class: "text-luxury-platinum", "Loading media..." }
                        }
                    } else if let Some(message) = media_error.clone() {
                        div { class: "text-center text-red-400",
                            p { "Failed to load media: {message}" }
                        }
                    } else if media.is_empty() {
                        div { id: "event-media-empty", class: "text-center text-luxury-platinum/60 py-8",
                            p { "No media files found" }
                        }
                    } else {
                        div { id: "event-media-gallery", class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4",
                            for asset in media.iter() {
                                {
                                    let asset = asset.clone();
                                    let size = format_file_size(asset.file_size);
                                    let created = format_media_date(&asset.created_at);
                                    let open = asset.clone();
                                    let delete_id = asset.id.clone();
                                    rsx! {
                                        div { class: "luxury-glass rounded-xl overflow-hidden group relative",
                                            div { class: "aspect-square relative",
                                                if asset.kind == MediaKind::Image && !asset.preview_url.is_empty() {
                                                    img {
                                                        src: "{asset.preview_url}",
                                                        alt: "{asset.original_filename}",
                                                        class: "w-full h-full object-cover cursor-pointer",
                                                        onclick: move |_| on_open_lightbox.call(open.clone()),
                                                    }
                                                } else {
                                                    div { class: "w-full h-full flex items-center justify-center bg-luxury-midnight-black/30",
                                                        Icon { name: IconName::Info, class: "h-12 w-12 text-luxury-gold".to_string() }
                                                    }
                                                }
                                                if can_edit {
                                                    button {
                                                        r#type: "button",
                                                        class: "absolute top-2 right-2 w-10 h-10 bg-red-500/20 rounded-full flex items-center justify-center text-red-400",
                                                        onclick: move |_| on_delete_media.call(delete_id.clone()),
                                                        Icon { name: IconName::Trash2, class: "h-5 w-5".to_string() }
                                                    }
                                                }
                                            }
                                            div { class: "p-3",
                                                h4 { class: "text-luxury-platinum font-medium text-sm truncate mb-1", "{asset.original_filename}" }
                                                div { class: "flex items-center justify-between text-xs text-luxury-platinum/60",
                                                    span { "{size}" }
                                                    span { "{created}" }
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
            if lightbox_open {
                if let Some(asset) = selected_media.clone() {
                    div {
                        class: "{lightbox_class}",
                        onanimationend: move |_| on_lightbox_end.call(()),
                        onclick: move |_| on_close_lightbox.call(()),
                        div { class: "max-w-5xl max-h-full flex flex-col",
                            div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-t-2xl",
                                h3 { class: "text-luxury-gold font-medium", "{asset.original_filename}" }
                                button {
                                    r#type: "button",
                                    onclick: move |_| on_close_lightbox.call(()),
                                    Icon { name: IconName::X, class: "h-6 w-6".to_string() }
                                }
                            }
                            if !asset.file_path.is_empty() {
                                img {
                                    src: "{asset.file_path}",
                                    alt: "{asset.original_filename}",
                                    class: "max-w-full max-h-[70vh] object-contain",
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
fn UploaderPanel(
    title: String,
    hint: String,
    accept: String,
    kind_label: String,
    pending: Vec<PendingFile>,
    uploading: bool,
    on_select: EventHandler<FormEvent>,
    on_clear: EventHandler<()>,
    on_upload: EventHandler<()>,
) -> Element {
    let count = pending.len();
    let total: u64 = pending.iter().map(|file| file.size).sum();
    let total_label = format!("Total size: {}", format_file_size(total));
    let upload_label = if count == 1 {
        "Upload 1 File".to_string()
    } else {
        format!("Upload {count} Files")
    };
    rsx! {
        div {
            h3 { class: "text-lg font-medium text-luxury-platinum mb-4", "{title}" }
            div { class: "border-dashed border-2 border-luxury-gold/30 rounded-lg p-6 bg-luxury-gold/5",
                h3 { class: "text-lg font-medium text-luxury-gold mb-2", "Upload {kind_label}" }
                p { class: "text-luxury-platinum/80 mb-4", "Drag and drop files here, or click to select" }
                input {
                    r#type: "file",
                    multiple: true,
                    accept: "{accept}",
                    onchange: move |evt| on_select.call(evt),
                }
            }
            p { class: "text-sm text-luxury-platinum/60 mt-2", "{hint}" }
            if !pending.is_empty() {
                div { class: "mt-4 space-y-3",
                    h4 { class: "text-lg font-medium text-luxury-gold", "Selected Files ({count})" }
                    for file in pending.iter() {
                        div { class: "flex items-center space-x-3 p-3 bg-luxury-midnight-black/30 rounded-lg",
                            p { class: "text-luxury-platinum font-medium truncate", "{file.name}" }
                            p { class: "text-luxury-platinum/60 text-sm", "{format_file_size(file.size)}" }
                        }
                    }
                    div { class: "flex items-center justify-between mt-4",
                        div { class: "text-luxury-platinum/80 text-sm", "{total_label}" }
                        div { class: "flex space-x-3",
                            button {
                                r#type: "button",
                                class: "px-4 py-2 border border-luxury-gold/30 text-luxury-platinum rounded-lg",
                                disabled: uploading,
                                onclick: move |_| on_clear.call(()),
                                "Clear All"
                            }
                            button {
                                r#type: "button",
                                class: "luxury-button px-6 py-2 disabled:opacity-50",
                                disabled: uploading || count == 0,
                                onclick: move |_| on_upload.call(()),
                                if uploading {
                                    span { "Uploading..." }
                                } else {
                                    span { "{upload_label}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
