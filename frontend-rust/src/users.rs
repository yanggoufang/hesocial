use serde_json::Value;

use crate::permissions::{AuthSnapshot, RouteGuard, permissions};

pub const USERS_API_PATH: &str = "/api/users";
pub const USER_STATS_API_PATH: &str = "/api/users/stats/overview";
pub const PAGE_SIZE: u32 = 20;
pub const USER_MANAGEMENT_FALLBACK: &str = "/admin";

pub const NETWORK_ERROR: &str = "Network error occurred";
pub const NETWORK_ERROR_ZH: &str = "發生網路錯誤";
pub const USERS_FETCH_FALLBACK: &str = "Failed to fetch users";
pub const USER_FETCH_FALLBACK: &str = "Failed to get user";
pub const UPDATE_FALLBACK: &str = "Failed to update user";
pub const VERIFY_FALLBACK: &str = "Failed to verify user";
pub const ROLE_FALLBACK: &str = "更新使用者角色失敗";
pub const DELETE_FALLBACK: &str = "刪除使用者失敗";
pub const STATS_FETCH_FALLBACK: &str = "Failed to get user statistics";
pub const EDIT_ACTION: &str = "edit";
pub const DELETE_ACTION: &str = "delete";

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UserFilters {
    pub search: String,
    pub role: String,
    pub membership_tier: String,
    pub verification_status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsersPagination {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}

impl Default for UsersPagination {
    fn default() -> Self {
        Self {
            page: 1,
            limit: PAGE_SIZE,
            total: 0,
            total_pages: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i64,
    pub profession: String,
    pub annual_income: i64,
    pub net_worth: i64,
    pub membership_tier: String,
    pub privacy_level: i64,
    pub is_verified: bool,
    pub verification_status: String,
    pub role: String,
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
    pub interests: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RoleCount {
    pub role: String,
    pub count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TierCount {
    pub membership_tier: String,
    pub count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VerificationCount {
    pub verification_status: String,
    pub count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UserStats {
    pub total_users: i64,
    pub users_by_role: Vec<RoleCount>,
    pub users_by_membership_tier: Vec<TierCount>,
    pub users_by_verification_status: Vec<VerificationCount>,
    pub recent_registrations: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsersPage {
    pub users: Vec<User>,
    pub pagination: UsersPagination,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EditUserData {
    pub first_name: String,
    pub last_name: String,
    pub age: i64,
    pub profession: String,
    pub annual_income: i64,
    pub net_worth: i64,
    pub membership_tier: String,
    pub privacy_level: i64,
    pub bio: String,
    pub interests: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UsersModal {
    #[default]
    None,
    Detail,
    Edit,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationBadge {
    Approved,
    Rejected,
    Pending,
}

pub fn user_api_path(id: &str) -> String {
    format!("{USERS_API_PATH}/{id}")
}

pub fn user_verify_api_path(id: &str) -> String {
    format!("{USERS_API_PATH}/{id}/verify")
}

pub fn user_role_api_path(id: &str) -> String {
    format!("{USERS_API_PATH}/{id}/role")
}

pub fn users_list_url(page: u32, filters: &UserFilters) -> String {
    let query = users_query_string(page, PAGE_SIZE, filters);
    if query.is_empty() {
        USERS_API_PATH.to_string()
    } else {
        format!("{USERS_API_PATH}?{query}")
    }
}

pub fn users_query_string(page: u32, limit: u32, filters: &UserFilters) -> String {
    let mut out = String::new();
    push_param(&mut out, "page", &page.to_string());
    push_param(&mut out, "limit", &limit.to_string());
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    if !filters.role.is_empty() {
        push_param(&mut out, "role", &filters.role);
    }
    if !filters.membership_tier.is_empty() {
        push_param(&mut out, "membershipTier", &filters.membership_tier);
    }
    if !filters.verification_status.is_empty() {
        push_param(&mut out, "verificationStatus", &filters.verification_status);
    }
    out
}

pub fn page_after_filter_change(_current_page: u32) -> u32 {
    1
}

pub fn total_pages(total: u32, page_size: u32) -> u32 {
    if page_size == 0 {
        0
    } else {
        total.div_ceil(page_size)
    }
}

pub fn page_in_range(new_page: u32, total_pages: u32) -> bool {
    new_page >= 1 && (total_pages == 0 || new_page <= total_pages)
}

pub fn pagination_range(page: u32, limit: u32, total: u32) -> (u32, u32) {
    let start = page
        .saturating_sub(1)
        .saturating_mul(limit)
        .saturating_add(1);
    let end = page.saturating_mul(limit).min(total);
    (start, end)
}

pub fn filters_are_active(filters: &UserFilters) -> bool {
    !filters.search.is_empty()
        || !filters.role.is_empty()
        || !filters.membership_tier.is_empty()
        || !filters.verification_status.is_empty()
}

pub fn user_management_guard(restoring: bool, snapshot: &AuthSnapshot) -> RouteGuard {
    if restoring {
        RouteGuard::Loading
    } else if !permissions(snapshot).manage_users {
        RouteGuard::Redirect(USER_MANAGEMENT_FALLBACK)
    } else {
        RouteGuard::Allow
    }
}

pub fn parse_users_response(status: u16, body: &str) -> Result<UsersPage, String> {
    let value = parse_success_root(status, body, USERS_FETCH_FALLBACK)?;
    let users = match value.get("data") {
        Some(Value::Array(rows)) => rows.iter().filter_map(parse_user).collect(),
        _ => return Err(USERS_FETCH_FALLBACK.to_string()),
    };
    Ok(UsersPage {
        users,
        pagination: parse_pagination(value.get("pagination")),
    })
}

pub fn parse_user_response(status: u16, body: &str) -> Result<User, String> {
    let value = parse_success_data(status, body, USER_FETCH_FALLBACK)?;
    parse_user(&value).ok_or_else(|| USER_FETCH_FALLBACK.to_string())
}

pub fn parse_user_stats_response(status: u16, body: &str) -> Result<UserStats, String> {
    let value = parse_success_data(status, body, STATS_FETCH_FALLBACK)?;
    Ok(UserStats {
        total_users: json_i64(value.get("totalUsers")).unwrap_or(0),
        users_by_role: value
            .get("usersByRole")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().filter_map(parse_role_count).collect())
            .unwrap_or_default(),
        users_by_membership_tier: value
            .get("usersByMembershipTier")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().filter_map(parse_tier_count).collect())
            .unwrap_or_default(),
        users_by_verification_status: value
            .get("usersByVerificationStatus")
            .and_then(Value::as_array)
            .map(|rows| rows.iter().filter_map(parse_verification_count).collect())
            .unwrap_or_default(),
        recent_registrations: json_i64(value.get("recentRegistrations")).unwrap_or(0),
    })
}

pub fn parse_update_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, UPDATE_FALLBACK)?;
    Ok(())
}

pub fn parse_verify_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, VERIFY_FALLBACK)?;
    Ok(())
}

pub fn parse_role_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, ROLE_FALLBACK)?;
    Ok(())
}

pub fn parse_delete_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, DELETE_FALLBACK)?;
    Ok(())
}

pub fn edit_data_from_user(user: &User) -> EditUserData {
    EditUserData {
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        age: user.age,
        profession: user.profession.clone(),
        annual_income: user.annual_income,
        net_worth: user.net_worth,
        membership_tier: user.membership_tier.clone(),
        privacy_level: user.privacy_level,
        bio: user.bio.clone().unwrap_or_default(),
        interests: user.interests.clone(),
    }
}

pub fn edit_payload(data: &EditUserData) -> Value {
    serde_json::json!({
        "firstName": data.first_name,
        "lastName": data.last_name,
        "age": data.age,
        "profession": data.profession,
        "annualIncome": data.annual_income,
        "netWorth": data.net_worth,
        "membershipTier": data.membership_tier,
        "privacyLevel": data.privacy_level,
        "bio": data.bio,
        "interests": data.interests,
    })
}

pub fn verify_payload(status: &str) -> Value {
    serde_json::json!({ "status": status })
}

pub fn role_payload(role: &str) -> Value {
    serde_json::json!({ "role": role })
}

pub fn is_valid_verify_status(status: &str) -> bool {
    matches!(status, "approved" | "rejected")
}

pub fn is_valid_role(role: &str) -> bool {
    matches!(role, "user" | "admin" | "super_admin")
}

pub fn is_valid_membership_tier(tier: &str) -> bool {
    matches!(tier, "Platinum" | "Diamond" | "Black Card")
}

pub fn is_valid_privacy_level(level: i64) -> bool {
    (1..=5).contains(&level)
}

pub fn parse_edit_int(raw: &str) -> Option<i64> {
    raw.trim().parse().ok()
}

pub fn shows_verify_actions(status: &str) -> bool {
    status == "pending"
}

pub fn verify_action_key(user_id: &str) -> String {
    format!("verify-{user_id}")
}

pub fn role_action_key(user_id: &str) -> String {
    format!("role-{user_id}")
}

pub fn action_is(action_loading: Option<&str>, key: &str) -> bool {
    action_loading == Some(key)
}

pub fn verification_badge(status: &str, is_verified: bool) -> VerificationBadge {
    if status == "approved" && is_verified {
        VerificationBadge::Approved
    } else if status == "rejected" {
        VerificationBadge::Rejected
    } else {
        VerificationBadge::Pending
    }
}

pub fn verification_badge_label(badge: VerificationBadge) -> &'static str {
    match badge {
        VerificationBadge::Approved => "已驗證",
        VerificationBadge::Rejected => "已拒絕",
        VerificationBadge::Pending => "待審核",
    }
}

