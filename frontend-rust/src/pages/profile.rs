use crate::icons::{Icon, IconName};
use crate::profile::{
    ACTIVITY_CREDIT, ACTIVITY_EVENTS_ATTENDED, ACTIVITY_MEMBER_SINCE_YEAR, ACTIVITY_TOTAL_SPENT_K,
    ACTIVITY_UPCOMING, ProfileUser, UPCOMING_EVENTS, display_age, display_full_name,
    display_optional, display_privacy_level, fetch_profile, membership_benefits,
    membership_color_class, profile_picture_src,
};
use dioxus::prelude::*;

#[component]
pub fn ProfileBody() -> Element {
    let mut loading = use_signal(|| true);
    let mut failed = use_signal(|| false);
    let mut profile = use_signal(|| None::<ProfileUser>);

    use_effect(move || {
        spawn(async move {
            match fetch_profile().await {
                Ok(user) => {
                    profile.set(Some(user));
                    failed.set(false);
                    loading.set(false);
                }
                Err(_) => {
                    failed.set(true);
                    loading.set(false);
                }
            }
        });
    });

    if loading() {
        return rsx! {
            ProfileStatus {
                id: "profile-stub".to_string(),
                message: "載入個人資料中...".to_string(),
                spinning: true,
            }
        };
    }
    if failed() || profile().is_none() {
        return rsx! {
            ProfileStatus {
                id: "profile-stub".to_string(),
                message: "無法載入個人資料".to_string(),
                spinning: false,
            }
        };
    }
    rsx! { ProfileScreen { profile: profile().unwrap() } }
}

#[component]
pub fn ProfileGuardLoading() -> Element {
    rsx! {
        ProfileStatus {
            id: "profile-stub".to_string(),
            message: "驗證存取權限中...".to_string(),
            spinning: true,
        }
    }
}

#[component]
fn ProfileStatus(id: String, message: String, spinning: bool) -> Element {
    rsx! {
        div {
            id: "{id}",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                if spinning {
                    div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                }
                p { class: "text-luxury-platinum", "{message}" }
            }
        }
    }
}

