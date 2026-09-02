use std::cell::Cell;
use std::rc::Rc;

use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session};
use crate::shell::{Presence, presence_class};
use crate::taxonomy::{
    CategoryForm, EventCategory, TaxonomyModal, Venue, VenueForm, category_form_from, close_modal,
    confirm_delete_id, delete_confirmation_message, event_management_guard, fetch_categories,
    fetch_venues, filter_categories, filter_venues, open_create, open_delete, open_edit,
    validate_category_form, validate_venue_form, venue_form_from,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum EventMgmtSection {
    Events,
    Categories,
    Venues,
}

#[component]
pub fn EventCategories() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match event_management_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            EventMgmtGuardLoading { id: "event-categories-guard-loading".to_string() }
        },
        RouteGuard::Redirect(path) => {
            navigator.replace(path);
            rsx! {
                p { id: "event-categories-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventCategoriesBody {} },
    }
}

#[component]
pub fn EventVenues() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match event_management_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! {
            EventMgmtGuardLoading { id: "event-venues-guard-loading".to_string() }
        },
        RouteGuard::Redirect(path) => {
            navigator.replace(path);
            rsx! {
                p { id: "event-venues-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { EventVenuesBody {} },
    }
}

#[component]
fn EventCategoriesBody() -> Element {
    let loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let categories = use_signal(Vec::<EventCategory>::new);
    let mut search = use_signal(String::new);
    let mut form = use_signal(CategoryForm::default);
    let mut form_error = use_signal(|| None::<String>);
    let mut selected = use_signal(|| None::<EventCategory>);
    let mut modal = use_signal(|| TaxonomyModal::None);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            start_categories_fetch(fetch_gen.clone(), loading, error, categories);
        }
    });

    rsx! {
        EventCategoriesScreen {
            can_manage: true,
            loading: loading(),
            error: error(),
            categories: categories(),
            search: search(),
            form: form(),
            form_error: form_error(),
            selected: selected(),
            modal: modal(),
            header_presence: Presence::Entering,
            on_search: move |value: String| search.set(value),
            on_open_create: move |_| {
                form.set(CategoryForm::default());
                form_error.set(None);
                selected.set(None);
                modal.set(open_create());
            },
            on_open_edit: move |category: EventCategory| {
                form.set(category_form_from(&category));
                form_error.set(None);
                selected.set(Some(category));
                modal.set(open_edit());
            },
            on_open_delete: move |category: EventCategory| {
                selected.set(Some(category));
                modal.set(open_delete());
            },
            on_form_name: move |value: String| {
                form.write().name = value;
            },
            on_close_modal: move |_| {
                modal.set(close_modal());
                form_error.set(None);
            },
            on_submit_form: move |_| {
                if let Err(message) = validate_category_form(&form()) {
                    form_error.set(Some(message));
                    return;
                }
                form_error.set(None);
                form.set(CategoryForm::default());
                selected.set(None);
                modal.set(close_modal());
            },
            on_confirm_delete: move |_| {
                if confirm_delete_id(selected().as_ref().map(|item| item.id.as_str())).is_none() {
                    return;
                }
                selected.set(None);
                modal.set(close_modal());
            },
            on_dismiss_error: move |_| error.set(None),
        }
    }
}

