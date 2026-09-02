use crate::icons::{Icon, IconName};
use crate::profile::{
    ACTIVITY_CREDIT, ACTIVITY_EVENTS_ATTENDED, ACTIVITY_MEMBER_SINCE_YEAR, ACTIVITY_TOTAL_SPENT_K,
    ACTIVITY_UPCOMING, PRIVACY_LEVEL_OPTIONS, PROFILE_ADD_INTEREST_PLACEHOLDER, PROFILE_CANCEL,
    PROFILE_EDIT_LABEL, PROFILE_SAVE, PROFILE_SAVING, ProfileEditForm, ProfileUser,
    UPCOMING_EVENTS, apply_profile_save_success, display_age, display_full_name, display_optional,
    display_privacy_level, fetch_profile, membership_benefits, membership_color_class,
    profile_edit_from_user, profile_partial_payload, profile_picture_src, update_profile,
    validate_profile_edit,
};
use crate::register::push_interest;
use crate::shell::{Presence, presence_after_animation_end, presence_class};
use dioxus::prelude::*;

#[component]
pub fn ProfileBody() -> Element {
    let mut loading = use_signal(|| true);
    let mut failed = use_signal(|| false);
    let mut profile = use_signal(|| None::<ProfileUser>);
    let mut editing = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut edit = use_signal(|| None::<ProfileEditForm>);
    let mut save_error = use_signal(|| None::<String>);
    let mut form_presence = use_signal(|| Presence::Hidden);

    use_effect(move || {
        spawn(async move {
            match fetch_profile().await {
                Ok(user) => {
                    edit.set(Some(profile_edit_from_user(&user)));
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

    rsx! {
        ProfileScreen {
            profile: profile().unwrap(),
            editable: true,
            editing: editing(),
            saving: saving(),
            edit: edit(),
            save_error: save_error(),
            form_presence: form_presence(),
            on_start_edit: move |_| {
                if editing() {
                    return;
                }
                let Some(user) = profile() else {
                    return;
                };
                edit.set(Some(profile_edit_from_user(&user)));
                save_error.set(None);
                editing.set(true);
                form_presence.set(Presence::Entering);
            },
            on_cancel: move |_| {
                let Some(user) = profile() else {
                    return;
                };
                edit.set(Some(profile_edit_from_user(&user)));
                save_error.set(None);
                editing.set(false);
                form_presence.set(Presence::Hidden);
            },
            on_first_name: move |value: String| {
                mutate_edit(edit, |form| form.first_name = value);
            },
            on_last_name: move |value: String| {
                mutate_edit(edit, |form| form.last_name = value);
            },
            on_profession: move |value: String| {
                mutate_edit(edit, |form| form.profession = value);
            },
            on_bio: move |value: String| {
                mutate_edit(edit, |form| form.bio = value);
            },
            on_privacy_level: move |value: i64| {
                mutate_edit(edit, |form| form.privacy_level = value);
            },
            on_new_interest: move |value: String| {
                mutate_edit(edit, |form| form.new_interest = value);
            },
            on_add_interest: move |_| {
                let Some(form) = edit() else {
                    return;
                };
                if let Some(next) = push_interest(&form.interests, &form.new_interest) {
                    mutate_edit(edit, |form| {
                        form.interests = next;
                        form.new_interest.clear();
                    });
                }
            },
            on_remove_interest: move |interest: String| {
                mutate_edit(edit, |form| {
                    form.interests.retain(|item| item != &interest);
                });
            },
            on_save: move |_| {
                if saving() {
                    return;
                }
                let Some(form) = edit() else {
                    return;
                };
                let Some(current) = profile() else {
                    return;
                };
                if let Err(message) = validate_profile_edit(&form) {
                    save_error.set(Some(message.to_string()));
                    return;
                }
                let payload = profile_partial_payload(&current, &form);
                saving.set(true);
                spawn(async move {
                    match update_profile(&payload).await {
                        Ok(user) => {
                            let outcome = apply_profile_save_success(user);
                            profile.set(Some(outcome.profile));
                            edit.set(Some(outcome.form));
                            editing.set(outcome.editing);
                            save_error.set(outcome.error);
                            form_presence.set(Presence::Hidden);
                        }
                        Err(message) => {
                            save_error.set(Some(message));
                        }
                    }
                    saving.set(false);
                });
            },
            on_form_animation_end: move |_| {
                form_presence.set(presence_after_animation_end(form_presence()));
            },
        }
    }
}

fn mutate_edit(mut edit: Signal<Option<ProfileEditForm>>, f: impl FnOnce(&mut ProfileEditForm)) {
    edit.with_mut(|slot| {
        if let Some(form) = slot {
            f(form);
        }
    });
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
pub fn ProfileScreen(
    profile: ProfileUser,
    #[props(default)] editing: bool,
    #[props(default)] saving: bool,
    #[props(default)] editable: bool,
    #[props(default)] edit: Option<ProfileEditForm>,
    #[props(default)] save_error: Option<String>,
    #[props(default)] form_presence: Presence,
    #[props(default)] on_start_edit: EventHandler<()>,
    #[props(default)] on_cancel: EventHandler<()>,
    #[props(default)] on_save: EventHandler<()>,
    #[props(default)] on_first_name: EventHandler<String>,
    #[props(default)] on_last_name: EventHandler<String>,
    #[props(default)] on_profession: EventHandler<String>,
    #[props(default)] on_bio: EventHandler<String>,
    #[props(default)] on_privacy_level: EventHandler<i64>,
    #[props(default)] on_new_interest: EventHandler<String>,
    #[props(default)] on_add_interest: EventHandler<()>,
    #[props(default)] on_remove_interest: EventHandler<String>,
    #[props(default)] on_form_animation_end: EventHandler<()>,
) -> Element {
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
    let show_form = editing && edit.is_some();
    let form = edit.clone().unwrap_or_default();
    let form_class = format!(
        "space-y-6 {}",
        presence_class(form_presence, "hs-enter", "hs-exit")
    );

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
                                if editable {
                                    button {
                                        r#type: "button",
                                        class: "mt-6 luxury-button-outline w-full",
                                        onclick: move |_| on_start_edit.call(()),
                                        Icon { name: IconName::Edit, class: "h-4 w-4 mr-2".to_string() }
                                        "{PROFILE_EDIT_LABEL}"
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
                            div { class: "flex items-center justify-between mb-6",
                                h2 { class: "text-2xl font-luxury font-semibold text-luxury-gold", "個人資訊" }
                                if editable && !editing {
                                    button {
                                        r#type: "button",
                                        class: "text-luxury-gold hover:text-luxury-gold/80 transition-colors",
                                        onclick: move |_| on_start_edit.call(()),
                                        Icon { name: IconName::Edit, class: "h-5 w-5".to_string() }
                                    }
                                }
                            }
                            if show_form {
                                div {
                                    class: "{form_class}",
                                    onanimationend: move |_| on_form_animation_end.call(()),
                                    if let Some(message) = save_error.clone() {
                                        div {
                                            id: "profile-edit-error",
                                            class: "p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm",
                                            "{message}"
                                        }
                                    }
                                    div { class: "grid grid-cols-2 gap-4",
                                        div {
                                            label { class: "block text-luxury-platinum text-sm font-medium mb-2", "名字" }
                                            input {
                                                r#type: "text",
                                                name: "firstName",
                                                value: "{form.first_name}",
                                                class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors",
                                                oninput: move |evt| on_first_name.call(evt.value()),
                                            }
                                        }
                                        div {
                                            label { class: "block text-luxury-platinum text-sm font-medium mb-2", "姓氏" }
                                            input {
                                                r#type: "text",
                                                name: "lastName",
                                                value: "{form.last_name}",
                                                class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors",
                                                oninput: move |evt| on_last_name.call(evt.value()),
                                            }
                                        }
                                    }
                                    div {
                                        label { class: "block text-luxury-platinum text-sm font-medium mb-2", "職業" }
                                        input {
                                            r#type: "text",
                                            name: "profession",
                                            value: "{form.profession}",
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors",
                                            oninput: move |evt| on_profession.call(evt.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-luxury-platinum text-sm font-medium mb-2", "個人簡介" }
                                        textarea {
                                            name: "bio",
                                            value: "{form.bio}",
                                            rows: 3,
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors resize-none",
                                            oninput: move |evt| on_bio.call(evt.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-luxury-platinum text-sm font-medium mb-2", "隱私等級" }
                                        select {
                                            name: "privacyLevel",
                                            value: "{form.privacy_level}",
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors",
                                            onchange: move |evt| {
                                                if let Ok(level) = evt.value().parse::<i64>() {
                                                    on_privacy_level.call(level);
                                                }
                                            },
                                            for (value, label) in PRIVACY_LEVEL_OPTIONS {
                                                option { value: "{value}", "{label}" }
                                            }
                                        }
                                    }
                                    div {
                                        label { class: "block text-luxury-platinum text-sm font-medium mb-2", "興趣愛好" }
                                        div { class: "flex space-x-2 mb-3",
                                            input {
                                                r#type: "text",
                                                value: "{form.new_interest}",
                                                class: "flex-1 bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-2 text-luxury-platinum focus:outline-none focus:border-luxury-gold transition-colors",
                                                placeholder: "{PROFILE_ADD_INTEREST_PLACEHOLDER}",
                                                oninput: move |evt| on_new_interest.call(evt.value()),
                                                onkeydown: move |evt| {
                                                    if evt.key() == Key::Enter {
                                                        evt.prevent_default();
                                                        on_add_interest.call(());
                                                    }
                                                },
                                            }
                                            button {
                                                r#type: "button",
                                                class: "px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold/30 transition-colors",
                                                onclick: move |_| on_add_interest.call(()),
                                                Icon { name: IconName::Plus, class: "h-4 w-4".to_string() }
                                            }
                                        }
                                        div { class: "flex flex-wrap gap-2",
                                            for interest in form.interests.iter() {
                                                {
                                                    let label = interest.clone();
                                                    rsx! {
                                                        span { class: "inline-flex items-center px-3 py-1 bg-luxury-gold/20 text-luxury-gold text-sm rounded-full",
                                                            "{interest}"
                                                            button {
                                                                r#type: "button",
                                                                class: "ml-2 hover:text-luxury-gold/80",
                                                                onclick: move |_| on_remove_interest.call(label.clone()),
                                                                Icon { name: IconName::X, class: "h-3 w-3".to_string() }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "flex space-x-4",
                                        button {
                                            r#type: "button",
                                            class: "flex-1 px-4 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors",
                                            onclick: move |_| on_cancel.call(()),
                                            "{PROFILE_CANCEL}"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "flex-1 luxury-button py-3 disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center justify-center",
                                            disabled: saving,
                                            onclick: move |_| on_save.call(()),
                                            if saving {
                                                div { class: "w-4 h-4 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin mr-2" }
                                                "{PROFILE_SAVING}"
                                            } else {
                                                Icon { name: IconName::Save, class: "h-4 w-4 mr-2".to_string() }
                                                "{PROFILE_SAVE}"
                                            }
                                        }
                                    }
                                }
                            } else {
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
}
