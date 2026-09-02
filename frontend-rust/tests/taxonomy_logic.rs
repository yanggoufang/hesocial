#![cfg(not(target_arch = "wasm32"))]

use dioxus::prelude::*;
use hesocial_frontend::pages::taxonomy::{EventCategoriesScreen, EventVenuesScreen};
use hesocial_frontend::permissions::{AuthSnapshot, Role, RouteGuard};
use hesocial_frontend::shell::Presence;
use hesocial_frontend::taxonomy::{
    CATEGORIES_API_PATH, CATEGORIES_FETCH_FALLBACK, CategoryForm, EVENT_MANAGEMENT_FALLBACK,
    EventCategory, NAME_REQUIRED, NETWORK_ERROR, TaxonomyModal, VENUES_API_PATH,
    VENUES_FETCH_FALLBACK, Venue, VenueForm, category_form_from, close_modal, confirm_delete_id,
    delete_confirmation_message, event_management_guard, filter_categories, filter_venues,
    matches_search, open_create, open_delete, open_edit, parse_categories_response,
    parse_venues_response, validate_category_form, validate_venue_form, venue_form_from,
};

fn snapshot(role: Option<Role>, authenticated: bool) -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated: authenticated,
        role,
        ..AuthSnapshot::default()
    }
}

fn category_list_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": 3,
                "name": "私人晚宴",
                "description": "Small private dinners",
                "icon": "utensils",
                "createdAt": "2024-03-05T10:00:00.000Z"
            },
            {
                "id": "4",
                "name": "遊艇派對",
                "description": null,
                "icon": null,
                "created_at": "2024-04-01T00:00:00.000Z"
            }
        ]
    }"#
    .to_string()
}

fn venue_list_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": 9,
                "name": "基隆港遊艇會",
                "address": "No. 1 Harbour Rd",
                "city": "Keelung",
                "rating": 5,
                "amenities": ["valet", "lounge"],
                "images": ["a.jpg"],
                "createdAt": "2024-01-15T00:00:00.000Z",
                "coordinates": {"lat": 25.128, "lng": 121.739}
            },
            {
                "id": "10",
                "name": "台北藝術空間",
                "address": "Zhongshan N Rd",
                "city": "Taipei",
                "rating": null,
                "amenities": "[\"wifi\",\"bar\"]",
                "images": null,
                "created_at": "2024-02-01T00:00:00.000Z"
            }
        ]
    }"#
    .to_string()
}

#[test]
fn api_paths_match_backend_worker_routes() {
    assert_eq!(CATEGORIES_API_PATH, "/api/categories");
    assert_eq!(VENUES_API_PATH, "/api/venues");
}

#[test]
fn parse_categories_numeric_and_string_ids() {
    let rows = parse_categories_response(200, &category_list_body()).expect("ok");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "3");
    assert_eq!(rows[0].name, "私人晚宴");
    assert_eq!(rows[0].description, "Small private dinners");
    assert_eq!(rows[0].icon, "utensils");
    assert_eq!(rows[0].created_at, "2024-03-05T10:00:00.000Z");
    assert_eq!(rows[1].id, "4");
    assert_eq!(rows[1].name, "遊艇派對");
    assert_eq!(rows[1].description, "");
    assert_eq!(rows[1].icon, "");
    assert_eq!(rows[1].created_at, "2024-04-01T00:00:00.000Z");
}

#[test]
fn parse_categories_empty_array() {
    let rows = parse_categories_response(200, r#"{"success":true,"data":[]}"#).expect("ok");
    assert!(rows.is_empty());
}

#[test]
fn parse_categories_skips_rows_without_id() {
    let body = r#"{
        "success": true,
        "data": [
            {"name": "missing-id"},
            {"id": 1, "name": "kept"}
        ]
    }"#;
    let rows = parse_categories_response(200, body).expect("ok");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "kept");
}

