use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IconName {
    Calendar,
    MapPin,
    Users,
    Star,
    Search,
    ChevronLeft,
    ChevronRight,
    Diamond,
}

impl IconName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Calendar => "calendar",
            Self::MapPin => "map-pin",
            Self::Users => "users",
            Self::Star => "star",
            Self::Search => "search",
            Self::ChevronLeft => "chevron-left",
            Self::ChevronRight => "chevron-right",
            Self::Diamond => "diamond",
        }
    }
}

#[component]
pub fn Icon(
    name: IconName,
    #[props(default)] class: String,
    #[props(default)] filled: bool,
) -> Element {
    let fill = if filled { "currentColor" } else { "none" };
    let icon = name.as_str();
    rsx! {
        svg {
            class: "{class}",
            fill,
            view_box: "0 0 24 24",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "data-icon": icon,
            match name {
                IconName::Calendar => rsx! {
                    path { d: "M8 2v4" }
                    path { d: "M16 2v4" }
                    rect { width: "18", height: "18", x: "3", y: "4", rx: "2" }
                    path { d: "M3 10h18" }
                },
                IconName::MapPin => rsx! {
                    path { d: "M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0" }
                    circle { cx: "12", cy: "10", r: "3" }
                },
                IconName::Users => rsx! {
                    path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                    circle { cx: "9", cy: "7", r: "4" }
                    path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
                    path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
                },
                IconName::Star => rsx! {
                    path { d: "M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z" }
                },
                IconName::Search => rsx! {
                    circle { cx: "11", cy: "11", r: "8" }
                    path { d: "m21 21-4.3-4.3" }
                },
                IconName::ChevronLeft => rsx! {
                    path { d: "m15 18-6-6 6-6" }
                },
                IconName::ChevronRight => rsx! {
                    path { d: "m9 18 6-6-6-6" }
                },
                IconName::Diamond => rsx! {
                    path { d: "M2.7 10.3a2.41 2.41 0 0 0 0 3.41l7.59 7.59a2.41 2.41 0 0 0 3.41 0l7.59-7.59a2.41 2.41 0 0 0 0-3.41l-7.59-7.59a2.41 2.41 0 0 0-3.41 0Z" }
                },
            }
        }
    }
}