#[component]
fn EventVenuesBody() -> Element {
    let loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let venues = use_signal(Vec::<Venue>::new);
    let mut search = use_signal(String::new);
    let mut form = use_signal(VenueForm::default);
    let mut form_error = use_signal(|| None::<String>);
    let mut selected = use_signal(|| None::<Venue>);
    let mut modal = use_signal(|| TaxonomyModal::None);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            start_venues_fetch(fetch_gen.clone(), loading, error, venues);
        }
    });

    rsx! {
        EventVenuesScreen {
            can_manage: true,
            loading: loading(),
            error: error(),
            venues: venues(),
            search: search(),
            form: form(),
            form_error: form_error(),
            selected: selected(),
            modal: modal(),
            header_presence: Presence::Entering,
            on_search: move |value: String| search.set(value),
            on_open_create: move |_| {
                form.set(VenueForm::default());
                form_error.set(None);
                selected.set(None);
                modal.set(open_create());
            },
            on_open_edit: move |venue: Venue| {
                form.set(venue_form_from(&venue));
                form_error.set(None);
                selected.set(Some(venue));
                modal.set(open_edit());
            },
            on_open_delete: move |venue: Venue| {
                selected.set(Some(venue));
                modal.set(open_delete());
            },
            on_form_name: move |value: String| {
                form.write().name = value;
            },
            on_close_modal: move |_| {
                modal.set(close_modal());
                form_error.set(None);
            },
            on_submit_form: move |_| {
                if let Err(message) = validate_venue_form(&form()) {
                    form_error.set(Some(message));
                    return;
                }
                form_error.set(None);
                form.set(VenueForm::default());
                selected.set(None);
                modal.set(close_modal());
            },
            on_confirm_delete: move |_| {
                if confirm_delete_id(selected().as_ref().map(|item| item.id.as_str())).is_none() {
                    return;
                }
                selected.set(None);
                modal.set(close_modal());
            },
            on_dismiss_error: move |_| error.set(None),
        }
    }
}

