use serde_json::Value;

pub const EVENTS_API_PATH: &str = "/api/events";
pub const PAGE_LIMIT: u32 = 9;
pub const PLACEHOLDER_IMAGE: &str = "/api/placeholder/400/300";

pub const CATEGORIES: &[FilterOption] = &[
    FilterOption {
        id: "all",
        name: "全部活動",
    },
    FilterOption {
        id: "dinner",
        name: "私人晚宴",
    },
    FilterOption {
        id: "yacht",
        name: "遊艇派對",
    },
    FilterOption {
        id: "art",
        name: "藝術沙龍",
    },
    FilterOption {
        id: "business",
        name: "商務社交",
    },
    FilterOption {
        id: "wine",
        name: "品酒會",
    },
];

pub const EXCLUSIVITY_LEVELS: &[FilterOption] = &[
    FilterOption {
        id: "all",
        name: "全部等級",
    },
    FilterOption {
        id: "VIP",
        name: "VIP",
    },
    FilterOption {
        id: "VVIP",
        name: "VVIP",
    },
    FilterOption {
        id: "invitation",
        name: "僅限邀請",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterOption {
    pub id: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventFilters {
    pub page: u32,
    pub limit: u32,
    pub search: String,
    pub category: String,
    pub exclusivity_level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventsView {
    pub events: Vec<Event>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Venue {
    pub name: String,
    pub address: String,
    pub rating: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pricing {
    pub vvip: Option<f64>,
    pub vip: Option<f64>,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub description: String,
    pub date_time: String,
    pub venue: Option<Venue>,
    pub exclusivity_level: Option<String>,
    pub pricing: Pricing,
    pub current_attendees: u32,
    pub capacity: u32,
    pub images: Option<Vec<String>>,
}

pub fn events_query_string(filters: &EventFilters) -> String {
    let mut out = String::new();
    push_param(&mut out, "page", &filters.page.to_string());
    push_param(&mut out, "limit", &filters.limit.to_string());
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    if filters.category != "all" && !filters.category.is_empty() {
        push_param(&mut out, "category", &filters.category);
    }
    if filters.exclusivity_level != "all" && !filters.exclusivity_level.is_empty() {
        push_param(&mut out, "exclusivityLevel", &filters.exclusivity_level);
    }
    out
}

pub fn page_after_filter_change(_current_page: u32) -> u32 {
    1
}

pub fn page_in_range(new_page: u32, total_pages: u32) -> bool {
    new_page >= 1 && new_page <= total_pages
}

pub fn collapse_on_error(requested_page: u32, limit: u32) -> Pagination {
    Pagination {
        page: requested_page,
        limit,
        total: 0,
        total_pages: 1,
    }
}

pub fn parse_events_response(body: &str, requested_page: u32, limit: u32) -> EventsView {
    let collapsed = EventsView {
        events: Vec::new(),
        pagination: collapse_on_error(requested_page, limit),
    };
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return collapsed;
    };
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return collapsed;
    }
    let events = value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_event).collect())
        .unwrap_or_default();
    let pagination = value
        .get("pagination")
        .map(|p| Pagination {
            page: json_u32(p.get("page")).unwrap_or(requested_page),
            limit: json_u32(p.get("limit")).unwrap_or(limit),
            total: json_u32(p.get("total")).unwrap_or(0),
            total_pages: json_u32(p.get("totalPages")).unwrap_or(1),
        })
        .unwrap_or_else(|| collapse_on_error(requested_page, limit));
    EventsView { events, pagination }
}

pub fn exclusivity_color(level: Option<&str>) -> &'static str {
    match level {
        Some("VIP") => "bg-blue-500/20 text-blue-400 border-blue-500/30",
        Some("VVIP") => "bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30",
        Some("僅限邀請") => "bg-purple-500/20 text-purple-400 border-purple-500/30",
        _ => "bg-gray-500/20 text-gray-400 border-gray-500/30",
    }
}

pub fn exclusivity_label(level: Option<&str>) -> String {
    level.unwrap_or("").to_string()
}

pub fn star_count(level: Option<&str>) -> u32 {
    match level {
        Some("VIP") => 2,
        Some("VVIP") | Some("Invitation Only") => 3,
        _ => 0,
    }
}

pub fn shows_diamond(level: Option<&str>) -> bool {
    level == Some("Invitation Only")
}

pub fn first_image(images: Option<&[String]>) -> String {
    images
        .and_then(|list| list.first())
        .filter(|url| !url.is_empty())
        .cloned()
        .unwrap_or_else(|| PLACEHOLDER_IMAGE.to_string())
}

pub fn format_price(vvip: Option<f64>, vip: Option<f64>) -> String {
    if let Some(amount) = js_truthy_amount(vvip).or_else(|| js_truthy_amount(vip)) {
        format!("NT$ {}", format_thousands(amount))
    } else {
        "價格洽詢".to_string()
    }
}

pub fn format_event_datetime(iso: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format_event_datetime_js(iso);
    }
    #[cfg(not(target_arch = "wasm32"))]
    iso.to_string()
}

