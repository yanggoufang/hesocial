pub const TOKEN_STORAGE_KEY: &str = "hesocial_token";
pub const LOGIN_API_PATH: &str = "/api/auth/login";
pub const GOOGLE_AUTH_PATH: &str = "/api/auth/google";
pub const LOGIN_FAILED_FALLBACK: &str = "登入失敗，請檢查您的電子郵件和密碼";
pub const GOOGLE_LOGIN_FAILED: &str = "Google 登入失敗，請稍後再試";
pub const OAUTH_LANDING_PATH: &str = "/complete-profile";
pub const OAUTH_LANDING_REDIRECT: &str = "/profile";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginOk {
    pub token: String,
    pub user: Option<crate::permissions::AuthUser>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootClaim {
    pub token: Option<String>,
    pub route: String,
}

pub fn password_input_type(show_password: bool) -> &'static str {
    if show_password { "text" } else { "password" }
}

pub fn bearer_authorization(token: &str) -> String {
    format!("Bearer {token}")
}

pub fn display_login_error(api_error: Option<&str>) -> String {
    match api_error.map(str::trim).filter(|s| !s.is_empty()) {
        Some(error) => error.to_string(),
        None => LOGIN_FAILED_FALLBACK.to_string(),
    }
}

pub fn parse_login_response(body: &str) -> Result<LoginOk, String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|_| LOGIN_FAILED_FALLBACK.to_string())?;
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
            Some(token) => Ok(LoginOk {
                token: token.to_string(),
                user: value
                    .pointer("/data/user")
                    .map(crate::permissions::AuthUser::from_json),
            }),
            None => Err(LOGIN_FAILED_FALLBACK.to_string()),
        }
    } else {
        let error = value.get("error").and_then(|v| v.as_str());
        Err(display_login_error(error))
    }
}

pub fn path_only(path_and_query: &str) -> &str {
    path_and_query.split('?').next().unwrap_or(path_and_query)
}

pub fn extract_oauth_token(input: &str) -> Option<String> {
    let query = if let Some((_, query)) = input.split_once('?') {
        query
    } else if input.starts_with('/') {
        return None;
    } else {
        input.trim_start_matches('?')
    };
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        if key == "token" && !value.is_empty() {
            return Some(percent_decode(value));
        }
    }
    None
}

pub fn apply_complete_profile_redirect(path_and_query: &str) -> String {
    match path_only(path_and_query) {
        OAUTH_LANDING_PATH => OAUTH_LANDING_REDIRECT.to_string(),
        other => other.to_string(),
    }
}

pub fn boot_claim_oauth(path_and_query: &str) -> BootClaim {
    let token = extract_oauth_token(path_and_query);
    let route = apply_complete_profile_redirect(path_and_query);
    BootClaim { token, route }
}

pub fn claim_oauth_token_on_boot() {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        let location = window.location();
        let pathname = location.pathname().unwrap_or_default();
        let search = location.search().unwrap_or_default();
        let claimed = boot_claim_oauth(&format!("{pathname}{search}"));
        if let Some(token) = claimed.token {
            store_token(&token);
        }
    }
}

pub fn store_token(token: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(TOKEN_STORAGE_KEY, token);
        }
    }
    let _ = token;
}

pub fn clear_token() {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(storage) = local_storage() {
            let _ = storage.remove_item(TOKEN_STORAGE_KEY);
        }
    }
}

pub fn read_stored_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        return local_storage()?.get_item(TOKEN_STORAGE_KEY).ok().flatten();
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn initiate_google_login() -> Result<(), ()> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().ok_or(())?;
        window
            .location()
            .set_href(GOOGLE_AUTH_PATH)
            .map_err(|_| ())?;
        return Ok(());
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(())
}

pub async fn login_with_password(email: &str, password: &str) -> Result<LoginOk, String> {
    #[cfg(target_arch = "wasm32")]
    {
        #[derive(serde::Serialize)]
        struct LoginBody<'a> {
            email: &'a str,
            password: &'a str,
        }

        let mut builder = gloo_net::http::Request::post(LOGIN_API_PATH);
        if let Some(token) = read_stored_token() {
            builder = builder.header("Authorization", &bearer_authorization(&token));
        }
        let response = builder
            .json(&LoginBody { email, password })
            .map_err(|_| LOGIN_FAILED_FALLBACK.to_string())?
            .send()
            .await
            .map_err(|_| LOGIN_FAILED_FALLBACK.to_string())?;
        let body = response.text().await.unwrap_or_default();
        return parse_login_response(&body);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (email, password);
        Err(LOGIN_FAILED_FALLBACK.to_string())
    }
}

fn percent_decode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = &value[i + 1..i + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte as char);
                    i += 3;
                    continue;
                }
                out.push('%');
                i += 1;
            }
            b'+' => {
                out.push(' ');
                i += 1;
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}
