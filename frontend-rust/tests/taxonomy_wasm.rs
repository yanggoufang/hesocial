#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::pages::taxonomy::{EventCategoriesScreen, EventVenuesScreen};
use hesocial_frontend::shell::Presence;
use hesocial_frontend::taxonomy::{
    CategoryForm, EventCategory, NAME_REQUIRED, TaxonomyModal, Venue, VenueForm,
};
use wasm_bindgen_test::wasm_bindgen_test;

fn sample_category() -> EventCategory {
    EventCategory {
        id: "3".to_string(),
        name: "私人晚宴".to_string(),
        description: "Small private dinners".to_string(),
        icon: "utensils".to_string(),
        created_at: "2024-03-05T10:00:00.000Z".to_string(),
    }
}

fn sample_venue() -> Venue {
    Venue {
        id: "9".to_string(),
        name: "基隆港遊艇會".to_string(),
        address: "No. 1 Harbour Rd".to_string(),
        city: "Keelung".to_string(),
        rating: Some(5.0),
        amenities: vec!["valet".to_string()],
        images: Vec::new(),
        created_at: "2024-01-15T00:00:00.000Z".to_string(),
        latitude: Some(25.128),
        longitude: Some(121.739),
    }
}

#[component]
fn CategoriesAt(
    can_manage: bool,
    loading: bool,
    error: Option<String>,
    categories: Vec<EventCategory>,
    search: String,
    form: CategoryForm,
    form_error: Option<String>,
    selected: Option<EventCategory>,
    modal: TaxonomyModal,
) -> Element {
    rsx! {
        EventCategoriesScreen {
            can_manage,
            loading,
            error,
            categories,
            search,
            form,
            form_error,
            selected,
            modal,
            header_presence: Presence::Shown,
        }
    }
}

#[component]
fn VenuesAt(
    can_manage: bool,
    loading: bool,
    error: Option<String>,
    venues: Vec<Venue>,
    search: String,
    form: VenueForm,
    form_error: Option<String>,
    selected: Option<Venue>,
    modal: TaxonomyModal,
) -> Element {
    rsx! {
        EventVenuesScreen {
            can_manage,
            loading,
            error,
            venues,
            search,
            form,
            form_error,
            selected,
            modal,
            header_presence: Presence::Shown,
        }
    }
}

