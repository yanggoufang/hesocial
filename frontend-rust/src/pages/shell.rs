use crate::icons::{Icon, IconName};
use crate::shell::{
    Presence, SessionKind, is_active_path, presence_class, presence_is_mounted, primary_nav_items,
    session_entries,
};
use dioxus::prelude::*;

fn nav_item_id(path: &str) -> &'static str {
    match path {
        "/" => "nav-item-home",
        "/events" => "nav-item-events",
        "/vvip" => "nav-item-vvip",
        _ => "nav-item",
    }
}

fn session_entry_id(kind: SessionKind) -> &'static str {
    match kind {
        SessionKind::Login => "nav-login",
        SessionKind::Register => "nav-register",
        SessionKind::Profile => "nav-profile",
        SessionKind::Registrations => "nav-registrations",
        SessionKind::Admin => "nav-admin",
        SessionKind::EventMgmt => "nav-event-mgmt",
        SessionKind::Sales => "nav-sales",
        SessionKind::SystemHealth => "nav-system",
        SessionKind::Logout => "nav-logout",
    }
}

fn nav_link_class(active: bool) -> &'static str {
    if active {
        "flex items-center space-x-1 px-4 py-2 rounded-lg transition-all duration-300 text-luxury-gold bg-luxury-gold/10"
    } else {
        "flex items-center space-x-1 px-4 py-2 rounded-lg transition-all duration-300 text-luxury-platinum hover:text-luxury-gold hover:bg-luxury-gold/5"
    }
}

fn dropdown_link_class() -> &'static str {
    "block px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors"
}