#[component]
pub fn ProfileScreen(profile: ProfileUser) -> Element {
    let name = display_full_name(profile.first_name.as_deref(), profile.last_name.as_deref());
    let email = display_optional(profile.email.as_deref());
    let profession = display_optional(profile.profession.as_deref());
    let bio = display_optional(profile.bio.as_deref());
    let age = display_age(profile.age);
    let privacy = display_privacy_level(profile.privacy_level);
    let picture = profile_picture_src(profile.profile_picture.as_deref());
    let tier_label = profile.membership_tier_label().unwrap_or("").to_string();
    let tier_color = membership_color_class(profile.membership_tier_label());
    let benefits = membership_benefits(profile.membership_tier_label());
    let interests = profile.interests.clone();

    rsx! {
        div {
            id: "profile-stub",
            class: "min-h-screen bg-luxury-midnight-black py-8",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                    div { class: "lg:col-span-1",
                        div { class: "luxury-glass p-6 rounded-2xl",
                            div { class: "text-center",
                                div { class: "relative inline-block mb-4",
                                    img {
                                        src: "{picture}",
                                        alt: "Profile",
                                        class: "w-24 h-24 rounded-full border-4 border-luxury-gold",
                                    }
                                    div { class: "absolute -bottom-1 -right-1 w-8 h-8 bg-luxury-gold rounded-full flex items-center justify-center",
                                        Icon { name: IconName::Crown, class: "h-4 w-4 text-luxury-midnight-black".to_string() }
                                    }
                                }
                                h1 {
                                    id: "profile-heading",
                                    class: "text-2xl font-luxury font-bold text-luxury-gold mb-2",
                                    "{name}"
                                }
                                div { class: "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium mb-4 {tier_color}",
                                    Icon { name: IconName::Crown, class: "h-4 w-4 mr-1".to_string() }
                                    "{tier_label} 會員"
                                }
                                p { class: "text-luxury-platinum/80 text-sm mb-6", "{bio}" }
                                div { class: "space-y-3 text-sm",
                                    div { class: "flex items-center justify-center text-luxury-platinum/80",
                                        Icon { name: IconName::Mail, class: "h-4 w-4 mr-2".to_string() }
                                        "{email}"
                                    }
                                    div { class: "flex items-center justify-center text-luxury-platinum/80",
                                        Icon { name: IconName::Briefcase, class: "h-4 w-4 mr-2".to_string() }
                                        "{profession}"
                                    }
                                    div { class: "flex items-center justify-center text-luxury-platinum/80",
                                        Icon { name: IconName::Calendar, class: "h-4 w-4 mr-2".to_string() }
                                        "會員自 {ACTIVITY_MEMBER_SINCE_YEAR} 年"
                                    }
                                }
                            }
                        }
                        div { class: "luxury-glass p-6 rounded-2xl mt-6",
                            h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4", "會員權益" }
                            ul { class: "space-y-2",
                                for benefit in benefits.iter() {
                                    li { class: "flex items-start text-luxury-platinum/80 text-sm",
                                        Icon { name: IconName::Star, class: "h-4 w-4 text-luxury-gold mr-2 mt-0.5 flex-shrink-0".to_string() }
                                        "{benefit}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "lg:col-span-2",
                        div { class: "grid grid-cols-2 md:grid-cols-4 gap-4 mb-8",
                            div { class: "luxury-glass p-4 rounded-xl text-center",
                                Icon { name: IconName::Users, class: "h-8 w-8 text-luxury-gold mx-auto mb-2".to_string() }
                                div { class: "text-2xl font-bold text-luxury-gold", "{ACTIVITY_EVENTS_ATTENDED}" }
                                div { class: "text-luxury-platinum/80 text-sm", "參與活動" }
                            }
                            div { class: "luxury-glass p-4 rounded-xl text-center",
                                Icon { name: IconName::Calendar, class: "h-8 w-8 text-luxury-gold mx-auto mb-2".to_string() }
                                div { class: "text-2xl font-bold text-luxury-gold", "{ACTIVITY_UPCOMING}" }
                                div { class: "text-luxury-platinum/80 text-sm", "即將參與" }
                            }
                            div { class: "luxury-glass p-4 rounded-xl text-center",
                                Icon { name: IconName::TrendingUp, class: "h-8 w-8 text-luxury-gold mx-auto mb-2".to_string() }
                                div { class: "text-2xl font-bold text-luxury-gold", "NT$ {ACTIVITY_TOTAL_SPENT_K}K" }
                                div { class: "text-luxury-platinum/80 text-sm", "累計消費" }
                            }
                            div { class: "luxury-glass p-4 rounded-xl text-center",
                                Icon { name: IconName::Award, class: "h-8 w-8 text-luxury-gold mx-auto mb-2".to_string() }
                                div { class: "text-2xl font-bold text-luxury-gold", "{ACTIVITY_CREDIT}" }
                                div { class: "text-luxury-platinum/80 text-sm", "信用評級" }
                            }
                        }
                        div { class: "luxury-glass p-6 rounded-2xl mb-8",
                            h2 { class: "text-2xl font-luxury font-semibold text-luxury-gold mb-6", "即將參與的活動" }
                            div { class: "space-y-4",
                                for event in UPCOMING_EVENTS.iter() {
                                    div { class: "flex items-center justify-between p-4 bg-luxury-midnight-black/30 rounded-lg",
                                        div {
                                            h3 { class: "text-luxury-platinum font-medium", "{event.name}" }
                                            p { class: "text-luxury-platinum/60 text-sm", "{event.date_label}" }
                                        }
                                        span {
                                            class: if event.confirmed {
                                                "px-3 py-1 rounded-full text-xs font-medium border bg-green-500/20 text-green-400 border-green-500/30"
                                            } else {
                                                "px-3 py-1 rounded-full text-xs font-medium border bg-yellow-500/20 text-yellow-400 border-yellow-500/30"
                                            },
                                            if event.confirmed { "已確認" } else { "待審核" }
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "luxury-glass p-6 rounded-2xl",
                            h2 { class: "text-2xl font-luxury font-semibold text-luxury-gold mb-6", "個人資訊" }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                                div { class: "space-y-4",
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "姓名" }
                                        p { class: "text-luxury-platinum font-medium", "{name}" }
                                    }
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "電子郵件" }
                                        p { class: "text-luxury-platinum font-medium", "{email}" }
                                    }
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "職業" }
                                        p { class: "text-luxury-platinum font-medium", "{profession}" }
                                    }
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "年齡" }
                                        p { class: "text-luxury-platinum font-medium", "{age}" }
                                    }
                                }
                                div { class: "space-y-4",
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "會員等級" }
                                        p { class: "font-medium {tier_color}", "{tier_label}" }
                                    }
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "隱私等級" }
                                        p { class: "text-luxury-platinum font-medium", "{privacy}" }
                                    }
                                    div {
                                        label { class: "text-luxury-platinum/60 text-sm", "興趣愛好" }
                                        div { class: "flex flex-wrap gap-2 mt-1",
                                            for interest in interests.iter() {
                                                span { class: "px-2 py-1 bg-luxury-gold/20 text-luxury-gold text-xs rounded-full",
                                                    "{interest}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