fn render_categories(
    can_manage: bool,
    loading: bool,
    error: Option<String>,
    categories: Vec<EventCategory>,
    search: String,
    form: CategoryForm,
    form_error: Option<String>,
    selected: Option<EventCategory>,
    modal: TaxonomyModal,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        CategoriesAt,
        CategoriesAtProps {
            can_manage,
            loading,
            error,
            categories,
            search,
            form,
            form_error,
            selected,
            modal,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn render_venues(
    can_manage: bool,
    loading: bool,
    error: Option<String>,
    venues: Vec<Venue>,
    search: String,
    form: VenueForm,
    form_error: Option<String>,
    selected: Option<Venue>,
    modal: TaxonomyModal,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        VenuesAt,
        VenuesAtProps {
            can_manage,
            loading,
            error,
            venues,
            search,
            form,
            form_error,
            selected,
            modal,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

#[wasm_bindgen_test]
fn categories_denied_copy() {
    let html = render_categories(
        false,
        false,
        None,
        Vec::new(),
        String::new(),
        CategoryForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(
        html.contains("id=\"event-categories-denied\""),
        "denied id missing: {html}"
    );
    assert!(
        html.contains("Access Denied"),
        "denied copy missing: {html}"
    );
    assert!(
        !html.contains("Category Management"),
        "heading must not render when denied: {html}"
    );
}

#[wasm_bindgen_test]
fn categories_loading_copy() {
    let html = render_categories(
        true,
        true,
        None,
        Vec::new(),
        String::new(),
        CategoryForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(
        html.contains("id=\"event-categories-loading\""),
        "loading id missing: {html}"
    );
    assert!(html.contains("Loading..."), "loading copy missing: {html}");
    assert!(html.contains("Category Management"));
    assert!(html.contains("Manage all aspects of your events, categories, and venues."));
    assert!(html.contains("Search categories..."));
    assert!(html.contains("New Category"));
    assert!(!html.contains("id=\"event-categories-table\""));
}

#[wasm_bindgen_test]
fn categories_empty_state() {
    let html = render_categories(
        true,
        false,
        None,
        Vec::new(),
        String::new(),
        CategoryForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(html.contains("id=\"event-categories-table\""));
    assert!(html.contains("id=\"event-categories-empty\""));
    assert!(html.contains("Categories"));
    assert!(!html.contains("id=\"event-categories-loading\""));
}

#[wasm_bindgen_test]
fn categories_populated_and_error() {
    let html = render_categories(
        true,
        false,
        Some("Failed to fetch categories".to_string()),
        vec![sample_category()],
        String::new(),
        CategoryForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(html.contains("id=\"event-categories-row-3\""));
    assert!(html.contains("私人晚宴"));
    assert!(html.contains("Small private dinners"));
    assert!(html.contains("utensils"));
    assert!(html.contains("id=\"event-categories-error\""));
    assert!(html.contains("Failed to fetch categories"));
    assert!(!html.contains("id=\"event-categories-empty\""));
    assert!(!html.contains("id=\"event-categories-loading\""));
}

#[wasm_bindgen_test]
fn categories_create_and_delete_modals() {
    let create = render_categories(
        true,
        false,
        None,
        vec![sample_category()],
        String::new(),
        CategoryForm::default(),
        Some(NAME_REQUIRED.to_string()),
        None,
        TaxonomyModal::Create,
    );
    assert!(create.contains("id=\"event-categories-form-modal\""));
    assert!(create.contains("Create New Category"));
    assert!(create.contains("Category Name"));
    assert!(create.contains("Cancel"));
    assert!(create.contains("Create"));
    assert!(create.contains(NAME_REQUIRED));

    let delete = render_categories(
        true,
        false,
        None,
        vec![sample_category()],
        String::new(),
        CategoryForm::default(),
        None,
        Some(sample_category()),
        TaxonomyModal::Delete,
    );
    assert!(delete.contains("id=\"event-categories-delete-modal\""));
    assert!(delete.contains("Delete Category"));
    assert!(delete.contains("Are you sure you want to delete"));
    assert!(delete.contains("私人晚宴"));
    assert!(delete.contains("Delete"));
}

#[wasm_bindgen_test]
fn venues_denied_loading_empty_populated_error_and_modals() {
    let denied = render_venues(
        false,
        false,
        None,
        Vec::new(),
        String::new(),
        VenueForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(denied.contains("id=\"event-venues-denied\""));
    assert!(denied.contains("Access Denied"));

    let loading = render_venues(
        true,
        true,
        None,
        Vec::new(),
        String::new(),
        VenueForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(loading.contains("id=\"event-venues-loading\""));
    assert!(loading.contains("Loading..."));
    assert!(loading.contains("Venue Management"));
    assert!(loading.contains("Search venues..."));
    assert!(loading.contains("New Venue"));
    assert!(!loading.contains("id=\"event-venues-table\""));

    let empty = render_venues(
        true,
        false,
        None,
        Vec::new(),
        String::new(),
        VenueForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(empty.contains("id=\"event-venues-table\""));
    assert!(empty.contains("id=\"event-venues-empty\""));

    let populated = render_venues(
        true,
        false,
        Some("Failed to fetch venues".to_string()),
        vec![sample_venue()],
        String::new(),
        VenueForm::default(),
        None,
        None,
        TaxonomyModal::None,
    );
    assert!(populated.contains("id=\"event-venues-row-9\""));
    assert!(populated.contains("基隆港遊艇會"));
    assert!(populated.contains("Keelung"));
    assert!(populated.contains("No. 1 Harbour Rd"));
    assert!(populated.contains("id=\"event-venues-error\""));
    assert!(populated.contains("Failed to fetch venues"));

    let create = render_venues(
        true,
        false,
        None,
        vec![sample_venue()],
        String::new(),
        VenueForm::default(),
        None,
        None,
        TaxonomyModal::Create,
    );
    assert!(create.contains("id=\"event-venues-form-modal\""));
    assert!(create.contains("Create New Venue"));
    assert!(create.contains("Venue Name"));

    let delete = render_venues(
        true,
        false,
        None,
        vec![sample_venue()],
        String::new(),
        VenueForm::default(),
        None,
        Some(sample_venue()),
        TaxonomyModal::Delete,
    );
    assert!(delete.contains("id=\"event-venues-delete-modal\""));
    assert!(delete.contains("Delete Venue"));
    assert!(delete.contains("Are you sure you want to delete"));
    assert!(delete.contains("基隆港遊艇會"));
}
