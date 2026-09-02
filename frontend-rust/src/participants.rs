use std::collections::HashMap;

use serde_json::Value;

pub const PAGE_SIZE: u32 = 12;
pub const NETWORK_ERROR: &str = "Network error occurred";
pub const ACCESS_FETCH_FALLBACK: &str = "Failed to check access";
pub const PARTICIPANTS_FETCH_FALLBACK: &str = "Failed to fetch participants";
pub const PRIVACY_FETCH_FALLBACK: &str = "Failed to fetch privacy settings";
pub const PRIVACY_UPDATE_FALLBACK: &str = "Failed to update privacy settings";
pub const PRIVACY_UPDATE_SUCCESS: &str = "隱私設定已成功更新";
pub const CONTACT_FALLBACK: &str = "Failed to send message";
pub const DETAIL_FETCH_FALLBACK: &str = "Failed to get participant details";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewerRelationship {
    Unpaid,
    PaidStandard,
    PaidPremium,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticipantViewAccess {
    pub can_view_participants: bool,
    pub max_privacy_level_visible: i64,
    pub can_see_contact_info: bool,
    pub can_initiate_contact: bool,
    pub participant_count_visible: bool,
    pub access_level: i64,
}

impl ParticipantViewAccess {
    pub const fn denied() -> Self {
        Self {
            can_view_participants: false,
            max_privacy_level_visible: 0,
            can_see_contact_info: false,
            can_initiate_contact: false,
            participant_count_visible: true,
            access_level: 0,
        }
    }
}

impl Default for ParticipantViewAccess {
    fn default() -> Self {
        Self::denied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipantRow {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub age: Option<i64>,
    pub profession: Option<String>,
    pub company: Option<String>,
    pub city: Option<String>,
    pub membership_tier: String,
    pub interests: Option<String>,
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
    pub effective_privacy_level: i64,
    pub can_contact: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FilteredParticipant {
    pub id: String,
    pub display_name: String,
    pub profession: Option<String>,
    pub company: Option<String>,
    pub membership_tier: String,
    pub interests: Vec<String>,
    pub profile_picture: Option<String>,
    pub age_range: Option<String>,
    pub city: Option<String>,
    pub bio: Option<String>,
    pub privacy_level: i64,
    pub can_contact: bool,
    pub contact_info: Option<ContactInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipantList {
    pub participants: Vec<FilteredParticipant>,
    pub total_count: u32,
    pub paid_participant_count: u32,
    pub unpaid_participant_count: u32,
    pub viewer_access: ParticipantViewAccess,
    pub participant_count_by_tier: HashMap<String, u32>,
}

impl ParticipantList {
    pub fn tier_count(&self, tier: &str) -> u32 {
        self.participant_count_by_tier
            .get(tier)
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipantAccessCheck {
    pub has_access: bool,
    pub access_level: ParticipantViewAccess,
    pub payment_required: bool,
    pub payment_status: String,
    pub registration_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipantDetail {
    pub participant: FilteredParticipant,
    pub viewer_access_level: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ParticipantFilters {
    pub search: String,
    pub membership_tier: String,
    pub profession: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivacySettings {
    pub privacy_level: i64,
    pub allow_contact: bool,
    pub show_in_list: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivacyLevelCopy {
    pub title: &'static str,
    pub description: &'static str,
    pub visibility: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParticipantsPhase {
    Loading,
    Paywall { payment_pending: bool },
    Error(String),
    Empty,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrivacyPhase {
    Loading,
    Error(String),
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactDraft {
    pub participant: FilteredParticipant,
    pub message: String,
    pub sending: bool,
    pub sent: bool,
}

pub fn participants_api_path(event_id: &str) -> String {
    format!("/api/events/{event_id}/participants")
}

pub fn participant_access_api_path(event_id: &str) -> String {
    format!("/api/events/{event_id}/participant-access")
}

pub fn participant_detail_api_path(event_id: &str, participant_id: &str) -> String {
    format!("/api/events/{event_id}/participants/{participant_id}")
}

pub fn participant_contact_api_path(event_id: &str, participant_id: &str) -> String {
    format!("/api/events/{event_id}/participants/{participant_id}/contact")
}

pub fn privacy_settings_api_path(event_id: &str) -> String {
    format!("/api/events/{event_id}/privacy-settings")
}

pub fn participants_query_string(page: u32, limit: u32, filters: &ParticipantFilters) -> String {
    let mut out = String::new();
    push_param(&mut out, "page", &page.to_string());
    push_param(&mut out, "limit", &limit.to_string());
    if !filters.membership_tier.is_empty() {
        push_param(&mut out, "membershipTier", &filters.membership_tier);
    }
    if !filters.profession.is_empty() {
        push_param(&mut out, "profession", &filters.profession);
    }
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    out
}

pub fn page_after_filter_change(_current_page: u32) -> u32 {
    1
}

pub fn total_pages(total_count: u32, page_size: u32) -> u32 {
    if page_size == 0 {
        0
    } else {
        total_count.div_ceil(page_size)
    }
}

pub fn viewer_relationship(
    payment_status: Option<&str>,
    membership_tier: &str,
) -> ViewerRelationship {
    if payment_status != Some("paid") {
        ViewerRelationship::Unpaid
    } else if matches!(membership_tier, "Diamond" | "Black Card") {
        ViewerRelationship::PaidPremium
    } else {
        ViewerRelationship::PaidStandard
    }
}

pub const fn participant_view_access(relationship: ViewerRelationship) -> ParticipantViewAccess {
    match relationship {
        ViewerRelationship::Unpaid => ParticipantViewAccess::denied(),
        ViewerRelationship::PaidStandard => ParticipantViewAccess {
            can_view_participants: true,
            max_privacy_level_visible: 3,
            can_see_contact_info: false,
            can_initiate_contact: true,
            participant_count_visible: true,
            access_level: 3,
        },
        ViewerRelationship::PaidPremium => ParticipantViewAccess {
            can_view_participants: true,
            max_privacy_level_visible: 5,
            can_see_contact_info: true,
            can_initiate_contact: true,
            participant_count_visible: true,
            access_level: 4,
        },
    }
}

pub fn mask_participant(
    participant: &ParticipantRow,
    access: ParticipantViewAccess,
) -> Option<FilteredParticipant> {
    let level = participant.effective_privacy_level;
    if !access.can_view_participants || level > access.max_privacy_level_visible {
        return None;
    }

    let mut masked = FilteredParticipant {
        id: participant.id.clone(),
        display_name: abbreviated_name(participant),
        membership_tier: participant.membership_tier.clone(),
        privacy_level: level,
        can_contact: participant.can_contact != 0 && access.can_initiate_contact,
        ..FilteredParticipant::default()
    };

    if level >= 1 {
        masked.profession =
            Some(profession_category(participant.profession.as_deref()).to_string());
        masked.interests = interests(participant.interests.as_deref());
        masked.profile_picture = participant.profile_picture.clone();
        masked.age_range = Some(age_range(participant.age).to_string());
    }

    if level >= 2 {
        if let Some(company) = participant.company.as_deref() {
            masked.company = Some(company_category(company).to_string());
        }
        if let Some(city) = participant.city.as_deref() {
            masked.city = Some(city.to_string());
        }
    }

    if level >= 3 {
        masked.display_name = format!("{} {}", participant.first_name, participant.last_name);
        if let Some(company) = participant.company.as_deref() {
            masked.company = Some(company.to_string());
        }
        if let Some(bio) = participant.bio.as_deref() {
            masked.bio = Some(bio.chars().take(200).collect());
        }
    }

    if level >= 4 && access.can_see_contact_info {
        if let Some(bio) = participant.bio.as_deref() {
            masked.bio = Some(bio.to_string());
        }
        masked.contact_info = Some(ContactInfo {
            email: Some(participant.email.clone()),
            phone: None,
        });
    }

    if level >= 5 && access.can_see_contact_info {
        masked.contact_info = Some(ContactInfo {
            email: Some(participant.email.clone()),
            phone: participant.phone.clone(),
        });
    }

    Some(masked)
}

pub fn parse_access_check_response(
    status: u16,
    body: &str,
) -> Result<ParticipantAccessCheck, String> {
    let value = parse_success_data(status, body, ACCESS_FETCH_FALLBACK)?;
    Ok(ParticipantAccessCheck {
        has_access: json_bool(value.get("hasAccess")).unwrap_or(false),
        access_level: parse_view_access(value.get("accessLevel")),
        payment_required: json_bool(value.get("paymentRequired")).unwrap_or(true),
        payment_status: json_string(value.get("paymentStatus"))
            .unwrap_or_else(|| "none".to_string()),
        registration_status: json_string(value.get("registrationStatus")),
    })
}

pub fn parse_participants_response(status: u16, body: &str) -> Result<ParticipantList, String> {
    let value = parse_success_data(status, body, PARTICIPANTS_FETCH_FALLBACK)?;
    let participants = value
        .get("participants")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_filtered_participant).collect())
        .ok_or(PARTICIPANTS_FETCH_FALLBACK)?;
    Ok(ParticipantList {
        participants,
        total_count: json_u32(value.get("totalCount")).unwrap_or(0),
        paid_participant_count: json_u32(value.get("paidParticipantCount")).unwrap_or(0),
        unpaid_participant_count: json_u32(value.get("unpaidParticipantCount")).unwrap_or(0),
        viewer_access: parse_view_access(value.get("viewerAccess")),
        participant_count_by_tier: parse_tier_counts(value.get("participantCountByTier")),
    })
}

pub fn parse_participant_detail_response(
    status: u16,
    body: &str,
) -> Result<ParticipantDetail, String> {
    let value = parse_success_data(status, body, DETAIL_FETCH_FALLBACK)?;
    let participant = value
        .get("participant")
        .and_then(parse_filtered_participant)
        .ok_or(DETAIL_FETCH_FALLBACK)?;
    Ok(ParticipantDetail {
        participant,
        viewer_access_level: json_i64(value.get("viewerAccess")).unwrap_or(0),
    })
}

pub fn parse_contact_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, CONTACT_FALLBACK)?;
    Ok(())
}

pub fn parse_privacy_settings_response(status: u16, body: &str) -> Result<PrivacySettings, String> {
    let value = parse_success_data(status, body, PRIVACY_FETCH_FALLBACK)?;
    let privacy_level = json_i64(value.get("privacy_level")).ok_or(PRIVACY_FETCH_FALLBACK)?;
    let allow_contact = json_bool(value.get("allow_contact")).ok_or(PRIVACY_FETCH_FALLBACK)?;
    let show_in_list = json_bool(value.get("show_in_list")).ok_or(PRIVACY_FETCH_FALLBACK)?;
    Ok(PrivacySettings {
        privacy_level,
        allow_contact,
        show_in_list,
    })
}

pub fn parse_privacy_update_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, PRIVACY_UPDATE_FALLBACK)?;
    Ok(())
}

pub fn privacy_settings_payload(settings: &PrivacySettings) -> Value {
    serde_json::json!({
        "privacyLevel": settings.privacy_level,
        "allowContact": settings.allow_contact,
        "showInList": settings.show_in_list,
    })
}

pub fn contact_payload(message: &str) -> Value {
    serde_json::json!({ "message": message })
}

pub fn can_send_contact(message: &str) -> bool {
    !message.trim().is_empty()
}

pub fn participants_phase(
    loading: bool,
    access: Option<&ParticipantAccessCheck>,
    list: Option<&ParticipantList>,
    error: Option<&str>,
) -> ParticipantsPhase {
    if loading && list.is_none() {
        return ParticipantsPhase::Loading;
    }
    if access.map(|check| check.has_access) != Some(true) {
        return ParticipantsPhase::Paywall {
            payment_pending: access
                .map(|check| check.payment_status == "pending")
                .unwrap_or(false),
        };
    }
    if let Some(error) = error.map(str::trim).filter(|value| !value.is_empty()) {
        return ParticipantsPhase::Error(error.to_string());
    }
    match list {
        Some(list) if !list.participants.is_empty() => ParticipantsPhase::Ready,
        _ => ParticipantsPhase::Empty,
    }
}

pub fn privacy_settings_phase(
    loading: bool,
    settings: Option<&PrivacySettings>,
    error: Option<&str>,
) -> PrivacyPhase {
    if loading {
        return PrivacyPhase::Loading;
    }
    if settings.is_some() {
        return PrivacyPhase::Ready;
    }
    PrivacyPhase::Error(
        error
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(PRIVACY_FETCH_FALLBACK)
            .to_string(),
    )
}

pub fn privacy_level_description(level: i64) -> Option<PrivacyLevelCopy> {
    match level {
        1 => Some(PrivacyLevelCopy {
            title: "公開資料",
            description: "顯示基本資訊：姓名縮寫、年齡範圍、職業類別、會員等級",
            visibility: "所有付費參與者可見",
        }),
        2 => Some(PrivacyLevelCopy {
            title: "半私人資料",
            description: "顯示完整名字、公司行業、經驗範圍、城市",
            visibility: "所有付費參與者可見",
        }),
        3 => Some(PrivacyLevelCopy {
            title: "選擇性分享",
            description: "顯示全名、公司名稱、具體興趣、專業成就",
            visibility: "所有付費參與者可見",
        }),
        4 => Some(PrivacyLevelCopy {
            title: "增強資料",
            description: "顯示聯絡資訊、社交連結、詳細履歷",
            visibility: "Diamond 和 Black Card 會員可見",
        }),
        5 => Some(PrivacyLevelCopy {
            title: "完全公開",
            description: "顯示直接聯絡方式、個人興趣、網絡連接",
            visibility: "Diamond 和 Black Card 會員可見",
        }),
        _ => None,
    }
}

pub fn privacy_level_indicator_class(level: i64) -> &'static str {
    match level {
        1 => "bg-green-500/20 text-green-400",
        2 => "bg-blue-500/20 text-blue-400",
        3 => "bg-yellow-500/20 text-yellow-400",
        4 => "bg-orange-500/20 text-orange-400",
        5 => "bg-red-500/20 text-red-400",
        _ => "bg-gray-500/20 text-gray-400",
    }
}

pub fn privacy_level_card_class(level: i64) -> &'static str {
    match level {
        1 => "border-green-500/30 bg-green-500/10 text-green-400",
        2 => "border-blue-500/30 bg-blue-500/10 text-blue-400",
        3 => "border-yellow-500/30 bg-yellow-500/10 text-yellow-400",
        4 => "border-orange-500/30 bg-orange-500/10 text-orange-400",
        5 => "border-red-500/30 bg-red-500/10 text-red-400",
        _ => "border-luxury-gold/20 bg-luxury-midnight-black/30",
    }
}

pub fn privacy_level_dot_class(level: i64) -> &'static str {
    match level {
        1 => "text-green-400",
        2 => "text-blue-400",
        3 => "text-yellow-400",
        4 => "text-orange-400",
        5 => "text-red-400",
        _ => "text-luxury-platinum",
    }
}

pub fn membership_tier_badge_class(tier: &str) -> &'static str {
    match tier {
        "Platinum" => "bg-gray-500/20 text-gray-300 border-gray-500/30",
        "Diamond" => "bg-blue-500/20 text-blue-400 border-blue-500/30",
        "Black Card" => "bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30",
        _ => "bg-gray-500/20 text-gray-300 border-gray-500/30",
    }
}

pub fn display_initial(display_name: &str) -> String {
    display_name
        .chars()
        .next()
        .map(|ch| ch.to_string())
        .unwrap_or_default()
}

pub fn visible_interests(interests: &[String]) -> &[String] {
    let end = interests.len().min(3);
    &interests[..end]
}

pub fn page_in_range(new_page: u32, total_pages: u32) -> bool {
    new_page >= 1 && (total_pages == 0 || new_page <= total_pages)
}

pub async fn fetch_participant_access(event_id: &str) -> Result<ParticipantAccessCheck, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = participant_access_api_path(event_id);
        return authorized_get(&url, ACCESS_FETCH_FALLBACK, parse_access_check_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = event_id;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_participants(
    event_id: &str,
    page: u32,
    filters: &ParticipantFilters,
) -> Result<ParticipantList, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!(
            "{}?{}",
            participants_api_path(event_id),
            participants_query_string(page, PAGE_SIZE, filters)
        );
        return authorized_get(
            &url,
            PARTICIPANTS_FETCH_FALLBACK,
            parse_participants_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, page, filters);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_participant_detail(
    event_id: &str,
    participant_id: &str,
) -> Result<ParticipantDetail, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = participant_detail_api_path(event_id, participant_id);
        return authorized_get(
            &url,
            DETAIL_FETCH_FALLBACK,
            parse_participant_detail_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, participant_id);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn initiate_contact(
    event_id: &str,
    participant_id: &str,
    message: &str,
) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = participant_contact_api_path(event_id, participant_id);
        return authorized_send(
            gloo_net::http::Request::post(&url),
            Some(contact_payload(message)),
            CONTACT_FALLBACK,
            parse_contact_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, participant_id, message);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_privacy_settings(event_id: &str) -> Result<PrivacySettings, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = privacy_settings_api_path(event_id);
        return authorized_get(
            &url,
            PRIVACY_FETCH_FALLBACK,
            parse_privacy_settings_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = event_id;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn update_privacy_settings(
    event_id: &str,
    settings: &PrivacySettings,
) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = privacy_settings_api_path(event_id);
        return authorized_send(
            gloo_net::http::Request::put(&url),
            Some(privacy_settings_payload(settings)),
            PRIVACY_UPDATE_FALLBACK,
            parse_privacy_update_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, settings);
        Err(NETWORK_ERROR.to_string())
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
    let builder = authorized_builder(builder)?.header("Content-Type", "application/json");
    let payload = body.unwrap_or(Value::Null);
    let request = builder
        .json(&payload)
        .map_err(|_| NETWORK_ERROR.to_string())?;
    let response = request
        .send()
        .await
        .map_err(|_| NETWORK_ERROR.to_string())?;
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

fn parse_success_root(_status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
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

fn parse_view_access(value: Option<&Value>) -> ParticipantViewAccess {
    let Some(value) = value else {
        return ParticipantViewAccess::denied();
    };
    ParticipantViewAccess {
        can_view_participants: json_bool(value.get("canViewParticipants")).unwrap_or(false),
        max_privacy_level_visible: json_i64(value.get("maxPrivacyLevelVisible")).unwrap_or(0),
        can_see_contact_info: json_bool(value.get("canSeeContactInfo")).unwrap_or(false),
        can_initiate_contact: json_bool(value.get("canInitiateContact")).unwrap_or(false),
        participant_count_visible: json_bool(value.get("participantCountVisible")).unwrap_or(false),
        access_level: json_i64(value.get("accessLevel")).unwrap_or(0),
    }
}

fn parse_filtered_participant(value: &Value) -> Option<FilteredParticipant> {
    let id = json_id(value.get("id"))?;
    let display_name = json_string(value.get("displayName")).unwrap_or_default();
    Some(FilteredParticipant {
        id,
        display_name,
        profession: json_string(value.get("profession")),
        company: json_string(value.get("company")),
        membership_tier: json_string(value.get("membershipTier")).unwrap_or_default(),
        interests: parse_interests_value(value.get("interests")),
        profile_picture: json_string(value.get("profilePicture")),
        age_range: json_string(value.get("ageRange")),
        city: json_string(value.get("city")),
        bio: json_string(value.get("bio")),
        privacy_level: json_i64(value.get("privacyLevel")).unwrap_or(0),
        can_contact: json_bool(value.get("canContact")).unwrap_or(false),
        contact_info: value.get("contactInfo").and_then(parse_contact_info),
    })
}

fn parse_contact_info(value: &Value) -> Option<ContactInfo> {
    if value.is_null() {
        return None;
    }
    Some(ContactInfo {
        email: json_string(value.get("email")),
        phone: json_string(value.get("phone")),
    })
}

fn parse_interests_value(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(Value::String(raw)) => interests(Some(raw)),
        _ => Vec::new(),
    }
}

fn parse_tier_counts(value: Option<&Value>) -> HashMap<String, u32> {
    let Some(Value::Object(map)) = value else {
        return HashMap::new();
    };
    map.iter()
        .filter_map(|(key, count)| json_u32(Some(count)).map(|n| (key.clone(), n)))
        .collect()
}

fn profession_category(profession: Option<&str>) -> &'static str {
    let Some(profession) = profession else {
        return "Professional";
    };
    let profession = profession.to_lowercase();
    let categories: [(&str, &[&str]); 9] = [
        (
            "Technology",
            &[
                "software",
                "engineer",
                "developer",
                "tech",
                "it",
                "data",
                "ai",
                "machine learning",
            ],
        ),
        (
            "Finance",
            &[
                "finance",
                "banking",
                "investment",
                "fund",
                "trading",
                "analyst",
                "wealth",
            ],
        ),
        (
            "Healthcare",
            &[
                "doctor",
                "physician",
                "medical",
                "health",
                "nurse",
                "surgeon",
            ],
        ),
        (
            "Business",
            &[
                "ceo",
                "manager",
                "director",
                "executive",
                "business",
                "entrepreneur",
            ],
        ),
        (
            "Legal",
            &["lawyer", "attorney", "legal", "counsel", "judge"],
        ),
        (
            "Real Estate",
            &["real estate", "property", "development", "construction"],
        ),
        ("Consulting", &["consultant", "consulting", "advisory"]),
        (
            "Education",
            &["professor", "teacher", "education", "academic"],
        ),
        (
            "Media",
            &["media", "journalist", "marketing", "advertising", "pr"],
        ),
    ];
    categories
        .iter()
        .find(|(_, keywords)| keywords.iter().any(|keyword| profession.contains(keyword)))
        .map_or("Professional", |(category, _)| category)
}

fn company_category(company: &str) -> &'static str {
    let company = company.to_lowercase();
    if ["tech", "software", "microsoft", "google", "apple"]
        .iter()
        .any(|keyword| company.contains(keyword))
    {
        "Technology Company"
    } else if ["bank", "financial", "investment"]
        .iter()
        .any(|keyword| company.contains(keyword))
    {
        "Financial Services"
    } else {
        "Private Company"
    }
}

fn age_range(age: Option<i64>) -> &'static str {
    match age.unwrap_or(0) {
        ..=24 => "18-24",
        25..=29 => "25-29",
        30..=34 => "30-34",
        35..=39 => "35-39",
        40..=44 => "40-44",
        45..=49 => "45-49",
        50..=54 => "50-54",
        55..=59 => "55-59",
        60..=64 => "60-64",
        _ => "65+",
    }
}

fn interests(raw: Option<&str>) -> Vec<String> {
    raw.and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .unwrap_or_default()
        .into_iter()
        .take(3)
        .collect()
}

fn abbreviated_name(participant: &ParticipantRow) -> String {
    let initial = participant.last_name.chars().next().unwrap_or_default();
    format!("{} {initial}.", participant.first_name)
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
