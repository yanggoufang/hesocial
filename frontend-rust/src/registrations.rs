use serde_json::Value;

use crate::events::Pagination;
use crate::permissions::{AuthUser, MembershipTier};

pub const USER_REGISTRATIONS_API_PATH: &str = "/api/registrations/user";
pub const REGISTRATIONS_API_PATH: &str = "/api/registrations";
pub const REGISTER_FOR_EVENT_API_PATH: &str = "/api/registrations/events";
pub const EVENT_DETAIL_API_PATH: &str = "/api/events";
pub const PAGE_SIZE: u32 = 10;

pub const FETCH_REGISTRATIONS_ERROR: &str = "無法獲取活動報名記錄";
pub const NETWORK_ERROR: &str = "發生網絡錯誤，請稍後再試";
pub const NETWORK_ERROR_SHORT: &str = "發生網絡錯誤";
pub const UPDATE_SUCCESS: &str = "報名資訊已成功更新";
pub const UPDATE_ERROR: &str = "更新報名資訊失敗";
pub const CANCEL_SUCCESS: &str = "活動報名已成功取消";
pub const CANCEL_ERROR: &str = "取消活動報名失敗";
pub const CANCEL_CONFIRM: &str = "您確定要取消此次活動報名嗎？此操作無法復原。";
pub const FETCH_EVENT_ERROR: &str = "Failed to fetch event details";
pub const NETWORK_EVENT_ERROR: &str = "Network error occurred";
pub const REGISTER_ERROR: &str = "Failed to register for event";
pub const REGISTER_NETWORK_ERROR: &str = "Network error occurred during registration";
pub const REGISTER_SUCCESS_NAV: &str = "Registration submitted successfully!";
pub const PRICE_ON_REQUEST: &str = "Price on request";
pub const EVENT_TITLE_FALLBACK: &str = "活動";
pub const EVENT_WHEN_FALLBACK: &str = "時間待確認";
pub const VENUE_FALLBACK: &str = "場地待確認";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationFilters {
    pub page: u32,
    pub limit: u32,
    pub status: String,
    pub payment_status: String,
    pub search: String,
}

