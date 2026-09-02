use serde_json::Value;

use crate::permissions::{AuthSnapshot, RouteGuard, permissions};

pub const CATEGORIES_API_PATH: &str = "/api/categories";
pub const VENUES_API_PATH: &str = "/api/venues";
pub const EVENT_MANAGEMENT_FALLBACK: &str = "/login";
pub const NETWORK_ERROR: &str = "Network error";
pub const CATEGORIES_FETCH_FALLBACK: &str = "Failed to fetch categories";
pub const VENUES_FETCH_FALLBACK: &str = "Failed to fetch venues";
pub const NAME_REQUIRED: &str = "Name is required";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EventCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Venue {
    pub id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub rating: Option<f64>,
    pub amenities: Vec<String>,
    pub images: Vec<String>,
    pub created_at: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CategoryForm {
    pub name: String,
    pub description: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VenueForm {
    pub name: String,
    pub city: String,
    pub address: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TaxonomyModal {
    #[default]
    None,
    Create,
    Edit,
    Delete,
}

pub fn event_management_guard(restoring: bool, snapshot: &AuthSnapshot) -> RouteGuard {
    if restoring {
        RouteGuard::Loading
    } else if !permissions(snapshot).event_management {
        RouteGuard::Redirect(EVENT_MANAGEMENT_FALLBACK)
    } else {
        RouteGuard::Allow
    }
}

pub fn open_create() -> TaxonomyModal {
    TaxonomyModal::Create
}

pub fn open_edit() -> TaxonomyModal {
    TaxonomyModal::Edit
}

pub fn open_delete() -> TaxonomyModal {
    TaxonomyModal::Delete
}

pub fn close_modal() -> TaxonomyModal {
    TaxonomyModal::None
}

pub fn delete_confirmation_message(name: &str) -> String {
    format!("Are you sure you want to delete \"{name}\"?")
}

pub fn confirm_delete_id(selected: Option<&str>) -> Option<String> {
    selected
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

pub fn matches_search(name: &str, search: &str) -> bool {
    let term = search.trim().to_lowercase();
    if term.is_empty() {
        true
    } else {
        name.to_lowercase().contains(&term)
    }
}

pub fn filter_categories(categories: &[EventCategory], search: &str) -> Vec<EventCategory> {
    categories
        .iter()
        .filter(|category| matches_search(&category.name, search))
        .cloned()
        .collect()
}

pub fn filter_venues(venues: &[Venue], search: &str) -> Vec<Venue> {
    venues
        .iter()
        .filter(|venue| matches_search(&venue.name, search))
        .cloned()
        .collect()
}

pub fn validate_required_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        Err(NAME_REQUIRED.to_string())
    } else {
        Ok(())
    }
}

pub fn validate_category_form(form: &CategoryForm) -> Result<(), String> {
    validate_required_name(&form.name)
}

pub fn validate_venue_form(form: &VenueForm) -> Result<(), String> {
    validate_required_name(&form.name)
}

pub fn category_form_from(category: &EventCategory) -> CategoryForm {
    CategoryForm {
        name: category.name.clone(),
        description: category.description.clone(),
        icon: category.icon.clone(),
    }
}

pub fn venue_form_from(venue: &Venue) -> VenueForm {
    VenueForm {
        name: venue.name.clone(),
        city: venue.city.clone(),
        address: venue.address.clone(),
    }
}

pub fn parse_categories_response(status: u16, body: &str) -> Result<Vec<EventCategory>, String> {
    let value = parse_success_root(status, body, CATEGORIES_FETCH_FALLBACK)?;
    match value.get("data") {
        Some(Value::Array(rows)) => Ok(rows.iter().filter_map(parse_category).collect()),
        _ => Err(CATEGORIES_FETCH_FALLBACK.to_string()),
    }
}

pub fn parse_venues_response(status: u16, body: &str) -> Result<Vec<Venue>, String> {
    let value = parse_success_root(status, body, VENUES_FETCH_FALLBACK)?;
    match value.get("data") {
        Some(Value::Array(rows)) => Ok(rows.iter().filter_map(parse_venue).collect()),
        _ => Err(VENUES_FETCH_FALLBACK.to_string()),
    }
}

pub async fn fetch_categories() -> Result<Vec<EventCategory>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            CATEGORIES_API_PATH,
            CATEGORIES_FETCH_FALLBACK,
            parse_categories_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn fetch_venues() -> Result<Vec<Venue>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            VENUES_API_PATH,
            VENUES_FETCH_FALLBACK,
            parse_venues_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

#[cfg(target_arch = "wasm32")]
async fn authorized_get<T>(
    url: &str,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let builder = authorized_builder(gloo_net::http::Request::get(url))?;
    let response = builder
        .send()
        .await
        .map_err(|_| NETWORK_ERROR.to_string())?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    parse(status, &text).map_err(|err| {
        if err.is_empty() {
            fallback.to_string()
        } else {
            err
        }
    })
}

#[cfg(target_arch = "wasm32")]
fn authorized_builder(
    builder: gloo_net::http::RequestBuilder,
) -> Result<gloo_net::http::RequestBuilder, String> {
    let token = crate::auth::read_stored_token().ok_or_else(|| NETWORK_ERROR.to_string())?;
    Ok(builder.header("Authorization", &crate::auth::bearer_authorization(&token)))
}

fn parse_success_root(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    if status == 401 || status == 403 {
        let value = serde_json::from_str::<Value>(body).unwrap_or(Value::Null);
        return Err(json_error_message(&value, fallback));
    }
    let value: Value = serde_json::from_str(body).map_err(|_| fallback.to_string())?;
    if value.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(value)
    } else {
        Err(json_error_message(&value, fallback))
    }
}

fn json_error_message(value: &Value, fallback: &'static str) -> String {
    value
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|error| !error.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn parse_category(value: &Value) -> Option<EventCategory> {
    let id = json_id(field(value, "id", "id"))?;
    Some(EventCategory {
        id,
        name: json_string(field(value, "name", "name")).unwrap_or_default(),
        description: json_string(field(value, "description", "description")).unwrap_or_default(),
        icon: json_string(field(value, "icon", "icon")).unwrap_or_default(),
        created_at: json_string(field(value, "createdAt", "created_at")).unwrap_or_default(),
    })
}

fn parse_venue(value: &Value) -> Option<Venue> {
    let id = json_id(field(value, "id", "id"))?;
    let coordinates = value.get("coordinates");
    Some(Venue {
        id,
        name: json_string(field(value, "name", "name")).unwrap_or_default(),
        address: json_string(field(value, "address", "address")).unwrap_or_default(),
        city: json_string(field(value, "city", "city")).unwrap_or_default(),
        rating: json_f64(field(value, "rating", "rating")),
        amenities: parse_string_list(field(value, "amenities", "amenities")),
        images: parse_string_list(field(value, "images", "images")),
        created_at: json_string(field(value, "createdAt", "created_at")).unwrap_or_default(),
        latitude: json_f64(coordinates.and_then(|item| item.get("lat"))),
        longitude: json_f64(coordinates.and_then(|item| item.get("lng"))),
    })
}

fn parse_string_list(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Null => Vec::new(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str()
                    .filter(|text| !text.is_empty())
                    .map(str::to_string)
            })
            .collect(),
        Value::String(text) => {
            if text.is_empty() {
                Vec::new()
            } else if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                parse_string_list(Some(&parsed))
            } else {
                vec![text.clone()]
            }
        }
        _ => Vec::new(),
    }
}

fn field<'a>(value: &'a Value, camel: &str, snake: &str) -> Option<&'a Value> {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .filter(|item| !item.is_null())
}

fn json_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn json_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}
