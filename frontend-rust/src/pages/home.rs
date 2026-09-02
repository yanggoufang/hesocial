use crate::icons::{Icon, IconName};
use crate::logic::{next_toggled, toggle_label};
use dioxus::prelude::*;

struct Feature {
    icon: IconName,
    title: &'static str,
    description: &'static str,
}

struct MembershipTier {
    name: &'static str,
    price: &'static str,
    period: &'static str,
    features: &'static [&'static str],
    popular: bool,
}

const FEATURES: &[Feature] = &[
    Feature {
        icon: IconName::Crown,
        title: "頂級會員制",
        description: "嚴格篩選的高淨值會員，確保最優質的社交圈層",
    },
    Feature {
        icon: IconName::Shield,
        title: "隱私保障",
        description: "企業級加密技術，絕對保護您的個人隱私與資料安全",
    },
    Feature {
        icon: IconName::Users,
        title: "AI智能配對",
        description: "基於興趣、背景和偏好的精準社交推薦系統",
    },
    Feature {
        icon: IconName::Calendar,
        title: "獨家活動",
        description: "精心策劃的高端社交活動，從私人晚宴到豪華遊艇派對",
    },
];

const MEMBERSHIP_TIERS: &[MembershipTier] = &[
    MembershipTier {
        name: "Platinum",
        price: "NT$50,000",
        period: "/年",
        features: &[
            "參與精選社交活動",
            "基本身份驗證",
            "標準客服支援",
            "月度活動推薦",
        ],
        popular: false,
    },
    MembershipTier {
        name: "Diamond",
        price: "NT$120,000",
        period: "/年",
        features: &[
            "所有 Platinum 權益",
            "VIP 活動優先預訂",
            "專屬社交顧問",
            "私人活動邀請",
            "高端場地折扣",
        ],
        popular: true,
    },
    MembershipTier {
        name: "Black Card",
        price: "邀請制",
        period: "",
        features: &[
            "所有 Diamond 權益",
            "獨家 VVIP 活動",
            "24/7 禮賓服務",
            "客製化活動規劃",
            "全球合作夥伴特權",
        ],
        popular: false,
    },
];

#[component]
pub fn Home() -> Element {
    let mut toggled = use_signal(|| false);
    rsx! {
        HomeScreen {
            toggled: toggled(),
            on_toggle: move |_| toggled.set(next_toggled(toggled())),
        }
    }
}

