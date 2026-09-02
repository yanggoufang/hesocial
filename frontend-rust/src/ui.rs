use crate::auth::{
    GOOGLE_LOGIN_FAILED, claim_oauth_token_on_boot, clear_token, login_with_password,
    password_input_type, read_stored_token, store_token, validate_stored_token,
};
#[cfg(feature = "admin-bundle")]
use crate::pages::admin::{Admin, AdminSystem};
#[cfg(feature = "admin-bundle")]
use crate::pages::adminanalytics::AdminAnalytics;
#[cfg(feature = "admin-bundle")]
use crate::pages::eventmgmt::{EventMedia, EventMgmt};
#[cfg(not(feature = "admin-bundle"))]
use crate::pages::events::{EventDetail, Events};
use crate::pages::home::Home;
#[cfg(not(feature = "admin-bundle"))]
use crate::pages::participants::{EventParticipants, EventPrivacySettings};
use crate::pages::profile::{ProfileBody, ProfileGuardLoading};
#[cfg(not(feature = "admin-bundle"))]
use crate::pages::registrations::{EventRegister, MyRegistrations};
#[cfg(feature = "admin-bundle")]
use crate::pages::sales::AdminSales;
#[cfg(feature = "admin-bundle")]
use crate::pages::taxonomy::{EventCategories, EventVenues};
#[cfg(feature = "admin-bundle")]
use crate::pages::users::AdminUsers;
#[cfg(not(feature = "admin-bundle"))]
use crate::pages::vvip::Vvip;
use crate::permissions::{RouteGuard, Session, permissions, user_route_guard};
use crate::register::{
    RegisterForm, push_interest, register_account, remove_interest, validate_step,
};
use crate::shell::{Presence, hard_navigate, presence_after_animation_end, presence_toggle};
use dioxus::prelude::*;
use std::str::FromStr;

pub use crate::pages::admin::{AdminScreen, AdminSystemScreen};
pub use crate::pages::adminanalytics::AdminAnalyticsScreen;
pub use crate::pages::eventmgmt::{EventMediaScreen, EventMgmtScreen};
pub use crate::pages::events::{EventCard, EventDetailScreen, EventsScreen};
pub use crate::pages::home::HomeScreen;
pub use crate::pages::participants::{EventParticipantsScreen, EventPrivacySettingsScreen};
pub use crate::pages::profile::ProfileScreen;
pub use crate::pages::register::RegisterScreen;
pub use crate::pages::registrations::{EventRegisterScreen, MyRegistrationsScreen};
pub use crate::pages::sales::AdminSalesScreen;
pub use crate::pages::shell::{Footer, NavbarScreen};
pub use crate::pages::taxonomy::{EventCategoriesScreen, EventVenuesScreen};
pub use crate::pages::users::AdminUsersScreen;
pub use crate::pages::vvip::{VvipRecruitmentScreen, VvipScreen};

