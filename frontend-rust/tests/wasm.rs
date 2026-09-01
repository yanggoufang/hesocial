#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::ui::Home;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn home_renders_heading_and_toggle_button() {
    let mut vdom = VirtualDom::new(Home);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("HeSocial"),
        "expected heading text in SSR markup, got: {html}"
    );
    assert!(
        html.contains("toggle-btn"),
        "expected toggle button id in SSR markup, got: {html}"
    );
}