impl Default for RegistrationFilters {
    fn default() -> Self {
        Self {
            page: 1,
            limit: PAGE_SIZE,
            status: String::new(),
            payment_status: String::new(),
            search: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RegistrationPricing {
    pub vip: Option<f64>,
    pub vvip: Option<f64>,
    pub general: Option<f64>,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Registration {
    pub id: String,
    pub user_id: String,
    pub event_id: String,
    pub status: String,
    pub payment_status: String,
    pub payment_intent_id: Option<String>,
    pub special_requests: Option<String>,
    pub event_name: Option<String>,
    pub event_description: Option<String>,
    pub event_date_time: Option<String>,
    pub registration_deadline: Option<String>,
    pub venue_name: Option<String>,
    pub venue_address: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub exclusivity_level: Option<String>,
    pub dress_code_label: String,
    pub capacity: u32,
    pub current_attendees: u32,
    pub pricing: RegistrationPricing,
    pub amenities: Vec<String>,
    pub privacy_guarantees: Vec<String>,
    pub requirements: Vec<String>,
    pub event_images: Vec<String>,
    pub category_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistrationsView {
    pub registrations: Vec<Registration>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RegisterEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub date_time: String,
    pub registration_deadline: String,
    pub venue_name: String,
    pub venue_address: String,
    pub category_name: String,
    pub exclusivity_level: Option<String>,
    pub dress_code_label: String,
    pub capacity: u32,
    pub current_attendees: u32,
    pub pricing: RegistrationPricing,
    pub images: Vec<String>,
    pub amenities: Vec<String>,
    pub privacy_guarantees: Vec<String>,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegisterUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub profession: String,
    pub membership_tier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRegistrationOk {
    pub registration_id: String,
    pub status: String,
    pub message: String,
}

pub fn default_filters() -> RegistrationFilters {
    RegistrationFilters::default()
}

pub fn registrations_query_string(filters: &RegistrationFilters) -> String {
    let mut out = String::new();
    push_param(&mut out, "page", &filters.page.to_string());
    push_param(&mut out, "limit", &filters.limit.to_string());
    if !filters.status.is_empty() {
        push_param(&mut out, "status", &filters.status);
    }
    if !filters.payment_status.is_empty() {
        push_param(&mut out, "paymentStatus", &filters.payment_status);
    }
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    out
}

pub fn page_after_filter_change(_current_page: u32) -> u32 {
    1
}

pub fn page_in_range(new_page: u32, total_pages: u32) -> bool {
    new_page >= 1 && new_page <= total_pages
}

pub fn pagination_range(page: u32, limit: u32, total: u32) -> (u32, u32) {
    let start = page
        .saturating_sub(1)
        .saturating_mul(limit)
        .saturating_add(1);
    let end = page.saturating_mul(limit).min(total);
    (start, end)
}

pub fn collapse_on_error(requested_page: u32, limit: u32) -> Pagination {
    Pagination {
        page: requested_page,
        limit,
        total: 0,
        total_pages: 1,
    }
}

pub fn parse_user_registrations_response(
    status: u16,
    body: &str,
    requested_page: u32,
    limit: u32,
) -> Result<RegistrationsView, String> {
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if !(200..300).contains(&status) {
        return Err(api_error(&value, FETCH_REGISTRATIONS_ERROR));
    }
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(api_error(&value, FETCH_REGISTRATIONS_ERROR));
    }
    let registrations = value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_registration).collect())
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
    Ok(RegistrationsView {
        registrations,
        pagination,
    })
}

pub fn parse_registration_detail_response(status: u16, body: &str) -> Result<Registration, String> {
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if !(200..300).contains(&status) {
        return Err(api_error(&value, "Failed to fetch registration details"));
    }
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(api_error(&value, "Failed to fetch registration details"));
    }
    parse_registration(value.get("data").unwrap_or(&Value::Null))
        .ok_or_else(|| "Failed to fetch registration details".to_string())
}

pub fn parse_create_registration_response(
    status: u16,
    body: &str,
) -> Result<CreateRegistrationOk, String> {
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if !(200..300).contains(&status) || value.get("success").and_then(Value::as_bool) != Some(true)
    {
        return Err(api_error(&value, REGISTER_ERROR));
    }
    let data = value.get("data").unwrap_or(&Value::Null);
    let registration_id =
        json_id(data.get("registrationId")).ok_or_else(|| api_error(&value, REGISTER_ERROR))?;
    Ok(CreateRegistrationOk {
        registration_id,
        status: json_string(data.get("status")).unwrap_or_default(),
        message: json_string(data.get("message")).unwrap_or_default(),
    })
}

pub fn parse_mutation_response(status: u16, body: &str, fallback: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if (200..300).contains(&status) && value.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(json_string(value.get("message")).unwrap_or_default())
    } else {
        Err(api_error(&value, fallback))
    }
}

pub fn parse_register_event_response(status: u16, body: &str) -> Result<RegisterEvent, String> {
    let value: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if !(200..300).contains(&status) {
        return Err(api_error(&value, FETCH_EVENT_ERROR));
    }
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return Err(api_error(&value, FETCH_EVENT_ERROR));
    }
    parse_register_event(value.get("data").unwrap_or(&Value::Null))
        .ok_or_else(|| FETCH_EVENT_ERROR.to_string())
}

pub fn parse_register_user_from_auth(user: &AuthUser) -> RegisterUser {
    RegisterUser {
        first_name: String::new(),
        last_name: String::new(),
        email: user.email.clone().unwrap_or_default(),
        profession: String::new(),
        membership_tier: membership_tier_label(user.membership_tier)
            .unwrap_or_default()
            .to_string(),
    }
}

pub fn parse_register_user_from_profile(value: &Value) -> RegisterUser {
    RegisterUser {
        first_name: json_string(value.get("firstName")).unwrap_or_default(),
        last_name: json_string(value.get("lastName")).unwrap_or_default(),
        email: json_string(value.get("email")).unwrap_or_default(),
        profession: json_string(value.get("profession")).unwrap_or_default(),
        membership_tier: json_string(value.get("membershipTier")).unwrap_or_default(),
    }
}

pub fn register_user_from_profile(profile: &crate::profile::ProfileUser) -> RegisterUser {
    RegisterUser {
        first_name: profile.first_name.clone().unwrap_or_default(),
        last_name: profile.last_name.clone().unwrap_or_default(),
        email: profile.email.clone().unwrap_or_default(),
        profession: profile.profession.clone().unwrap_or_default(),
        membership_tier: profile
            .membership_tier_label()
            .unwrap_or_default()
            .to_string(),
    }
}