pub fn membership_tier_label(tier: &str) -> String {
    match tier {
        "Platinum" => "白金卡".to_string(),
        "Diamond" => "鑽石卡".to_string(),
        "Black Card" => "黑卡".to_string(),
        other => other.to_string(),
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

pub fn role_label(role: &str) -> &'static str {
    match role {
        "super_admin" => "超級管理員",
        "admin" => "管理員",
        _ => "使用者",
    }
}

pub fn role_badge_class(role: &str) -> &'static str {
    match role {
        "admin" => "bg-purple-100 text-purple-800",
        "super_admin" => "bg-red-100 text-red-800",
        _ => "bg-gray-100 text-gray-800",
    }
}

pub fn verification_badge_class(badge: VerificationBadge) -> &'static str {
    match badge {
        VerificationBadge::Approved => "bg-green-100 text-green-800 border border-green-300",
        VerificationBadge::Rejected => "bg-red-100 text-red-800 border border-red-300",
        VerificationBadge::Pending => "bg-yellow-100 text-yellow-800 border border-yellow-300",
    }
}

pub fn user_initials(first_name: &str, last_name: &str) -> String {
    let mut out = String::new();
    if let Some(first) = first_name.chars().next() {
        out.push(first);
    }
    if let Some(last) = last_name.chars().next() {
        out.push(last);
    }
    out
}

