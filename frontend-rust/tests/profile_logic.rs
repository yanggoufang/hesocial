#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::profile::{
    PRIVACY_LEVEL_OPTIONS, PROFILE_ADD_INTEREST_PLACEHOLDER, PROFILE_API_PATH, PROFILE_CANCEL,
    PROFILE_EDIT_LABEL, PROFILE_SAVE, PROFILE_SAVING, PROFILE_UPDATE_FALLBACK, ProfileEditForm,
    ProfileUser, apply_profile_save_failure, apply_profile_save_success, interests_are_bindable,
    parse_profile_age, parse_update_profile_response, profile_edit_from_user, profile_full_payload,
    profile_partial_payload, validate_profile_edit,
};
use hesocial_frontend::register::{
    MAX_INTERESTS, REGISTER_AGE_RANGE, REGISTER_INTEREST_REQUIRED, REGISTER_REQUIRED,
    push_interest, remove_interest,
};
use serde_json::{Value, json};

fn complete_user() -> ProfileUser {
    ProfileUser::from_json(&json!({
        "id": "1",
        "email": "ok@example.com",
        "firstName": "Wei",
        "lastName": "Chen",
        "age": 42,
        "profession": "投資人",
        "annualIncome": 8000000,
        "netWorth": 50000000,
        "membershipTier": "Diamond",
        "privacyLevel": 4,
        "isVerified": true,
        "verificationStatus": "approved",
        "role": "user",
        "profilePicture": "https://media.example/p.jpg",
        "bio": "喜歡藝術與航海",
        "interests": ["藝術", "遊艇"]
    }))
}

fn google_user() -> ProfileUser {
    ProfileUser::from_json(&json!({
        "id": "g-1",
        "email": "google@example.com",
        "firstName": "Ada",
        "lastName": "Li",
        "age": null,
        "profession": null,
        "annualIncome": null,
        "netWorth": null,
        "membershipTier": "Platinum",
        "privacyLevel": 3,
        "isVerified": false,
        "verificationStatus": "pending",
        "role": "user",
        "profilePicture": null,
        "bio": null,
        "interests": null
    }))
}

fn valid_form() -> ProfileEditForm {
    profile_edit_from_user(&complete_user())
}

fn success_body(user: &Value) -> String {
    json!({
        "success": true,
        "data": { "user": user },
        "message": "Profile updated successfully"
    })
    .to_string()
}

#[test]
fn update_uses_the_same_profile_path() {
    assert_eq!(PROFILE_API_PATH, "/api/auth/profile");
    assert_eq!(PROFILE_UPDATE_FALLBACK, "Failed to update profile");
}

#[test]
fn react_copy_is_verbatim() {
    assert_eq!(PROFILE_EDIT_LABEL, "編輯個人資料");
    assert_eq!(PROFILE_CANCEL, "取消");
    assert_eq!(PROFILE_SAVE, "儲存");
    assert_eq!(PROFILE_SAVING, "儲存中...");
    assert_eq!(PROFILE_ADD_INTEREST_PLACEHOLDER, "添加新興趣");
    assert_eq!(
        PRIVACY_LEVEL_OPTIONS,
        &[
            (1, "1 - 完全公開"),
            (2, "2 - 基本資訊公開"),
            (3, "3 - 僅會員可見"),
            (4, "4 - 僅同等級會員可見"),
            (5, "5 - 完全私密"),
        ]
    );
}

#[test]
fn edit_form_copies_profile_fields_without_inventing_values() {
    let form = profile_edit_from_user(&complete_user());
    assert_eq!(form.first_name, "Wei");
    assert_eq!(form.last_name, "Chen");
    assert_eq!(form.age, "42");
    assert_eq!(form.profession, "投資人");
    assert_eq!(form.bio, "喜歡藝術與航海");
    assert_eq!(form.privacy_level, 4);
    assert_eq!(form.interests, vec!["藝術".to_string(), "遊艇".to_string()]);
    assert!(form.new_interest.is_empty());

    let google = profile_edit_from_user(&google_user());
    assert_eq!(google.first_name, "Ada");
    assert_eq!(google.last_name, "Li");
    assert!(google.age.is_empty());
    assert!(google.profession.is_empty());
    assert!(google.bio.is_empty());
    assert_eq!(google.privacy_level, 3);
    assert!(google.interests.is_empty());
}