fn start_categories_fetch(
    fetch_gen: Rc<Cell<u32>>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut categories: Signal<Vec<EventCategory>>,
) {
    let request_id = fetch_gen.get() + 1;
    fetch_gen.set(request_id);
    loading.set(true);
    error.set(None);
    spawn(async move {
        let result = fetch_categories().await;
        if fetch_gen.get() != request_id {
            return;
        }
        match result {
            Ok(fetched) => {
                categories.set(fetched);
                error.set(None);
            }
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn start_venues_fetch(
    fetch_gen: Rc<Cell<u32>>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut venues: Signal<Vec<Venue>>,
) {
    let request_id = fetch_gen.get() + 1;
    fetch_gen.set(request_id);
    loading.set(true);
    error.set(None);
    spawn(async move {
        let result = fetch_venues().await;
        if fetch_gen.get() != request_id {
            return;
        }
        match result {
            Ok(fetched) => {
                venues.set(fetched);
                error.set(None);
            }
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

#[component]
pub fn EventCategoriesScreen(
    can_manage: bool,
    loading: bool,
    error: Option<String>,
    categories: Vec<EventCategory>,
    search: String,
    form: CategoryForm,
    form_error: Option<String>,
    selected: Option<EventCategory>,
    modal: TaxonomyModal,
    header_presence: Presence,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_open_create: EventHandler<()>,
    #[props(default)] on_open_edit: EventHandler<EventCategory>,
    #[props(default)] on_open_delete: EventHandler<EventCategory>,
    #[props(default)] on_form_name: EventHandler<String>,
    #[props(default)] on_close_modal: EventHandler<()>,
    #[props(default)] on_submit_form: EventHandler<()>,
    #[props(default)] on_confirm_delete: EventHandler<()>,
    #[props(default)] on_dismiss_error: EventHandler<()>,
) -> Element {
    if !can_manage {
        return rsx! { AccessDenied { id: "event-categories-denied".to_string() } };
    }

    let filtered = filter_categories(&categories, &search);
    let empty = !loading && filtered.is_empty();
    let show_create = matches!(modal, TaxonomyModal::Create | TaxonomyModal::Edit);
    let show_delete = modal == TaxonomyModal::Delete;
    let delete_name = selected.as_ref().map(|item| item.name.clone());
    let form_name = form.name.clone();
    let form_error_text = form_error.clone();
    let error_text = error.clone();

    rsx! {
        EventMgmtShell {
            id: "event-categories-page".to_string(),
            section: EventMgmtSection::Categories,
            header_presence,
            div { class: "space-y-6",
                Toolbar {
                    heading: "Categories".to_string(),
                    search_id: "event-categories-search".to_string(),
                    search,
                    search_placeholder: "Search categories...".to_string(),
                    create_id: "event-categories-new".to_string(),
                    create_label: "New Category".to_string(),
                    on_search,
                    on_open_create,
                }
                if let Some(message) = error_text {
                    ErrorBanner {
                        id: "event-categories-error".to_string(),
                        message,
                        on_dismiss_error,
                    }
                }
                div { class: "luxury-glass overflow-x-auto rounded-lg",
                    if loading {
                        div {
                            id: "event-categories-loading",
                            class: "p-6 text-center text-luxury-platinum/70",
                            "Loading..."
                        }
                    } else {
                        table {
                            id: "event-categories-table",
                            class: "w-full text-sm text-left text-luxury-platinum/80",
                            thead { class: "text-xs text-luxury-gold uppercase bg-luxury-gold/10",
                                tr {
                                    th { scope: "col", class: "px-6 py-3", "Name" }
                                    th { scope: "col", class: "px-6 py-3", "Description" }
                                    th { scope: "col", class: "px-6 py-3", "Icon" }
                                    th { scope: "col", class: "px-6 py-3 text-right", "Actions" }
                                }
                            }
                            tbody {
                                if empty {
                                    tr { id: "event-categories-empty",
                                        td {
                                            colspan: "4",
                                            class: "px-6 py-4 text-center text-luxury-platinum/70",
                                        }
                                    }
                                }
                                for category in filtered {
                                    CategoryRow {
                                        category,
                                        on_open_edit,
                                        on_open_delete,
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if show_create {
                NameFormModal {
                    id: "event-categories-form-modal".to_string(),
                    title: "Create New Category".to_string(),
                    placeholder: "Category Name".to_string(),
                    name: form_name,
                    form_error: form_error_text,
                    on_form_name,
                    on_close_modal,
                    on_submit_form,
                }
            }
            if show_delete {
                if let Some(name) = delete_name {
                    ConfirmDeleteModal {
                        id: "event-categories-delete-modal".to_string(),
                        title: "Delete Category".to_string(),
                        message: delete_confirmation_message(&name),
                        on_close_modal,
                        on_confirm_delete,
                    }
                }
            }
        }
    }
}

#[component]
pub fn EventVenuesScreen(
    can_manage: bool,
    loading: bool,
    error: Option<String>,
    venues: Vec<Venue>,
    search: String,
    form: VenueForm,
    form_error: Option<String>,
    selected: Option<Venue>,
    modal: TaxonomyModal,
    header_presence: Presence,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_open_create: EventHandler<()>,
    #[props(default)] on_open_edit: EventHandler<Venue>,
    #[props(default)] on_open_delete: EventHandler<Venue>,
    #[props(default)] on_form_name: EventHandler<String>,
    #[props(default)] on_close_modal: EventHandler<()>,
    #[props(default)] on_submit_form: EventHandler<()>,
    #[props(default)] on_confirm_delete: EventHandler<()>,
    #[props(default)] on_dismiss_error: EventHandler<()>,
) -> Element {
    if !can_manage {
        return rsx! { AccessDenied { id: "event-venues-denied".to_string() } };
    }

    let filtered = filter_venues(&venues, &search);
    let empty = !loading && filtered.is_empty();
    let show_create = matches!(modal, TaxonomyModal::Create | TaxonomyModal::Edit);
    let show_delete = modal == TaxonomyModal::Delete;
    let delete_name = selected.as_ref().map(|item| item.name.clone());
    let form_name = form.name.clone();
    let form_error_text = form_error.clone();
    let error_text = error.clone();

    rsx! {
        EventMgmtShell {
            id: "event-venues-page".to_string(),
            section: EventMgmtSection::Venues,
            header_presence,
            div { class: "space-y-6",
                Toolbar {
                    heading: "Venues".to_string(),
                    search_id: "event-venues-search".to_string(),
                    search,
                    search_placeholder: "Search venues...".to_string(),
                    create_id: "event-venues-new".to_string(),
                    create_label: "New Venue".to_string(),
                    on_search,
                    on_open_create,
                }
                if let Some(message) = error_text {
                    ErrorBanner {
                        id: "event-venues-error".to_string(),
                        message,
                        on_dismiss_error,
                    }
                }
                div { class: "luxury-glass overflow-x-auto rounded-lg",
                    if loading {
                        div {
                            id: "event-venues-loading",
                            class: "p-6 text-center text-luxury-platinum/70",
                            "Loading..."
                        }
                    } else {
                        table {
                            id: "event-venues-table",
                            class: "w-full text-sm text-left text-luxury-platinum/80",
                            thead { class: "text-xs text-luxury-gold uppercase bg-luxury-gold/10",
                                tr {
                                    th { scope: "col", class: "px-6 py-3", "Name" }
                                    th { scope: "col", class: "px-6 py-3", "City" }
                                    th { scope: "col", class: "px-6 py-3", "Address" }
                                    th { scope: "col", class: "px-6 py-3 text-right", "Actions" }
                                }
                            }
                            tbody {
                                if empty {
                                    tr { id: "event-venues-empty",
                                        td {
                                            colspan: "4",
                                            class: "px-6 py-4 text-center text-luxury-platinum/70",
                                        }
                                    }
                                }
                                for venue in filtered {
                                    VenueRow {
                                        venue,
                                        on_open_edit,
                                        on_open_delete,
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if show_create {
                NameFormModal {
                    id: "event-venues-form-modal".to_string(),
                    title: "Create New Venue".to_string(),
                    placeholder: "Venue Name".to_string(),
                    name: form_name,
                    form_error: form_error_text,
                    on_form_name,
                    on_close_modal,
                    on_submit_form,
                }
            }
            if show_delete {
                if let Some(name) = delete_name {
                    ConfirmDeleteModal {
                        id: "event-venues-delete-modal".to_string(),
                        title: "Delete Venue".to_string(),
                        message: delete_confirmation_message(&name),
                        on_close_modal,
                        on_confirm_delete,
                    }
                }
            }
        }
    }
}

#[component]
fn EventMgmtShell(
    id: String,
    section: EventMgmtSection,
    header_presence: Presence,
    children: Element,
) -> Element {
    let title = match section {
        EventMgmtSection::Events => "Event Dashboard",
        EventMgmtSection::Categories => "Category Management",
        EventMgmtSection::Venues => "Venue Management",
    };
    let header_class = format!(
        "mb-8 {}",
        presence_class(header_presence, "hs-enter", "hs-exit")
    );
    let nav_class = format!(
        "luxury-glass p-2 rounded-xl mb-8 {}",
        presence_class(header_presence, "hs-enter", "hs-exit")
    );
    let events_class = nav_item_class(section == EventMgmtSection::Events);
    let categories_class = nav_item_class(section == EventMgmtSection::Categories);
    let venues_class = nav_item_class(section == EventMgmtSection::Venues);

    rsx! {
        div {
            id,
            class: "p-4 sm:p-6 lg:p-8 bg-luxury-midnight-black min-h-screen text-luxury-platinum",
            div { class: "{header_class}",
                h1 { class: "text-3xl font-bold text-luxury-gold font-luxury tracking-wider", "{title}" }
                p { class: "text-luxury-platinum/60 mt-1",
                    "Manage all aspects of your events, categories, and venues."
                }
            }
            div { class: "{nav_class}",
                nav { class: "flex items-center space-x-2 sm:space-x-4",
                    a {
                        id: "event-mgmt-nav-events",
                        href: "/event-mgmt",
                        class: "{events_class}",
                        Icon { name: IconName::Briefcase, class: "w-4 h-4 mr-2".to_string() }
                        span { "Events" }
                    }
                    a {
                        id: "event-mgmt-nav-categories",
                        href: "/event-mgmt/categories",
                        class: "{categories_class}",
                        Icon { name: IconName::Filter, class: "w-4 h-4 mr-2".to_string() }
                        span { "Categories" }
                    }
                    a {
                        id: "event-mgmt-nav-venues",
                        href: "/event-mgmt/venues",
                        class: "{venues_class}",
                        Icon { name: IconName::MapPin, class: "w-4 h-4 mr-2".to_string() }
                        span { "Venues" }
                    }
                }
            }
            main { {children} }
        }
    }
}

fn nav_item_class(active: bool) -> &'static str {
    if active {
        "flex-1 sm:flex-initial flex items-center justify-center px-3 py-2 rounded-lg transition-all duration-300 text-sm sm:text-base font-medium bg-luxury-gold/20 text-luxury-gold shadow-inner"
    } else {
        "flex-1 sm:flex-initial flex items-center justify-center px-3 py-2 rounded-lg transition-all duration-300 text-sm sm:text-base font-medium text-luxury-platinum/70 hover:bg-luxury-gold/10 hover:text-luxury-platinum"
    }
}

#[component]
fn Toolbar(
    heading: String,
    search_id: String,
    search: String,
    search_placeholder: String,
    create_id: String,
    create_label: String,
    on_search: EventHandler<String>,
    on_open_create: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "luxury-glass p-4 rounded-lg",
            div { class: "flex flex-col md:flex-row justify-between items-center mb-4",
                h2 { class: "text-xl font-bold text-luxury-platinum mb-4 md:mb-0", "{heading}" }
                div { class: "flex items-center gap-4 w-full md:w-auto",
                    div { class: "relative flex-grow",
                        Icon {
                            name: IconName::Search,
                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                        }
                        input {
                            id: "{search_id}",
                            r#type: "text",
                            placeholder: "{search_placeholder}",
                            value: "{search}",
                            class: "luxury-input w-full pl-10 bg-white/10 border border-white/20 rounded-lg text-luxury-platinum placeholder:text-luxury-platinum/50 py-2 pr-3",
                            oninput: move |evt| on_search.call(evt.value()),
                        }
                    }
                    button {
                        id: "{create_id}",
                        r#type: "button",
                        class: "luxury-button-primary luxury-button inline-flex items-center",
                        onclick: move |_| on_open_create.call(()),
                        Icon { name: IconName::Plus, class: "h-4 w-4 mr-2".to_string() }
                        "{create_label}"
                    }
                }
            }
        }
    }
}

#[component]
fn ErrorBanner(id: String, message: String, on_dismiss_error: EventHandler<()>) -> Element {
    rsx! {
        div {
            id,
            class: "bg-red-900/50 text-red-200 border border-red-700 p-4 rounded-lg",
            onclick: move |_| on_dismiss_error.call(()),
            "{message}"
        }
    }
}

#[component]
fn CategoryRow(
    category: EventCategory,
    on_open_edit: EventHandler<EventCategory>,
    on_open_delete: EventHandler<EventCategory>,
) -> Element {
    let name = category.name.clone();
    let description = category.description.clone();
    let icon = category.icon.clone();
    let edit_item = category.clone();
    let delete_item = category.clone();
    let row_id = format!("event-categories-row-{}", category.id);
    rsx! {
        tr { id: "{row_id}", class: "border-b border-luxury-gold/10 hover:bg-luxury-gold/5",
            td { class: "px-6 py-4 font-medium text-white", "{name}" }
            td { class: "px-6 py-4", "{description}" }
            td { class: "px-6 py-4", "{icon}" }
            td { class: "px-6 py-4 text-right space-x-2",
                button {
                    id: "event-categories-edit-{category.id}",
                    r#type: "button",
                    class: "luxury-button-icon inline-flex items-center text-luxury-gold",
                    onclick: move |_| on_open_edit.call(edit_item.clone()),
                    Icon { name: IconName::Edit, class: "h-4 w-4".to_string() }
                }
                button {
                    id: "event-categories-delete-{category.id}",
                    r#type: "button",
                    class: "luxury-button-icon-danger inline-flex items-center text-red-400",
                    onclick: move |_| on_open_delete.call(delete_item.clone()),
                    Icon { name: IconName::Trash2, class: "h-4 w-4".to_string() }
                }
            }
        }
    }
}

#[component]
fn VenueRow(
    venue: Venue,
    on_open_edit: EventHandler<Venue>,
    on_open_delete: EventHandler<Venue>,
) -> Element {
    let name = venue.name.clone();
    let city = venue.city.clone();
    let address = venue.address.clone();
    let edit_item = venue.clone();
    let delete_item = venue.clone();
    let row_id = format!("event-venues-row-{}", venue.id);
    rsx! {
        tr { id: "{row_id}", class: "border-b border-luxury-gold/10 hover:bg-luxury-gold/5",
            td { class: "px-6 py-4 font-medium text-white", "{name}" }
            td { class: "px-6 py-4", "{city}" }
            td { class: "px-6 py-4", "{address}" }
            td { class: "px-6 py-4 text-right space-x-2",
                button {
                    id: "event-venues-edit-{venue.id}",
                    r#type: "button",
                    class: "luxury-button-icon inline-flex items-center text-luxury-gold",
                    onclick: move |_| on_open_edit.call(edit_item.clone()),
                    Icon { name: IconName::Edit, class: "h-4 w-4".to_string() }
                }
                button {
                    id: "event-venues-delete-{venue.id}",
                    r#type: "button",
                    class: "luxury-button-icon-danger inline-flex items-center text-red-400",
                    onclick: move |_| on_open_delete.call(delete_item.clone()),
                    Icon { name: IconName::Trash2, class: "h-4 w-4".to_string() }
                }
            }
        }
    }
}

#[component]
fn NameFormModal(
    id: String,
    title: String,
    placeholder: String,
    name: String,
    form_error: Option<String>,
    on_form_name: EventHandler<String>,
    on_close_modal: EventHandler<()>,
    on_submit_form: EventHandler<()>,
) -> Element {
    let error_id = format!("{id}-error");
    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50",
            div {
                id,
                class: "luxury-glass p-6 rounded-lg text-center w-full max-w-md hs-enter",
                h3 { class: "text-lg font-bold text-luxury-gold mb-4", "{title}" }
                input {
                    r#type: "text",
                    placeholder: "{placeholder}",
                    value: "{name}",
                    class: "luxury-input w-full mb-4 bg-white/10 border border-white/20 rounded-lg text-luxury-platinum placeholder:text-luxury-platinum/50 px-3 py-2",
                    oninput: move |evt| on_form_name.call(evt.value()),
                }
                if let Some(message) = form_error {
                    p { id: "{error_id}", class: "text-red-300 text-sm mb-4", "{message}" }
                }
                div { class: "mt-6 space-x-4",
                    button {
                        r#type: "button",
                        class: "luxury-button-secondary luxury-button-outline",
                        onclick: move |_| on_close_modal.call(()),
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        class: "luxury-button-primary luxury-button",
                        onclick: move |_| on_submit_form.call(()),
                        "Create"
                    }
                }
            }
        }
    }
}

#[component]
fn ConfirmDeleteModal(
    id: String,
    title: String,
    message: String,
    on_close_modal: EventHandler<()>,
    on_confirm_delete: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50",
            div {
                id,
                class: "luxury-glass p-6 rounded-lg text-center hs-enter",
                h3 { class: "text-lg font-bold text-luxury-gold mb-4", "{title}" }
                p { "{message}" }
                div { class: "mt-6 space-x-4",
                    button {
                        r#type: "button",
                        class: "luxury-button-secondary luxury-button-outline",
                        onclick: move |_| on_close_modal.call(()),
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        class: "luxury-button-danger px-8 py-3 bg-red-700 text-white font-medium rounded-lg",
                        onclick: move |_| on_confirm_delete.call(()),
                        "Delete"
                    }
                }
            }
        }
    }
}

#[component]
fn AccessDenied(id: String) -> Element {
    rsx! {
        div { id, class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center text-luxury-platinum",
            "Access Denied"
        }
    }
}

#[component]
fn EventMgmtGuardLoading(id: String) -> Element {
    rsx! {
        div {
            id,
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center hs-enter",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                p { class: "text-luxury-platinum", "驗證存取權限中..." }
            }
        }
    }
}