#[component]
pub fn HomeScreen(toggled: bool, #[props(default)] on_toggle: EventHandler<()>) -> Element {
    rsx! {
        div { id: "home", class: "min-h-screen",
            section { class: "relative h-screen flex items-center justify-center overflow-hidden",
                div { class: "absolute inset-0 luxury-gradient" }
                div { class: "absolute inset-0 bg-luxury-midnight-black/50" }
                div {
                    class: "relative z-10 text-center max-w-4xl mx-auto px-4 hs-enter",
                    style: "--hs-from: 50px",
                    h1 {
                        id: "scaffold-heading",
                        class: "text-6xl md:text-8xl font-luxury font-bold text-luxury-gold mb-6",
                        "HeSocial"
                    }
                    p { class: "text-xl md:text-2xl text-luxury-platinum mb-8 leading-relaxed",
                        "專為高淨值人士打造的頂級社交平台"
                        br {}
                        "在奢華環境中遇見志同道合的菁英"
                    }
                    div { class: "flex flex-col sm:flex-row gap-4 justify-center",
                        a {
                            id: "home-apply",
                            href: "/register",
                            class: "luxury-button text-lg px-12 py-4",
                            "申請加入"
                        }
                        a {
                            id: "home-explore",
                            href: "/events",
                            class: "luxury-button-outline text-lg px-12 py-4",
                            "探索活動"
                        }
                    }
                    button {
                        id: "toggle-btn",
                        class: "luxury-button mt-8",
                        r#type: "button",
                        onclick: move |_| on_toggle.call(()),
                        "{toggle_label(toggled)}"
                    }
                }
                div { class: "absolute bottom-10 left-1/2 transform -translate-x-1/2 hs-enter-page",
                    div { class: "w-6 h-10 border-2 border-luxury-gold rounded-full flex justify-center",
                        div { class: "w-1 h-3 bg-luxury-gold rounded-full mt-2 animate-bounce" }
                    }
                }
            }

            section { class: "py-20 bg-luxury-midnight-black",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div {
                        class: "text-center mb-16 hs-enter",
                        style: "--hs-from: 50px",
                        h2 { class: "text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6",
                            "為什麼選擇 HeSocial"
                        }
                        p { class: "text-xl text-luxury-platinum/80 max-w-3xl mx-auto",
                            "我們致力於為台灣最優秀的企業家、投資人和專業人士提供最高品質的社交體驗"
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8",
                        for (index, feature) in FEATURES.iter().enumerate() {
                            div {
                                class: "luxury-glass p-8 rounded-2xl hover:bg-luxury-gold/5 transition-all duration-300 group hs-enter",
                                style: "--hs-from: 50px; animation-delay: {index as f32 * 0.2}s",
                                Icon {
                                    name: feature.icon,
                                    class: "h-12 w-12 text-luxury-gold mb-6 group-hover:scale-110 transition-transform duration-300".to_string(),
                                }
                                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                    "{feature.title}"
                                }
                                p { class: "text-luxury-platinum/80 leading-relaxed",
                                    "{feature.description}"
                                }
                            }
                        }
                    }
                }
            }

            section { class: "py-20 luxury-gradient",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div {
                        class: "text-center mb-16 hs-enter",
                        style: "--hs-from: 50px",
                        h2 { class: "text-4xl md:text-5xl font-luxury font-bold text-luxury-midnight-black mb-6",
                            "會員方案"
                        }
                        p { class: "text-xl text-luxury-midnight-black/80 max-w-3xl mx-auto",
                            "選擇最適合您的會員等級，享受專屬的尊榮服務與社交體驗"
                        }
                    }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-8",
                        for (index, tier) in MEMBERSHIP_TIERS.iter().enumerate() {
                            {
                                let card_class = if tier.popular {
                                    "relative p-8 rounded-2xl transition-all duration-300 hover:scale-105 bg-luxury-gold text-luxury-midnight-black shadow-2xl hs-enter"
                                } else {
                                    "relative p-8 rounded-2xl transition-all duration-300 hover:scale-105 luxury-glass hover:bg-white/10 hs-enter"
                                };
                                let dot_class = if tier.popular {
                                    "w-2 h-2 rounded-full mr-3 bg-luxury-midnight-black"
                                } else {
                                    "w-2 h-2 rounded-full mr-3 bg-luxury-gold"
                                };
                                let cta_class = if tier.popular {
                                    "w-full py-3 px-6 rounded-lg font-medium transition-all duration-300 flex items-center justify-center bg-luxury-midnight-black text-luxury-gold hover:bg-luxury-midnight-black/90"
                                } else {
                                    "w-full py-3 px-6 rounded-lg font-medium transition-all duration-300 flex items-center justify-center luxury-button-outline"
                                };
                                let cta_label = if tier.name == "Black Card" {
                                    "申請邀請"
                                } else {
                                    "立即申請"
                                };
                                rsx! {
                                    div {
                                        class: "{card_class}",
                                        style: "--hs-from: 50px; animation-delay: {index as f32 * 0.2}s",
                                        if tier.popular {
                                            div { class: "absolute -top-4 left-1/2 transform -translate-x-1/2",
                                                div { class: "bg-luxury-midnight-black text-luxury-gold px-4 py-2 rounded-full text-sm font-medium flex items-center",
                                                    Icon {
                                                        name: IconName::Star,
                                                        class: "h-4 w-4 mr-1".to_string(),
                                                    }
                                                    "最受歡迎"
                                                }
                                            }
                                        }
                                        div { class: "text-center mb-8",
                                            h3 { class: "text-2xl font-luxury font-bold mb-2", "{tier.name}" }
                                            div { class: "flex items-baseline justify-center",
                                                span { class: "text-4xl font-bold", "{tier.price}" }
                                                span { class: "text-lg ml-1", "{tier.period}" }
                                            }
                                        }
                                        ul { class: "space-y-4 mb-8",
                                            for feature in tier.features {
                                                li { class: "flex items-center",
                                                    div { class: "{dot_class}" }
                                                    span { class: "text-sm", "{feature}" }
                                                }
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "{cta_class}",
                                            "{cta_label}"
                                            Icon {
                                                name: IconName::ArrowRight,
                                                class: "h-4 w-4 ml-2".to_string(),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section { class: "py-20 bg-luxury-midnight-black",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center",
                    div { class: "hs-enter", style: "--hs-from: 50px",
                        h2 { class: "text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6",
                            "準備開始您的尊榮社交之旅？"
                        }
                        p { class: "text-xl text-luxury-platinum/80 mb-8 max-w-3xl mx-auto",
                            "加入台灣最頂級的社交圈，與成功人士建立深度連結，開拓無限可能"
                        }
                        a {
                            id: "home-cta-register",
                            href: "/register",
                            class: "luxury-button text-lg px-12 py-4 inline-flex items-center",
                            "立即申請加入"
                            Icon {
                                name: IconName::ArrowRight,
                                class: "h-5 w-5 ml-2".to_string(),
                            }
                        }
                    }
                }
            }
        }
    }
}