pub fn membership_tier_label(tier: Option<MembershipTier>) -> Option<&'static str> {
    match tier {
        Some(MembershipTier::Platinum) => Some("Platinum"),
        Some(MembershipTier::Diamond) => Some("Diamond"),
        Some(MembershipTier::BlackCard) => Some("Black Card"),
        None => None,
    }
}

pub fn membership_tier_badge_class(tier: &str) -> &'static str {
    match tier {
        "Platinum" => "bg-gray-100 text-gray-800 border-gray-300",
        "Diamond" => "bg-blue-100 text-blue-800 border-blue-300",
        "Black Card" => "bg-black text-white border-black",
        _ => "bg-gray-100 text-gray-800 border-gray-300",
    }
}

pub fn status_label(status: &str) -> &str {
    match status {
        "pending" => "審核中",
        "approved" | "confirmed" => "已核准",
        "rejected" => "已婉拒",
        "cancelled" => "已取消",
        "waitlisted" => "候補中",
        other => other,
    }
}

pub fn status_class(status: &str) -> &'static str {
    match status {
        "pending" => "bg-yellow-900/50 text-yellow-300 border-yellow-700",
        "approved" | "confirmed" => "bg-green-900/50 text-green-300 border-green-700",
        "rejected" => "bg-red-900/50 text-red-300 border-red-700",
        "cancelled" => "bg-gray-700/50 text-gray-300 border-gray-500",
        _ => "",
    }
}

pub fn payment_label(status: &str) -> &str {
    match status {
        "pending" => "待付款",
        "paid" => "已付款",
        "refunded" => "已退款",
        other => other,
    }
}

pub fn payment_class(status: &str) -> &'static str {
    match status {
        "pending" => "bg-yellow-900/50 text-yellow-300",
        "paid" => "bg-green-900/50 text-green-300",
        "refunded" => "bg-blue-900/50 text-blue-300",
        _ => "",
    }
}

pub fn event_title(name: Option<&str>) -> String {
    name.map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(EVENT_TITLE_FALLBACK)
        .to_string()
}

pub fn venue_label(name: Option<&str>) -> String {
    name.map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(VENUE_FALLBACK)
        .to_string()
}

pub fn event_when_label(iso: Option<&str>) -> String {
    match iso.map(str::trim).filter(|s| !s.is_empty()) {
        Some(value) => format_list_datetime(value),
        None => EVENT_WHEN_FALLBACK.to_string(),
    }
}

pub fn register_dress_code_text(code: i32) -> &'static str {
    match code {
        1 => "Casual",
        2 => "Business Casual",
        3 => "Business Formal",
        4 => "Cocktail Attire",
        5 => "Black Tie",
        _ => "Not specified",
    }
}

pub fn register_exclusivity_class(level: Option<&str>) -> &'static str {
    match level {
        Some("VIP") => "bg-purple-100 text-purple-800 border-purple-300",
        Some("VVIP") => "bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30",
        Some("Invitation Only") => "bg-red-100 text-red-800 border-red-300",
        _ => "bg-gray-100 text-gray-800 border-gray-300",
    }
}

pub fn format_event_price(pricing: &RegistrationPricing) -> String {
    let currency = if pricing.currency.is_empty() {
        "TWD"
    } else {
        pricing.currency.as_str()
    };
    if let Some(general) = js_truthy_amount(pricing.general) {
        return format_currency(general, currency);
    }
    match (
        js_truthy_amount(pricing.vip),
        js_truthy_amount(pricing.vvip),
    ) {
        (Some(vip), Some(vvip)) => format!(
            "{} - {}",
            format_currency(vip, currency),
            format_currency(vvip, currency)
        ),
        (Some(vip), None) => format_currency(vip, currency),
        _ => PRICE_ON_REQUEST.to_string(),
    }
}

pub fn format_list_datetime(iso: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format_list_datetime_js(iso);
    }
    #[cfg(not(target_arch = "wasm32"))]
    iso.to_string()
}

pub fn format_register_datetime(iso: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return crate::events::format_event_datetime(iso);
    }
    #[cfg(not(target_arch = "wasm32"))]
    iso.to_string()
}

