#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::eventmgmt::{
    EventCategory, EventFormData, EventMgmtFilters, EventMgmtModal, EventVenue, EventsPagination,
    ManagedEvent, MediaAsset, MediaKind, MediaTab, PAGE_LIMIT,
};
use hesocial_frontend::pages::eventmgmt::{EventMediaScreen, EventMgmtScreen};
use hesocial_frontend::shell::Presence;
use wasm_bindgen_test::wasm_bindgen_test;

fn sample_event() -> ManagedEvent {
    ManagedEvent {
        id: "2".to_string(),
        title: "Autumn Yacht Social".to_string(),
        description: "Sunset cruise".to_string(),
        start_datetime: "2026-10-10T09:00:00.000Z".to_string(),
        end_datetime: "2026-10-10T12:00:00.000Z".to_string(),
        venue_name: "Keelung Luxury Yacht".to_string(),
        price_platinum: 18000.0,
        price_diamond: 28000.0,
        currency: "TWD".to_string(),
        status: "published".to_string(),
        approval_status: "approved".to_string(),
        ..ManagedEvent::default()
    }
}

fn draft_event() -> ManagedEvent {
    ManagedEvent {
        id: "3".to_string(),
        title: "Private Dinner".to_string(),
        start_datetime: "2026-11-01T18:00:00.000Z".to_string(),
        venue_name: "Taipei Residence".to_string(),
        status: "draft".to_string(),
        approval_status: "pending".to_string(),
        currency: "TWD".to_string(),
        ..ManagedEvent::default()
    }
}

fn sample_category() -> EventCategory {
    EventCategory {
        id: "4".to_string(),
        name: "遊艇派對".to_string(),
        slug: "yacht".to_string(),
    }
}

fn sample_venue() -> EventVenue {
    EventVenue {
        id: "8".to_string(),
        name: "Keelung Luxury Yacht".to_string(),
        city: "Keelung".to_string(),
        capacity_max: 40,
    }
}

fn sample_media() -> MediaAsset {
    MediaAsset {
        id: "m1".to_string(),
        kind: MediaKind::Image,
        file_path: "https://media.ahexagram.com/events/hero-original.jpg".to_string(),
        preview_url: "https://media.ahexagram.com/events/hero-medium.jpg".to_string(),
        original_filename: "hero.jpg".to_string(),
        file_size: 2048,
        mime_type: "image/jpeg".to_string(),
        uploaded_by: "1".to_string(),
        created_at: "2026-08-01T00:00:00.000Z".to_string(),
    }
}

#[component]
fn MgmtAt(
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
) -> Element {
    rsx! {
        EventMgmtScreen {
            can_manage,
            can_delete,
            loading,
            error,
            events,
            categories,
            venues,
            filters,
            pagination,
            selected_event,
            form_data,
            modal,
            form_presence,
            delete_presence,
            action_loading,
        }
    }
}