#[test]
fn parse_categories_error_empty_and_malformed() {
    assert_eq!(
        parse_categories_response(200, "not-json").unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_categories_response(200, r#"{"success":false}"#).unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_categories_response(200, r#"{"success":false,"error":"nope"}"#).unwrap_err(),
        "nope"
    );
    assert_eq!(
        parse_categories_response(200, r#"{"success":true}"#).unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_categories_response(200, r#"{"success":true,"data":null}"#).unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_categories_response(0, "").unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
}

#[test]
fn parse_categories_401_and_403_use_backend_error_strings() {
    assert_eq!(
        parse_categories_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_categories_response(401, "").unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_categories_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
    assert_eq!(
        parse_categories_response(403, r#"{"success":false}"#).unwrap_err(),
        CATEGORIES_FETCH_FALLBACK
    );
}

#[test]
fn parse_venues_list_and_nested_fields() {
    let rows = parse_venues_response(200, &venue_list_body()).expect("ok");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "9");
    assert_eq!(rows[0].name, "基隆港遊艇會");
    assert_eq!(rows[0].address, "No. 1 Harbour Rd");
    assert_eq!(rows[0].city, "Keelung");
    assert_eq!(rows[0].rating, Some(5.0));
    assert_eq!(
        rows[0].amenities,
        vec!["valet".to_string(), "lounge".to_string()]
    );
    assert_eq!(rows[0].images, vec!["a.jpg".to_string()]);
    assert_eq!(rows[0].created_at, "2024-01-15T00:00:00.000Z");
    assert_eq!(rows[0].latitude, Some(25.128));
    assert_eq!(rows[0].longitude, Some(121.739));
    assert_eq!(rows[1].id, "10");
    assert_eq!(rows[1].city, "Taipei");
    assert_eq!(rows[1].rating, None);
    assert_eq!(
        rows[1].amenities,
        vec!["wifi".to_string(), "bar".to_string()]
    );
    assert!(rows[1].images.is_empty());
    assert_eq!(rows[1].latitude, None);
    assert_eq!(rows[1].longitude, None);
}

#[test]
fn parse_venues_empty_array() {
    let rows = parse_venues_response(200, r#"{"success":true,"data":[]}"#).expect("ok");
    assert!(rows.is_empty());
}

#[test]
fn parse_venues_error_empty_and_malformed() {
    assert_eq!(
        parse_venues_response(200, "not-json").unwrap_err(),
        VENUES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_venues_response(200, r#"{"success":false}"#).unwrap_err(),
        VENUES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_venues_response(200, r#"{"success":false,"error":"Failed to get venues"}"#)
            .unwrap_err(),
        "Failed to get venues"
    );
    assert_eq!(
        parse_venues_response(200, r#"{"success":true,"data":{}}"#).unwrap_err(),
        VENUES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_venues_response(0, "").unwrap_err(),
        VENUES_FETCH_FALLBACK
    );
}

#[test]
fn parse_venues_401_and_403_use_backend_error_strings() {
    assert_eq!(
        parse_venues_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_venues_response(401, "").unwrap_err(),
        VENUES_FETCH_FALLBACK
    );
    assert_eq!(
        parse_venues_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
    assert_eq!(
        parse_venues_response(403, r#"{"success":false}"#).unwrap_err(),
        VENUES_FETCH_FALLBACK
    );
}

#[test]
fn search_filter_is_case_insensitive_and_empty_returns_all() {
    let rows = parse_categories_response(200, &category_list_body()).expect("ok");
    assert_eq!(filter_categories(&rows, "").len(), 2);
    assert_eq!(filter_categories(&rows, "   ").len(), 2);
    let dinner = filter_categories(&rows, "晚宴");
    assert_eq!(dinner.len(), 1);
    assert_eq!(dinner[0].id, "3");
    let yacht = filter_categories(&rows, "遊艇");
    assert_eq!(yacht.len(), 1);
    assert!(matches_search("Private Dinner", "dinner"));
    assert!(matches_search("Private Dinner", "PRIVATE"));
    assert!(!matches_search("Private Dinner", "yacht"));

    let venues = parse_venues_response(200, &venue_list_body()).expect("ok");
    assert_eq!(filter_venues(&venues, "").len(), 2);
    let keelung = filter_venues(&venues, "keelung");
    assert!(keelung.is_empty());
    let harbour = filter_venues(&venues, "遊艇");
    assert_eq!(harbour.len(), 1);
    assert_eq!(harbour[0].id, "9");
}

#[test]
fn category_form_validation_requires_non_blank_name() {
    assert_eq!(
        validate_category_form(&CategoryForm::default()).unwrap_err(),
        NAME_REQUIRED
    );
    assert_eq!(
        validate_category_form(&CategoryForm {
            name: "   ".to_string(),
            ..CategoryForm::default()
        })
        .unwrap_err(),
        NAME_REQUIRED
    );
    validate_category_form(&CategoryForm {
        name: " 藝術沙龍 ".to_string(),
        description: "optional".to_string(),
        icon: String::new(),
    })
    .expect("ok");
}

#[test]
fn venue_form_validation_requires_non_blank_name() {
    assert_eq!(
        validate_venue_form(&VenueForm::default()).unwrap_err(),
        NAME_REQUIRED
    );
    assert_eq!(
        validate_venue_form(&VenueForm {
            name: "\t\n".to_string(),
            city: "Taipei".to_string(),
            address: "Somewhere".to_string(),
        })
        .unwrap_err(),
        NAME_REQUIRED
    );
    validate_venue_form(&VenueForm {
        name: "台北藝術空間".to_string(),
        city: String::new(),
        address: String::new(),
    })
    .expect("name-only create form is valid");
}

#[test]
fn forms_round_trip_from_parsed_entities() {
    let categories = parse_categories_response(200, &category_list_body()).expect("ok");
    let form = category_form_from(&categories[0]);
    assert_eq!(form.name, "私人晚宴");
    assert_eq!(form.description, "Small private dinners");
    assert_eq!(form.icon, "utensils");

    let venues = parse_venues_response(200, &venue_list_body()).expect("ok");
    let form = venue_form_from(&venues[0]);
    assert_eq!(form.name, "基隆港遊艇會");
    assert_eq!(form.city, "Keelung");
    assert_eq!(form.address, "No. 1 Harbour Rd");
}

#[test]
fn delete_confirmation_flow_opens_cancels_and_requires_selection() {
    assert_eq!(open_create(), TaxonomyModal::Create);
    assert_eq!(open_edit(), TaxonomyModal::Edit);
    assert_eq!(open_delete(), TaxonomyModal::Delete);
    assert_eq!(close_modal(), TaxonomyModal::None);

    assert_eq!(
        delete_confirmation_message("私人晚宴"),
        "Are you sure you want to delete \"私人晚宴\"?"
    );
    assert_eq!(
        delete_confirmation_message("基隆港遊艇會"),
        "Are you sure you want to delete \"基隆港遊艇會\"?"
    );

    assert_eq!(confirm_delete_id(None), None);
    assert_eq!(confirm_delete_id(Some("")), None);
    assert_eq!(confirm_delete_id(Some("3")), Some("3".to_string()));
    assert_eq!(close_modal(), TaxonomyModal::None);
}

#[test]
fn event_management_guard_has_loading_redirect_and_allow() {
    assert_eq!(
        event_management_guard(true, &snapshot(None, false)),
        RouteGuard::Loading
    );
    assert_eq!(
        event_management_guard(false, &snapshot(None, false)),
        RouteGuard::Redirect(EVENT_MANAGEMENT_FALLBACK)
    );
    assert_eq!(EVENT_MANAGEMENT_FALLBACK, "/login");
    assert_eq!(
        event_management_guard(false, &snapshot(Some(Role::User), true)),
        RouteGuard::Redirect("/login")
    );
    assert_eq!(
        event_management_guard(false, &snapshot(Some(Role::Admin), true)),
        RouteGuard::Allow
    );
    assert_eq!(
        event_management_guard(false, &snapshot(Some(Role::SuperAdmin), true)),
        RouteGuard::Allow
    );
}

#[test]
fn network_error_copy_matches_react() {
    assert_eq!(NETWORK_ERROR, "Network error");
    assert_eq!(CATEGORIES_FETCH_FALLBACK, "Failed to fetch categories");
    assert_eq!(VENUES_FETCH_FALLBACK, "Failed to fetch venues");
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

#[test]
fn categories_ssr_denied_loading_empty_populated_error() {
    let denied = render_categories(
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
    assert!(denied.contains("id=\"event-categories-denied\""));
    assert!(denied.contains("Access Denied"));
    assert!(!denied.contains("Category Management"));

    let loading = render_categories(
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
    assert!(loading.contains("id=\"event-categories-loading\""));
    assert!(loading.contains("Loading..."));
    assert!(loading.contains("Category Management"));
    assert!(loading.contains("Manage all aspects of your events, categories, and venues."));
    assert!(loading.contains("Search categories..."));
    assert!(loading.contains("New Category"));
    assert!(!loading.contains("id=\"event-categories-table\""));

    let empty = render_categories(
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
    assert!(empty.contains("id=\"event-categories-table\""));
    assert!(empty.contains("id=\"event-categories-empty\""));

    let populated = render_categories(
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
    assert!(populated.contains("id=\"event-categories-row-3\""));
    assert!(populated.contains("私人晚宴"));
    assert!(populated.contains("Small private dinners"));
    assert!(populated.contains("id=\"event-categories-error\""));
    assert!(populated.contains("Failed to fetch categories"));
    assert!(!populated.contains("id=\"event-categories-empty\""));
}

#[test]
fn categories_ssr_create_and_delete_modals() {
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

#[test]
fn venues_ssr_denied_loading_empty_populated_error_and_modals() {
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