pub fn now_ms() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        return js_sys::Date::now();
    }
    #[cfg(not(target_arch = "wasm32"))]
    0.0
}

pub fn parse_iso_ms(iso: &str) -> Option<f64> {
    let iso = iso.trim();
    if iso.is_empty() {
        return None;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
        let ms = date.get_time();
        if ms.is_nan() {
            return None;
        }
        return Some(ms);
    }
    #[cfg(not(target_arch = "wasm32"))]
    parse_iso_ms_native(iso)
}

pub fn can_edit(status: &str, event_date_time: Option<&str>, now_ms: f64) -> bool {
    if status != "pending" {
        return false;
    }
    let Some(iso) = event_date_time.map(str::trim).filter(|s| !s.is_empty()) else {
        return false;
    };
    parse_iso_ms(iso).is_some_and(|event_ms| event_ms > now_ms)
}

pub fn can_cancel(status: &str, event_date_time: Option<&str>, now_ms: f64) -> bool {
    if status == "cancelled" || status == "rejected" {
        return false;
    }
    let Some(iso) = event_date_time.map(str::trim).filter(|s| !s.is_empty()) else {
        return false;
    };
    parse_iso_ms(iso).is_some_and(|event_ms| (event_ms - now_ms) / 3_600_000.0 > 24.0)
}

pub fn success_message_from_query(query: &str) -> Option<String> {
    let query = query.trim_start_matches('?');
    for pair in query.split('&') {
        if pair == "registered=1" || pair.starts_with("registered=1") {
            return Some(REGISTER_SUCCESS_NAV.to_string());
        }
    }
    None
}

pub fn boot_success_message() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window()?.location().search().ok()?;
        return success_message_from_query(&search);
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn confirm_cancel() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window()
            .and_then(|window| window.confirm_with_message(CANCEL_CONFIRM).ok())
            .unwrap_or(false);
    }
    #[cfg(not(target_arch = "wasm32"))]
    false
}

pub async fn fetch_user_registrations(
    filters: &RegistrationFilters,
) -> Result<RegistrationsView, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!(
            "{}?{}",
            USER_REGISTRATIONS_API_PATH,
            registrations_query_string(filters)
        );
        let Some((status, body)) = send_authorized("GET", &url, None).await else {
            return Err(NETWORK_ERROR.to_string());
        };
        return parse_user_registrations_response(status, &body, filters.page, filters.limit);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filters;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_registration_detail(id: &str) -> Result<Registration, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{REGISTRATIONS_API_PATH}/{id}");
        let Some((status, body)) = send_authorized("GET", &url, None).await else {
            return Err("Failed to fetch registration details".to_string());
        };
        return parse_registration_detail_response(status, &body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err("Failed to fetch registration details".to_string())
    }
}

pub async fn create_registration(
    event_id: &str,
    special_requests: &str,
) -> Result<CreateRegistrationOk, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{REGISTER_FOR_EVENT_API_PATH}/{event_id}");
        let payload = serde_json::json!({ "specialRequests": special_requests });
        let Some((status, body)) = send_authorized("POST", &url, Some(payload.to_string())).await
        else {
            return Err(REGISTER_NETWORK_ERROR.to_string());
        };
        return parse_create_registration_response(status, &body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, special_requests);
        Err(REGISTER_NETWORK_ERROR.to_string())
    }
}

pub async fn update_registration(id: &str, special_requests: &str) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{REGISTRATIONS_API_PATH}/{id}");
        let payload = serde_json::json!({ "specialRequests": special_requests });
        let Some((status, body)) = send_authorized("PUT", &url, Some(payload.to_string())).await
        else {
            return Err(NETWORK_ERROR_SHORT.to_string());
        };
        return parse_mutation_response(status, &body, UPDATE_ERROR);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, special_requests);
        Err(NETWORK_ERROR_SHORT.to_string())
    }
}

pub async fn cancel_registration(id: &str) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{REGISTRATIONS_API_PATH}/{id}");
        let Some((status, body)) = send_authorized("DELETE", &url, None).await else {
            return Err(NETWORK_ERROR_SHORT.to_string());
        };
        return parse_mutation_response(status, &body, CANCEL_ERROR);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_ERROR_SHORT.to_string())
    }
}