#[test]
fn validate_rejects_blank_required_fields_with_register_copy() {
    let mut form = valid_form();
    form.first_name.clear();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_REQUIRED));

    form = valid_form();
    form.first_name = "   ".to_string();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_REQUIRED));

    form = valid_form();
    form.last_name.clear();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_REQUIRED));

    form = valid_form();
    form.last_name = "\t".to_string();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_REQUIRED));

    form = valid_form();
    form.profession.clear();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_REQUIRED));

    form = valid_form();
    form.profession = " ".to_string();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_REQUIRED));
}

#[test]
fn validate_age_uses_register_range_only_when_present() {
    let mut form = valid_form();
    form.age.clear();
    assert_eq!(validate_profile_edit(&form), Ok(()));

    form.age = "17".to_string();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_AGE_RANGE));

    form.age = "101".to_string();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_AGE_RANGE));

    form.age = "abc".to_string();
    assert_eq!(validate_profile_edit(&form), Err(REGISTER_AGE_RANGE));

    form.age = "18".to_string();
    assert_eq!(validate_profile_edit(&form), Ok(()));

    form.age = "100".to_string();
    assert_eq!(validate_profile_edit(&form), Ok(()));

    form.age = " 42 ".to_string();
    assert_eq!(validate_profile_edit(&form), Ok(()));
}

#[test]
fn validate_rejects_empty_interests_with_register_copy() {
    let mut form = valid_form();
    form.interests.clear();
    assert_eq!(
        validate_profile_edit(&form),
        Err(REGISTER_INTEREST_REQUIRED)
    );
    assert!(
        !interests_are_bindable(&form.interests),
        "empty interests must not bind through js_truthy / COALESCE"
    );
}

#[test]
fn validate_accepts_a_complete_form() {
    assert_eq!(validate_profile_edit(&valid_form()), Ok(()));
}

#[test]
fn parse_profile_age_reads_integers_and_rejects_blank() {
    assert_eq!(parse_profile_age(""), None);
    assert_eq!(parse_profile_age("   "), None);
    assert_eq!(parse_profile_age("42"), Some(42));
    assert_eq!(parse_profile_age(" 18 "), Some(18));
    assert_eq!(parse_profile_age("abc"), None);
}

#[test]
fn full_payload_sends_every_editable_field_with_real_values() {
    let body = profile_full_payload(&valid_form());
    assert_eq!(
        body,
        json!({
            "firstName": "Wei",
            "lastName": "Chen",
            "age": 42,
            "profession": "投資人",
            "bio": "喜歡藝術與航海",
            "interests": ["藝術", "遊艇"],
            "privacyLevel": 4
        })
    );
}

#[test]
fn full_payload_omits_empty_age_and_unselected_privacy() {
    let mut form = profile_edit_from_user(&google_user());
    form.profession = "工程師".to_string();
    form.interests = vec!["程式".to_string()];
    form.privacy_level = 0;
    let body = profile_full_payload(&form);
    assert_eq!(body.get("firstName"), Some(&json!("Ada")));
    assert_eq!(body.get("lastName"), Some(&json!("Li")));
    assert!(body.get("age").is_none());
    assert_eq!(body.get("profession"), Some(&json!("工程師")));
    assert_eq!(body.get("bio"), Some(&json!("")));
    assert_eq!(body.get("interests"), Some(&json!(["程式"])));
    assert!(body.get("privacyLevel").is_none());
}

#[test]
fn partial_payload_sends_only_changed_fields() {
    let original = complete_user();
    let mut form = profile_edit_from_user(&original);
    form.profession = "創投人".to_string();
    let body = profile_partial_payload(&original, &form);
    assert_eq!(body, json!({ "profession": "創投人" }));
    assert!(body.get("firstName").is_none());
    assert!(body.get("lastName").is_none());
    assert!(body.get("age").is_none());
    assert!(body.get("bio").is_none());
    assert!(body.get("interests").is_none());
    assert!(body.get("privacyLevel").is_none());
}

#[test]
fn partial_payload_for_a_full_edit_includes_every_changed_field() {
    let original = complete_user();
    let form = ProfileEditForm {
        first_name: "Ada".to_string(),
        last_name: "Li".to_string(),
        age: "36".to_string(),
        profession: "創投人".to_string(),
        bio: "新簡介".to_string(),
        privacy_level: 2,
        interests: vec!["程式".to_string()],
        new_interest: String::new(),
    };
    let body = profile_partial_payload(&original, &form);
    assert_eq!(
        body,
        json!({
            "firstName": "Ada",
            "lastName": "Li",
            "age": 36,
            "profession": "創投人",
            "bio": "新簡介",
            "interests": ["程式"],
            "privacyLevel": 2
        })
    );
}

