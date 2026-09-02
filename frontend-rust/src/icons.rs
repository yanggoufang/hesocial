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
    Menu,
    X,
    Crown,
    User,
    LogOut,
    Settings,
    Shield,
    Activity,
    TrendingUp,
    Mail,
    Phone,
    Briefcase,
    Award,
    ArrowRight,
    Clock,
    Heart,
    Share2,
    Shirt,
    Check,
    AlertCircle,
    Lock,
    DollarSign,
    Plus,
    Eye,
    EyeOff,
}

impl IconName {
    pub fn from_nav_icon(name: &str) -> Option<Self> {
        match name {
            "crown" => Some(Self::Crown),
            "user" => Some(Self::User),
            "log-out" => Some(Self::LogOut),
            "calendar" => Some(Self::Calendar),
            "settings" => Some(Self::Settings),
            "shield" => Some(Self::Shield),
            "activity" => Some(Self::Activity),
            "trending-up" => Some(Self::TrendingUp),
            "menu" => Some(Self::Menu),
            "x" => Some(Self::X),
            _ => None,
        }
    }

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
            Self::Menu => "menu",
            Self::X => "x",
            Self::Crown => "crown",
            Self::User => "user",
            Self::LogOut => "log-out",
            Self::Settings => "settings",
            Self::Shield => "shield",
            Self::Activity => "activity",
            Self::TrendingUp => "trending-up",
            Self::Mail => "mail",
            Self::Phone => "phone",
            Self::Briefcase => "briefcase",
            Self::Award => "award",
            Self::ArrowRight => "arrow-right",
            Self::Clock => "clock",
            Self::Heart => "heart",
            Self::Share2 => "share-2",
            Self::Shirt => "shirt",
            Self::Check => "check",
            Self::AlertCircle => "alert-circle",
            Self::Lock => "lock",
            Self::DollarSign => "dollar-sign",
            Self::Plus => "plus",
            Self::Eye => "eye",
            Self::EyeOff => "eye-off",
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
                IconName::Menu => rsx! {
                    line { x1: "4", x2: "20", y1: "12", y2: "12" }
                    line { x1: "4", x2: "20", y1: "6", y2: "6" }
                    line { x1: "4", x2: "20", y1: "18", y2: "18" }
                },
                IconName::X => rsx! {
                    path { d: "M18 6 6 18" }
                    path { d: "m6 6 12 12" }
                },
                IconName::Crown => rsx! {
                    path { d: "M11.562 3.266a.5.5 0 0 1 .876 0L15.39 8.87a1 1 0 0 0 1.516.294L21.183 5.5a.5.5 0 0 1 .798.519l-2.834 10.246a1 1 0 0 1-.956.734H5.81a1 1 0 0 1-.957-.734L2.02 6.02a.5.5 0 0 1 .798-.519l4.276 3.664a1 1 0 0 0 1.516-.294z" }
                    path { d: "M5 21h14" }
                },
                IconName::User => rsx! {
                    path { d: "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" }
                    circle { cx: "12", cy: "7", r: "4" }
                },
                IconName::LogOut => rsx! {
                    path { d: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" }
                    polyline { points: "16 17 21 12 16 7" }
                    line { x1: "21", x2: "9", y1: "12", y2: "12" }
                },
                IconName::Settings => rsx! {
                    path { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }
                    circle { cx: "12", cy: "12", r: "3" }
                },
                IconName::Shield => rsx! {
                    path { d: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" }
                },
                IconName::Activity => rsx! {
                    path { d: "M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2" }
                },
                IconName::TrendingUp => rsx! {
                    polyline { points: "22 7 13.5 15.5 8.5 10.5 2 17" }
                    polyline { points: "16 7 22 7 22 13" }
                },
                IconName::Mail => rsx! {
                    rect { width: "20", height: "16", x: "2", y: "4", rx: "2" }
                    path { d: "m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" }
                },
                IconName::Phone => rsx! {
                    path { d: "M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z" }
                },
                IconName::Briefcase => rsx! {
                    path { d: "M16 20V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" }
                    rect { width: "20", height: "14", x: "2", y: "6", rx: "2" }
                },
                IconName::Award => rsx! {
                    path { d: "m15.477 12.89 1.515 8.526a.5.5 0 0 1-.81.47l-3.58-2.687a1 1 0 0 0-1.197 0l-3.586 2.686a.5.5 0 0 1-.81-.469l1.514-8.526" }
                    circle { cx: "12", cy: "8", r: "6" }
                },
                IconName::ArrowRight => rsx! {
                    path { d: "M5 12h14" }
                    path { d: "m12 5 7 7-7 7" }
                },
                IconName::Clock => rsx! {
                    circle { cx: "12", cy: "12", r: "10" }
                    polyline { points: "12 6 12 12 16 14" }
                },
                IconName::Heart => rsx! {
                    path { d: "M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z" }
                },
                IconName::Share2 => rsx! {
                    circle { cx: "18", cy: "5", r: "3" }
                    circle { cx: "6", cy: "12", r: "3" }
                    circle { cx: "18", cy: "19", r: "3" }
                    line { x1: "8.59", x2: "15.42", y1: "13.51", y2: "17.49" }
                    line { x1: "15.41", x2: "8.59", y1: "6.51", y2: "10.49" }
                },
                IconName::Shirt => rsx! {
                    path { d: "M20.38 3.46 16 2a4 4 0 0 1-8 0L3.62 3.46a2 2 0 0 0-1.34 2.23l.58 3.47a1 1 0 0 0 .99.84H6v10c0 1.1.9 2 2 2h8a2 2 0 0 0 2-2V10h2.15a1 1 0 0 0 .99-.84l.58-3.47a2 2 0 0 0-1.34-2.23z" }
                },
                IconName::Check => rsx! {
                    path { d: "M20 6 9 17l-5-5" }
                },
                IconName::AlertCircle => rsx! {
                    circle { cx: "12", cy: "12", r: "10" }
                    line { x1: "12", x2: "12", y1: "8", y2: "12" }
                    line { x1: "12", x2: "12.01", y1: "16", y2: "16" }
                },
                IconName::Lock => rsx! {
                    rect { width: "18", height: "11", x: "3", y: "11", rx: "2", ry: "2" }
                    path { d: "M7 11V7a5 5 0 0 1 10 0v4" }
                },
                IconName::DollarSign => rsx! {
                    line { x1: "12", x2: "12", y1: "2", y2: "22" }
                    path { d: "M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" }
                },
                IconName::Plus => rsx! {
                    path { d: "M5 12h14" }
                    path { d: "M12 5v14" }
                },
                IconName::Eye => rsx! {
                    path { d: "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" }
                    circle { cx: "12", cy: "12", r: "3" }
                },
                IconName::EyeOff => rsx! {
                    path { d: "M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" }
                    path { d: "M14.084 14.158a3 3 0 0 1-4.242-4.242" }
                    path { d: "M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" }
                    path { d: "m2 2 20 20" }
                },
            }
        }
    }
}