pub fn display_name(user: &User) -> String {
    format!("{} {}", user.first_name, user.last_name)
}

pub fn pending_verification_count(stats: &UserStats) -> i64 {
    stats
        .users_by_verification_status
        .iter()
        .find(|row| row.verification_status == "pending")
        .map(|row| row.count)
        .unwrap_or(0)
}

pub fn admin_count(stats: &UserStats) -> i64 {
    stats
        .users_by_role
        .iter()
        .filter(|row| row.role == "admin" || row.role == "super_admin")
        .map(|row| row.count)
        .sum()
}

pub fn format_currency(amount: i64) -> String {
    format!("NT${}", format_thousands(amount))
}

pub fn format_joined_date(iso: &str) -> String {
    let date = iso.split('T').next().unwrap_or(iso);
    let mut parts = date.split('-');
    let Some(year) = parts.next() else {
        return iso.to_string();
    };
    let Some(month) = parts.next() else {
        return iso.to_string();
    };
    let Some(day) = parts.next() else {
        return iso.to_string();
    };
    if year.is_empty() || month.is_empty() || day.is_empty() {
        return iso.to_string();
    }
    let month = month.trim_start_matches('0');
    let day = day.trim_start_matches('0');
    let month = if month.is_empty() { "0" } else { month };
    let day = if day.is_empty() { "0" } else { day };
    format!("{year}/{month}/{day}")
}