#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Home {},
        #[route("/login")]
        Login {},
        #[route("/register")]
        #[cfg(not(feature = "admin-bundle"))]
        Register {},
        #[route("/forgot-password")]
        #[cfg(not(feature = "admin-bundle"))]
        ForgotPassword {},
        #[route("/events")]
        #[cfg(not(feature = "admin-bundle"))]
        Events {},
        #[route("/events/:id/register")]
        #[cfg(not(feature = "admin-bundle"))]
        EventRegister { id: String },
        #[route("/events/:id/participants")]
        #[cfg(not(feature = "admin-bundle"))]
        EventParticipants { id: String },
        #[route("/events/:id/privacy-settings")]
        #[cfg(not(feature = "admin-bundle"))]
        EventPrivacySettings { id: String },
        #[route("/events/:id")]
        #[cfg(not(feature = "admin-bundle"))]
        EventDetail { id: String },
        #[route("/vvip")]
        #[cfg(not(feature = "admin-bundle"))]
        Vvip {},
        #[route("/profile/registrations")]
        #[cfg(not(feature = "admin-bundle"))]
        MyRegistrations {},
        #[redirect("/dashboard", || Route::Profile {})]
        #[redirect("/complete-profile", || Route::Profile {})]
        #[route("/profile")]
        Profile {},
        #[route("/admin")]
        #[cfg(feature = "admin-bundle")]
        Admin {},
        #[route("/admin/users")]
        #[cfg(feature = "admin-bundle")]
        AdminUsers {},
        #[route("/admin/analytics")]
        #[cfg(feature = "admin-bundle")]
        AdminAnalytics {},
        #[route("/event-mgmt")]
        #[cfg(feature = "admin-bundle")]
        EventMgmt {},
        #[route("/event-mgmt/categories")]
        #[cfg(feature = "admin-bundle")]
        EventCategories {},
        #[route("/event-mgmt/venues")]
        #[cfg(feature = "admin-bundle")]
        EventVenues {},
        #[route("/event-mgmt/media/:event_id")]
        #[cfg(feature = "admin-bundle")]
        EventMedia { event_id: String },
        #[route("/admin/sales")]
        #[cfg(feature = "admin-bundle")]
        AdminSales {},
        #[route("/admin/system")]
        #[cfg(feature = "admin-bundle")]
        AdminSystem {},
}

#[component]
pub fn App() -> Element {
    claim_oauth_token_on_boot();
    let mut session = use_signal(|| {
        let token = read_stored_token();
        Session {
            restoring: token.is_some(),
            token,
            user: None,
        }
    });
    use_context_provider(|| session);
    use_future(move || async move {
        if session.peek().restoring {
            session.set(validate_stored_token().await);
        }
    });
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
fn Shell() -> Element {
    let local = use_signal(|| Session {
        token: read_stored_token(),
        user: None,
        restoring: false,
    });
    let mut session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let navigator = use_navigator();
    let route = use_route::<Route>();
    let pathname = route.to_string();
    let mut user_menu = use_signal(|| Presence::Hidden);
    let mut mobile = use_signal(|| Presence::Hidden);
    let snapshot = session().snapshot();
    let can = permissions(&snapshot);

    rsx! {
        div { class: "min-h-screen bg-luxury-midnight-black text-luxury-platinum",
            NavbarScreen {
                pathname,
                is_authenticated: snapshot.is_authenticated,
                view_admin: can.view_admin,
                user_menu: user_menu(),
                mobile: mobile(),
                on_toggle_user_menu: move |_| user_menu.set(presence_toggle(user_menu())),
                on_user_menu_animation_end: move |_| {
                    user_menu.set(presence_after_animation_end(user_menu()));
                },
                on_toggle_mobile: move |_| mobile.set(presence_toggle(mobile())),
                on_mobile_animation_end: move |_| {
                    mobile.set(presence_after_animation_end(mobile()));
                },
                on_close_user_menu: move |_| user_menu.set(Presence::Exiting),
                on_close_mobile: move |_| mobile.set(Presence::Exiting),
                on_logout: move |_| {
                    clear_token();
                    session.set(Session::default());
                    user_menu.set(Presence::Hidden);
                    mobile.set(Presence::Hidden);
                },
                on_navigate: move |path: String| {
                    match Route::from_str(&path) {
                        Ok(next) => {
                            navigator.push(next);
                        }
                        Err(_) => hard_navigate(&path),
                    }
                },
            }
            main { class: "pt-20",
                SuspenseBoundary {
                    fallback: |_| rsx! { RouteChunkLoading {} },
                    Outlet::<Route> {}
                }
            }
            Footer {}
        }
    }
}

#[component]
fn RouteChunkLoading() -> Element {
    rsx! {
        div { id: "route-chunk-loading", class: "min-h-[50vh] flex items-center justify-center",
            div { class: "w-10 h-10 border-2 border-luxury-gold border-t-transparent rounded-full animate-spin" }
        }
    }
}

#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let session = try_use_context::<Signal<Session>>();
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut show_password = use_signal(|| false);
    let mut submitting = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        LoginScreen {
            email: email(),
            password: password(),
            show_password: show_password(),
            submitting: submitting(),
            error: error(),
            on_email: move |value: String| email.set(value),
            on_password: move |value: String| password.set(value),
            on_toggle_password: move |_| show_password.set(!show_password()),
            on_google: move |_| {
                if submitting() {
                    return;
                }
                match crate::auth::initiate_google_login() {
                    Ok(()) => {}
                    Err(()) => error.set(Some(GOOGLE_LOGIN_FAILED.to_string())),
                }
            },
            on_submit: move |_| {
                if submitting() {
                    return;
                }
                submitting.set(true);
                error.set(None);
                let email_val = email();
                let password_val = password();
                spawn(async move {
                    match login_with_password(&email_val, &password_val).await {
                        Ok(ok) => {
                            store_token(&ok.token);
                            if let Some(mut session) = session {
                                session.set(Session {
                                    token: Some(ok.token.clone()),
                                    user: ok.user.clone(),
                                    restoring: false,
                                });
                            }
                            navigator.push(Route::Home {});
                        }
                        Err(msg) => {
                            error.set(Some(msg));
                            submitting.set(false);
                        }
                    }
                });
            },
        }
    }
}

