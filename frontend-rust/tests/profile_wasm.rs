#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::pages::profile::ProfileScreen;
use hesocial_frontend::profile::{
    PRIVACY_LEVEL_OPTIONS, PROFILE_ADD_INTEREST_PLACEHOLDER, PROFILE_CANCEL, PROFILE_EDIT_LABEL,
    PROFILE_SAVE, PROFILE_SAVING, ProfileEditForm, ProfileUser, profile_edit_from_user,
};
use hesocial_frontend::register::REGISTER_INTEREST_REQUIRED;
use hesocial_frontend::shell::Presence;
use wasm_bindgen_test::wasm_bindgen_test;

fn complete_user() -> ProfileUser {
    ProfileUser::from_json(&serde_json::json!({
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

#[component]
fn ProfileAt(
    profile: ProfileUser,
    editing: bool,
    saving: bool,
    editable: bool,
    edit: Option<ProfileEditForm>,
    save_error: Option<String>,
    form_presence: Presence,
) -> Element {
    rsx! {
        ProfileScreen {
            profile,
            editing,
            saving,
            editable,
            edit,
            save_error,
            form_presence,
        }
    }
}

fn render_profile(
    profile: ProfileUser,
    editing: bool,
    saving: bool,
    editable: bool,
    edit: Option<ProfileEditForm>,
    save_error: Option<String>,
    form_presence: Presence,
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        ProfileAt,
        ProfileAtProps {
            profile,
            editing,
            saving,
            editable,
            edit,
            save_error,
            form_presence,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn render_view(editable: bool) -> String {
    render_profile(
        complete_user(),
        false,
        false,
        editable,
        None,
        None,
        Presence::Hidden,
    )
}

fn render_edit(saving: bool, error: Option<String>, form: ProfileEditForm) -> String {
    render_profile(
        complete_user(),
        true,
        saving,
        true,
        Some(form),
        error,
        Presence::Entering,
    )
}

#[wasm_bindgen_test]
fn profile_view_without_editable_stays_read_only() {
    let html = render_view(false);
    assert!(html.contains("id=\"profile-heading\""), "heading: {html}");
    assert!(
        !html.contains(PROFILE_EDIT_LABEL),
        "default props must not show edit: {html}"
    );
    assert!(
        !html.contains("name=\"firstName\""),
        "default props must not wire the form: {html}"
    );
}

#[wasm_bindgen_test]
fn profile_view_editable_shows_react_edit_controls() {
    let html = render_view(true);
    assert!(
        html.contains(PROFILE_EDIT_LABEL),
        "sidebar edit missing: {html}"
    );
    assert!(
        html.contains("data-icon=\"edit\""),
        "edit icon missing: {html}"
    );
    assert!(html.contains("Wei Chen"), "name missing: {html}");
    assert!(html.contains("個人資訊"), "section missing: {html}");
    assert!(
        !html.contains("name=\"firstName\""),
        "view must not render the form: {html}"
    );
    assert!(
        !html.contains(PROFILE_SAVE),
        "save must not show in view: {html}"
    );
}

#[wasm_bindgen_test]
fn profile_edit_markup_matches_react_copy() {
    let form = profile_edit_from_user(&complete_user());
    let html = render_edit(false, None, form);
    for needle in [
        "名字",
        "姓氏",
        "職業",
        "個人簡介",
        "隱私等級",
        "興趣愛好",
        PROFILE_ADD_INTEREST_PLACEHOLDER,
        PROFILE_CANCEL,
        PROFILE_SAVE,
        "name=\"firstName\"",
        "name=\"lastName\"",
        "name=\"profession\"",
        "name=\"bio\"",
        "name=\"privacyLevel\"",
        "Wei",
        "Chen",
        "投資人",
        "喜歡藝術與航海",
        "藝術",
        "遊艇",
        "data-icon=\"plus\"",
        "data-icon=\"x\"",
        "data-icon=\"save\"",
        "hs-enter",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in edit markup, got: {html}"
        );
    }
    for (_, label) in PRIVACY_LEVEL_OPTIONS {
        assert!(
            html.contains(*label),
            "privacy option {label} missing: {html}"
        );
    }
    assert!(
        !html.contains(PROFILE_SAVING),
        "idle save must not say 儲存中: {html}"
    );
    assert!(
        html.contains(PROFILE_EDIT_LABEL),
        "sidebar edit stays visible while editing: {html}"
    );
}

#[wasm_bindgen_test]
fn profile_saving_markup() {
    let form = profile_edit_from_user(&complete_user());
    let html = render_edit(true, None, form);
    assert!(html.contains(PROFILE_SAVING), "saving copy missing: {html}");
    assert!(html.contains("disabled"), "save must disable: {html}");
    assert!(
        html.contains("name=\"firstName\""),
        "form stays mounted while saving: {html}"
    );
    assert!(html.contains("Wei"), "typed first name missing: {html}");
}

#[wasm_bindgen_test]
fn profile_error_keeps_typed_form() {
    let mut form = profile_edit_from_user(&complete_user());
    form.first_name = "Typed".to_string();
    form.bio = "尚未送出的簡介".to_string();
    let html = render_edit(false, Some(REGISTER_INTEREST_REQUIRED.to_string()), form);
    assert!(
        html.contains("id=\"profile-edit-error\""),
        "error id missing: {html}"
    );
    assert!(
        html.contains(REGISTER_INTEREST_REQUIRED),
        "error copy missing: {html}"
    );
    assert!(html.contains("Typed"), "typed first name dropped: {html}");
    assert!(html.contains("尚未送出的簡介"), "typed bio dropped: {html}");
    assert!(
        html.contains("name=\"firstName\""),
        "form dropped after error: {html}"
    );
    assert!(
        html.contains(PROFILE_SAVE),
        "save missing after error: {html}"
    );
}
