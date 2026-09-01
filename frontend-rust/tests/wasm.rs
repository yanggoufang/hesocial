#![cfg(target_arch = "wasm32")]

use std::rc::Rc;

use dioxus::history::{History, MemoryHistory};
use dioxus::prelude::*;
use dioxus::router::components::HistoryProvider;
use hesocial_frontend::ui::{LoginScreen, Route};
use wasm_bindgen_test::wasm_bindgen_test;

#[component]
fn At(path: String) -> Element {
    rsx! {
        HistoryProvider {
            history: move |_| {
                Rc::new(MemoryHistory::with_initial_path(path.clone())) as Rc<dyn History>
            },
            Router::<Route> {}
        }
    }
}

fn render_path(path: &str) -> String {
    let mut vdom = VirtualDom::new_with_props(At, AtProps { path: path.into() });
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn opening_tag<'a>(html: &'a str, id: &str) -> &'a str {
    let needle = format!("id=\"{id}\"");
    let Some(id_at) = html.find(&needle) else {
        return "";
    };
    let start = html[..id_at].rfind('<').unwrap_or(id_at);
    let end = html[id_at..]
        .find('>')
        .map(|rel| id_at + rel + 1)
        .unwrap_or(html.len());
    &html[start..end]
}

#[wasm_bindgen_test]
fn home_renders_heading_and_toggle_button() {
    let html = render_path("/");
    assert!(
        html.contains("HeSocial"),
        "expected heading text in SSR markup, got: {html}"
    );
    assert!(
        html.contains("toggle-btn"),
        "expected toggle button id in SSR markup, got: {html}"
    );
}

#[wasm_bindgen_test]
fn login_renders_traditional_chinese_copy() {
    let html = render_path("/login");
    for needle in [
        "歡迎回來",
        "登入您的尊榮帳戶",
        "電子郵件",
        "請輸入您的電子郵件",
        "密碼",
        "請輸入您的密碼",
        "記住我",
        "忘記密碼？",
        "登入",
        "或使用",
        "Google",
        "LinkedIn (即將推出)",
        "還沒有帳戶？",
        "立即申請加入",
    ] {
        assert!(
            html.contains(needle),
            "expected {needle:?} in login markup, got: {html}"
        );
    }
    assert!(
        html.contains("href=\"/forgot-password\""),
        "forgot-password link missing: {html}"
    );
    assert!(
        html.contains("href=\"/register\""),
        "register link missing: {html}"
    );
}

#[wasm_bindgen_test]
fn login_linkedin_is_disabled_and_submit_is_not() {
    let html = render_path("/login");
    let linkedin = opening_tag(&html, "linkedin-login");
    assert!(!linkedin.is_empty(), "linkedin button missing: {html}");
    assert!(
        linkedin.contains("disabled=") || linkedin.contains(" disabled>"),
        "linkedin must be permanently disabled, tag={linkedin} html={html}"
    );

    let submit = opening_tag(&html, "login-submit");
    assert!(
        !submit.contains("disabled=") && !submit.contains(" disabled>"),
        "submit must be enabled at rest, tag={submit} html={html}"
    );
}

#[component]
fn SubmittingLogin() -> Element {
    rsx! {
        LoginScreen {
            email: String::new(),
            password: String::new(),
            show_password: false,
            submitting: true,
            error: None,
        }
    }
}

#[wasm_bindgen_test]
fn login_submit_disabled_while_in_flight() {
    let mut vdom = VirtualDom::new(SubmittingLogin);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    let submit = opening_tag(&html, "login-submit");
    assert!(
        submit.contains("disabled=") || submit.contains(" disabled>"),
        "submit must be disabled while a request is in flight, tag={submit} html={html}"
    );
    assert!(
        html.contains("登入中..."),
        "expected in-flight label, got: {html}"
    );
}

#[wasm_bindgen_test]
fn login_password_field_starts_masked() {
    let html = render_path("/login");
    let field = opening_tag(&html, "password");
    assert!(
        field.contains("type=\"password\"") || field.contains("type='password'"),
        "password must start masked, tag={field} html={html}"
    );
}
