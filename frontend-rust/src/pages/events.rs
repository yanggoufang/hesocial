use std::cell::Cell;
use std::rc::Rc;

use crate::events::{
    CATEGORIES, EXCLUSIVITY_LEVELS, Event, EventDetail as EventDetailData, EventFilters,
    PAGE_LIMIT, Pagination, detail_exclusivity_color, dress_code_text, exclusivity_color,
    exclusivity_label, fetch_event_detail, fetch_events, first_image, format_event_date,
    format_event_datetime, format_event_time, format_price, gallery_image, is_full,
    occupancy_percent, page_after_filter_change, page_in_range, price_kind_label, shows_diamond,
    spots_remaining, star_count, venue_star_count, wrap_image_index,
};
use crate::icons::{Icon, IconName};
use crate::permissions::Session;
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
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let mut loading = use_signal(|| true);
    let mut event = use_signal(|| None::<EventDetailData>);
    let mut image_index = use_signal(|| 0usize);
    let mut event_id = use_signal(|| id.clone());
    if event_id.peek().as_str() != id {
        event_id.set(id.clone());
    }
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let id = event_id();
            let request_id = fetch_gen.get() + 1;
            fetch_gen.set(request_id);
            loading.set(true);
            event.set(None);
            image_index.set(0);
            let fetch_gen = fetch_gen.clone();
            spawn(async move {
                let fetched = fetch_event_detail(&id).await;
                if fetch_gen.get() != request_id {
                    return;
                }
                event.set(fetched);
                loading.set(false);
            });
        }
    });

    let snapshot = session().snapshot();
    rsx! {
        EventDetailScreen {
            loading: loading(),
            event: event(),
            image_index: image_index(),
            is_authenticated: snapshot.is_authenticated,
            on_prev_image: move |len: usize| {
                image_index.set(wrap_image_index(image_index() as i32 - 1, len));
            },
            on_next_image: move |len: usize| {
                image_index.set(wrap_image_index(image_index() as i32 + 1, len));
            },
            on_select_image: move |index: usize| image_index.set(index),
        }
    }
}

