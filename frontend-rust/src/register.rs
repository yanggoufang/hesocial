pub const REGISTER_REQUIRED: &str = "請填寫所有必填欄位";
pub const REGISTER_PASSWORD_MISMATCH: &str = "密碼確認不相符";
pub const REGISTER_PASSWORD_SHORT: &str = "密碼長度至少需要8個字元";
pub const REGISTER_AGE_RANGE: &str = "年齡必須在18-100歲之間";
pub const REGISTER_INCOME_ASSET: &str = "請填寫收入與資產資訊";
pub const REGISTER_INCOME_MIN: &str = "年收入需達500萬元以上才符合申請資格";
pub const REGISTER_NETWORTH_MIN: &str = "淨資產需達3000萬元以上才符合申請資格";
pub const REGISTER_INTEREST_REQUIRED: &str = "請至少添加一個興趣";
pub const REGISTER_FAILED_FALLBACK: &str = "註冊失敗，請稍後再試";
pub const MAX_INTERESTS: usize = 10;
pub const MIN_PASSWORD_LEN: usize = 8;
pub const MIN_AGE: i32 = 18;
pub const MAX_AGE: i32 = 100;
pub const MIN_INCOME_WAN: i64 = 500;
pub const MIN_NET_WORTH_WAN: i64 = 3000;
pub const WAN: i64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MembershipTierOption {
    pub value: &'static str,
    pub label: &'static str,
    pub price: &'static str,
    pub description: &'static str,
}

pub const MEMBERSHIP_TIERS: &[MembershipTierOption] = &[
    MembershipTierOption {
        value: "Platinum",
        label: "Platinum",
        price: "NT$50,000/年",
        description: "基礎尊榮會員，享受精選活動與服務",
    },
    MembershipTierOption {
        value: "Diamond",
        label: "Diamond",
        price: "NT$120,000/年",
        description: "VIP會員專屬活動與專人顧問服務",
    },
    MembershipTierOption {
        value: "Black Card",
        label: "Black Card",
        price: "邀請制",
        description: "最高等級會員，享受所有獨家特權",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterForm {
    pub step: u8,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub first_name: String,
    pub last_name: String,
    pub age: String,
    pub profession: String,
    pub annual_income: String,
    pub net_worth: String,
    pub membership_tier: String,
    pub bio: String,
    pub interests: Vec<String>,
    pub new_interest: String,
    pub show_password: bool,
    pub show_confirm_password: bool,
    pub submitting: bool,
    pub error: Option<String>,
}

impl Default for RegisterForm {
    fn default() -> Self {
        Self {
            step: 1,
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            first_name: String::new(),
            last_name: String::new(),
            age: String::new(),
            profession: String::new(),
            annual_income: String::new(),
            net_worth: String::new(),
            membership_tier: "Platinum".to_string(),
            bio: String::new(),
            interests: Vec::new(),
            new_interest: String::new(),
            show_password: false,
            show_confirm_password: false,
            submitting: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub age: i32,
    pub profession: String,
    pub annual_income: i64,
    pub net_worth: i64,
    pub bio: String,
    pub interests: Vec<String>,
}

pub fn step_title(step: u8) -> &'static str {
    match step {
        1 => "步驟 1: 帳戶設定",
        2 => "步驟 2: 個人資訊",
        3 => "步驟 3: 會員資格",
        _ => "",
    }
}

pub fn validate_step(form: &RegisterForm) -> Result<(), &'static str> {
    match form.step {
        1 => validate_step1(form),
        2 => validate_step2(form),
        3 => validate_step3(form),
        _ => Ok(()),
    }
}

pub fn validate_step1(form: &RegisterForm) -> Result<(), &'static str> {
    if form.email.trim().is_empty() || form.password.is_empty() || form.confirm_password.is_empty()
    {
        return Err(REGISTER_REQUIRED);
    }
    if form.password != form.confirm_password {
        return Err(REGISTER_PASSWORD_MISMATCH);
    }
    if form.password.len() < MIN_PASSWORD_LEN {
        return Err(REGISTER_PASSWORD_SHORT);
    }
    Ok(())
}

pub fn validate_step2(form: &RegisterForm) -> Result<(), &'static str> {
    if form.first_name.trim().is_empty()
        || form.last_name.trim().is_empty()
        || form.age.trim().is_empty()
        || form.profession.trim().is_empty()
    {
        return Err(REGISTER_REQUIRED);
    }
    let age = parse_int(&form.age).unwrap_or(i32::MIN as i64);
    if age < MIN_AGE as i64 || age > MAX_AGE as i64 {
        return Err(REGISTER_AGE_RANGE);
    }
    Ok(())
}

