use crate::logic::{next_toggled, toggle_label};
use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=Playfair+Display:wght@400;500;600;700&display=swap",
        }
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        Router::<Route> {}
    }
}

#[component]
pub fn Home() -> Element {
    let mut toggled = use_signal(|| false);

    rsx! {
        main { class: "min-h-screen bg-luxury-midnight-black text-luxury-platinum p-8",
            h1 {
                id: "scaffold-heading",
                class: "font-luxury text-luxury-gold text-4xl mb-6",
                "HeSocial"
            }
            button {
                id: "toggle-btn",
                class: "luxury-button",
                r#type: "button",
                onclick: move |_| toggled.set(next_toggled(toggled())),
                "{toggle_label(toggled())}"
            }
        }
    }
}