#[component]
pub fn LoginScreen(
    email: String,
    password: String,
    show_password: bool,
    submitting: bool,
    error: Option<String>,
    #[props(default)] on_email: EventHandler<String>,
    #[props(default)] on_password: EventHandler<String>,
    #[props(default)] on_toggle_password: EventHandler<()>,
    #[props(default)] on_submit: EventHandler<()>,
    #[props(default)] on_google: EventHandler<()>,
) -> Element {
    let password_type = password_input_type(show_password);
    let submit_disabled = submitting;

    rsx! {
        div { class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center px-4",
            div { class: "max-w-md w-full",
                div { class: "luxury-glass p-8 rounded-2xl",
                    div { class: "text-center mb-8",
                        div { class: "flex items-center justify-center mb-4",
                            svg {
                                class: "h-12 w-12 text-luxury-gold",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                stroke_width: "1.5",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M11.48 3.499a.562.562 0 011.04 0l2.125 5.111a.563.563 0 00.475.345l5.518.442c.499.04.701.663.321.988l-4.204 3.602a.563.563 0 00-.182.557l1.285 5.385a.562.562 0 01-.84.61l-4.725-2.885a.563.563 0 00-.586 0L6.007 20.54a.562.562 0 01-.84-.61l1.285-5.386a.562.562 0 00-.182-.557L2.066 10.385a.563.563 0 01.321-.988l5.518-.442a.563.563 0 00.475-.345L10.505 3.5z",
                                }
                            }
                        }
                        h1 {
                            id: "login-heading",
                            class: "text-3xl font-luxury font-bold text-luxury-gold mb-2",
                            "歡迎回來"
                        }
                        p { class: "text-luxury-platinum/80", "登入您的尊榮帳戶" }
                    }

                    if let Some(message) = error {
                        div {
                            id: "login-error",
                            class: "mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm",
                            "{message}"
                        }
                    }

                    form {
                        class: "space-y-6",
                        onsubmit: move |evt| {
                            evt.prevent_default();
                            on_submit.call(());
                        },
                        div {
                            label {
                                r#for: "email",
                                class: "block text-luxury-platinum text-sm font-medium mb-2",
                                "電子郵件"
                            }
                            div { class: "relative",
                                svg {
                                    class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke: "currentColor",
                                    stroke_width: "1.5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75",
                                    }
                                }
                                input {
                                    r#type: "email",
                                    id: "email",
                                    name: "email",
                                    value: "{email}",
                                    required: true,
                                    class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                    placeholder: "請輸入您的電子郵件",
                                    oninput: move |evt| on_email.call(evt.value()),
                                }
                            }
                        }

                        div {
                            label {
                                r#for: "password",
                                class: "block text-luxury-platinum text-sm font-medium mb-2",
                                "密碼"
                            }
                            div { class: "relative",
                                svg {
                                    class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke: "currentColor",
                                    stroke_width: "1.5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M16.5 10.5V6.75a4.5 4.5 0 10-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 002.25-2.25v-6.75a2.25 2.25 0 00-2.25-2.25H6.75a2.25 2.25 0 00-2.25 2.25v6.75a2.25 2.25 0 002.25 2.25z",
                                    }
                                }
                                input {
                                    r#type: "{password_type}",
                                    id: "password",
                                    name: "password",
                                    value: "{password}",
                                    required: true,
                                    class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 pr-12 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                    placeholder: "請輸入您的密碼",
                                    oninput: move |evt| on_password.call(evt.value()),
                                }
                                button {
                                    id: "password-toggle",
                                    r#type: "button",
                                    class: "absolute right-3 top-1/2 transform -translate-y-1/2 text-luxury-gold hover:text-luxury-gold/80 transition-colors",
                                    onclick: move |_| on_toggle_password.call(()),
                                    if show_password {
                                        svg {
                                            class: "h-5 w-5",
                                            fill: "none",
                                            view_box: "0 0 24 24",
                                            stroke: "currentColor",
                                            stroke_width: "1.5",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                d: "M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88",
                                            }
                                        }
                                    } else {
                                        svg {
                                            class: "h-5 w-5",
                                            fill: "none",
                                            view_box: "0 0 24 24",
                                            stroke: "currentColor",
                                            stroke_width: "1.5",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                d: "M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z",
                                            }
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z",
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "flex items-center justify-between",
                            label { class: "flex items-center",
                                input {
                                    id: "remember-me",
                                    r#type: "checkbox",
                                    class: "w-4 h-4 text-luxury-gold bg-luxury-midnight-black border-luxury-gold/30 rounded focus:ring-luxury-gold focus:ring-2",
                                }
                                span { class: "ml-2 text-sm text-luxury-platinum/80", "記住我" }
                            }
                            a {
                                id: "forgot-password-link",
                                href: "/forgot-password",
                                class: "text-sm text-luxury-gold hover:text-luxury-gold/80 transition-colors",
                                "忘記密碼？"
                            }
                        }

                        button {
                            id: "login-submit",
                            r#type: "submit",
                            class: "w-full luxury-button py-3 disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: submit_disabled,
                            if submitting {
                                div { class: "flex items-center justify-center",
                                    div { class: "w-5 h-5 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin mr-2" }
                                    "登入中..."
                                }
                            } else {
                                "登入"
                            }
                        }
                    }

                    div { class: "mt-8",
                        div { class: "relative",
                            div { class: "absolute inset-0 flex items-center",
                                div { class: "w-full border-t border-luxury-gold/20" }
                            }
                            div { class: "relative flex justify-center text-sm",
                                span { class: "px-2 bg-luxury-midnight-black text-luxury-platinum/60",
                                    "或使用"
                                }
                            }
                        }

                        div { class: "mt-6 grid grid-cols-2 gap-3",
                            button {
                                id: "google-login",
                                r#type: "button",
                                class: "w-full inline-flex justify-center py-2 px-4 border border-luxury-gold/30 rounded-lg shadow-sm bg-luxury-midnight-black/50 text-sm font-medium text-luxury-platinum hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                                disabled: submitting,
                                onclick: move |_| on_google.call(()),
                                svg {
                                    class: "w-5 h-5",
                                    view_box: "0 0 24 24",
                                    path {
                                        fill: "currentColor",
                                        d: "M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z",
                                    }
                                    path {
                                        fill: "currentColor",
                                        d: "M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z",
                                    }
                                    path {
                                        fill: "currentColor",
                                        d: "M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z",
                                    }
                                    path {
                                        fill: "currentColor",
                                        d: "M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z",
                                    }
                                }
                                span { class: "ml-2", "Google" }
                            }
                            button {
                                id: "linkedin-login",
                                r#type: "button",
                                class: "w-full inline-flex justify-center py-2 px-4 border border-luxury-gold/30 rounded-lg shadow-sm bg-luxury-midnight-black/50 text-sm font-medium text-luxury-platinum/50 cursor-not-allowed",
                                disabled: true,
                                svg {
                                    class: "w-5 h-5",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path { d: "M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z" }
                                }
                                span { class: "ml-2", "LinkedIn (即將推出)" }
                            }
                        }
                    }

                    div { class: "mt-8 text-center",
                        p { class: "text-luxury-platinum/60 text-sm",
                            "還沒有帳戶？"
                            a {
                                id: "register-link",
                                href: "/register",
                                class: "text-luxury-gold hover:text-luxury-gold/80 font-medium ml-1 transition-colors",
                                "立即申請加入"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let session = try_use_context::<Signal<Session>>();
    let mut form = use_signal(RegisterForm::default);

    rsx! {
        RegisterScreen {
            form: form(),
            on_email: move |value: String| form.write().email = value,
            on_password: move |value: String| form.write().password = value,
            on_confirm_password: move |value: String| form.write().confirm_password = value,
            on_first_name: move |value: String| form.write().first_name = value,
            on_last_name: move |value: String| form.write().last_name = value,
            on_age: move |value: String| form.write().age = value,
            on_profession: move |value: String| form.write().profession = value,
            on_annual_income: move |value: String| form.write().annual_income = value,
            on_net_worth: move |value: String| form.write().net_worth = value,
            on_membership_tier: move |value: String| form.write().membership_tier = value,
            on_bio: move |value: String| form.write().bio = value,
            on_new_interest: move |value: String| form.write().new_interest = value,
            on_toggle_password: move |_| {
                let show = form.peek().show_password;
                form.write().show_password = !show;
            },
            on_toggle_confirm: move |_| {
                let show = form.peek().show_confirm_password;
                form.write().show_confirm_password = !show;
            },
            on_add_interest: move |_| {
                let current = form.peek().interests.clone();
                let raw = form.peek().new_interest.clone();
                if let Some(next) = push_interest(&current, &raw) {
                    let mut form = form.write();
                    form.interests = next;
                    form.new_interest.clear();
                }
            },
            on_remove_interest: move |interest: String| {
                let current = form.peek().interests.clone();
                form.write().interests = remove_interest(&current, &interest);
            },
            on_prev: move |_| {
                let step = form.peek().step;
                if step > 1 {
                    let mut form = form.write();
                    form.step = step - 1;
                    form.error = None;
                }
            },
            on_next: move |_| {
                let snapshot = form.peek().clone();
                if snapshot.submitting {
                    return;
                }
                match validate_step(&snapshot) {
                    Err(message) => form.write().error = Some(message.to_string()),
                    Ok(()) if snapshot.step < 3 => {
                        let mut form = form.write();
                        form.error = None;
                        form.step = snapshot.step + 1;
                    }
                    Ok(()) => {
                        form.write().submitting = true;
                        form.write().error = None;
                        spawn(async move {
                            match register_account(&snapshot).await {
                                Ok(ok) => {
                                    store_token(&ok.token);
                                    if let Some(mut session) = session {
                                        session.set(Session {
                                            token: Some(ok.token.clone()),
                                            user: ok.user.clone(),
                                            restoring: false,
                                        });
                                    }
                                    navigator.push(Route::Profile {});
                                }
                                Err(message) => {
                                    let mut form = form.write();
                                    form.error = Some(message);
                                    form.submitting = false;
                                }
                            }
                        });
                    }
                }
            },
        }
    }
}

#[component]
pub fn ForgotPassword() -> Element {
    rsx! {
        main {
            id: "forgot-password-stub",
            class: "min-h-screen bg-luxury-midnight-black text-luxury-platinum p-8",
            h1 { "忘記密碼" }
        }
    }
}

#[component]
pub fn Profile() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match user_route_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { ProfileGuardLoading {} },
        RouteGuard::Redirect(_) => {
            navigator.replace(Route::Login {});
            rsx! {
                p { id: "profile-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { ProfileBody {} },
    }
}
