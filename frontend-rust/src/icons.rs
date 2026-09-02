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
    Ticket,
    Edit,
    Filter,
    RefreshCw,
    CreditCard,
    ExternalLink,
    ArrowLeft,
    MessageCircle,
    Info,
    Save,
    Building,
    AlertTriangle,
    Zap,
    Database,
    Cloud,
    BarChart3,
    CheckCircle,
    XCircle,
    Server,
    HardDrive,
    Trash2,
    UserCheck,
    Download,
    Target,
    PieChart,
    TrendingDown,
    Image,
    FileText,
    Upload,
    LayoutGrid,
    Map,
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
            Self::Ticket => "ticket",
            Self::Edit => "edit",
            Self::Filter => "filter",
            Self::RefreshCw => "refresh-cw",
            Self::CreditCard => "credit-card",
            Self::ExternalLink => "external-link",
            Self::ArrowLeft => "arrow-left",
            Self::MessageCircle => "message-circle",
            Self::Info => "info",
            Self::Save => "save",
            Self::Building => "building",
            Self::AlertTriangle => "alert-triangle",
            Self::Zap => "zap",
            Self::Database => "database",
            Self::Cloud => "cloud",
            Self::BarChart3 => "bar-chart-3",
            Self::CheckCircle => "check-circle",
            Self::XCircle => "x-circle",
            Self::Server => "server",
            Self::HardDrive => "hard-drive",
            Self::Trash2 => "trash-2",
            Self::UserCheck => "user-check",
            Self::Download => "download",
            Self::Target => "target",
            Self::PieChart => "pie-chart",
            Self::TrendingDown => "trending-down",
            Self::Image => "image",
            Self::FileText => "file-text",
            Self::Upload => "upload",
            Self::LayoutGrid => "layout-grid",
            Self::Map => "map",
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
                IconName::Ticket => rsx! {
                    path { d: "M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" }
                    path { d: "M13 5v2" }
                    path { d: "M13 17v2" }
                    path { d: "M13 11v2" }
                },
                IconName::Edit => rsx! {
                    path { d: "M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" }
                    path { d: "M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z" }
                },
                IconName::Filter => rsx! {
                    polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
                },
                IconName::RefreshCw => rsx! {
                    path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
                    path { d: "M21 3v5h-5" }
                    path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
                    path { d: "M8 16H3v5" }
                },
                IconName::CreditCard => rsx! {
                    rect { width: "20", height: "14", x: "2", y: "5", rx: "2" }
                    line { x1: "2", x2: "22", y1: "10", y2: "10" }
                },
                IconName::ExternalLink => rsx! {
                    path { d: "M15 3h6v6" }
                    path { d: "M10 14 21 3" }
                    path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
                },
                IconName::ArrowLeft => rsx! {
                    path { d: "m12 19-7-7 7-7" }
                    path { d: "M19 12H5" }
                },
                IconName::MessageCircle => rsx! {
                    path { d: "M7.9 20A9 9 0 1 0 4 16.1L2 22Z" }
                },
                IconName::Info => rsx! {
                    circle { cx: "12", cy: "12", r: "10" }
                    path { d: "M12 16v-4" }
                    path { d: "M12 8h.01" }
                },
                IconName::Save => rsx! {
                    path { d: "M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" }
                    path { d: "M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" }
                    path { d: "M7 3v4a1 1 0 0 0 1 1h7" }
                },
                IconName::Building => rsx! {
                    rect { width: "16", height: "20", x: "4", y: "2", rx: "2", ry: "2" }
                    path { d: "M9 22v-4h6v4" }
                    path { d: "M8 6h.01" }
                    path { d: "M16 6h.01" }
                    path { d: "M12 6h.01" }
                    path { d: "M12 10h.01" }
                    path { d: "M12 14h.01" }
                    path { d: "M16 10h.01" }
                    path { d: "M16 14h.01" }
                    path { d: "M8 10h.01" }
                    path { d: "M8 14h.01" }
                },
                IconName::AlertTriangle => rsx! {
                    path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" }
                    path { d: "M12 9v4" }
                    path { d: "M12 17h.01" }
                },
                IconName::Database => rsx! {
                    ellipse { cx: "12", cy: "5", rx: "9", ry: "3" }
                    path { d: "M3 5V19A9 3 0 0 0 21 19V5" }
                    path { d: "M3 12A9 3 0 0 0 21 12" }
                },
                IconName::Cloud => rsx! {
                    path { d: "M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z" }
                },
                IconName::BarChart3 => rsx! {
                    path { d: "M3 3v16a2 2 0 0 0 2 2h16" }
                    path { d: "M18 17V9" }
                    path { d: "M13 17V5" }
                    path { d: "M8 17v-3" }
                },
                IconName::CheckCircle => rsx! {
                    path { d: "M21.801 10A10 10 0 1 1 17 3.335" }
                    path { d: "m9 11 3 3L22 4" }
                },
                IconName::XCircle => rsx! {
                    circle { cx: "12", cy: "12", r: "10" }
                    path { d: "m15 9-6 6" }
                    path { d: "m9 9 6 6" }
                },
                IconName::Server => rsx! {
                    rect { width: "20", height: "8", x: "2", y: "2", rx: "2", ry: "2" }
                    rect { width: "20", height: "8", x: "2", y: "14", rx: "2", ry: "2" }
                    line { x1: "6", x2: "6.01", y1: "6", y2: "6" }
                    line { x1: "6", x2: "6.01", y1: "18", y2: "18" }
                },
                IconName::HardDrive => rsx! {
                    line { x1: "22", x2: "2", y1: "12", y2: "12" }
                    path { d: "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" }
                    line { x1: "6", x2: "6.01", y1: "16", y2: "16" }
                    line { x1: "10", x2: "10.01", y1: "16", y2: "16" }
                },
                IconName::Trash2 => rsx! {
                    path { d: "M3 6h18" }
                    path { d: "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" }
                    path { d: "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" }
                    line { x1: "10", x2: "10", y1: "11", y2: "17" }
                    line { x1: "14", x2: "14", y1: "11", y2: "17" }
                },
                IconName::UserCheck => rsx! {
                    path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                    circle { cx: "9", cy: "7", r: "4" }
                    polyline { points: "16 11 18 13 22 9" }
                },
                IconName::Download => rsx! {
                    path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                    polyline { points: "7 10 12 15 17 10" }
                    line { x1: "12", x2: "12", y1: "15", y2: "3" }
                },
                IconName::Target => rsx! {
                    circle { cx: "12", cy: "12", r: "10" }
                    circle { cx: "12", cy: "12", r: "6" }
                    circle { cx: "12", cy: "12", r: "2" }
                },
                IconName::PieChart => rsx! {
                    path { d: "M21.21 15.89A10 10 0 1 1 8 2.83" }
                    path { d: "M22 12A10 10 0 0 0 12 2v10z" }
                },
                IconName::Image => rsx! {
                    rect { width: "18", height: "18", x: "3", y: "3", rx: "2", ry: "2" }
                    circle { cx: "9", cy: "9", r: "2" }
                    path { d: "m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" }
                },
                IconName::FileText => rsx! {
                    path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
                    path { d: "M14 2v4a2 2 0 0 0 2 2h4" }
                    path { d: "M10 9H8" }
                    path { d: "M16 13H8" }
                    path { d: "M16 17H8" }
                },
                IconName::Upload => rsx! {
                    path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
                    polyline { points: "17 8 12 3 7 8" }
                    line { x1: "12", x2: "12", y1: "3", y2: "15" }
                },
                IconName::LayoutGrid => rsx! {
                    rect { width: "7", height: "7", x: "3", y: "3", rx: "1" }
                    rect { width: "7", height: "7", x: "14", y: "3", rx: "1" }
                    rect { width: "7", height: "7", x: "14", y: "14", rx: "1" }
                    rect { width: "7", height: "7", x: "3", y: "14", rx: "1" }
                },
                IconName::Map => rsx! {
                    path { d: "M14.106 5.553a2 2 0 0 0 1.788 0l3.659-1.83A1 1 0 0 1 21 4.619v12.764a1 1 0 0 1-.553.894l-4.553 2.277a2 2 0 0 1-1.788 0l-4.212-2.106a2 2 0 0 0-1.788 0l-3.659 1.83A1 1 0 0 1 3 19.381V6.618a1 1 0 0 1 .553-.894l4.553-2.277a2 2 0 0 1 1.788 0z" }
                    path { d: "M15 5.764v15" }
                    path { d: "M9 3.236v15" }
                },
                IconName::TrendingDown => rsx! {
                    polyline { points: "22 17 13.5 8.5 8.5 13.5 2 7" }
                    polyline { points: "16 17 22 17 22 11" }
                },
                IconName::Zap => rsx! {
                    path { d: "M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z" }
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