#[component]
pub fn NavbarScreen(
    pathname: String,
    is_authenticated: bool,
    view_admin: bool,
    user_menu: Presence,
    mobile: Presence,
    #[props(default)] on_toggle_user_menu: EventHandler<()>,
    #[props(default)] on_user_menu_animation_end: EventHandler<()>,
    #[props(default)] on_toggle_mobile: EventHandler<()>,
    #[props(default)] on_mobile_animation_end: EventHandler<()>,
    #[props(default)] on_logout: EventHandler<()>,
    #[props(default)] on_close_user_menu: EventHandler<()>,
    #[props(default)] on_close_mobile: EventHandler<()>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let entries = session_entries(is_authenticated, view_admin);
    let mobile_open = presence_is_mounted(mobile);
    let user_menu_mounted = presence_is_mounted(user_menu);

    rsx! {
        nav { id: "nav", class: "fixed top-0 w-full z-50 luxury-glass",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "flex justify-between items-center h-20",
                    a {
                        id: "nav-brand",
                        href: "/",
                        class: "flex items-center space-x-2",
                        onclick: move |evt| {
                            evt.prevent_default();
                            on_navigate.call("/".to_string());
                        },
                        Icon {
                            name: IconName::Crown,
                            class: "h-8 w-8 text-luxury-gold".to_string(),
                        }
                        span { class: "text-2xl font-luxury font-bold text-luxury-gold",
                            "HeSocial"
                        }
                    }

                    div { class: "hidden md:flex items-center space-x-8",
                        for item in primary_nav_items() {
                            a {
                                id: nav_item_id(item.path),
                                href: "{item.path}",
                                class: nav_link_class(is_active_path(&pathname, item.path)),
                                onclick: move |evt| {
                                    evt.prevent_default();
                                    on_navigate.call(item.path.to_string());
                                },
                                if let Some(icon) = item.icon.and_then(IconName::from_nav_icon) {
                                    Icon { name: icon, class: "h-4 w-4".to_string() }
                                }
                                span { "{item.name}" }
                            }
                        }
                    }

                    div { class: "hidden md:flex items-center space-x-4",
                        if is_authenticated {
                            div { class: "relative",
                                button {
                                    id: "nav-user-button",
                                    r#type: "button",
                                    class: "flex items-center space-x-2 p-2 rounded-lg hover:bg-luxury-gold/10 transition-colors",
                                    onclick: move |_| on_toggle_user_menu.call(()),
                                    div { class: "w-8 h-8 bg-luxury-gold rounded-full flex items-center justify-center",
                                        Icon {
                                            name: IconName::User,
                                            class: "h-4 w-4 text-luxury-midnight-black".to_string(),
                                        }
                                    }
                                }
                                if user_menu_mounted {
                                    div {
                                        id: "nav-user-menu",
                                        class: "absolute right-0 mt-2 w-48 bg-white/95 backdrop-blur-sm rounded-lg shadow-xl border border-luxury-gold/20 z-[60] {presence_class(user_menu, \"hs-dropdown-enter\", \"hs-dropdown-exit\")}",
                                        onanimationend: move |_| on_user_menu_animation_end.call(()),
                                        for entry in entries.iter().copied() {
                                            if entry.kind == SessionKind::Admin {
                                                div { class: "border-t border-luxury-gold/20 my-2" }
                                            }
                                            if entry.kind == SessionKind::Logout {
                                                div { class: "border-t border-luxury-gold/20 my-2" }
                                                button {
                                                    id: session_entry_id(entry.kind),
                                                    r#type: "button",
                                                    class: "block w-full text-left px-4 py-3 text-sm text-gray-700 hover:bg-luxury-gold/10 transition-colors",
                                                    onclick: move |_| on_logout.call(()),
                                                    Icon {
                                                        name: IconName::LogOut,
                                                        class: "inline h-4 w-4 mr-2".to_string(),
                                                    }
                                                    "{entry.label}"
                                                }
                                            } else if let Some(href) = entry.href {
                                                a {
                                                    id: session_entry_id(entry.kind),
                                                    href,
                                                    class: dropdown_link_class(),
                                                    onclick: move |evt| {
                                                        evt.prevent_default();
                                                        on_navigate.call(href.to_string());
                                                        on_close_user_menu.call(());
                                                    },
                                                    if let Some(icon) = session_icon(entry.kind) {
                                                        Icon { name: icon, class: "inline h-4 w-4 mr-2".to_string() }
                                                    }
                                                    "{entry.label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            div { class: "flex items-center space-x-4",
                                for entry in entries.iter().copied() {
                                    if let Some(href) = entry.href {
                                        a {
                                            id: session_entry_id(entry.kind),
                                            href,
                                            class: if entry.kind == SessionKind::Register {
                                                "luxury-button"
                                            } else {
                                                "text-luxury-platinum hover:text-luxury-gold transition-colors"
                                            },
                                            onclick: move |evt| {
                                                evt.prevent_default();
                                                on_navigate.call(href.to_string());
                                            },
                                            "{entry.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button {
                        id: "nav-mobile-toggle",
                        r#type: "button",
                        class: "md:hidden p-2 rounded-lg hover:bg-luxury-gold/10 transition-colors",
                        onclick: move |_| on_toggle_mobile.call(()),
                        Icon {
                            name: if mobile_open { IconName::X } else { IconName::Menu },
                            class: "h-6 w-6 text-luxury-gold".to_string(),
                        }
                    }
                }
            }

            if mobile_open {
                div {
                    id: "nav-mobile-panel",
                    class: "md:hidden luxury-glass border-t border-luxury-gold/20 {presence_class(mobile, \"hs-mobile-enter\", \"hs-mobile-exit\")}",
                    onanimationend: move |_| on_mobile_animation_end.call(()),
                    div { class: "hs-mobile-inner px-4 py-4 space-y-2",
                        for item in primary_nav_items() {
                            a {
                                href: "{item.path}",
                                class: nav_link_class(is_active_path(&pathname, item.path)),
                                onclick: move |evt| {
                                    evt.prevent_default();
                                    on_navigate.call(item.path.to_string());
                                    on_close_mobile.call(());
                                },
                                if let Some(icon) = item.icon.and_then(IconName::from_nav_icon) {
                                    Icon { name: icon, class: "h-4 w-4".to_string() }
                                }
                                span { "{item.name}" }
                            }
                        }
                        if !is_authenticated {
                            div { class: "pt-4 border-t border-luxury-gold/20 space-y-2",
                                a {
                                    href: "/login",
                                    class: "block px-4 py-3 text-luxury-platinum hover:text-luxury-gold transition-colors",
                                    onclick: move |evt| {
                                        evt.prevent_default();
                                        on_navigate.call("/login".to_string());
                                        on_close_mobile.call(());
                                    },
                                    "登入"
                                }
                                a {
                                    href: "/register",
                                    class: "block px-4 py-3 bg-luxury-gold text-luxury-midnight-black rounded-lg font-medium transition-colors hover:bg-luxury-gold/90",
                                    onclick: move |evt| {
                                        evt.prevent_default();
                                        on_navigate.call("/register".to_string());
                                        on_close_mobile.call(());
                                    },
                                    "註冊"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn session_icon(kind: SessionKind) -> Option<IconName> {
    match kind {
        SessionKind::Profile => Some(IconName::User),
        SessionKind::Registrations => Some(IconName::Calendar),
        SessionKind::Admin => Some(IconName::Shield),
        SessionKind::EventMgmt => Some(IconName::Settings),
        SessionKind::Sales => Some(IconName::TrendingUp),
        SessionKind::SystemHealth => Some(IconName::Activity),
        SessionKind::Logout => Some(IconName::LogOut),
        SessionKind::Login | SessionKind::Register => None,
    }
}

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { id: "footer", class: "bg-luxury-midnight-black border-t border-luxury-gold/20 mt-20",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12",
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-8",
                    div { class: "space-y-4",
                        div { class: "flex items-center space-x-2",
                            Icon {
                                name: IconName::Crown,
                                class: "h-8 w-8 text-luxury-gold".to_string(),
                            }
                            span { class: "text-2xl font-luxury font-bold text-luxury-gold",
                                "HeSocial"
                            }
                        }
                        p { class: "text-luxury-platinum/80 text-sm leading-relaxed",
                            "專為高淨值人士打造的頂級社交平台，提供獨家尊榮體驗與精緻社交活動。"
                        }
                    }
                    div { class: "space-y-4",
                        h3 { class: "text-luxury-gold font-luxury font-semibold", "服務項目" }
                        ul { class: "space-y-2 text-sm",
                            FooterLink { label: "私人晚宴" }
                            FooterLink { label: "豪華遊艇派對" }
                            FooterLink { label: "藝術品鑑會" }
                            FooterLink { label: "商務社交" }
                        }
                    }
                    div { class: "space-y-4",
                        h3 { class: "text-luxury-gold font-luxury font-semibold", "會員專區" }
                        ul { class: "space-y-2 text-sm",
                            FooterLink { label: "Platinum 會員" }
                            FooterLink { label: "Diamond 會員" }
                            FooterLink { label: "Black Card 會員" }
                            FooterLink { label: "專屬顧問服務" }
                        }
                    }
                    div { class: "space-y-4",
                        h3 { class: "text-luxury-gold font-luxury font-semibold", "聯絡我們" }
                        div { class: "space-y-3 text-sm",
                            div { class: "flex items-center space-x-3",
                                Icon {
                                    name: IconName::Phone,
                                    class: "h-4 w-4 text-luxury-gold".to_string(),
                                }
                                span { class: "text-luxury-platinum/80", "+886-2-2345-6789" }
                            }
                            div { class: "flex items-center space-x-3",
                                Icon {
                                    name: IconName::Mail,
                                    class: "h-4 w-4 text-luxury-gold".to_string(),
                                }
                                span { class: "text-luxury-platinum/80", "concierge@hesocial.com" }
                            }
                            div { class: "flex items-center space-x-3",
                                Icon {
                                    name: IconName::MapPin,
                                    class: "h-4 w-4 text-luxury-gold".to_string(),
                                }
                                span { class: "text-luxury-platinum/80", "台北市信義區松仁路" }
                            }
                        }
                    }
                }
                div { class: "mt-12 pt-8 border-t border-luxury-gold/20",
                    div { class: "flex flex-col md:flex-row justify-between items-center",
                        p { class: "text-luxury-platinum/60 text-sm",
                            "© 2024 HeSocial. 版權所有 | 隱私政策 | 服務條款"
                        }
                        div { class: "flex items-center space-x-4 mt-4 md:mt-0",
                            span { class: "text-luxury-platinum/60 text-sm", "企業級安全認證" }
                            div { class: "flex items-center space-x-2",
                                div { class: "w-2 h-2 bg-green-500 rounded-full" }
                                span { class: "text-luxury-platinum/60 text-sm", "系統正常運行" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FooterLink(label: &'static str) -> Element {
    rsx! {
        li {
            a {
                href: "#",
                class: "text-luxury-platinum/80 hover:text-luxury-gold transition-colors",
                "{label}"
            }
        }
    }
}


