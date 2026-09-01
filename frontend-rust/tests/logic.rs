use hesocial_frontend::auth::{
    LOGIN_FAILED_FALLBACK, TOKEN_STORAGE_KEY, apply_complete_profile_redirect,
    bearer_authorization, boot_claim_oauth, display_login_error, extract_oauth_token,
    parse_login_response, password_input_type,
};
use hesocial_frontend::logic::{next_toggled, toggle_label};

#[test]
fn off_state_uses_off_label() {
    assert_eq!(toggle_label(false), "Off");
}

#[test]
fn on_state_uses_on_label() {
    assert_eq!(toggle_label(true), "On");
}

#[test]
fn toggling_flips_boolean_state() {
    assert!(next_toggled(false));
    assert!(!next_toggled(true));
}

#[test]
fn password_is_masked_until_toggled() {
    assert_eq!(password_input_type(false), "password");
    assert_eq!(password_input_type(true), "text");
}

#[test]
fn authenticated_requests_use_hesocial_token_bearer_header() {
    assert_eq!(TOKEN_STORAGE_KEY, "hesocial_token");
    assert_eq!(bearer_authorization("abc.def.ghi"), "Bearer abc.def.ghi");
}

#[test]
fn login_error_prefers_api_error_string() {
    assert_eq!(
        display_login_error(Some("Invalid email or password")),
        "Invalid email or password"
    );
}

#[test]
fn login_error_falls_back_when_api_error_missing() {
    assert_eq!(display_login_error(None), LOGIN_FAILED_FALLBACK);
    assert_eq!(display_login_error(Some("")), LOGIN_FAILED_FALLBACK);
    assert_eq!(display_login_error(Some("   ")), LOGIN_FAILED_FALLBACK);
}

#[test]
fn parse_login_success_extracts_token() {
    let body = r#"{"success":true,"data":{"token":"jwt-1","user":{"email":"a@b.c"}}}"#;
    let ok = parse_login_response(body).expect("success body");
    assert_eq!(ok.token, "jwt-1");
}

#[test]
fn parse_login_uses_backend_401_error_string() {
    let body = r#"{"success":false,"error":"Invalid email or password"}"#;
    let err = parse_login_response(body).expect_err("401 body");
    assert_eq!(err, "Invalid email or password");
}

#[test]
fn parse_login_missing_error_uses_chinese_fallback() {
    let body = r#"{"success":false}"#;
    let err = parse_login_response(body).expect_err("empty error");
    assert_eq!(err, LOGIN_FAILED_FALLBACK);
}

#[test]
fn parse_login_malformed_body_uses_chinese_fallback() {
    assert_eq!(
        parse_login_response("not-json").unwrap_err(),
        LOGIN_FAILED_FALLBACK
    );
}

#[test]
fn extract_oauth_token_from_complete_profile_query() {
    assert_eq!(
        extract_oauth_token("/complete-profile?token=jwt-from-google").as_deref(),
        Some("jwt-from-google")
    );
    assert_eq!(
        extract_oauth_token("?token=jwt-from-google").as_deref(),
        Some("jwt-from-google")
    );
    assert_eq!(extract_oauth_token("/complete-profile"), None);
    assert_eq!(extract_oauth_token("/profile"), None);
}

#[test]
fn complete_profile_redirect_drops_the_query_string() {
    assert_eq!(
        apply_complete_profile_redirect("/complete-profile?token=jwt-from-google"),
        "/profile"
    );
    assert_eq!(
        extract_oauth_token("/profile"),
        None,
        "reading the token after the redirect loses it"
    );
}

#[test]
fn boot_claims_oauth_token_before_complete_profile_redirect() {
    let claimed = boot_claim_oauth("/complete-profile?token=jwt-from-google");
    assert_eq!(
        claimed.token.as_deref(),
        Some("jwt-from-google"),
        "token must be taken from the landing URL, not from the post-redirect path"
    );
    assert_eq!(claimed.route, "/profile");
    assert_ne!(claimed.route, "/login");
}