pub async fn fetch_users(page: u32, filters: &UserFilters) -> Result<UsersPage, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = users_list_url(page, filters);
        return authorized_get(&url, USERS_FETCH_FALLBACK, parse_users_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (page, filters);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_user_stats() -> Result<UserStats, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            USER_STATS_API_PATH,
            STATS_FETCH_FALLBACK,
            parse_user_stats_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn fetch_user(id: &str) -> Result<User, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = user_api_path(id);
        return authorized_get(&url, USER_FETCH_FALLBACK, parse_user_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn update_user(id: &str, data: &EditUserData) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = user_api_path(id);
        return authorized_send(
            gloo_net::http::Request::put(&url),
            Some(edit_payload(data)),
            UPDATE_FALLBACK,
            parse_update_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, data);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn delete_user(id: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = user_api_path(id);
        return authorized_send(
            gloo_net::http::Request::delete(&url),
            None,
            DELETE_FALLBACK,
            parse_delete_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_ERROR_ZH.to_string())
    }
}

pub async fn verify_user(id: &str, status: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = user_verify_api_path(id);
        return authorized_send(
            gloo_net::http::Request::post(&url),
            Some(verify_payload(status)),
            VERIFY_FALLBACK,
            parse_verify_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, status);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn update_user_role(id: &str, role: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = user_role_api_path(id);
        return authorized_send(
            gloo_net::http::Request::post(&url),
            Some(role_payload(role)),
            ROLE_FALLBACK,
            parse_role_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, role);
        Err(NETWORK_ERROR_ZH.to_string())
    }
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
    finish_response(response, fallback, parse).await
}

#[cfg(target_arch = "wasm32")]
async fn authorized_send<T>(
    builder: gloo_net::http::RequestBuilder,
    body: Option<Value>,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let builder = authorized_builder(builder)?;
    let response = if let Some(payload) = body {
        let request = builder
            .header("Content-Type", "application/json")
            .json(&payload)
            .map_err(|_| NETWORK_ERROR.to_string())?;
        request
            .send()
            .await
            .map_err(|_| NETWORK_ERROR.to_string())?
    } else {
        builder
            .send()
            .await
            .map_err(|_| NETWORK_ERROR_ZH.to_string())?
    };
    finish_response(response, fallback, parse).await
}

#[cfg(target_arch = "wasm32")]
fn authorized_builder(
    builder: gloo_net::http::RequestBuilder,
) -> Result<gloo_net::http::RequestBuilder, String> {
    let token = crate::auth::read_stored_token().ok_or_else(|| NETWORK_ERROR.to_string())?;
    Ok(builder.header("Authorization", &crate::auth::bearer_authorization(&token)))
}

#[cfg(target_arch = "wasm32")]
async fn finish_response<T>(
    response: gloo_net::http::Response,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
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

fn parse_success_data(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    let value = parse_success_root(status, body, fallback)?;
    match value.get("data") {
        Some(Value::Null) | None => Err(fallback.to_string()),
        Some(data) => Ok(data.clone()),
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

fn parse_pagination(value: Option<&Value>) -> UsersPagination {
    let Some(value) = value else {
        return UsersPagination::default();
    };
    UsersPagination {
        page: json_u32(value.get("page")).unwrap_or(1),
        limit: json_u32(value.get("limit")).unwrap_or(PAGE_SIZE),
        total: json_u32(value.get("total")).unwrap_or(0),
        total_pages: json_u32(value.get("totalPages")).unwrap_or(1),
    }
}

fn parse_user(value: &Value) -> Option<User> {
    let id = json_id(field(value, "id", "id"))?;
    Some(User {
        id,
        email: json_string(field(value, "email", "email")).unwrap_or_default(),
        first_name: json_string(field(value, "firstName", "first_name")).unwrap_or_default(),
        last_name: json_string(field(value, "lastName", "last_name")).unwrap_or_default(),
        age: json_i64(field(value, "age", "age")).unwrap_or(0),
        profession: json_string(field(value, "profession", "profession")).unwrap_or_default(),
        annual_income: json_i64(field(value, "annualIncome", "annual_income")).unwrap_or(0),
        net_worth: json_i64(field(value, "netWorth", "net_worth")).unwrap_or(0),
        membership_tier: json_string(field(value, "membershipTier", "membership_tier"))
            .unwrap_or_default(),
        privacy_level: json_i64(field(value, "privacyLevel", "privacy_level")).unwrap_or(0),
        is_verified: json_bool(field(value, "isVerified", "is_verified")).unwrap_or(false),
        verification_status: json_string(field(value, "verificationStatus", "verification_status"))
            .unwrap_or_default(),
        role: json_string(field(value, "role", "role")).unwrap_or_default(),
        profile_picture: json_string(field(value, "profilePicture", "profile_picture")),
        bio: json_string(field(value, "bio", "bio")),
        interests: parse_interests(field(value, "interests", "interests")),
        created_at: json_string(field(value, "createdAt", "created_at")).unwrap_or_default(),
        updated_at: json_string(field(value, "updatedAt", "updated_at")).unwrap_or_default(),
    })
}

fn parse_role_count(value: &Value) -> Option<RoleCount> {
    Some(RoleCount {
        role: json_string(value.get("role"))?,
        count: json_i64(value.get("count")).unwrap_or(0),
    })
}

fn parse_tier_count(value: &Value) -> Option<TierCount> {
    let membership_tier = json_string(value.get("membership_tier"))
        .or_else(|| json_string(value.get("membershipTier")))?;
    Some(TierCount {
        membership_tier,
        count: json_i64(value.get("count")).unwrap_or(0),
    })
}

fn parse_verification_count(value: &Value) -> Option<VerificationCount> {
    let verification_status = json_string(value.get("verification_status"))
        .or_else(|| json_string(value.get("verificationStatus")))?;
    Some(VerificationCount {
        verification_status,
        count: json_i64(value.get("count")).unwrap_or(0),
    })
}

fn parse_interests(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(raw)) => serde_json::from_str::<Vec<String>>(raw).unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn field<'a>(value: &'a Value, camel: &str, snake: &str) -> Option<&'a Value> {
    value
        .get(camel)
        .or_else(|| value.get(snake))
        .filter(|item| !item.is_null())
}

fn format_thousands(amount: i64) -> String {
    let negative = amount < 0;
    let mut digits = amount.abs().to_string();
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

fn json_i64(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_u64().map(|v| v as i64))
            .or_else(|| n.as_f64().map(|v| v as i64)),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_u32(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(n) => n
            .as_u64()
            .map(|v| v as u32)
            .or_else(|| n.as_i64().and_then(|v| u32::try_from(v).ok()))
            .or_else(|| n.as_f64().and_then(|v| u32::try_from(v as i64).ok())),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_bool(value: Option<&Value>) -> Option<bool> {
    match value? {
        Value::Bool(flag) => Some(*flag),
        Value::Number(n) => n
            .as_i64()
            .map(|v| v != 0)
            .or_else(|| n.as_f64().map(|v| v != 0.0)),
        Value::String(s) => match s.as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}
