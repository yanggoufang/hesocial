use std::cell::Cell;
use std::rc::Rc;

use crate::events::{format_event_date, format_event_time};
use crate::icons::{Icon, IconName};
use crate::permissions::Session;
use crate::vvip::{
    CATEGORY_FILTERS, CategoryFilter, PERKS, PreviewBundle, PreviewStatus, VvipEvent, VvipStats,
    VvipSurface, category_count, event_image, fetch_preview, fetch_vvip_events, filter_events,
    format_vvip_price, recruitment_join_href, vvip_stats, vvip_surface,
};
use dioxus::prelude::*;

const PERK_ICONS: [IconName; 4] = [
    IconName::Crown,
    IconName::Diamond,
    IconName::Shield,
    IconName::Award,
];

#[component]
pub fn Vvip() -> Element {
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let mut loading = use_signal(|| true);
    let mut events = use_signal(Vec::<VvipEvent>::new);
    let mut preview = use_signal(|| PreviewBundle {
        status: PreviewStatus::SignedOut,
        attendees: Vec::new(),
    });
    let mut selected_category = use_signal(|| "all".to_string());
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let token = session().token.clone();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let fetched = fetch_vvip_events().await;
                let bundle = fetch_preview(token.as_deref(), &fetched).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                events.set(fetched);
                preview.set(bundle);
                loading.set(false);
            });
        }
    });

    let current = session();
    let snapshot = current.snapshot();
    let surface = vvip_surface(current.restoring, &snapshot);

    match surface {
        VvipSurface::Loading => rsx! { VvipLoading {} },
        VvipSurface::Recruitment => rsx! {
            VvipRecruitmentScreen {
                loading: loading(),
                events: events(),
                preview: preview(),
                is_authenticated: snapshot.is_authenticated,
            }
        },
        VvipSurface::Content => rsx! {
            VvipScreen {
                loading: loading(),
                events: events(),
                selected_category: selected_category(),
                on_category: move |value: String| selected_category.set(value),
            }
        },
    }
}

#[component]
fn VvipLoading() -> Element {
    rsx! {
        div {
            id: "vvip-loading",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                p { class: "text-luxury-platinum", "載入VIP頁面中..." }
            }
        }
    }
}

#[component]
pub fn VvipScreen(
    loading: bool,
    events: Vec<VvipEvent>,
    selected_category: String,
    #[props(default)] on_category: EventHandler<String>,
) -> Element {
    let filtered = filter_events(&events, &selected_category);
    let stats = vvip_stats(&events, &[]);
    let filters: Vec<(&CategoryFilter, u32)> = CATEGORY_FILTERS
        .iter()
        .map(|filter| (filter, category_count(&events, filter.id)))
        .collect();
    rsx! {
        div {
            id: "vvip-content",
            class: "min-h-screen bg-luxury-midnight-black",
            VvipHero {}
            VvipStatsRow { stats }
            VvipPerks {}
            section { class: "py-20 bg-luxury-midnight-black/50",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div {
                        class: "text-center mb-12 hs-enter",
                        style: "--hs-from: 30px",
                        h2 { class: "text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6",
                            "獨家活動"
                        }
                        p { class: "text-xl text-luxury-platinum/80 max-w-3xl mx-auto",
                            "僅限 VVIP 會員參與的頂級社交體驗"
                        }
                    }
                    div { class: "flex flex-wrap justify-center gap-4 mb-12 hs-enter-filters",
                        for (filter, count) in filters {
                            button {
                                id: "vvip-category-{filter.id}",
                                r#type: "button",
                                class: if selected_category == filter.id {
                                    "px-6 py-3 rounded-lg font-medium transition-all duration-300 bg-luxury-gold text-luxury-midnight-black"
                                } else {
                                    "px-6 py-3 rounded-lg font-medium transition-all duration-300 bg-luxury-gold/20 text-luxury-gold hover:bg-luxury-gold/30"
                                },
                                onclick: move |_| on_category.call(filter.id.to_string()),
                                "{filter.name} ({count})"
                            }
                        }
                    }
                    if loading {
                        div {
                            id: "vvip-events-loading",
                            class: "text-center text-luxury-gold text-2xl py-12",
                            "載入中..."
                        }
                    } else if filtered.is_empty() {
                        div {
                            id: "vvip-empty-category",
                            class: "text-center py-12 hs-enter",
                            Icon {
                                name: IconName::Lock,
                                class: "h-16 w-16 text-luxury-gold/50 mx-auto mb-4".to_string(),
                            }
                            h3 { class: "text-xl font-luxury text-luxury-gold mb-2",
                                "暫無此類別活動"
                            }
                            p { class: "text-luxury-platinum/60",
                                "請選擇其他類別或稍後再來查看"
                            }
                        }
                    } else {
                        div { id: "vvip-events-grid", class: "grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-8",
                            for (index, event) in filtered.into_iter().enumerate() {
                                VvipEventCard { event, index: index as u32 }
                            }
                        }
                    }
                }
            }
            VvipCta {}
        }
    }
}