pub async fn update_payment_status(
    id: &str,
    payment_status: &str,
    payment_intent_id: Option<&str>,
) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{REGISTRATIONS_API_PATH}/{id}/payment");
        let payload = serde_json::json!({
            "paymentStatus": payment_status,
            "paymentIntentId": payment_intent_id,
        });
        let Some((status, body)) = send_authorized("POST", &url, Some(payload.to_string())).await
        else {
            return Err("Failed to update payment status".to_string());
        };
        return parse_mutation_response(status, &body, "Failed to update payment status");
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, payment_status, payment_intent_id);
        Err("Failed to update payment status".to_string())
    }
}

pub async fn fetch_register_event(id: &str) -> Result<RegisterEvent, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{EVENT_DETAIL_API_PATH}/{id}");
        let Some((status, body)) = send_authorized("GET", &url, None).await else {
            return Err(NETWORK_EVENT_ERROR.to_string());
        };
        return parse_register_event_response(status, &body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_EVENT_ERROR.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
async fn send_authorized(method: &str, url: &str, body: Option<String>) -> Option<(u16, String)> {
    let token = crate::auth::read_stored_token()?;
    let auth = crate::auth::bearer_authorization(&token);
    let builder = match method {
        "POST" => gloo_net::http::Request::post(url),
        "PUT" => gloo_net::http::Request::put(url),
        "DELETE" => gloo_net::http::Request::delete(url),
        _ => gloo_net::http::Request::get(url),
    };
    let builder = builder.header("Authorization", &auth);
    let request = match body {
        Some(body) => builder
            .header("Content-Type", "application/json")
            .body(body)
            .ok()?,
        None => builder.build().ok()?,
    };
    let response = request.send().await.ok()?;
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    Some((status, text))
}

#[cfg(target_arch = "wasm32")]
fn format_list_datetime_js(iso: &str) -> String {
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_str(iso));
    if date.get_time().is_nan() {
        return iso.to_string();
    }
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"year".into(), &"numeric".into());
    let _ = js_sys::Reflect::set(&opts, &"month".into(), &"long".into());
    let _ = js_sys::Reflect::set(&opts, &"day".into(), &"numeric".into());
    let _ = js_sys::Reflect::set(&opts, &"hour".into(), &"2-digit".into());
    let _ = js_sys::Reflect::set(&opts, &"minute".into(), &"2-digit".into());
    let _ = js_sys::Reflect::set(&opts, &"hour12".into(), &JsValue::from_bool(false));
    date.to_locale_string("zh-TW", &opts).into()
}

fn parse_registration(value: &Value) -> Option<Registration> {
    let id = json_id(value.get("id"))?;
    Some(Registration {
        id,
        user_id: json_id(value.get("userId")).unwrap_or_default(),
        event_id: json_id(value.get("eventId")).unwrap_or_default(),
        status: json_string(value.get("status")).unwrap_or_default(),
        payment_status: json_string(value.get("paymentStatus")).unwrap_or_default(),
        payment_intent_id: json_string(value.get("paymentIntentId")),
        special_requests: json_string(value.get("specialRequests")),
        event_name: json_string(value.get("eventName")),
        event_description: json_string(value.get("eventDescription")),
        event_date_time: json_string(value.get("eventDateTime")),
        registration_deadline: json_string(value.get("registrationDeadline")),
        venue_name: json_string(value.get("venueName")),
        venue_address: json_string(value.get("venueAddress")),
        created_at: json_string(value.get("createdAt")).unwrap_or_default(),
        updated_at: json_string(value.get("updatedAt")).unwrap_or_default(),
        exclusivity_level: json_string(value.get("exclusivityLevel")),
        dress_code_label: parse_dress_code_label(value.get("dressCode")),
        capacity: json_u32(value.get("capacity")).unwrap_or(0),
        current_attendees: json_u32(value.get("currentAttendees")).unwrap_or(0),
        pricing: parse_pricing(value.get("pricing")),
        amenities: parse_string_list(value.get("amenities")),
        privacy_guarantees: parse_string_list(value.get("privacyGuarantees")),
        requirements: parse_string_list(value.get("requirements")),
        event_images: parse_string_list(value.get("eventImages")),
        category_name: json_string(value.get("categoryName")),
    })
}

