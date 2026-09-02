#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub name: &'static str,
    pub path: &'static str,
    pub icon: Option<&'static str>,
}

pub const PRIMARY_NAV_ITEMS: &[NavItem] = &[
    NavItem {
        name: "首頁",
        path: "/",
        icon: None,
    },
    NavItem {
        name: "精選活動",
        path: "/events",
        icon: None,
    },
    NavItem {
        name: "VVIP專區",
        path: "/vvip",
        icon: Some("crown"),
    },
];

pub fn primary_nav_items() -> &'static [NavItem] {
    PRIMARY_NAV_ITEMS
}

pub fn is_active_path(current: &str, item_path: &str) -> bool {
    current == item_path
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionKind {
    Login,
    Register,
    Profile,
    Registrations,
    Admin,
    EventMgmt,
    Sales,
    SystemHealth,
    Logout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionEntry {
    pub kind: SessionKind,
    pub href: Option<&'static str>,
    pub label: &'static str,
}

pub fn session_entries(is_authenticated: bool, view_admin: bool) -> Vec<SessionEntry> {
    if !is_authenticated {
        return vec![
            SessionEntry {
                kind: SessionKind::Login,
                href: Some("/login"),
                label: "登入",
            },
            SessionEntry {
                kind: SessionKind::Register,
                href: Some("/register"),
                label: "註冊",
            },
        ];
    }

    let mut entries = vec![
        SessionEntry {
            kind: SessionKind::Profile,
            href: Some("/profile"),
            label: "個人檔案",
        },
        SessionEntry {
            kind: SessionKind::Registrations,
            href: Some("/profile/registrations"),
            label: "我的報名",
        },
    ];
    if view_admin {
        entries.extend_from_slice(&[
            SessionEntry {
                kind: SessionKind::Admin,
                href: Some("/admin"),
                label: "管理後台",
            },
            SessionEntry {
                kind: SessionKind::EventMgmt,
                href: Some("/event-mgmt"),
                label: "活動管理",
            },
            SessionEntry {
                kind: SessionKind::Sales,
                href: Some("/admin/sales"),
                label: "銷售管理",
            },
            SessionEntry {
                kind: SessionKind::SystemHealth,
                href: Some("/admin/system"),
                label: "系統健康",
            },
        ]);
    }
    entries.push(SessionEntry {
        kind: SessionKind::Logout,
        href: None,
        label: "登出",
    });
    entries
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Presence {
    #[default]
    Hidden,
    Entering,
    Shown,
    Exiting,
}

pub fn presence_is_mounted(presence: Presence) -> bool {
    !matches!(presence, Presence::Hidden)
}

pub fn presence_toggle(presence: Presence) -> Presence {
    match presence {
        Presence::Hidden | Presence::Exiting => Presence::Entering,
        Presence::Entering | Presence::Shown => Presence::Exiting,
    }
}

pub fn presence_after_animation_end(presence: Presence) -> Presence {
    match presence {
        Presence::Entering => Presence::Shown,
        Presence::Exiting => Presence::Hidden,
        other => other,
    }
}

pub fn presence_class(presence: Presence, enter: &'static str, exit: &'static str) -> &'static str {
    match presence {
        Presence::Entering | Presence::Shown => enter,
        Presence::Exiting => exit,
        Presence::Hidden => "",
    }
}