pub async fn fetch_events(filters: &EventFilters) -> EventsView {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{EVENTS_API_PATH}?{}", events_query_string(filters));
        let response = match gloo_net::http::Request::get(&url).send().await {
            Ok(response) => response,
            Err(_) => return empty_view(filters),
        };
        if !(200..300).contains(&response.status()) {
            return empty_view(filters);
        }
        let body = response.text().await.unwrap_or_default();
        return parse_events_response(&body, filters.page, filters.limit);
    }
    #[cfg(not(target_arch = "wasm32"))]
    empty_view(filters)
}

#[cfg(target_arch = "wasm32")]
fn format_event_datetime_js(iso: &str) -> String {
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_str(iso));
    if date.get_time().is_nan() {
        return iso.to_string();
    }
    let date_opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&date_opts, &"year".into(), &"numeric".into());
    let _ = js_sys::Reflect::set(&date_opts, &"month".into(), &"long".into());
    let _ = js_sys::Reflect::set(&date_opts, &"day".into(), &"numeric".into());
    let _ = js_sys::Reflect::set(&date_opts, &"weekday".into(), &"long".into());
    let date_part = date.to_locale_date_string("zh-TW", &date_opts);
    let time_part = format!("{:02}:{:02}", date.get_hours(), date.get_minutes());
    format!("{date_part} {time_part}")
}

fn empty_view(filters: &EventFilters) -> EventsView {
    EventsView {
        events: Vec::new(),
        pagination: collapse_on_error(filters.page, filters.limit),
    }
}

fn parse_event(value: &Value) -> Option<Event> {
    let id = json_id(value.get("id"))?;
    let name = json_string(value.get("name")).filter(|s| !s.is_empty())?;
    Some(Event {
        id,
        name,
        description: json_string(value.get("description")).unwrap_or_default(),
        date_time: json_string(value.get("dateTime"))
            .or_else(|| json_string(value.get("date_time")))
            .unwrap_or_default(),
        venue: value.get("venue").and_then(parse_venue),
        exclusivity_level: json_string(value.get("exclusivityLevel")),
        pricing: parse_pricing(value.get("pricing")),
        current_attendees: json_u32(value.get("currentAttendees")).unwrap_or(0),
        capacity: json_u32(value.get("capacity")).unwrap_or(0),
        images: value.get("images").and_then(parse_images),
    })
}

fn parse_venue(value: &Value) -> Option<Venue> {
    if value.is_null() {
        return None;
    }
    Some(Venue {
        name: json_string(value.get("name")).unwrap_or_default(),
        address: json_string(value.get("address")).unwrap_or_default(),
        rating: json_f64(value.get("rating")).unwrap_or(0.0),
    })
}

fn parse_pricing(value: Option<&Value>) -> Pricing {
    let Some(value) = value else {
        return Pricing::default();
    };
    Pricing {
        vvip: json_f64(value.get("vvip")),
        vip: json_f64(value.get("vip")),
        currency: json_string(value.get("currency")).unwrap_or_default(),
    }
}

fn parse_images(value: &Value) -> Option<Vec<String>> {
    if value.is_null() {
        return None;
    }
    let Value::Array(items) = value else {
        return None;
    };
    Some(
        items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
    )
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

fn json_u32(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(n) => n.as_u64().map(|v| v as u32),
        Value::String(s) => s.parse().ok(),
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

fn js_truthy_amount(value: Option<f64>) -> Option<f64> {
    value.filter(|amount| *amount != 0.0)
}

fn format_thousands(amount: f64) -> String {
    let negative = amount < 0.0;
    let abs = amount.abs();
    let whole = abs.trunc() as i64;
    let mut digits = whole.to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        grouped.insert_str(0, &format!(",{}", &digits[split..]));
        digits.truncate(split);
    }
    grouped.insert_str(0, &digits);
    if negative {
        grouped.insert(0, '-');
    }
    grouped
}

fn push_param(out: &mut String, key: &str, value: &str) {
    if !out.is_empty() {
        out.push('&');
    }
    out.push_str(key);
    out.push('=');
    out.push_str(&form_encode(value));
}

fn form_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
