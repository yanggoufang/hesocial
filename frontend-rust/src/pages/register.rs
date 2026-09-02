use crate::auth::password_input_type;
use crate::icons::{Icon, IconName};
use crate::register::{MEMBERSHIP_TIERS, RegisterForm, step_title};
use dioxus::prelude::*;

#[component]
pub fn RegisterScreen(
    form: RegisterForm,
    #[props(default)] on_email: EventHandler<String>,
    #[props(default)] on_password: EventHandler<String>,
    #[props(default)] on_confirm_password: EventHandler<String>,
    #[props(default)] on_first_name: EventHandler<String>,
    #[props(default)] on_last_name: EventHandler<String>,
    #[props(default)] on_age: EventHandler<String>,
    #[props(default)] on_profession: EventHandler<String>,
    #[props(default)] on_annual_income: EventHandler<String>,
    #[props(default)] on_net_worth: EventHandler<String>,
    #[props(default)] on_membership_tier: EventHandler<String>,
    #[props(default)] on_bio: EventHandler<String>,
    #[props(default)] on_new_interest: EventHandler<String>,
    #[props(default)] on_toggle_password: EventHandler<()>,
    #[props(default)] on_toggle_confirm: EventHandler<()>,
    #[props(default)] on_add_interest: EventHandler<()>,
    #[props(default)] on_remove_interest: EventHandler<String>,
    #[props(default)] on_prev: EventHandler<()>,
    #[props(default)] on_next: EventHandler<()>,
) -> Element {
    let password_type = password_input_type(form.show_password);
    let confirm_type = password_input_type(form.show_confirm_password);
    let prev_disabled = form.step == 1;
    let next_label = if form.step == 3 {
        "提交申請"
    } else {
        "下一步"
    };
    let title = step_title(form.step);
    let bio_len = form.bio.len();

    rsx! {
        div { id: "register", class: "min-h-screen bg-luxury-midnight-black py-8 px-4",
            div { class: "max-w-2xl mx-auto",
                div {
                    class: "luxury-glass p-8 rounded-2xl hs-enter",
                    style: "--hs-from: 30px",
                    div { class: "text-center mb-8",
                        div { class: "flex items-center justify-center mb-4",
                            Icon {
                                name: IconName::Crown,
                                class: "h-12 w-12 text-luxury-gold".to_string(),
                            }
                        }
                        h1 {
                            id: "register-heading",
                            class: "text-3xl font-luxury font-bold text-luxury-gold mb-2",
                            "加入 HeSocial"
                        }
                        p { class: "text-luxury-platinum/80",
                            "申請成為尊榮會員，開啟您的頂級社交之旅"
                        }
                    }

                    div { class: "flex items-center justify-center mb-8",
                        for step in 1..=3u8 {
                            div { class: "flex items-center",
                                {
                                    let bubble = if step <= form.step {
                                        "w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium bg-luxury-gold text-luxury-midnight-black"
                                    } else {
                                        "w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium bg-luxury-gold/20 text-luxury-gold"
                                    };
                                    rsx! {
                                        div { class: "{bubble}", "{step}" }
                                    }
                                }
                                if step < 3 {
                                    {
                                        let line = if step < form.step {
                                            "w-12 h-0.5 mx-2 bg-luxury-gold"
                                        } else {
                                            "w-12 h-0.5 mx-2 bg-luxury-gold/20"
                                        };
                                        rsx! { div { class: "{line}" } }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "text-center mb-6",
                        h2 { id: "register-step-title", class: "text-xl font-luxury text-luxury-gold",
                            "{title}"
                        }
                    }

                    if let Some(message) = form.error.clone() {
                        div {
                            id: "register-error",
                            class: "mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-400 text-sm",
                            "{message}"
                        }
                    }

                    form {
                        onsubmit: move |evt| {
                            evt.prevent_default();
                            on_next.call(());
                        },
                        if form.step == 1 {
                            div { class: "space-y-6",
                                div {
                                    label {
                                        r#for: "email",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "電子郵件 *"
                                    }
                                    div { class: "relative",
                                        Icon {
                                            name: IconName::Mail,
                                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                        }
                                        input {
                                            r#type: "email",
                                            id: "email",
                                            name: "email",
                                            value: "{form.email}",
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
                                        "密碼 *"
                                    }
                                    div { class: "relative",
                                        Icon {
                                            name: IconName::Lock,
                                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                        }
                                        input {
                                            r#type: "{password_type}",
                                            id: "password",
                                            name: "password",
                                            value: "{form.password}",
                                            required: true,
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 pr-12 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "至少8個字元，包含大小寫字母、數字和特殊符號",
                                            oninput: move |evt| on_password.call(evt.value()),
                                        }
                                        button {
                                            id: "register-password-toggle",
                                            r#type: "button",
                                            class: "absolute right-3 top-1/2 transform -translate-y-1/2 text-luxury-gold hover:text-luxury-gold/80 transition-colors",
                                            onclick: move |_| on_toggle_password.call(()),
                                            if form.show_password {
                                                Icon { name: IconName::EyeOff, class: "h-5 w-5".to_string() }
                                            } else {
                                                Icon { name: IconName::Eye, class: "h-5 w-5".to_string() }
                                            }
                                        }
                                    }
                                }
                                div {
                                    label {
                                        r#for: "confirmPassword",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "確認密碼 *"
                                    }
                                    div { class: "relative",
                                        Icon {
                                            name: IconName::Lock,
                                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                        }
                                        input {
                                            r#type: "{confirm_type}",
                                            id: "confirmPassword",
                                            name: "confirmPassword",
                                            value: "{form.confirm_password}",
                                            required: true,
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 pr-12 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "請再次輸入密碼",
                                            oninput: move |evt| on_confirm_password.call(evt.value()),
                                        }
                                        button {
                                            id: "register-confirm-toggle",
                                            r#type: "button",
                                            class: "absolute right-3 top-1/2 transform -translate-y-1/2 text-luxury-gold hover:text-luxury-gold/80 transition-colors",
                                            onclick: move |_| on_toggle_confirm.call(()),
                                            if form.show_confirm_password {
                                                Icon { name: IconName::EyeOff, class: "h-5 w-5".to_string() }
                                            } else {
                                                Icon { name: IconName::Eye, class: "h-5 w-5".to_string() }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if form.step == 2 {
                            div { class: "space-y-6",
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label {
                                            r#for: "firstName",
                                            class: "block text-luxury-platinum text-sm font-medium mb-2",
                                            "名字 *"
                                        }
                                        div { class: "relative",
                                            Icon {
                                                name: IconName::User,
                                                class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                            }
                                            input {
                                                r#type: "text",
                                                id: "firstName",
                                                name: "firstName",
                                                value: "{form.first_name}",
                                                required: true,
                                                class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                                placeholder: "名字",
                                                oninput: move |evt| on_first_name.call(evt.value()),
                                            }
                                        }
                                    }
                                    div {
                                        label {
                                            r#for: "lastName",
                                            class: "block text-luxury-platinum text-sm font-medium mb-2",
                                            "姓氏 *"
                                        }
                                        input {
                                            r#type: "text",
                                            id: "lastName",
                                            name: "lastName",
                                            value: "{form.last_name}",
                                            required: true,
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "姓氏",
                                            oninput: move |evt| on_last_name.call(evt.value()),
                                        }
                                    }
                                }
                                div {
                                    label {
                                        r#for: "age",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "年齡 *"
                                    }
                                    input {
                                        r#type: "number",
                                        id: "age",
                                        name: "age",
                                        value: "{form.age}",
                                        required: true,
                                        min: "18",
                                        max: "100",
                                        class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                        placeholder: "請輸入年齡",
                                        oninput: move |evt| on_age.call(evt.value()),
                                    }
                                }
                                div {
                                    label {
                                        r#for: "profession",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "職業 *"
                                    }
                                    div { class: "relative",
                                        Icon {
                                            name: IconName::Briefcase,
                                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                        }
                                        input {
                                            r#type: "text",
                                            id: "profession",
                                            name: "profession",
                                            value: "{form.profession}",
                                            required: true,
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "例：企業執行長、投資銀行家、醫師",
                                            oninput: move |evt| on_profession.call(evt.value()),
                                        }
                                    }
                                }
                            }
                        }

                        if form.step == 3 {
                            div { class: "space-y-6",
                                div {
                                    label {
                                        r#for: "annualIncome",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "年收入 (新台幣萬元) *"
                                    }
                                    div { class: "relative",
                                        Icon {
                                            name: IconName::DollarSign,
                                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                        }
                                        input {
                                            r#type: "number",
                                            id: "annualIncome",
                                            name: "annualIncome",
                                            value: "{form.annual_income}",
                                            required: true,
                                            min: "500",
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "請輸入年收入 (萬元)",
                                            oninput: move |evt| on_annual_income.call(evt.value()),
                                        }
                                    }
                                    p { class: "text-luxury-platinum/60 text-xs mt-1",
                                        "申請資格：年收入需達新台幣500萬元以上"
                                    }
                                }
                                div {
                                    label {
                                        r#for: "netWorth",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "淨資產 (新台幣萬元) *"
                                    }
                                    div { class: "relative",
                                        Icon {
                                            name: IconName::DollarSign,
                                            class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                                        }
                                        input {
                                            r#type: "number",
                                            id: "netWorth",
                                            name: "netWorth",
                                            value: "{form.net_worth}",
                                            required: true,
                                            min: "3000",
                                            class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "請輸入淨資產 (萬元)",
                                            oninput: move |evt| on_net_worth.call(evt.value()),
                                        }
                                    }
                                    p { class: "text-luxury-platinum/60 text-xs mt-1",
                                        "申請資格：淨資產需達新台幣3000萬元以上"
                                    }
                                }
                                div {
                                    label {
                                        r#for: "membershipTier",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "會員等級 *"
                                    }
                                    div { class: "space-y-3",
                                        for tier in MEMBERSHIP_TIERS {
                                            label { class: "flex items-start space-x-3 cursor-pointer",
                                                input {
                                                    r#type: "radio",
                                                    name: "membershipTier",
                                                    value: "{tier.value}",
                                                    checked: form.membership_tier == tier.value,
                                                    class: "mt-1 w-4 h-4 text-luxury-gold bg-luxury-midnight-black border-luxury-gold/30 focus:ring-luxury-gold focus:ring-2",
                                                    onchange: move |_| on_membership_tier.call(tier.value.to_string()),
                                                }
                                                div { class: "flex-1",
                                                    div { class: "flex items-center justify-between",
                                                        span { class: "text-luxury-gold font-medium", "{tier.label}" }
                                                        span { class: "text-luxury-platinum/80 text-sm", "{tier.price}" }
                                                    }
                                                    p { class: "text-luxury-platinum/60 text-sm", "{tier.description}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                div {
                                    label {
                                        r#for: "interests",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "興趣愛好 *"
                                    }
                                    div { class: "flex space-x-2 mb-3",
                                        input {
                                            r#type: "text",
                                            id: "interests",
                                            value: "{form.new_interest}",
                                            maxlength: "50",
                                            class: "flex-1 bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-2 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors",
                                            placeholder: "添加興趣（例：藝術收藏、高爾夫、紅酒品鑑）",
                                            oninput: move |evt| on_new_interest.call(evt.value()),
                                            onkeydown: move |evt| {
                                                if evt.key() == dioxus::prelude::Key::Enter {
                                                    evt.prevent_default();
                                                    on_add_interest.call(());
                                                }
                                            },
                                        }
                                        button {
                                            id: "register-add-interest",
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
                                    p { class: "text-luxury-platinum/60 text-xs mt-1",
                                        "請至少添加1個興趣，最多10個"
                                    }
                                }
                                div {
                                    label {
                                        r#for: "bio",
                                        class: "block text-luxury-platinum text-sm font-medium mb-2",
                                        "個人簡介 (選填)"
                                    }
                                    textarea {
                                        id: "bio",
                                        name: "bio",
                                        rows: 3,
                                        maxlength: "500",
                                        class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold transition-colors resize-none",
                                        placeholder: "請簡述您的背景與興趣（選填，最多500字）",
                                        value: "{form.bio}",
                                        oninput: move |evt| on_bio.call(evt.value()),
                                    }
                                    p { class: "text-luxury-platinum/60 text-xs mt-1", "{bio_len}/500" }
                                }
                            }
                        }

                        div { class: "flex justify-between mt-8",
                            button {
                                id: "register-prev",
                                r#type: "button",
                                class: "px-6 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                                disabled: prev_disabled,
                                onclick: move |_| on_prev.call(()),
                                "上一步"
                            }
                            button {
                                id: "register-next",
                                r#type: "button",
                                class: "px-6 py-3 luxury-button disabled:opacity-50 disabled:cursor-not-allowed",
                                disabled: form.submitting,
                                onclick: move |_| on_next.call(()),
                                if form.submitting {
                                    div { class: "flex items-center",
                                        div { class: "w-5 h-5 border-2 border-luxury-midnight-black border-t-transparent rounded-full animate-spin mr-2" }
                                        "提交申請中..."
                                    }
                                } else {
                                    "{next_label}"
                                }
                            }
                        }
                    }

                    div { class: "mt-8 text-center",
                        p { class: "text-luxury-platinum/60 text-sm",
                            "已有帳戶？"
                            a {
                                id: "register-login-link",
                                href: "/login",
                                class: "text-luxury-gold hover:text-luxury-gold/80 font-medium ml-1 transition-colors",
                                "立即登入"
                            }
                        }
                    }
                }
            }
        }
    }
}
