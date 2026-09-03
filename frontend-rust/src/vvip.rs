use serde_json::Value;

use crate::permissions::{AuthSnapshot, permissions};

pub const VVIP_EVENTS_API_PATH: &str = "/api/events";
pub const VVIP_EVENTS_LIMIT: u32 = 50;
pub const VVIP_PLACEHOLDER_IMAGE: &str = "/api/placeholder/600/400";
pub const PREVIEW_EVENT_LIMIT: usize = 3;

pub const CATEGORY_FILTERS: &[CategoryFilter] = &[
    CategoryFilter {
        id: "all",
        name: "全部活動",
    },
    CategoryFilter {
        id: "dining",
        name: "頂級餐飲",
    },
    CategoryFilter {
        id: "travel",
        name: "奢華旅遊",
    },
    CategoryFilter {
        id: "art",
        name: "藝術收藏",
    },
    CategoryFilter {
        id: "business",
        name: "商務社交",
    },
];

pub const PERKS: &[Perk] = &[
    Perk {
        title: "專屬禮賓服務",
        description: "24/7 專人服務，滿足您的每一個需求",
    },
    Perk {
        title: "獨家活動優先權",
        description: "搶先參與最頂級、最獨特的社交活動",
    },
    Perk {
        title: "最高隱私保障",
        description: "軍用級加密技術，絕對保護您的身份與隱私",
    },
    Perk {
        title: "客製化體驗",
        description: "根據您的喜好量身打造專屬活動與服務",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryFilter {
    pub id: &'static str,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Perk {
    pub title: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VvipSurface {
    Loading,
    Recruitment,
    Content,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VvipEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub date_time: String,
    pub location: String,
    pub pricing_vvip: Option<f64>,
    pub pricing_vip: Option<f64>,
    pub current_attendees: u32,
    pub capacity: u32,
    pub category_id: String,
    pub category_name: String,
    pub exclusivity_level: Option<String>,
    pub images: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantsView {
    pub participants: Vec<Value>,
    pub total_count: u32,
    pub paid_participant_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticipantsFetch {
    Ok(ParticipantsView),
    Unauthorized,
    Forbidden,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewStatus {
    SignedOut,
    Unauthorized,
    Restricted,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewAttendee {
    pub profession: Option<String>,
    pub industry: Option<String>,
    pub membership_tier: Option<String>,
    pub profile_picture: Option<String>,
    pub gender: Option<String>,
    pub event_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewBundle {
    pub status: PreviewStatus,
    pub attendees: Vec<PreviewAttendee>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessCheck {
    pub has_access: bool,
    pub payment_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VvipStats {
    pub member_count: Option<u32>,
    pub event_count: u32,
    pub venue_count: u32,
}

pub fn vvip_surface(restoring: bool, snapshot: &AuthSnapshot) -> VvipSurface {
    if restoring {
        VvipSurface::Loading
    } else if permissions(snapshot).access_vvip {
        VvipSurface::Content
    } else {
        VvipSurface::Recruitment
    }
}

pub fn classify_category(name: &str, slug: Option<&str>) -> &'static str {
    let slug = slug.unwrap_or("");
    let combined = format!("{slug} {name}").to_lowercase();
    if matches_any(
        &combined,
        &["dinner", "dining", "wine", "晚宴", "餐飲", "品酒"],
    ) {
        "dining"
    } else if matches_any(&combined, &["yacht", "travel", "遊艇", "旅遊"]) {
        "travel"
    } else if matches_any(&combined, &["art", "藝術"]) {
        "art"
    } else if matches_any(&combined, &["business", "商務"]) {
        "business"
    } else {
        "other"
    }
}

pub fn filter_events(events: &[VvipEvent], category: &str) -> Vec<VvipEvent> {
    if category == "all" || category.is_empty() {
        events.to_vec()
    } else {
        events
            .iter()
            .filter(|event| event.category_id == category)
            .cloned()
            .collect()
    }
}

pub fn category_count(events: &[VvipEvent], id: &str) -> u32 {
    filter_events(events, id).len() as u32
}

pub fn parse_vvip_events_response(body: &str) -> Vec<VvipEvent> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return Vec::new();
    }
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(parse_vvip_event).collect())
        .unwrap_or_default()
}

pub fn parse_participants_response(status: u16, body: &str) -> ParticipantsFetch {
    match status {
        401 => ParticipantsFetch::Unauthorized,
        403 => ParticipantsFetch::Forbidden,
        200..=299 => parse_participants_ok(body),
        _ => ParticipantsFetch::Empty,
    }
}

pub fn parse_participant_access_response(status: u16, body: &str) -> Option<AccessCheck> {
    if !(200..300).contains(&status) {
        return None;
    }
    let value: Value = serde_json::from_str(body).ok()?;
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let data = value.get("data")?;
    Some(AccessCheck {
        has_access: data
            .get("hasAccess")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        payment_required: data
            .get("paymentRequired")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

pub fn preview_attendee(value: &Value, event_id: &str) -> Option<PreviewAttendee> {
    if !value.is_object() {
        return None;
    }
    Some(PreviewAttendee {
        profession: json_string(value.get("profession")),
        industry: json_string(value.get("company")),
        membership_tier: json_string(value.get("membershipTier")),
        profile_picture: json_string(value.get("profilePicture"))
            .or_else(|| json_string(value.get("profile_picture"))),
        gender: json_string(value.get("gender")),
        event_id: event_id.to_string(),
        display_name: None,
        email: None,
        phone: None,
    })
}

pub fn preview_bundle(has_token: bool, fetches: &[ParticipantsFetch]) -> PreviewBundle {
    if !has_token {
        return PreviewBundle {
            status: PreviewStatus::SignedOut,
            attendees: Vec::new(),
        };
    }
    if fetches
        .iter()
        .any(|fetch| matches!(fetch, ParticipantsFetch::Unauthorized))
    {
        return PreviewBundle {
            status: PreviewStatus::Unauthorized,
            attendees: Vec::new(),
        };
    }
    let attendees = fetches
        .iter()
        .filter_map(|fetch| match fetch {
            ParticipantsFetch::Ok(view) => Some(&view.participants),
            _ => None,
        })
        .flatten()
        .filter_map(|value| preview_attendee(value, ""))
        .collect::<Vec<_>>();
    if attendees.is_empty()
        && fetches
            .iter()
            .any(|fetch| matches!(fetch, ParticipantsFetch::Forbidden))
        && fetches
            .iter()
            .all(|fetch| !matches!(fetch, ParticipantsFetch::Ok(_)))
    {
        return PreviewBundle {
            status: PreviewStatus::Restricted,
            attendees: Vec::new(),
        };
    }
    PreviewBundle {
        status: PreviewStatus::Ready,
        attendees,
    }
}

pub fn filter_preview_by_interest(
    attendees: Vec<PreviewAttendee>,
    interested_in: Option<&str>,
) -> Vec<PreviewAttendee> {
    let Some(pref) = interested_in else {
        return attendees;
    };
    match pref {
        "female" | "male" => attendees
            .into_iter()
            .filter(|a| {
                let g = a.gender.as_deref().unwrap_or("");
                g == pref || g.is_empty() || g == "prefer_not_to_say"
            })
            .collect(),
        "everyone" | "prefer_not_to_say" | "" => attendees,
        _ => attendees,
    }
}

pub fn vvip_stats(events: &[VvipEvent], preview: &[PreviewAttendee]) -> VvipStats {
    let mut venues: Vec<&str> = events
        .iter()
        .map(|event| event.location.as_str())
        .filter(|location| !location.is_empty())
        .collect();
    venues.sort_unstable();
    venues.dedup();
    VvipStats {
        member_count: if preview.is_empty() {
            None
        } else {
            Some(preview.len() as u32)
        },
        event_count: events.len() as u32,
        venue_count: venues.len() as u32,
    }
}

pub fn recruitment_join_href(is_authenticated: bool) -> &'static str {
    if is_authenticated {
        "/profile"
    } else {
        "/register"
    }
}

pub fn event_image(event: &VvipEvent) -> String {
    event
        .images
        .first()
        .filter(|url| !url.is_empty())
        .cloned()
        .unwrap_or_else(|| VVIP_PLACEHOLDER_IMAGE.to_string())
}

pub fn format_vvip_price(event: &VvipEvent) -> String {
    crate::events::format_price(event.pricing_vvip, event.pricing_vip)
}

pub fn participants_path(event_id: &str) -> String {
    format!("/api/events/{event_id}/participants")
}

pub fn participant_access_path(event_id: &str) -> String {
    format!("/api/events/{event_id}/participant-access")
}

pub async fn fetch_vvip_events() -> Vec<VvipEvent> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{VVIP_EVENTS_API_PATH}?page=1&limit={VVIP_EVENTS_LIMIT}");
        let response = match gloo_net::http::Request::get(&url).send().await {
            Ok(response) => response,
            Err(_) => return Vec::new(),
        };
        if !(200..300).contains(&response.status()) {
            return Vec::new();
        }
        let body = response.text().await.unwrap_or_default();
        return parse_vvip_events_response(&body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    Vec::new()
}

pub async fn fetch_participant_access(event_id: &str, token: Option<&str>) -> Option<AccessCheck> {
    let token = token.filter(|value| !value.is_empty())?;
    #[cfg(target_arch = "wasm32")]
    {
        let url = participant_access_path(event_id);
        let response = match authorized_get(&url, token).await {
            Ok(response) => response,
            Err(_) => return None,
        };
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return parse_participant_access_response(status, &body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, token);
        None
    }
}

pub async fn fetch_participants(event_id: &str, token: Option<&str>) -> ParticipantsFetch {
    let Some(token) = token.filter(|value| !value.is_empty()) else {
        return ParticipantsFetch::Unauthorized;
    };
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("{}?page=1&limit=20", participants_path(event_id));
        let response = match authorized_get(&url, token).await {
            Ok(response) => response,
            Err(_) => return ParticipantsFetch::Empty,
        };
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return parse_participants_response(status, &body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, token);
        ParticipantsFetch::Empty
    }
}

pub async fn fetch_preview(token: Option<&str>, events: &[VvipEvent]) -> PreviewBundle {
    let Some(token) = token.filter(|value| !value.is_empty()) else {
        return preview_bundle(false, &[]);
    };
    #[cfg(target_arch = "wasm32")]
    {
        let mut fetches = Vec::new();
        let mut attendees = Vec::new();
        for event in events
            .iter()
            .filter(|event| event.current_attendees > 0)
            .take(PREVIEW_EVENT_LIMIT)
        {
            match fetch_participant_access(&event.id, Some(token)).await {
                None => {
                    fetches.push(ParticipantsFetch::Unauthorized);
                    break;
                }
                Some(check) if !check.has_access => {
                    fetches.push(ParticipantsFetch::Forbidden);
                }
                Some(_) => {
                    let fetch = fetch_participants(&event.id, Some(token)).await;
                    if let ParticipantsFetch::Ok(view) = &fetch {
                        attendees.extend(
                            view.participants
                                .iter()
                                .filter_map(|value| preview_attendee(value, &event.id)),
                        );
                    }
                    if matches!(fetch, ParticipantsFetch::Unauthorized) {
                        fetches.push(fetch);
                        break;
                    }
                    fetches.push(fetch);
                }
            }
        }
        let mut bundle = preview_bundle(true, &fetches);
        if bundle.status == PreviewStatus::Ready {
            bundle.attendees = attendees;
        }
        return bundle;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (token, events);
        preview_bundle(true, &[])
    }
}

#[cfg(target_arch = "wasm32")]
async fn authorized_get(
    url: &str,
    token: &str,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    gloo_net::http::Request::get(url)
        .header("Authorization", &crate::auth::bearer_authorization(token))
        .send()
        .await
}

fn parse_participants_ok(body: &str) -> ParticipantsFetch {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return ParticipantsFetch::Empty;
    };
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        return ParticipantsFetch::Empty;
    }
    let Some(data) = value.get("data") else {
        return ParticipantsFetch::Empty;
    };
    let participants = data
        .get("participants")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    ParticipantsFetch::Ok(ParticipantsView {
        participants,
        total_count: json_u32(data.get("totalCount")).unwrap_or(0),
        paid_participant_count: json_u32(data.get("paidParticipantCount")).unwrap_or(0),
    })
}

fn parse_vvip_event(value: &Value) -> Option<VvipEvent> {
    let id = json_id(value.get("id"))?;
    let name = json_string(value.get("name")).filter(|s| !s.is_empty())?;
    let category = value.get("category");
    let category_name = json_string(category.and_then(|item| item.get("name")))
        .or_else(|| json_string(value.get("category_name")))
        .unwrap_or_default();
    let category_slug = json_string(category.and_then(|item| item.get("slug")))
        .or_else(|| json_string(category.and_then(|item| item.get("id"))));
    let location = json_string(value.get("venue").and_then(|venue| venue.get("name")))
        .or_else(|| json_string(value.get("venue").and_then(|venue| venue.get("address"))))
        .or_else(|| json_string(value.get("venue_name")))
        .unwrap_or_default();
    let pricing = value.get("pricing");
    Some(VvipEvent {
        id,
        name,
        description: json_string(value.get("description")).unwrap_or_default(),
        date_time: json_string(value.get("dateTime"))
            .or_else(|| json_string(value.get("date_time")))
            .unwrap_or_default(),
        location,
        pricing_vvip: json_f64(pricing.and_then(|item| item.get("vvip"))),
        pricing_vip: json_f64(pricing.and_then(|item| item.get("vip"))),
        current_attendees: json_u32(value.get("currentAttendees")).unwrap_or(0),
        capacity: json_u32(value.get("capacity")).unwrap_or(0),
        category_id: classify_category(category_name.as_str(), category_slug.as_deref())
            .to_string(),
        category_name,
        exclusivity_level: json_string(value.get("exclusivityLevel")),
        images: parse_string_list(value.get("images")),
        tags: parse_string_list(value.get("tags")),
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
            .filter_map(|item| item.as_str().map(str::to_string))
            .filter(|item| !item.is_empty())
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

fn matches_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
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
        Value::String(s) if !s.is_empty() => Some(s.clone()),
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