fn parse_register_event(value: &Value) -> Option<RegisterEvent> {
    let id = json_id(value.get("id"))?;
    let name = json_string(value.get("name")).filter(|s| !s.is_empty())?;
    let venue = value.get("venue");
    let category = value.get("category");
    Some(RegisterEvent {
        id,
        name,
        description: json_string(value.get("description")).unwrap_or_default(),
        date_time: json_string(value.get("dateTime"))
            .or_else(|| json_string(value.get("date_time")))
            .unwrap_or_default(),
        registration_deadline: json_string(value.get("registrationDeadline"))
            .or_else(|| json_string(value.get("registration_deadline")))
            .unwrap_or_default(),
        venue_name: json_string(value.get("venueName"))
            .or_else(|| venue.and_then(|v| json_string(v.get("name"))))
            .unwrap_or_default(),
        venue_address: json_string(value.get("venueAddress"))
            .or_else(|| venue.and_then(|v| json_string(v.get("address"))))
            .unwrap_or_default(),
        category_name: json_string(value.get("categoryName"))
            .or_else(|| category.and_then(|v| json_string(v.get("name"))))
            .unwrap_or_default(),
        exclusivity_level: json_string(value.get("exclusivityLevel")),
        dress_code_label: parse_dress_code_label(value.get("dressCode")),
        capacity: json_u32(value.get("capacity")).unwrap_or(0),
        current_attendees: json_u32(value.get("currentAttendees")).unwrap_or(0),
        pricing: parse_pricing(value.get("pricing")),
        images: parse_string_list(value.get("images")),
        amenities: parse_string_list(value.get("amenities")),
        privacy_guarantees: parse_string_list(value.get("privacyGuarantees")),
        requirements: parse_string_list(value.get("requirements")),
    })
}

fn parse_pricing(value: Option<&Value>) -> RegistrationPricing {
    let Some(value) = value else {
        return RegistrationPricing::default();
    };
    RegistrationPricing {
        vip: json_f64(value.get("vip")),
        vvip: json_f64(value.get("vvip")),
        general: json_f64(value.get("general")),
        currency: json_string(value.get("currency")).unwrap_or_default(),
    }
}

fn parse_dress_code_label(value: Option<&Value>) -> String {
    match value {
        Some(Value::Number(n)) => {
            register_dress_code_text(n.as_i64().unwrap_or(0) as i32).to_string()
        }
        Some(Value::String(text)) => {
            if let Ok(code) = text.parse::<i32>() {
                register_dress_code_text(code).to_string()
            } else if text.is_empty() {
                register_dress_code_text(0).to_string()
            } else {
                text.clone()
            }
        }
        _ => register_dress_code_text(0).to_string(),
    }
}

fn parse_string_list(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Null => Vec::new(),
        Value::Array(items) => items.iter().filter_map(parse_list_item).collect(),
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

fn parse_list_item(value: &Value) -> Option<String> {
    match value {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Object(_) => json_string(value.get("description")).filter(|s| !s.is_empty()),
        _ => None,
    }
}

fn api_error(value: &Value, fallback: &str) -> String {
    value
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback)
        .to_string()
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

fn format_currency(amount: f64, currency: &str) -> String {
    let grouped = format_thousands(amount);
    match currency {
        "TWD" | "twd" => format!("NT${grouped}"),
        other => format!("{other} {grouped}"),
    }
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

#[cfg(not(target_arch = "wasm32"))]
fn parse_iso_ms_native(iso: &str) -> Option<f64> {
    let iso = iso.trim();
    let (date, rest) = iso.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: u32 = date_parts.next()?.parse().ok()?;
    let day: u32 = date_parts.next()?.parse().ok()?;
    let time = rest.trim_end_matches('Z');
    let (hms, frac) = time.split_once('.').unwrap_or((time, "0"));
    let mut time_parts = hms.split(':');
    let hour: u32 = time_parts.next()?.parse().ok()?;
    let minute: u32 = time_parts.next()?.parse().ok()?;
    let second: u32 = time_parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let millis: u32 = frac
        .chars()
        .filter(char::is_ascii_digit)
        .take(3)
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    let days = days_from_civil(year, month, day)?;
    let unix = days * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60 + i64::from(second);
    Some(unix as f64 * 1_000.0 + f64::from(millis))
}

#[cfg(not(target_arch = "wasm32"))]
fn days_from_civil(mut year: i64, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let month = month as i64;
    let day = day as i64;
    year -= i64::from(month <= 2);
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}
