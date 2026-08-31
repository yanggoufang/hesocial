//! Hand-written Google OAuth 2.0 authorization-code flow, mirroring the
//! Express `passport-google-oauth20` setup in `backend/src/config/passport.ts`
//! and `backend/src/routes/authRoutes.ts`.
//!
//! Everything here is host-testable pure logic; the wasm32 worker crate owns
//! the actual HTTP calls to Google behind the small request/response seam
//! (`GOOGLE_TOKEN_URL` form POST in, `GOOGLE_USERINFO_URL` Bearer GET out).

use serde_json::Value;

pub const GOOGLE_AUTHORIZATION_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
pub const GOOGLE_CALLBACK_PATH: &str = "/api/auth/google/callback";

/// Express passes `scope: ['profile', 'email']`; passport joins with a space.
pub const OAUTH_SCOPES: &str = "profile email";

pub const STATE_COOKIE_NAME: &str = "google_oauth_state";
pub const STATE_COOKIE_PATH: &str = "/api/auth/google";
pub const STATE_COOKIE_MAX_AGE_SECONDS: u64 = 600;

/// Mirrors Node's `querystring.escape` (same character set as
/// `encodeURIComponent`): alphanumerics plus `-_.!~*'()` stay literal,
/// everything else is UTF-8 percent-encoded with uppercase hex.
pub fn percent_encode(value: &str) -> String {
    const UNRESERVED: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.!~*'()";

    let mut encoded = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        if UNRESERVED.contains(byte) {
            encoded.push(char::from(*byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

/// Consent URL in the exact parameter order passport-oauth2 emits:
/// `response_type`, `redirect_uri`, `scope`, `state`, then `client_id` last.
/// (Express sends no `state`; ours is additive CSRF protection, R7.)
pub fn google_consent_url(client_id: &str, redirect_uri: &str, state: &str) -> String {
    format!(
        "{GOOGLE_AUTHORIZATION_URL}?response_type=code&redirect_uri={}&scope={}&state={}&client_id={}",
        percent_encode(redirect_uri),
        percent_encode(OAUTH_SCOPES),
        percent_encode(state),
        percent_encode(client_id),
    )
}

/// Form body for the authorization-code exchange. node-oauth sends the client
/// credentials in the POST body (no Basic header); parameter order is not
/// contract-relevant here.
pub fn token_exchange_body(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> String {
    format!(
        "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
        percent_encode(client_id),
        percent_encode(client_secret),
        percent_encode(code),
        percent_encode(redirect_uri),
    )
}

pub fn state_set_cookie(state: &str) -> String {
    format!(
        "{STATE_COOKIE_NAME}={state}; Path={STATE_COOKIE_PATH}; HttpOnly; Secure; SameSite=Lax; Max-Age={STATE_COOKIE_MAX_AGE_SECONDS}"
    )
}

pub fn state_clear_cookie() -> String {
    format!(
        "{STATE_COOKIE_NAME}=; Path={STATE_COOKIE_PATH}; HttpOnly; Secure; SameSite=Lax; Max-Age=0"
    )
}

/// Reads one cookie out of a raw `Cookie` header value.
pub fn read_cookie<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header.split(';').map(str::trim).find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then_some(value)
    })
}

pub fn random_oauth_state() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub enum CallbackAction {
    /// State matched and a code is present: proceed to the token exchange.
    ExchangeCode(String),
    /// Anything user-facing went wrong: `/login?error=oauth_failed`.
    RedirectOauthFailed,
}

/// Validates the callback query against the state cookie. Express/passport
/// had no state check at all (NullStore); the cookie check is the additive
/// R7 mitigation and failures map to the same `oauth_failed` redirect the
/// Express handler uses for a missing user.
pub fn evaluate_callback(
    error: Option<&str>,
    code: Option<&str>,
    state: Option<&str>,
    cookie_state: Option<&str>,
) -> CallbackAction {
    if error.is_some() {
        return CallbackAction::RedirectOauthFailed;
    }
    let Some(code) = code.filter(|code| !code.is_empty()) else {
        return CallbackAction::RedirectOauthFailed;
    };
    let (Some(state), Some(cookie_state)) = (state, cookie_state) else {
        return CallbackAction::RedirectOauthFailed;
    };
    if state.is_empty() || state != cookie_state {
        return CallbackAction::RedirectOauthFailed;
    }
    CallbackAction::ExchangeCode(code.to_owned())
}

/// Normalized subset of the OpenID userinfo document, matching what the
/// Express strategy reads off the passport profile.
pub struct GoogleProfile {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub picture: Option<String>,
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.is_empty())
}

/// Mirrors the Express fallbacks:
/// `givenName || displayName.split(' ')[0] || ''` and
/// `familyName || displayName.split(' ').slice(1).join(' ') || ''`.
/// No email at all is a hard failure in Express too.
pub fn parse_google_userinfo(json: &Value) -> Option<GoogleProfile> {
    let email = json.get("email").and_then(Value::as_str)?;
    let display_name = json.get("name").and_then(Value::as_str).unwrap_or("");

    let first_name = non_empty(json.get("given_name").and_then(Value::as_str))
        .or_else(|| display_name.split(' ').next())
        .unwrap_or_default()
        .to_owned();
    let last_name = non_empty(json.get("family_name").and_then(Value::as_str))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            display_name
                .split(' ')
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ")
        });
    let picture = json
        .get("picture")
        .and_then(Value::as_str)
        .map(str::to_owned);

    Some(GoogleProfile {
        email: email.to_owned(),
        first_name,
        last_name,
        picture,
    })
}

