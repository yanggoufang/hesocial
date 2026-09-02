use std::cell::Cell;
use std::rc::Rc;

use crate::events::{
    CATEGORIES, EXCLUSIVITY_LEVELS, Event, EventFilters, PAGE_LIMIT, Pagination, exclusivity_color,
    exclusivity_label, fetch_events, first_image, format_event_datetime, format_price,
    page_after_filter_change, page_in_range, shows_diamond, star_count,
};
use crate::icons::{Icon, IconName};
use dioxus::prelude::*;

#[component]
pub fn Events() -> Element {
    let mut search = use_signal(String::new);
    let mut category = use_signal(|| "all".to_string());
    let mut level = use_signal(|| "all".to_string());
    let mut events = use_signal(Vec::<Event>::new);
    let mut loading = use_signal(|| true);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));
    let mut pagination = use_signal(|| Pagination {
        page: 1,
        limit: PAGE_LIMIT,
        total: 0,
        total_pages: 1,
    });

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let search_val = search();
            let category_val = category();
            let level_val = level();
            let page = page_after_filter_change(1);
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let view = fetch_events(&EventFilters {
                    page,
                    limit: PAGE_LIMIT,
                    search: search_val,
                    category: category_val,
                    exclusivity_level: level_val,
                })
                .await;
                if fetch_gen.get() != request_id {
                    return;
                }
                events.set(view.events);
                pagination.set(view.pagination);
                loading.set(false);
            });
        }
    });

    rsx! {
        EventsScreen {
            search: search(),
            category: category(),
            level: level(),
            events: events(),
            loading: loading(),
            pagination: pagination(),
            on_search: move |value: String| search.set(value),
            on_category: move |value: String| category.set(value),
            on_level: move |value: String| level.set(value),
            on_page: move |new_page: u32| {
                if loading() || !page_in_range(new_page, pagination().total_pages) {
                    return;
                }
                let request_id = fetch_gen.get() + 1;
                fetch_gen.set(request_id);
                loading.set(true);
                let search_val = search();
                let category_val = category();
                let level_val = level();
                let fetch_gen = fetch_gen.clone();
                spawn(async move {
                    let view = fetch_events(&EventFilters {
                        page: new_page,
                        limit: PAGE_LIMIT,
                        search: search_val,
                        category: category_val,
                        exclusivity_level: level_val,
                    })
                    .await;
                    if fetch_gen.get() != request_id {
                        return;
                    }
                    events.set(view.events);
                    pagination.set(view.pagination);
                    loading.set(false);
                });
            },
        }
    }
}

#[component]
pub fn EventDetail(id: String) -> Element {
    rsx! {
        main {
            id: "event-detail-stub",
            class: "min-h-screen bg-luxury-midnight-black text-luxury-platinum p-8",
            h1 { "活動詳情" }
            p { id: "event-detail-id", "{id}" }
        }
    }
}