#[component]
pub fn EventDetailScreen(
    loading: bool,
    event: Option<EventDetailData>,
    image_index: usize,
    is_authenticated: bool,
    #[props(default)] on_prev_image: EventHandler<usize>,
    #[props(default)] on_next_image: EventHandler<usize>,
    #[props(default)] on_select_image: EventHandler<usize>,
) -> Element {
    if loading {
        return rsx! {
            div {
                id: "event-detail-loading",
                class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
                div { class: "luxury-glass p-8 rounded-2xl text-center",
                    div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                    p { class: "text-luxury-platinum", "載入活動資訊中..." }
                }
            }
        };
    }

    let Some(event) = event else {
        return rsx! {
            div {
                id: "event-detail-not-found",
                class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
                div { class: "luxury-glass p-8 rounded-2xl text-center",
                    Icon {
                        name: IconName::AlertCircle,
                        class: "h-12 w-12 text-red-400 mx-auto mb-4".to_string(),
                    }
                    h2 { class: "text-xl font-luxury text-luxury-gold mb-2", "活動未找到" }
                    p { class: "text-luxury-platinum/80 mb-4",
                        "抱歉，您要查看的活動不存在或已被移除。"
                    }
                    a {
                        id: "event-detail-back-to-list",
                        href: "/events",
                        class: "luxury-button",
                        "返回活動列表"
                    }
                }
            }
        };
    };

    let image_count = event.images.len();
    let hero_src = gallery_image(&event.images, image_index);
    let level = event.exclusivity_level.as_deref();
    let badge_class = detail_exclusivity_color(level);
    let badge_label = exclusivity_label(level);
    let date_label = format_event_date(&event.date_time);
    let time_label = format_event_time(&event.date_time);
    let deadline_label = format_event_date(&event.registration_deadline);
    let dress = dress_code_text(event.dress_code);
    let attendees = format!("{}/{} 人", event.current_attendees, event.capacity);
    let price = format_price(event.pricing.vvip, event.pricing.vip);
    let price_kind = price_kind_label(event.pricing.vvip, event.pricing.vip);
    let filled = occupancy_percent(event.current_attendees, event.capacity);
    let remaining = spots_remaining(event.current_attendees, event.capacity);
    let full = is_full(event.current_attendees, event.capacity);
    let stars = venue_star_count(event.venue_rating);
    let register_href = format!("/events/{}/register", event.id);
    let participants_href = format!("/events/{}/participants", event.id);
    let register_label = if full { "已額滿" } else { "立即報名" };

    rsx! {
        div { id: "event-detail", class: "min-h-screen bg-luxury-midnight-black",
            div { class: "relative h-[60vh] overflow-hidden",
                div { class: "relative h-full",
                    img {
                        src: "{hero_src}",
                        alt: "{event.name}",
                        class: "w-full h-full object-cover",
                    }
                    div { class: "absolute inset-0 bg-luxury-midnight-black/50" }
                    if image_count > 1 {
                        button {
                            id: "event-detail-prev-image",
                            r#type: "button",
                            class: "absolute left-4 top-1/2 transform -translate-y-1/2 w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors",
                            onclick: move |_| on_prev_image.call(image_count),
                            Icon { name: IconName::ChevronLeft, class: "h-6 w-6".to_string() }
                        }
                        button {
                            id: "event-detail-next-image",
                            r#type: "button",
                            class: "absolute right-4 top-1/2 transform -translate-y-1/2 w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors",
                            onclick: move |_| on_next_image.call(image_count),
                            Icon { name: IconName::ChevronRight, class: "h-6 w-6".to_string() }
                        }
                        div { class: "absolute bottom-4 left-1/2 transform -translate-x-1/2 flex space-x-2",
                            for index in 0..image_count {
                                {
                                    let dot_class = if index == image_index {
                                        "w-3 h-3 rounded-full transition-colors bg-luxury-gold"
                                    } else {
                                        "w-3 h-3 rounded-full transition-colors bg-white/30"
                                    };
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            id: "event-image-{index}",
                                            class: "{dot_class}",
                                            onclick: move |_| on_select_image.call(index),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "absolute top-8 left-8",
                    a {
                        id: "event-detail-back",
                        href: "/events",
                        class: "inline-flex items-center px-4 py-2 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-lg text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors",
                        Icon { name: IconName::ChevronLeft, class: "h-5 w-5 mr-1".to_string() }
                        "返回"
                    }
                }
                div { class: "absolute top-8 right-8 flex space-x-2",
                    button {
                        r#type: "button",
                        class: "w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors",
                        Icon { name: IconName::Heart, class: "h-5 w-5".to_string() }
                    }
                    button {
                        r#type: "button",
                        class: "w-12 h-12 bg-luxury-midnight-black/50 backdrop-blur-sm rounded-full flex items-center justify-center text-luxury-gold hover:bg-luxury-midnight-black/70 transition-colors",
                        Icon { name: IconName::Share2, class: "h-5 w-5".to_string() }
                    }
                }
            }

            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                    div { class: "lg:col-span-2",
                        div {
                            class: "space-y-8 hs-enter",
                            style: "--hs-from: 30px",
                            div {
                                div { class: "flex items-start justify-between mb-4",
                                    div {
                                        span { class: "inline-block px-3 py-1 rounded-full text-xs font-medium border {badge_class} mb-2",
                                            "{badge_label}"
                                        }
                                        h1 {
                                            id: "event-detail-heading",
                                            class: "text-4xl font-luxury font-bold text-luxury-gold mb-2",
                                            "{event.name}"
                                        }
                                        p { class: "text-luxury-platinum/80 text-lg",
                                            "主辦：{event.organizer}"
                                        }
                                    }
                                }
                                div { class: "flex flex-wrap gap-2",
                                    for tag in event.tags.iter() {
                                        span { class: "px-3 py-1 bg-luxury-gold/20 text-luxury-gold text-sm rounded-full",
                                            "{tag}"
                                        }
                                    }
                                }
                            }
                            div { class: "luxury-glass p-6 rounded-2xl",
                                h2 { class: "text-2xl font-luxury font-semibold text-luxury-gold mb-4",
                                    "活動詳情"
                                }
                                div { class: "prose prose-invert max-w-none",
                                    p { class: "text-luxury-platinum/80 leading-relaxed whitespace-pre-line",
                                        "{event.description}"
                                    }
                                }
                            }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                                div { class: "luxury-glass p-6 rounded-2xl",
                                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                        "時間地點"
                                    }
                                    div { class: "space-y-3",
                                        div { class: "flex items-start",
                                            Icon { name: IconName::Calendar, class: "h-5 w-5 text-luxury-gold mr-3 mt-0.5".to_string() }
                                            div {
                                                p { class: "text-luxury-platinum font-medium", "{date_label}" }
                                                p { class: "text-luxury-platinum/80 text-sm", "{time_label}" }
                                            }
                                        }
                                        div { class: "flex items-start",
                                            Icon { name: IconName::MapPin, class: "h-5 w-5 text-luxury-gold mr-3 mt-0.5".to_string() }
                                            div {
                                                p { class: "text-luxury-platinum font-medium", "{event.venue_name}" }
                                                p { class: "text-luxury-platinum/80 text-sm", "{event.venue_address}" }
                                            }
                                        }
                                        div { class: "flex items-center",
                                            Icon { name: IconName::Clock, class: "h-5 w-5 text-luxury-gold mr-3".to_string() }
                                            div {
                                                p { class: "text-luxury-platinum font-medium", "報名截止" }
                                                p { class: "text-luxury-platinum/80 text-sm", "{deadline_label}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "luxury-glass p-6 rounded-2xl",
                                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                        "活動規格"
                                    }
                                    div { class: "space-y-3",
                                        div { class: "flex items-center",
                                            Icon { name: IconName::Users, class: "h-5 w-5 text-luxury-gold mr-3".to_string() }
                                            div {
                                                p { class: "text-luxury-platinum font-medium", "參與人數" }
                                                p { class: "text-luxury-platinum/80 text-sm", "{attendees}" }
                                            }
                                        }
                                        div { class: "flex items-center",
                                            Icon { name: IconName::Shirt, class: "h-5 w-5 text-luxury-gold mr-3".to_string() }
                                            div {
                                                p { class: "text-luxury-platinum font-medium", "服裝規範" }
                                                p { class: "text-luxury-platinum/80 text-sm", "{dress}" }
                                            }
                                        }
                                        div { class: "flex items-center",
                                            Icon { name: IconName::Crown, class: "h-5 w-5 text-luxury-gold mr-3".to_string() }
                                            div {
                                                p { class: "text-luxury-platinum font-medium", "獨家等級" }
                                                p { class: "text-luxury-platinum/80 text-sm", "{badge_label}" }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                                div { class: "luxury-glass p-6 rounded-2xl",
                                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                        "專屬服務"
                                    }
                                    ul { class: "space-y-2",
                                        for amenity in event.amenities.iter() {
                                            li { class: "flex items-center text-luxury-platinum/80",
                                                Icon { name: IconName::Check, class: "h-4 w-4 text-luxury-gold mr-2".to_string() }
                                                "{amenity}"
                                            }
                                        }
                                    }
                                }
                                div { class: "luxury-glass p-6 rounded-2xl",
                                    h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                        "隱私保障"
                                    }
                                    ul { class: "space-y-2",
                                        for guarantee in event.privacy_guarantees.iter() {
                                            li { class: "flex items-center text-luxury-platinum/80",
                                                Icon { name: IconName::Shield, class: "h-4 w-4 text-luxury-gold mr-2".to_string() }
                                                "{guarantee}"
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "luxury-glass p-6 rounded-2xl",
                                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                    "參與要求"
                                }
                                div { class: "space-y-2",
                                    for req in event.requirements.iter() {
                                        div { class: "flex items-center text-luxury-platinum/80",
                                            Icon { name: IconName::AlertCircle, class: "h-4 w-4 text-luxury-gold mr-2".to_string() }
                                            "{req}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "lg:col-span-1",
                        div {
                            class: "sticky top-24 hs-enter",
                            style: "--hs-from: 30px",
                            div { class: "luxury-glass p-6 rounded-2xl",
                                div { class: "text-center mb-6",
                                    div { class: "text-3xl font-bold text-luxury-gold mb-2", "{price}" }
                                    p { class: "text-luxury-platinum/80 text-sm", "{price_kind}" }
                                }
                                div { class: "space-y-4 mb-6",
                                    div { class: "flex justify-between items-center",
                                        span { class: "text-luxury-platinum/80", "報名人數" }
                                        span { class: "text-luxury-gold font-medium",
                                            "{event.current_attendees}/{event.capacity}"
                                        }
                                    }
                                    div { class: "w-full bg-luxury-midnight-black/50 rounded-full h-2",
                                        div {
                                            class: "bg-luxury-gold h-2 rounded-full transition-all duration-300",
                                            style: "width: {filled}%",
                                        }
                                    }
                                    div { class: "flex justify-between items-center text-sm",
                                        span { class: "text-luxury-platinum/80", "剩餘名額" }
                                        span { class: "text-luxury-gold", "{remaining} 個" }
                                    }
                                }
                                if is_authenticated {
                                    div { class: "space-y-3 mb-4",
                                        if full {
                                            button {
                                                id: "event-detail-register",
                                                r#type: "button",
                                                class: "w-full luxury-button py-3",
                                                disabled: true,
                                                "{register_label}"
                                            }
                                        } else {
                                            a {
                                                id: "event-detail-register",
                                                href: "{register_href}",
                                                class: "w-full luxury-button py-3 inline-block text-center",
                                                "{register_label}"
                                            }
                                        }
                                        a {
                                            id: "event-detail-participants",
                                            href: "{participants_href}",
                                            class: "w-full px-4 py-3 border border-luxury-gold/30 text-luxury-gold rounded-lg hover:bg-luxury-gold/10 transition-colors flex items-center justify-center gap-2",
                                            Icon { name: IconName::Users, class: "w-4 h-4".to_string() }
                                            "查看參與者"
                                        }
                                    }
                                } else {
                                    div { class: "space-y-2 mb-4",
                                        a {
                                            id: "event-detail-login",
                                            href: "/login",
                                            class: "w-full luxury-button py-3 inline-block text-center",
                                            "登入後報名"
                                        }
                                        p { class: "text-center text-luxury-platinum/60 text-xs",
                                            "需要登入會員帳號才能報名活動"
                                        }
                                    }
                                }
                                div { class: "text-center",
                                    p { class: "text-luxury-platinum/60 text-xs", "* 所有報名需經過審核" }
                                    p { class: "text-luxury-platinum/60 text-xs", "* 48小時內無條件退款" }
                                }
                            }
                            div { class: "luxury-glass p-6 rounded-2xl mt-6",
                                h3 { class: "text-xl font-luxury font-semibold text-luxury-gold mb-4",
                                    "場地資訊"
                                }
                                div { class: "space-y-3",
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-luxury-platinum font-medium", "{event.venue_name}" }
                                        div { class: "flex items-center",
                                            for _ in 0..stars {
                                                Icon {
                                                    name: IconName::Star,
                                                    class: "h-4 w-4 text-luxury-gold fill-current".to_string(),
                                                    filled: true,
                                                }
                                            }
                                        }
                                    }
                                    p { class: "text-luxury-platinum/80 text-sm", "{event.venue_address}" }
                                    div { class: "space-y-1",
                                        for amenity in event.venue_amenities.iter() {
                                            div { class: "flex items-center text-luxury-platinum/80 text-sm",
                                                Icon { name: IconName::Check, class: "h-3 w-3 text-luxury-gold mr-2".to_string() }
                                                "{amenity}"
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