/// Mirrors the Express check `!user.age || !user.profession ||
/// !user.annualIncome || !user.netWorth` (JS falsy: null, 0, '').
pub fn needs_profile_completion(
    age: Option<i64>,
    profession: Option<&str>,
    annual_income: Option<i64>,
    net_worth: Option<i64>,
) -> bool {
    age.is_none_or(|value| value == 0)
        || profession.is_none_or(str::is_empty)
        || annual_income.is_none_or(|value| value == 0)
        || net_worth.is_none_or(|value| value == 0)
}

/// `${corsOrigins[0]}/complete-profile?token=` for incomplete profiles,
/// `${corsOrigins[0]}/dashboard?token=` otherwise. The token rides in the
/// redirect query exactly like Express (frontend `authService` reads it).
pub fn success_redirect_url(frontend_origin: &str, token: &str, needs_completion: bool) -> String {
    let page = if needs_completion {
        "complete-profile"
    } else {
        "dashboard"
    };
    format!("{frontend_origin}/{page}?token={token}")
}

pub fn failure_redirect_url(frontend_origin: &str, error: &str) -> String {
    format!("{frontend_origin}/login?error={error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn percent_encode_matches_querystring_escape() {
        assert_eq!(percent_encode("profile email"), "profile%20email");
        assert_eq!(
            percent_encode("https://api.example.com/api/auth/google/callback"),
            "https%3A%2F%2Fapi.example.com%2Fapi%2Fauth%2Fgoogle%2Fcallback"
        );
        assert_eq!(percent_encode("-_.!~*'()"), "-_.!~*'()");
        assert_eq!(percent_encode("a+b=c&d"), "a%2Bb%3Dc%26d");
        assert_eq!(percent_encode("中文"), "%E4%B8%AD%E6%96%87");
        assert_eq!(percent_encode(""), "");
    }

    #[test]
    fn consent_url_uses_passport_param_order_and_encoding() {
        let url = google_consent_url(
            "client-123.apps.googleusercontent.com",
            "https://api.hesocial.com/api/auth/google/callback",
            "deadbeef",
        );
        assert_eq!(
            url,
            "https://accounts.google.com/o/oauth2/v2/auth?response_type=code&redirect_uri=https%3A%2F%2Fapi.hesocial.com%2Fapi%2Fauth%2Fgoogle%2Fcallback&scope=profile%20email&state=deadbeef&client_id=client-123.apps.googleusercontent.com"
        );
    }

    #[test]
    fn state_cookie_is_httponly_secure_lax_and_short_lived() {
        let cookie = state_set_cookie("abc123");
        assert_eq!(
            cookie,
            "google_oauth_state=abc123; Path=/api/auth/google; HttpOnly; Secure; SameSite=Lax; Max-Age=600"
        );
        assert_eq!(
            state_clear_cookie(),
            "google_oauth_state=; Path=/api/auth/google; HttpOnly; Secure; SameSite=Lax; Max-Age=0"
        );
    }

    #[test]
    fn read_cookie_finds_only_the_exact_name() {
        let header = "session=aaa; google_oauth_state=state-1; other=bbb";
        assert_eq!(read_cookie(header, "google_oauth_state"), Some("state-1"));
        assert_eq!(read_cookie(header, "missing"), None);
        assert_eq!(
            read_cookie("google_oauth_state=only", "google_oauth_state"),
            Some("only")
        );
        assert_eq!(read_cookie("", "google_oauth_state"), None);
        // Prefix collisions must not match.
        assert_eq!(
            read_cookie("google_oauth_state_extra=x", "google_oauth_state"),
            None
        );
    }

    #[test]
    fn callback_requires_code_and_matching_state() {
        // Happy path.
        match evaluate_callback(None, Some("code-1"), Some("s1"), Some("s1")) {
            CallbackAction::ExchangeCode(code) => assert_eq!(code, "code-1"),
            CallbackAction::RedirectOauthFailed => panic!("valid callback should exchange"),
        }

        // Google returned an error (e.g. access_denied).
        assert!(matches!(
            evaluate_callback(
                Some("access_denied"),
                Some("code-1"),
                Some("s1"),
                Some("s1")
            ),
            CallbackAction::RedirectOauthFailed
        ));
        // Missing code.
        assert!(matches!(
            evaluate_callback(None, None, Some("s1"), Some("s1")),
            CallbackAction::RedirectOauthFailed
        ));
        assert!(matches!(
            evaluate_callback(None, Some(""), Some("s1"), Some("s1")),
            CallbackAction::RedirectOauthFailed
        ));
        // Missing cookie, missing query state, or a mismatch.
        assert!(matches!(
            evaluate_callback(None, Some("code-1"), Some("s1"), None),
            CallbackAction::RedirectOauthFailed
        ));
        assert!(matches!(
            evaluate_callback(None, Some("code-1"), None, Some("s1")),
            CallbackAction::RedirectOauthFailed
        ));
        assert!(matches!(
            evaluate_callback(None, Some("code-1"), Some("s1"), Some("s2")),
            CallbackAction::RedirectOauthFailed
        ));
        assert!(matches!(
            evaluate_callback(None, Some("code-1"), Some(""), Some("")),
            CallbackAction::RedirectOauthFailed
        ));
    }

    #[test]
    fn token_exchange_body_is_form_encoded_with_all_fields() {
        let body = token_exchange_body("id", "sec ret", "co/de", "https://x/cb");
        assert_eq!(
            body,
            "client_id=id&client_secret=sec%20ret&code=co%2Fde&grant_type=authorization_code&redirect_uri=https%3A%2F%2Fx%2Fcb"
        );
    }

    #[test]
    fn userinfo_parsing_mirrors_the_passport_profile_fallbacks() {
        let full = parse_google_userinfo(&json!({
            "sub": "123",
            "email": "user@example.com",
            "given_name": "Given",
            "family_name": "Family",
            "name": "Given Family",
            "picture": "https://photo.example.com/p.jpg"
        }))
        .expect("profile with email parses");
        assert_eq!(full.email, "user@example.com");
        assert_eq!(full.first_name, "Given");
        assert_eq!(full.last_name, "Family");
        assert_eq!(
            full.picture.as_deref(),
            Some("https://photo.example.com/p.jpg")
        );

        // Only a display name: split like Express does.
        let split = parse_google_userinfo(&json!({
            "email": "user@example.com",
            "name": "First Middle Last"
        }))
        .expect("display-name fallback parses");
        assert_eq!(split.first_name, "First");
        assert_eq!(split.last_name, "Middle Last");
        assert_eq!(split.picture, None);

        // Single-token display name yields an empty last name, like
        // `displayName.split(' ').slice(1).join(' ') || ''`.
        let single = parse_google_userinfo(&json!({
            "email": "user@example.com",
            "name": "Madonna"
        }))
        .expect("single name parses");
        assert_eq!(single.first_name, "Madonna");
        assert_eq!(single.last_name, "");

        // Empty given/family names fall back to the display name split.
        let empty_names = parse_google_userinfo(&json!({
            "email": "user@example.com",
            "given_name": "",
            "family_name": "",
            "name": "Fallback Name"
        }))
        .expect("empty names fall back");
        assert_eq!(empty_names.first_name, "Fallback");
        assert_eq!(empty_names.last_name, "Name");

        // No email at all is a hard failure (Express: done(Error)).
        assert!(parse_google_userinfo(&json!({"sub": "123"})).is_none());
    }

    #[test]
    fn profile_completion_mirrors_js_falsy_checks() {
        // New Google user: everything null.
        assert!(needs_profile_completion(None, None, None, None));
        // Zero and empty-string are falsy in JS too.
        assert!(needs_profile_completion(
            Some(0),
            Some(""),
            Some(0),
            Some(0)
        ));
        assert!(needs_profile_completion(
            Some(30),
            Some("Engineer"),
            None,
            Some(1)
        ));
        // Fully populated profile goes straight to the dashboard.
        assert!(!needs_profile_completion(
            Some(35),
            Some("Engineer"),
            Some(5_000_000),
            Some(30_000_000)
        ));
    }

    #[test]
    fn redirect_urls_match_the_express_targets() {
        assert_eq!(
            success_redirect_url("http://localhost:3000", "tok", true),
            "http://localhost:3000/complete-profile?token=tok"
        );
        assert_eq!(
            success_redirect_url("http://localhost:3000", "tok", false),
            "http://localhost:3000/dashboard?token=tok"
        );
        assert_eq!(
            failure_redirect_url("http://localhost:3000", "oauth_failed"),
            "http://localhost:3000/login?error=oauth_failed"
        );
    }

    #[test]
    fn random_states_are_32_hex_chars_and_unique() {
        let first = random_oauth_state().expect("randomness available");
        let second = random_oauth_state().expect("randomness available");
        assert_eq!(first.len(), 32);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, second);
    }
}
