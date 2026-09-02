use serde_json::Value;

use crate::permissions::{AuthUser, MembershipTier};

pub const PROFILE_API_PATH: &str = "/api/auth/profile";
pub const PROFILE_PLACEHOLDER_IMAGE: &str = "/api/placeholder/150/150";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileUser {
    pub auth: AuthUser,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub age: Option<i64>,
    pub profession: Option<String>,
    pub annual_income: Option<i64>,
    pub net_worth: Option<i64>,
    pub privacy_level: Option<i64>,
    pub bio: Option<String>,
    pub interests: Vec<String>,
    pub profile_picture: Option<String>,
    pub created_at: Option<String>,
}

impl ProfileUser {
    pub fn from_json(value: &Value) -> Self {
        Self {
            auth: AuthUser::from_json(value),
            first_name: json_string(value.get("firstName")),
            last_name: json_string(value.get("lastName")),
            age: json_i64(value.get("age")),
            profession: json_string(value.get("profession")),
            annual_income: json_i64(value.get("annualIncome")),
            net_worth: json_i64(value.get("netWorth")),
            privacy_level: json_i64(value.get("privacyLevel")),
            bio: json_string(value.get("bio")),
            interests: parse_interests(value.get("interests")),
            profile_picture: json_string(value.get("profilePicture")),
            created_at: json_string(value.get("createdAt")),
        }
    }

    pub fn membership_tier_label(&self) -> Option<&str> {
        match self.auth.membership_tier {
            Some(MembershipTier::Platinum) => Some("Platinum"),
            Some(MembershipTier::Diamond) => Some("Diamond"),
            Some(MembershipTier::BlackCard) => Some("Black Card"),
            None => None,
        }
    }
}

impl std::ops::Deref for ProfileUser {
    type Target = AuthUser;
    fn deref(&self) -> &Self::Target {
        &self.auth
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    Failed,
}

pub fn parse_profile_response(body: &str) -> Result<ProfileUser, ProfileError> {
    let value: Value = serde_json::from_str(body).map_err(|_| ProfileError::Failed)?;
    let success = value
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !success {
        return Err(ProfileError::Failed);
    }
    let user = value.pointer("/data/user").ok_or(ProfileError::Failed)?;
    if user.is_null() {
        return Err(ProfileError::Failed);
    }
    Ok(ProfileUser::from_json(user))
}

pub fn display_optional(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string()
}

pub fn display_optional_i64(value: Option<i64>) -> String {
    match value {
        Some(n) => n.to_string(),
        None => String::new(),
    }
}

pub fn display_age(age: Option<i64>) -> String {
    match age {
        Some(n) => format!("{n} 歲"),
        None => " 歲".to_string(),
    }
}

pub fn display_full_name(first: Option<&str>, last: Option<&str>) -> String {
    format!("{} {}", display_optional(first), display_optional(last))
}

pub fn display_privacy_level(level: Option<i64>) -> String {
    match level {
        Some(n) => format!("Level {n}"),
        None => "Level ".to_string(),
    }
}

pub fn profile_picture_src(url: Option<&str>) -> String {
    match url.map(str::trim).filter(|s| !s.is_empty()) {
        Some(src) => src.to_string(),
        None => PROFILE_PLACEHOLDER_IMAGE.to_string(),
    }
}

pub fn membership_color_class(tier: Option<&str>) -> &'static str {
    match tier {
        Some("Platinum") => "text-gray-400",
        Some("Diamond") => "text-blue-400",
        Some("Black Card") => "text-luxury-gold",
        _ => "text-luxury-platinum",
    }
}

pub fn membership_benefits(tier: Option<&str>) -> &'static [&'static str] {
    match tier {
        Some("Platinum") => &["參與精選社交活動", "基本身份驗證", "標準客服支援"],
        Some("Diamond") => &[
            "VIP活動優先預訂",
            "專屬社交顧問",
            "私人活動邀請",
            "高端場地折扣",
        ],
        Some("Black Card") => &[
            "獨家VVIP活動",
            "24/7禮賓服務",
            "客製化活動規劃",
            "全球合作夥伴特權",
        ],
        _ => &[],
    }
}

pub const ACTIVITY_EVENTS_ATTENDED: u32 = 15;
pub const ACTIVITY_UPCOMING: u32 = 3;
pub const ACTIVITY_TOTAL_SPENT_K: u32 = 450;
pub const ACTIVITY_CREDIT: &str = "A+";
pub const ACTIVITY_MEMBER_SINCE_YEAR: i32 = 2023;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpcomingEvent {
    pub name: &'static str,
    pub date_label: &'static str,
    pub confirmed: bool,
}

pub const UPCOMING_EVENTS: &[UpcomingEvent] = &[
    UpcomingEvent {
        name: "星空下的法式晚宴",
        date_label: "2024年12月27日",
        confirmed: true,
    },
    UpcomingEvent {
        name: "私人遊艇品酒之夜",
        date_label: "2024年12月15日",
        confirmed: false,
    },
    UpcomingEvent {
        name: "當代藝術收藏家沙龍",
        date_label: "2024年12月8日",
        confirmed: true,
    },
];

pub async fn fetch_profile() -> Result<ProfileUser, ProfileError> {
    #[cfg(target_arch = "wasm32")]
    {
        let token = crate::auth::read_stored_token().ok_or(ProfileError::Failed)?;
        let response = gloo_net::http::Request::get(PROFILE_API_PATH)
            .header("Authorization", &crate::auth::bearer_authorization(&token))
            .send()
            .await
            .map_err(|_| ProfileError::Failed)?;
        if !(200..300).contains(&response.status()) {
            return Err(ProfileError::Failed);
        }
        let body = response.text().await.unwrap_or_default();
        return parse_profile_response(&body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(ProfileError::Failed)
}

fn json_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn json_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_u64().map(|n| n as i64))
            .or_else(|| v.as_f64().map(|n| n as i64))
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