pub fn validate_step3(form: &RegisterForm) -> Result<(), &'static str> {
    if form.annual_income.trim().is_empty() || form.net_worth.trim().is_empty() {
        return Err(REGISTER_INCOME_ASSET);
    }
    let income = parse_int(&form.annual_income).unwrap_or(0);
    if income < MIN_INCOME_WAN {
        return Err(REGISTER_INCOME_MIN);
    }
    let net_worth = parse_int(&form.net_worth).unwrap_or(0);
    if net_worth < MIN_NET_WORTH_WAN {
        return Err(REGISTER_NETWORTH_MIN);
    }
    if form.interests.is_empty() {
        return Err(REGISTER_INTEREST_REQUIRED);
    }
    Ok(())
}

pub fn push_interest(interests: &[String], raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || interests.iter().any(|item| item == trimmed)
        || interests.len() >= MAX_INTERESTS
    {
        return None;
    }
    let mut next = interests.to_vec();
    next.push(trimmed.to_string());
    Some(next)
}

pub fn remove_interest(interests: &[String], interest: &str) -> Vec<String> {
    interests
        .iter()
        .filter(|item| item.as_str() != interest)
        .cloned()
        .collect()
}

pub fn registration_payload(form: &RegisterForm) -> RegisterPayload {
    RegisterPayload {
        email: form.email.clone(),
        password: form.password.clone(),
        first_name: form.first_name.clone(),
        last_name: form.last_name.clone(),
        age: parse_int(&form.age).unwrap_or(0) as i32,
        profession: form.profession.clone(),
        annual_income: parse_int(&form.annual_income).unwrap_or(0) * WAN,
        net_worth: parse_int(&form.net_worth).unwrap_or(0) * WAN,
        bio: form.bio.clone(),
        interests: form.interests.clone(),
    }
}

pub fn parse_register_response(body: &str) -> Result<crate::auth::LoginOk, String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| REGISTER_FAILED_FALLBACK.to_string())?;
    let success = value
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if success {
        match value
            .pointer("/data/token")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(token) => Ok(crate::auth::LoginOk {
                token: token.to_string(),
                user: value
                    .pointer("/data/user")
                    .map(crate::permissions::AuthUser::from_json),
            }),
            None => Err(REGISTER_FAILED_FALLBACK.to_string()),
        }
    } else {
        let error = value
            .get("error")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());
        Err(error.unwrap_or(REGISTER_FAILED_FALLBACK).to_string())
    }
}

pub const REGISTER_API_PATH: &str = "/api/auth/register";

pub async fn register_account(form: &RegisterForm) -> Result<crate::auth::LoginOk, String> {
    #[cfg(target_arch = "wasm32")]
    {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            email: &'a str,
            password: &'a str,
            #[serde(rename = "firstName")]
            first_name: &'a str,
            #[serde(rename = "lastName")]
            last_name: &'a str,
            age: i32,
            profession: &'a str,
            #[serde(rename = "annualIncome")]
            annual_income: i64,
            #[serde(rename = "netWorth")]
            net_worth: i64,
            bio: &'a str,
            interests: &'a [String],
        }

        let payload = registration_payload(form);
        let mut builder = gloo_net::http::Request::post(REGISTER_API_PATH);
        if let Some(token) = crate::auth::read_stored_token() {
            builder = builder.header("Authorization", &crate::auth::bearer_authorization(&token));
        }
        let response = builder
            .json(&Body {
                email: &payload.email,
                password: &payload.password,
                first_name: &payload.first_name,
                last_name: &payload.last_name,
                age: payload.age,
                profession: &payload.profession,
                annual_income: payload.annual_income,
                net_worth: payload.net_worth,
                bio: &payload.bio,
                interests: &payload.interests,
            })
            .map_err(|_| REGISTER_FAILED_FALLBACK.to_string())?
            .send()
            .await
            .map_err(|_| REGISTER_FAILED_FALLBACK.to_string())?;
        let body = response.text().await.unwrap_or_default();
        return parse_register_response(&body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = form;
        Err(REGISTER_FAILED_FALLBACK.to_string())
    }
}

fn parse_int(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut end = 0;
    let bytes = trimmed.as_bytes();
    if bytes[0] == b'+' || bytes[0] == b'-' {
        end = 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == 0 || (end == 1 && !bytes[0].is_ascii_digit()) {
        return None;
    }
    trimmed[..end].parse().ok()
}