fn render_mgmt(
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
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        MgmtAt,
        MgmtAtProps {
            can_manage,
            can_delete,
            loading,
            error,
            events,
            categories,
            venues,
            filters,
            pagination,
            selected_event,
            form_data,
            modal,
            form_presence,
            delete_presence,
            action_loading,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn render_default(loading: bool, error: Option<String>, events: Vec<ManagedEvent>) -> String {
    let total = events.len() as u32;
    render_mgmt(
        true,
        false,
        loading,
        error,
        events,
        vec![sample_category()],
        vec![sample_venue()],
        EventMgmtFilters::list_default(),
        EventsPagination {
            page: 1,
            limit: PAGE_LIMIT,
            total,
            total_pages: 1,
        },
        None,
        EventFormData::default(),
        EventMgmtModal::None,
        Presence::Hidden,
        Presence::Hidden,
        None,
    )
}

#[component]
fn MediaAt(
    loading: bool,
    error: Option<String>,
    event: Option<ManagedEvent>,
    tab: MediaTab,
    media: Vec<MediaAsset>,
    media_loading: bool,
    media_error: Option<String>,
    can_edit: bool,
) -> Element {
    rsx! {
        EventMediaScreen {
            loading,
            error,
            event,
            tab,
            media,
            media_loading,
            media_error,
            pending_images: Vec::new(),
            pending_docs: Vec::new(),
            upload_errors: Vec::new(),
            action_loading: None,
            selected_media: None,
            lightbox_presence: Presence::Hidden,
            can_edit,
        }
    }
}

fn render_media(
    loading: bool,
    error: Option<String>,
    event: Option<ManagedEvent>,
    tab: MediaTab,
    media: Vec<MediaAsset>,
    media_loading: bool,
    media_error: Option<String>,
    can_edit: bool,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        MediaAt,
        MediaAtProps {
            loading,
            error,
            event,
            tab,
            media,
            media_loading,
            media_error,
            can_edit,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn event_mgmt_denied_copy() {
    let html = render_mgmt(
        false,
        false,
        false,
        None,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        EventMgmtFilters::list_default(),
        EventsPagination::default(),
        None,
        EventFormData::default(),
        EventMgmtModal::None,
        Presence::Hidden,
        Presence::Hidden,
        None,
    );
    assert!(
        html.contains("id=\"event-mgmt-denied\""),
        "denied id missing: {html}"
    );
    assert!(
        html.contains("Access Denied"),
        "denied title missing: {html}"
    );
    assert!(
        html.contains("You need admin privileges to access this page."),
        "denied body missing: {html}"
    );
    assert!(
        !html.contains("Event Dashboard"),
        "heading must not render when denied: {html}"
    );
}

#[wasm_bindgen_test]
fn event_mgmt_loading_copy() {
    let html = render_default(true, None, Vec::new());
    assert!(
        html.contains("id=\"event-mgmt-loading\""),
        "loading id missing: {html}"
    );
    assert!(
        html.contains("Loading events..."),
        "loading copy missing: {html}"
    );
    assert!(
        html.contains("Event Dashboard"),
        "heading missing while loading: {html}"
    );
    assert!(html.contains("Create Event"), "create copy missing: {html}");
    assert!(
        html.contains("Search by title..."),
        "search placeholder missing: {html}"
    );
    assert!(
        !html.contains("id=\"event-mgmt-table\""),
        "table must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn event_mgmt_empty_table() {
    let html = render_default(false, None, Vec::new());
    assert!(
        html.contains("id=\"event-mgmt-empty\""),
        "empty id missing: {html}"
    );
    assert!(
        html.contains("No events found."),
        "empty copy missing: {html}"
    );
    assert!(html.contains("Event List"), "list heading missing: {html}");
    assert!(
        html.contains("All Statuses"),
        "status filter missing: {html}"
    );
    assert!(
        html.contains("All Categories"),
        "category filter missing: {html}"
    );
    assert!(html.contains("All Venues"), "venue filter missing: {html}");
    assert!(html.contains("遊艇派對"), "category option missing: {html}");
}

#[wasm_bindgen_test]
fn event_mgmt_populated_row() {
    let html = render_default(false, None, vec![sample_event(), draft_event()]);
    assert!(
        html.contains("id=\"event-mgmt-table\""),
        "table missing: {html}"
    );
    assert!(
        html.contains("Autumn Yacht Social"),
        "title missing: {html}"
    );
    assert!(
        html.contains("Keelung Luxury Yacht"),
        "venue missing: {html}"
    );
    assert!(html.contains("Plat"), "platinum price missing: {html}");
    assert!(html.contains("Dia"), "diamond price missing: {html}");
    assert!(html.contains("published"), "status missing: {html}");
    assert!(
        html.contains("id=\"event-mgmt-approve-3\""),
        "approve missing for draft: {html}"
    );
    assert!(
        !html.contains("id=\"event-mgmt-approve-2\""),
        "published event must not show approve: {html}"
    );
}

#[wasm_bindgen_test]
fn event_mgmt_error_banner() {
    let html = render_default(
        false,
        Some("Failed to load event data".to_string()),
        Vec::new(),
    );
    assert!(
        html.contains("id=\"event-mgmt-error\""),
        "error id missing: {html}"
    );
    assert!(
        html.contains("Failed to load event data"),
        "error copy missing: {html}"
    );
}

#[wasm_bindgen_test]
fn event_mgmt_create_form_and_delete_modal() {
    let html = render_mgmt(
        true,
        true,
        false,
        None,
        vec![sample_event()],
        vec![sample_category()],
        vec![sample_venue()],
        EventMgmtFilters::list_default(),
        EventsPagination {
            page: 1,
            limit: 10,
            total: 1,
            total_pages: 1,
        },
        Some(sample_event()),
        EventFormData::default(),
        EventMgmtModal::Form,
        Presence::Shown,
        Presence::Hidden,
        None,
    );
    assert!(html.contains("Edit Event"), "edit title missing: {html}");
    assert!(
        html.contains("Event Title *"),
        "title label missing: {html}"
    );
    assert!(
        html.contains("Create New Event") || html.contains("Update Event"),
        "save missing: {html}"
    );
    assert!(
        html.contains("Basic Information"),
        "section missing: {html}"
    );
    assert!(
        html.contains("Schedule & Venue"),
        "schedule missing: {html}"
    );

    let delete_html = render_mgmt(
        true,
        true,
        false,
        None,
        vec![sample_event()],
        Vec::new(),
        Vec::new(),
        EventMgmtFilters::list_default(),
        EventsPagination::default(),
        Some(sample_event()),
        EventFormData::default(),
        EventMgmtModal::Delete,
        Presence::Hidden,
        Presence::Shown,
        None,
    );
    assert!(
        delete_html.contains("Delete Event"),
        "delete title missing: {delete_html}"
    );
    assert!(
        delete_html.contains("Are you sure you want to delete"),
        "delete copy missing: {delete_html}"
    );
}

#[wasm_bindgen_test]
fn event_media_loading_copy() {
    let html = render_media(
        true,
        None,
        None,
        MediaTab::All,
        Vec::new(),
        false,
        None,
        true,
    );
    assert!(
        html.contains("id=\"event-media-loading\""),
        "loading id missing: {html}"
    );
    assert!(
        !html.contains("Media Management"),
        "heading must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn event_media_not_found() {
    let html = render_media(
        false,
        Some("Error loading event".to_string()),
        None,
        MediaTab::All,
        Vec::new(),
        false,
        None,
        true,
    );
    assert!(
        html.contains("id=\"event-media-error\""),
        "error id missing: {html}"
    );
    assert!(
        html.contains("Event Not Found"),
        "not found title missing: {html}"
    );
    assert!(
        html.contains("Error loading event"),
        "error copy missing: {html}"
    );
    assert!(html.contains("Back to Events"), "back copy missing: {html}");
}

#[wasm_bindgen_test]
fn event_media_empty_gallery() {
    let html = render_media(
        false,
        None,
        Some(sample_event()),
        MediaTab::All,
        Vec::new(),
        false,
        None,
        true,
    );
    assert!(html.contains("Media Management"), "heading missing: {html}");
    assert!(
        html.contains("Autumn Yacht Social"),
        "title missing: {html}"
    );
    assert!(
        html.contains("Upload New Media"),
        "upload heading missing: {html}"
    );
    assert!(
        html.contains("Event Images"),
        "images heading missing: {html}"
    );
    assert!(
        html.contains("Event Documents"),
        "docs heading missing: {html}"
    );
    assert!(
        html.contains("Current Media"),
        "gallery heading missing: {html}"
    );
    assert!(
        html.contains("id=\"event-media-empty\""),
        "empty id missing: {html}"
    );
    assert!(
        html.contains("No media files found"),
        "empty copy missing: {html}"
    );
    assert!(html.contains("All Media"), "all tab missing: {html}");
    assert!(
        html.contains("TBD") || html.contains("Keelung Luxury Yacht"),
        "venue missing: {html}"
    );
}

#[wasm_bindgen_test]
fn event_media_populated_gallery_uses_api_urls() {
    let html = render_media(
        false,
        None,
        Some(sample_event()),
        MediaTab::Images,
        vec![sample_media()],
        false,
        None,
        true,
    );
    assert!(
        html.contains("id=\"event-media-gallery\""),
        "gallery missing: {html}"
    );
    assert!(html.contains("hero.jpg"), "filename missing: {html}");
    assert!(
        html.contains("https://media.ahexagram.com/events/hero-medium.jpg"),
        "preview url missing: {html}"
    );
    assert!(
        !html.contains("https://media.hesocial.com"),
        "must not invent default CDN: {html}"
    );
}

#[wasm_bindgen_test]
fn event_media_gallery_error() {
    let html = render_media(
        false,
        None,
        Some(sample_event()),
        MediaTab::All,
        Vec::new(),
        false,
        Some("Failed to get event media".to_string()),
        true,
    );
    assert!(
        html.contains("Failed to load media: Failed to get event media"),
        "media error missing: {html}"
    );
}