#[component]
pub fn EventsScreen(
    search: String,
    category: String,
    level: String,
    events: Vec<Event>,
    loading: bool,
    pagination: Pagination,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_category: EventHandler<String>,
    #[props(default)] on_level: EventHandler<String>,
    #[props(default)] on_page: EventHandler<u32>,
) -> Element {
    let prev_disabled = pagination.page <= 1 || loading;
    let next_disabled = pagination.page >= pagination.total_pages || loading;
    rsx! {
        div { class: "min-h-screen bg-luxury-midnight-black py-8",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div {
                    class: "text-center mb-12 hs-enter",
                    style: "--hs-from: 30px",
                    h1 {
                        id: "events-heading",
                        class: "text-4xl md:text-6xl font-luxury font-bold text-luxury-gold mb-6",
                        "精選活動"
                    }
                    p { class: "text-xl text-luxury-platinum/80 max-w-3xl mx-auto",
                        "探索為您精心策劃的頂級社交活動，與志同道合的菁英建立深度連結"
                    }
                }

                div { class: "luxury-glass p-6 rounded-2xl mb-8 hs-enter-filters",
                    div { class: "flex flex-col lg:flex-row gap-4 items-center",
                        div { class: "flex-1 relative w-full",
                            Icon {
                                name: IconName::Search,
                                class: "absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-luxury-gold".to_string(),
                            }
                            input {
                                r#type: "text",
                                id: "events-search",
                                value: "{search}",
                                placeholder: "搜尋活動名稱或關鍵字...",
                                class: "w-full bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-10 py-3 text-luxury-platinum placeholder-luxury-platinum/50 focus:outline-none focus:border-luxury-gold",
                                oninput: move |evt| on_search.call(evt.value()),
                            }
                        }
                        div { class: "flex flex-col sm:flex-row gap-4 w-full lg:w-auto",
                            select {
                                id: "events-category",
                                value: "{category}",
                                class: "bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold",
                                onchange: move |evt| on_category.call(evt.value()),
                                for opt in CATEGORIES {
                                    option {
                                        value: "{opt.id}",
                                        selected: category == opt.id,
                                        "{opt.name}"
                                    }
                                }
                            }
                            select {
                                id: "events-level",
                                value: "{level}",
                                class: "bg-luxury-midnight-black/50 border border-luxury-gold/30 rounded-lg px-4 py-3 text-luxury-platinum focus:outline-none focus:border-luxury-gold",
                                onchange: move |evt| on_level.call(evt.value()),
                                for opt in EXCLUSIVITY_LEVELS {
                                    option {
                                        value: "{opt.id}",
                                        selected: level == opt.id,
                                        "{opt.name}"
                                    }
                                }
                            }
                        }
                    }
                }

                if loading {
                    div {
                        id: "events-loading",
                        class: "text-center text-luxury-gold text-2xl",
                        "載入中..."
                    }
                } else if !events.is_empty() {
                    div { id: "events-grid", class: "grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-8",
                        for (index, event) in events.iter().cloned().enumerate() {
                            EventCard { event, index: index as u32 }
                        }
                    }
                    div { class: "flex items-center justify-center mt-12 space-x-4 text-luxury-platinum hs-enter-page",
                        button {
                            id: "events-prev",
                            r#type: "button",
                            class: "luxury-button-outline p-2 rounded-full disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: prev_disabled,
                            onclick: move |_| on_page.call(pagination.page.saturating_sub(1)),
                            Icon {
                                name: IconName::ChevronLeft,
                                class: "h-5 w-5".to_string(),
                            }
                        }
                        span { id: "events-page-label", class: "font-semibold",
                            "第 {pagination.page} / {pagination.total_pages} 頁"
                        }
                        button {
                            id: "events-next",
                            r#type: "button",
                            class: "luxury-button-outline p-2 rounded-full disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: next_disabled,
                            onclick: move |_| on_page.call(pagination.page + 1),
                            Icon {
                                name: IconName::ChevronRight,
                                class: "h-5 w-5".to_string(),
                            }
                        }
                    }
                } else {
                    div {
                        id: "events-empty",
                        class: "text-center text-luxury-platinum/80 text-xl py-20",
                        p { "找不到符合條件的活動。" }
                        p { "請嘗試調整您的篩選條件，或稍後再試。" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn EventCard(event: Event, index: u32) -> Element {
    let image = first_image(event.images.as_deref());
    let level = event.exclusivity_level.as_deref();
    let badge_class = exclusivity_color(level);
    let badge_label = exclusivity_label(level);
    let venue_name = event
        .venue
        .as_ref()
        .map(|venue| venue.name.as_str())
        .unwrap_or("");
    let price = format_price(event.pricing.vvip, event.pricing.vip);
    let when = format_event_datetime(&event.date_time);
    let delay = index as f32 * 0.1;
    let stars = star_count(level);
    let diamond = shows_diamond(level);
    let detail_href = format!("/events/{}", event.id);
    let card_id = format!("event-card-{}", event.id);
    let badge_id = format!("event-badge-{}", event.id);
    let attendees = format!("{}/{} 人", event.current_attendees, event.capacity);

    rsx! {
        div {
            id: "{card_id}",
            class: "relative luxury-glass rounded-2xl hover:bg-luxury-gold/5 transition-all duration-300 group hs-enter",
            style: "--hs-from: 50px; animation-delay: {delay}s",
            div { class: "relative",
                div { class: "rounded-t-2xl overflow-hidden",
                    img {
                        src: "{image}",
                        alt: "{event.name}",
                        class: "w-full h-48 object-cover group-hover:scale-105 transition-transform duration-300",
                    }
                }
                div { class: "absolute top-4 left-4",
                    span {
                        id: "{badge_id}",
                        class: "px-3 py-1 rounded-full text-xs font-medium border {badge_class}",
                        "{badge_label}"
                    }
                }
                div { class: "absolute top-4 right-4 flex items-center space-x-1",
                    if diamond {
                        Icon {
                            name: IconName::Diamond,
                            class: "h-4 w-4 text-white fill-current mr-1".to_string(),
                            filled: true,
                        }
                    }
                    for _ in 0..stars {
                        Icon {
                            name: IconName::Star,
                            class: "h-4 w-4 text-luxury-gold fill-current".to_string(),
                            filled: true,
                        }
                    }
                }
            }
            div { class: "p-6",
                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-2 group-hover:text-luxury-gold/90 transition-colors",
                    "{event.name}"
                }
                p { class: "text-luxury-platinum/80 text-sm mb-4 line-clamp-2", "{event.description}" }
                div { class: "space-y-3 mb-6",
                    div { class: "flex items-center text-luxury-platinum/70 text-sm",
                        Icon {
                            name: IconName::Calendar,
                            class: "h-4 w-4 mr-2 text-luxury-gold".to_string(),
                        }
                        span { "{when}" }
                    }
                    div { class: "flex items-center text-luxury-platinum/70 text-sm",
                        Icon {
                            name: IconName::MapPin,
                            class: "h-4 w-4 mr-2 text-luxury-gold".to_string(),
                        }
                        span { "{venue_name}" }
                    }
                    div { class: "flex items-center text-luxury-platinum/70 text-sm",
                        Icon {
                            name: IconName::Users,
                            class: "h-4 w-4 mr-2 text-luxury-gold".to_string(),
                        }
                        span { "{attendees}" }
                    }
                }
                div { class: "flex items-center justify-between",
                    div { class: "text-luxury-gold font-semibold", "{price}" }
                    a {
                        href: "{detail_href}",
                        class: "px-4 py-2 bg-luxury-gold/20 text-luxury-gold rounded-lg hover:bg-luxury-gold hover:text-luxury-midnight-black transition-all duration-300 text-sm font-medium",
                        "查看詳情"
                    }
                }
            }
        }
    }
}