#[test]
fn empty_interests_are_omitted_from_put_bodies() {
    let original = complete_user();
    let mut form = profile_edit_from_user(&original);
    form.interests.clear();
    assert!(!interests_are_bindable(&form.interests));
    let full = profile_full_payload(&form);
    assert!(
        full.get("interests").is_none(),
        "empty array must not be sent: {full}"
    );
    let partial = profile_partial_payload(&original, &form);
    assert!(
        partial.get("interests").is_none(),
        "cleared interests must not bind NULL-via-empty-array: {partial}"
    );
}

#[test]
fn parse_update_success_returns_server_user() {
    let body = success_body(&json!({
        "id": "1",
        "email": "ok@example.com",
        "firstName": "Ada",
        "lastName": "Chen",
        "age": 42,
        "profession": "投資人",
        "membershipTier": "Diamond",
        "privacyLevel": 4,
        "isVerified": true,
        "verificationStatus": "approved",
        "role": "user",
        "bio": "喜歡藝術與航海",
        "interests": ["藝術", "遊艇"]
    }));
    let user = parse_update_profile_response(200, &body).expect("ok");
    assert_eq!(user.first_name.as_deref(), Some("Ada"));
    assert_eq!(user.last_name.as_deref(), Some("Chen"));
    assert_eq!(user.interests, vec!["藝術".to_string(), "遊艇".to_string()]);
}

#[test]
fn parse_update_401_uses_backend_error_string() {
    assert_eq!(
        parse_update_profile_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_update_profile_response(401, "").unwrap_err(),
        PROFILE_UPDATE_FALLBACK
    );
}

#[test]
fn parse_update_success_false_uses_error_body() {
    assert_eq!(
        parse_update_profile_response(
            200,
            r#"{"success":false,"error":"Failed to update profile"}"#
        )
        .unwrap_err(),
        "Failed to update profile"
    );
    assert_eq!(
        parse_update_profile_response(200, r#"{"success":false}"#).unwrap_err(),
        PROFILE_UPDATE_FALLBACK
    );
    assert_eq!(
        parse_update_profile_response(500, "not-json").unwrap_err(),
        PROFILE_UPDATE_FALLBACK
    );
    assert_eq!(
        parse_update_profile_response(200, r#"{"success":true}"#).unwrap_err(),
        PROFILE_UPDATE_FALLBACK
    );
    assert_eq!(
        parse_update_profile_response(200, r#"{"success":true,"data":{"user":null}}"#).unwrap_err(),
        PROFILE_UPDATE_FALLBACK
    );
}

#[test]
fn successful_save_adopts_server_profile_not_local_form() {
    let mut form = valid_form();
    form.first_name = "Local".to_string();
    form.interests.clear();
    let mut server = complete_user();
    server.first_name = Some("Ada".to_string());
    server.interests = vec!["藝術".to_string(), "遊艇".to_string()];
    let outcome = apply_profile_save_success(server);
    assert_eq!(outcome.profile.first_name.as_deref(), Some("Ada"));
    assert_eq!(
        outcome.profile.interests,
        vec!["藝術".to_string(), "遊艇".to_string()]
    );
    assert_eq!(outcome.form.first_name, "Ada");
    assert_eq!(
        outcome.form.interests,
        vec!["藝術".to_string(), "遊艇".to_string()]
    );
    assert!(!outcome.editing);
    assert!(outcome.error.is_none());
}

#[test]
fn failed_save_preserves_typed_form() {
    let profile = complete_user();
    let mut form = valid_form();
    form.first_name = "Typed".to_string();
    form.bio = "尚未送出的簡介".to_string();
    let outcome = apply_profile_save_failure(
        profile.clone(),
        form.clone(),
        "Access token required".into(),
    );
    assert_eq!(outcome.profile, profile);
    assert_eq!(outcome.form.first_name, "Typed");
    assert_eq!(outcome.form.bio, "尚未送出的簡介");
    assert!(outcome.editing);
    assert_eq!(outcome.error.as_deref(), Some("Access token required"));
}

#[test]
fn interest_helpers_match_register_limits() {
    let added = push_interest(&["藝術".to_string()], " 遊艇 ").expect("added");
    assert_eq!(added, vec!["藝術".to_string(), "遊艇".to_string()]);
    assert!(push_interest(&added, "藝術").is_none());
    assert!(push_interest(&added, "   ").is_none());
    let ten: Vec<String> = (0..MAX_INTERESTS).map(|i| format!("i{i}")).collect();
    assert!(push_interest(&ten, "extra").is_none());
    assert_eq!(remove_interest(&added, "藝術"), vec!["遊艇".to_string()]);
}