#[component]
pub fn VvipRecruitmentScreen(
    loading: bool,
    events: Vec<VvipEvent>,
    preview: PreviewBundle,
    is_authenticated: bool,
) -> Element {
    let stats = vvip_stats(&events, &preview.attendees);
    let join_href = recruitment_join_href(is_authenticated);
    let degraded = matches!(
        preview.status,
        PreviewStatus::SignedOut | PreviewStatus::Unauthorized | PreviewStatus::Restricted
    ) || (preview.status == PreviewStatus::Ready && preview.attendees.is_empty());
    let degrade_copy = match preview.status {
        PreviewStatus::SignedOut | PreviewStatus::Unauthorized => {
            "登入後可預覽正在參加活動的優質會員（身分已遮蔽）"
        }
        PreviewStatus::Restricted => "完成活動報名後即可查看參加者預覽",
        PreviewStatus::Ready => "目前尚無公開的參加者預覽",
    };
    rsx! {
        div {
            id: "vvip-recruitment",
            class: "min-h-screen bg-luxury-midnight-black",
            VvipHero {}
            VvipStatsRow { stats }
            VvipPerks {}
            section { class: "py-20 bg-luxury-midnight-black/50",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div {
                        class: "text-center mb-12 hs-enter",
                        style: "--hs-from: 30px",
                        h2 { class: "text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6",
                            "正在參加活動的優質會員"
                        }
                        p { class: "text-xl text-luxury-platinum/80 max-w-3xl mx-auto",
                            "職業、產業與會員等級可見，身分已遮蔽"
                        }
                        p { class: "text-luxury-gold mt-4",
                            "僅限 Diamond 以上且已完成身份驗證的會員進入"
                        }
                    }
                    if loading {
                        div {
                            id: "vvip-preview-loading",
                            class: "text-center text-luxury-gold text-2xl py-12",
                            "載入中..."
                        }
                    } else if degraded {
                        div {
                            id: "vvip-preview-degraded",
                            class: "luxury-glass p-8 rounded-2xl text-center hs-enter",
                            Icon {
                                name: IconName::Lock,
                                class: "h-12 w-12 text-luxury-gold mx-auto mb-4".to_string(),
                            }
                            p { class: "text-luxury-platinum/80", "{degrade_copy}" }
                        }
                    } else {
                        div {
                            id: "vvip-preview",
                            class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for (index, attendee) in preview.attendees.iter().cloned().enumerate() {
                                div {
                                    id: "vvip-preview-card-{index}",
                                    class: "luxury-glass p-6 rounded-2xl hs-enter",
                                    style: "--hs-from: 30px; animation-delay: {index as f32 * 0.1}s",
                                    if let Some(profession) = attendee.profession.clone() {
                                        p { class: "text-luxury-gold font-semibold mb-2", "{profession}" }
                                    }
                                    if let Some(industry) = attendee.industry.clone() {
                                        p { class: "text-luxury-platinum/80 text-sm mb-2", "{industry}" }
                                    }
                                    if let Some(tier) = attendee.membership_tier.clone() {
                                        p { class: "text-luxury-gold text-sm", "{tier}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section { class: "py-20",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div {
                        class: "luxury-glass p-12 rounded-2xl text-center hs-enter",
                        style: "--hs-from: 30px",
                        Icon {
                            name: IconName::Crown,
                            class: "h-16 w-16 text-luxury-gold mx-auto mb-6".to_string(),
                        }
                        h2 { class: "text-4xl font-luxury font-bold text-luxury-gold mb-6",
                            "申請 VVIP 會員資格"
                        }
                        p { class: "text-xl text-luxury-platinum/80 mb-8 max-w-3xl mx-auto",
                            "VVIP 會員採邀請制，需經過嚴格審核。"
                            br {}
                            "如您符合資格，我們的專屬顧問將與您聯繫。"
                        }
                        div { class: "flex flex-col sm:flex-row gap-4 justify-center",
                            a {
                                id: "vvip-join-cta",
                                href: "{join_href}",
                                class: "luxury-button px-8 py-4 text-lg",
                                "申請邀請函"
                            }
                            a {
                                id: "vvip-advisor-cta",
                                href: "{join_href}",
                                class: "luxury-button-outline px-8 py-4 text-lg",
                                "聯繫專屬顧問"
                            }
                        }
                        div { class: "mt-8 text-luxury-platinum/60 text-sm",
                            p { "* VVIP 會員需滿足特定財富與社會地位要求" }
                            p { "* 年費 NT$ 500,000 起，享受全年無限制專屬服務" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VvipHero() -> Element {
    rsx! {
        section { class: "relative py-20 overflow-hidden",
            div { class: "absolute inset-0 luxury-gradient opacity-20" }
            div { class: "relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center",
                div { class: "hs-enter", style: "--hs-from: 30px",
                    div { class: "inline-flex items-center justify-center w-16 h-16 bg-luxury-gold rounded-full mb-6",
                        Icon {
                            name: IconName::Crown,
                            class: "h-8 w-8 text-luxury-midnight-black".to_string(),
                        }
                    }
                    h1 {
                        id: "vvip-heading",
                        class: "text-5xl md:text-7xl font-luxury font-bold text-luxury-gold mb-6",
                        "VVIP 專區"
                    }
                    p { class: "text-xl md:text-2xl text-luxury-platinum/80 max-w-4xl mx-auto leading-relaxed",
                        "專為最頂級會員打造的獨家體驗空間"
                        br {}
                        "享受前所未有的奢華與尊榮"
                    }
                }
            }
        }
    }
}

#[component]
fn VvipStatsRow(stats: VvipStats) -> Element {
    rsx! {
        section { class: "py-16 bg-luxury-midnight-black/50",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "grid grid-cols-2 md:grid-cols-3 gap-8 hs-enter", style: "--hs-from: 30px",
                    if let Some(members) = stats.member_count {
                        div { class: "text-center",
                            div { class: "text-4xl font-bold text-luxury-gold mb-2", "{members}" }
                            div { class: "text-luxury-platinum/80", "VVIP 會員" }
                        }
                    }
                    div { class: "text-center",
                        div { class: "text-4xl font-bold text-luxury-gold mb-2", "{stats.event_count}" }
                        div { class: "text-luxury-platinum/80", "獨家活動" }
                    }
                    div { class: "text-center",
                        div { class: "text-4xl font-bold text-luxury-gold mb-2", "{stats.venue_count}" }
                        div { class: "text-luxury-platinum/80", "專屬場地" }
                    }
                }
            }
        }
    }
}

#[component]
fn VvipPerks() -> Element {
    rsx! {
        section { class: "py-20",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div {
                    class: "text-center mb-16 hs-enter",
                    style: "--hs-from: 30px",
                    h2 { class: "text-4xl md:text-5xl font-luxury font-bold text-luxury-gold mb-6",
                        "VVIP 專屬特權"
                    }
                    p { class: "text-xl text-luxury-platinum/80 max-w-3xl mx-auto",
                        "超越一般會員的頂級服務體驗"
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8",
                    for (index, (perk, icon)) in PERKS.iter().zip(PERK_ICONS).enumerate() {
                        div {
                            class: "luxury-glass p-8 rounded-2xl text-center hover:bg-luxury-gold/5 transition-all duration-300 group hs-enter",
                            style: "--hs-from: 30px; animation-delay: {index as f32 * 0.1}s",
                            Icon {
                                name: icon,
                                class: "h-12 w-12 text-luxury-gold mx-auto mb-6 group-hover:scale-110 transition-transform duration-300".to_string(),
                            }
                            h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                "{perk.title}"
                            }
                            p { class: "text-luxury-platinum/80 leading-relaxed",
                                "{perk.description}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn VvipCta() -> Element {
    rsx! {
        section { class: "py-20",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div {
                    class: "luxury-glass p-12 rounded-2xl text-center hs-enter",
                    style: "--hs-from: 30px",
                    Icon {
                        name: IconName::Crown,
                        class: "h-16 w-16 text-luxury-gold mx-auto mb-6".to_string(),
                    }
                    h2 { class: "text-4xl font-luxury font-bold text-luxury-gold mb-6",
                        "申請 VVIP 會員資格"
                    }
                    p { class: "text-xl text-luxury-platinum/80 mb-8 max-w-3xl mx-auto",
                        "VVIP 會員採邀請制，需經過嚴格審核。"
                        br {}
                        "如您符合資格，我們的專屬顧問將與您聯繫。"
                    }
                    div { class: "flex flex-col sm:flex-row gap-4 justify-center",
                        button { class: "luxury-button px-8 py-4 text-lg", "申請邀請函" }
                        button { class: "luxury-button-outline px-8 py-4 text-lg", "聯繫專屬顧問" }
                    }
                    div { class: "mt-8 text-luxury-platinum/60 text-sm",
                        p { "* VVIP 會員需滿足特定財富與社會地位要求" }
                        p { "* 年費 NT$ 500,000 起，享受全年無限制專屬服務" }
                    }
                }
            }
        }
    }
}

#[component]
fn VvipEventCard(event: VvipEvent, index: u32) -> Element {
    let image = event_image(&event);
    let when = format!(
        "{} {}",
        format_event_date(&event.date_time),
        format_event_time(&event.date_time)
    );
    let attendees = format!("{}/{} 人", event.current_attendees, event.capacity);
    let price = format_vvip_price(&event);
    let detail_href = format!("/events/{}", event.id);
    let delay = index as f32 * 0.1;
    let tags: Vec<String> = event.tags.iter().take(4).cloned().collect();
    let level = event.exclusivity_level.clone().unwrap_or_default();
    rsx! {
        div {
            id: "vvip-event-{event.id}",
            class: "luxury-glass rounded-2xl overflow-hidden hover:bg-luxury-gold/5 transition-all duration-300 group hs-enter",
            style: "--hs-from: 30px; animation-delay: {delay}s",
            div { class: "relative",
                img {
                    src: "{image}",
                    alt: "{event.name}",
                    class: "w-full h-48 object-cover group-hover:scale-105 transition-transform duration-300",
                }
                if !level.is_empty() {
                    div { class: "absolute top-4 left-4",
                        span { class: "px-3 py-1 bg-luxury-gold text-luxury-midnight-black text-xs font-medium rounded-full",
                            "{level}"
                        }
                    }
                }
                div { class: "absolute top-4 right-4",
                    div { class: "w-10 h-10 bg-luxury-midnight-black/70 backdrop-blur-sm rounded-full flex items-center justify-center",
                        Icon {
                            name: IconName::Lock,
                            class: "h-5 w-5 text-luxury-gold".to_string(),
                        }
                    }
                }
            }
            div { class: "p-6",
                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-3 group-hover:text-luxury-gold/90 transition-colors",
                    "{event.name}"
                }
                p { class: "text-luxury-platinum/80 text-sm mb-4 line-clamp-2 leading-relaxed",
                    "{event.description}"
                }
                div { class: "space-y-2 mb-4",
                    div { class: "flex items-center text-luxury-platinum/70 text-sm",
                        Icon {
                            name: IconName::Calendar,
                            class: "h-4 w-4 mr-2 text-luxury-gold flex-shrink-0".to_string(),
                        }
                        span { "{when}" }
                    }
                    div { class: "flex items-center text-luxury-platinum/70 text-sm",
                        Icon {
                            name: IconName::Users,
                            class: "h-4 w-4 mr-2 text-luxury-gold flex-shrink-0".to_string(),
                        }
                        span { "{attendees}" }
                    }
                }
                if !tags.is_empty() {
                    div { class: "flex flex-wrap gap-2 mb-6",
                        for tag in tags {
                            span { class: "px-3 py-1 bg-luxury-gold/20 text-luxury-gold text-xs rounded-full font-medium",
                                "{tag}"
                            }
                        }
                    }
                }
                div { class: "flex items-center justify-between pt-4 border-t border-luxury-gold/20",
                    div { class: "text-2xl font-bold text-luxury-gold", "{price}" }
                    a {
                        href: "{detail_href}",
                        class: "px-6 py-2 bg-luxury-gold text-luxury-midnight-black rounded-lg hover:bg-luxury-gold/90 transition-all duration-300 text-sm font-semibold shadow-lg hover:shadow-luxury-gold/25",
                        "查看詳情"
                    }
                }
            }
        }
    }
}
