use std::cell::Cell;
use std::rc::Rc;

use crate::icons::{Icon, IconName};
use crate::permissions::{RouteGuard, Session, permissions};
use crate::shell::{
    Presence, presence_after_animation_end, presence_class, presence_is_mounted, presence_toggle,
};
use crate::users::{
    DELETE_ACTION, EDIT_ACTION, EditUserData, PAGE_SIZE, User, UserFilters, UserStats, UsersModal,
    UsersPage, UsersPagination, VerificationBadge, action_is, admin_count, delete_user,
    display_name, edit_data_from_user, fetch_user_stats, fetch_users, filters_are_active,
    format_currency, format_joined_date, membership_tier_badge_class, membership_tier_label,
    page_after_filter_change, page_in_range, pagination_range, parse_edit_int,
    pending_verification_count, role_badge_class, role_label, shows_verify_actions, update_user,
    user_initials, user_management_guard, verification_badge, verification_badge_class,
    verification_badge_label, verify_action_key, verify_user,
};
use dioxus::prelude::*;

#[component]
pub fn AdminUsers() -> Element {
    let navigator = use_navigator();
    let local = use_signal(Session::default);
    let session = try_use_context::<Signal<Session>>().unwrap_or(local);
    let current = session();
    match user_management_guard(current.restoring, &current.snapshot()) {
        RouteGuard::Loading => rsx! { AdminUsersGuardLoading {} },
        RouteGuard::Redirect(path) => {
            navigator.replace(path);
            rsx! {
                p { id: "admin-users-unauth", "redirecting" }
            }
        }
        RouteGuard::Allow => rsx! { AdminUsersBody { session } },
    }
}

#[component]
fn AdminUsersGuardLoading() -> Element {
    rsx! {
        div {
            id: "admin-users-guard-loading",
            class: "min-h-screen bg-luxury-midnight-black flex items-center justify-center",
            div { class: "luxury-glass p-8 rounded-2xl text-center",
                div { class: "w-12 h-12 border-4 border-luxury-gold border-t-transparent rounded-full animate-spin mx-auto mb-4" }
                p { class: "text-luxury-platinum", "驗證存取權限中..." }
            }
        }
    }
}

#[component]
fn AdminUsersBody(session: Signal<Session>) -> Element {
    let loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let users = use_signal(Vec::<User>::new);
    let stats = use_signal(|| None::<UserStats>);
    let pagination = use_signal(UsersPagination::default);
    let mut current_page = use_signal(|| 1u32);
    let mut filters = use_signal(UserFilters::default);
    let mut filters_presence = use_signal(|| Presence::Hidden);
    let mut selected_user = use_signal(|| None::<User>);
    let mut edit_data = use_signal(|| None::<EditUserData>);
    let mut modal = use_signal(|| UsersModal::None);
    let mut action_loading = use_signal(|| None::<String>);
    let fetch_gen = use_hook(|| Rc::new(Cell::new(0u32)));

    use_effect({
        let fetch_gen = fetch_gen.clone();
        move || {
            let page = current_page();
            let active_filters = filters();
            start_users_fetch(
                fetch_gen.clone(),
                loading,
                error,
                users,
                pagination,
                page,
                active_filters.clone(),
            );
            start_stats_fetch(stats);
        }
    });

    let can_delete = permissions(&session().snapshot()).manage_super_admin;

    rsx! {
        AdminUsersScreen {
            can_manage: true,
            can_delete,
            loading: loading(),
            error: error(),
            users: users(),
            stats: stats(),
            current_page: current_page(),
            pagination: pagination(),
            filters: filters(),
            filters_presence: filters_presence(),
            selected_user: selected_user(),
            edit_data: edit_data(),
            modal: modal(),
            action_loading: action_loading(),
            on_refresh: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    start_users_fetch(
                        fetch_gen.clone(),
                        loading,
                        error,
                        users,
                        pagination,
                        current_page(),
                        filters(),
                    );
                }
            },
            on_dismiss_error: move |_| error.set(None),
            on_toggle_filters: move |_| filters_presence.set(presence_toggle(filters_presence())),
            on_filters_animation_end: move |_| {
                filters_presence.set(presence_after_animation_end(filters_presence()));
            },
            on_search: move |value: String| {
                filters.write().search = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_role: move |value: String| {
                filters.write().role = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_membership_tier: move |value: String| {
                filters.write().membership_tier = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_verification_status: move |value: String| {
                filters.write().verification_status = value;
                current_page.set(page_after_filter_change(current_page()));
            },
            on_clear_filters: move |_| {
                filters.set(UserFilters::default());
                current_page.set(page_after_filter_change(current_page()));
            },
            on_page: move |new_page: u32| {
                if loading() || !page_in_range(new_page, pagination().total_pages) {
                    return;
                }
                current_page.set(new_page);
            },
            on_open_detail: move |user: User| {
                selected_user.set(Some(user));
                modal.set(UsersModal::Detail);
            },
            on_open_edit: move |user: User| {
                edit_data.set(Some(edit_data_from_user(&user)));
                selected_user.set(Some(user));
                modal.set(UsersModal::Edit);
            },
            on_open_delete: move |user: User| {
                selected_user.set(Some(user));
                modal.set(UsersModal::Delete);
            },
            on_close_modal: move |_| {
                modal.set(UsersModal::None);
                selected_user.set(None);
                edit_data.set(None);
            },
            on_edit_data: move |data: EditUserData| edit_data.set(Some(data)),
            on_save_edit: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    let Some(user) = selected_user() else {
                        return;
                    };
                    let Some(data) = edit_data() else {
                        return;
                    };
                    if action_is(action_loading().as_deref(), EDIT_ACTION) {
                        return;
                    }
                    action_loading.set(Some(EDIT_ACTION.to_string()));
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        let result = update_user(&user.id, &data).await;
                        action_loading.set(None);
                        match result {
                            Ok(()) => {
                                modal.set(UsersModal::None);
                                selected_user.set(None);
                                edit_data.set(None);
                                start_users_fetch(
                                    fetch_gen,
                                    loading,
                                    error,
                                    users,
                                    pagination,
                                    current_page(),
                                    filters(),
                                );
                            }
                            Err(message) => error.set(Some(message)),
                        }
                    });
                }
            },
            on_verify_approve: {
                let fetch_gen = fetch_gen.clone();
                move |user_id: String| {
                    start_verify(
                        fetch_gen.clone(),
                        loading,
                        error,
                        users,
                        pagination,
                        action_loading,
                        current_page(),
                        filters(),
                        user_id,
                        "approved",
                    );
                }
            },
            on_verify_reject: {
                let fetch_gen = fetch_gen.clone();
                move |user_id: String| {
                    start_verify(
                        fetch_gen.clone(),
                        loading,
                        error,
                        users,
                        pagination,
                        action_loading,
                        current_page(),
                        filters(),
                        user_id,
                        "rejected",
                    );
                }
            },
            on_confirm_delete: {
                let fetch_gen = fetch_gen.clone();
                move |_| {
                    let Some(user) = selected_user() else {
                        return;
                    };
                    if action_is(action_loading().as_deref(), DELETE_ACTION) {
                        return;
                    }
                    action_loading.set(Some(DELETE_ACTION.to_string()));
                    let fetch_gen = fetch_gen.clone();
                    spawn(async move {
                        let result = delete_user(&user.id).await;
                        action_loading.set(None);
                        match result {
                            Ok(()) => {
                                modal.set(UsersModal::None);
                                selected_user.set(None);
                                start_users_fetch(
                                    fetch_gen,
                                    loading,
                                    error,
                                    users,
                                    pagination,
                                    current_page(),
                                    filters(),
                                );
                            }
                            Err(message) => error.set(Some(message)),
                        }
                    });
                }
            },
        }
    }
}

fn start_users_fetch(
    fetch_gen: Rc<Cell<u32>>,
    mut loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    mut users: Signal<Vec<User>>,
    mut pagination: Signal<UsersPagination>,
    page: u32,
    filters: UserFilters,
) {
    let request_id = fetch_gen.get() + 1;
    fetch_gen.set(request_id);
    loading.set(true);
    error.set(None);
    spawn(async move {
        let result = fetch_users(page, &filters).await;
        if fetch_gen.get() != request_id {
            return;
        }
        match result {
            Ok(UsersPage {
                users: fetched,
                pagination: next,
            }) => {
                users.set(fetched);
                pagination.set(next);
                error.set(None);
            }
            Err(message) => {
                error.set(Some(message));
            }
        }
        loading.set(false);
    });
}

fn start_stats_fetch(mut stats: Signal<Option<UserStats>>) {
    spawn(async move {
        if let Ok(fetched) = fetch_user_stats().await {
            stats.set(Some(fetched));
        }
    });
}

fn start_verify(
    fetch_gen: Rc<Cell<u32>>,
    loading: Signal<bool>,
    mut error: Signal<Option<String>>,
    users: Signal<Vec<User>>,
    pagination: Signal<UsersPagination>,
    mut action_loading: Signal<Option<String>>,
    page: u32,
    filters: UserFilters,
    user_id: String,
    status: &'static str,
) {
    let key = verify_action_key(&user_id);
    if action_is(action_loading().as_deref(), &key) {
        return;
    }
    action_loading.set(Some(key));
    spawn(async move {
        let result = verify_user(&user_id, status).await;
        action_loading.set(None);
        match result {
            Ok(()) => {
                start_users_fetch(fetch_gen, loading, error, users, pagination, page, filters);
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[component]
pub fn AdminUsersScreen(
    can_manage: bool,
    can_delete: bool,
    loading: bool,
    error: Option<String>,
    users: Vec<User>,
    stats: Option<UserStats>,
    current_page: u32,
    pagination: UsersPagination,
    filters: UserFilters,
    filters_presence: Presence,
    selected_user: Option<User>,
    edit_data: Option<EditUserData>,
    modal: UsersModal,
    action_loading: Option<String>,
    #[props(default)] on_refresh: EventHandler<()>,
    #[props(default)] on_dismiss_error: EventHandler<()>,
    #[props(default)] on_toggle_filters: EventHandler<()>,
    #[props(default)] on_filters_animation_end: EventHandler<()>,
    #[props(default)] on_search: EventHandler<String>,
    #[props(default)] on_role: EventHandler<String>,
    #[props(default)] on_membership_tier: EventHandler<String>,
    #[props(default)] on_verification_status: EventHandler<String>,
    #[props(default)] on_clear_filters: EventHandler<()>,
    #[props(default)] on_page: EventHandler<u32>,
    #[props(default)] on_open_detail: EventHandler<User>,
    #[props(default)] on_open_edit: EventHandler<User>,
    #[props(default)] on_open_delete: EventHandler<User>,
    #[props(default)] on_close_modal: EventHandler<()>,
    #[props(default)] on_edit_data: EventHandler<EditUserData>,
    #[props(default)] on_save_edit: EventHandler<()>,
    #[props(default)] on_verify_approve: EventHandler<String>,
    #[props(default)] on_verify_reject: EventHandler<String>,
    #[props(default)] on_confirm_delete: EventHandler<()>,
) -> Element {
    if !can_manage {
        return rsx! {
            div {
                id: "admin-users-denied",
                class: "min-h-screen bg-gray-50 flex items-center justify-center",
                div { class: "text-center",
                    Icon { name: IconName::Shield, class: "w-16 h-16 text-gray-400 mx-auto mb-4".to_string() }
                    h1 { class: "text-2xl font-bold text-gray-900 mb-2", "拒絕存取" }
                    p { class: "text-gray-600", "您沒有權限存取此頁面。" }
                }
            }
        };
    }

    let total_users = pagination.total;
    let users_heading = format!("Users ({total_users} total)");
    let show_filters = presence_is_mounted(filters_presence);
    let filter_toggle = if show_filters {
        "隱藏篩選"
    } else {
        "顯示篩選"
    };
    let filters_class = format!(
        "grid grid-cols-1 md:grid-cols-3 gap-4 {}",
        presence_class(filters_presence, "hs-enter", "hs-exit")
    );
    let show_clear = filters_are_active(&filters);
    let show_pagination = pagination.total_pages > 1;
    let (range_start, range_end) = pagination_range(current_page, PAGE_SIZE, pagination.total);
    let range_copy = format!("Showing {range_start} to {range_end} of {total_users} results");
    let page_copy = format!("Page {current_page} of {}", pagination.total_pages);
    let empty = !loading && users.is_empty();
    let prev_disabled = current_page == 1;
    let next_disabled = current_page == pagination.total_pages;
    let prev_page = current_page.saturating_sub(1).max(1);
    let next_page = current_page.saturating_add(1);
    let edit_busy = action_is(action_loading.as_deref(), EDIT_ACTION);
    let delete_busy = action_is(action_loading.as_deref(), DELETE_ACTION);

    rsx! {
        div { id: "admin-users-page", class: "min-h-screen bg-gray-50",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8",
                div { class: "mb-8",
                    div { class: "flex items-center justify-between",
                        div {
                            h1 { class: "text-3xl font-bold text-gray-900 flex items-center gap-3",
                                Icon { name: IconName::Users, class: "w-8 h-8 text-purple-600".to_string() }
                                "使用者管理"
                            }
                            p { class: "text-gray-600 mt-2", "管理平台會員及其權限" }
                        }
                        button {
                            id: "admin-users-refresh",
                            r#type: "button",
                            class: "inline-flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors",
                            onclick: move |_| on_refresh.call(()),
                            Icon { name: IconName::RefreshCw, class: "w-4 h-4".to_string() }
                            "重新整理"
                        }
                    }
                    if let Some(stats) = stats {
                        StatsCards { stats }
                    }
                }

                if let Some(message) = error {
                    div {
                        id: "admin-users-error",
                        class: "mb-6 bg-red-50 border border-red-200 rounded-lg p-4",
                        div { class: "flex items-center",
                            Icon { name: IconName::X, class: "w-5 h-5 text-red-600 mr-2".to_string() }
                            p { class: "text-red-800", "{message}" }
                            button {
                                id: "admin-users-error-dismiss",
                                r#type: "button",
                                class: "ml-auto text-red-600 hover:text-red-800",
                                onclick: move |_| on_dismiss_error.call(()),
                                Icon { name: IconName::X, class: "w-4 h-4".to_string() }
                            }
                        }
                    }
                }

                div { class: "bg-white rounded-lg shadow-sm border border-gray-200 mb-6",
                    div { class: "p-4 border-b border-gray-200",
                        div { class: "flex items-center justify-between",
                            h3 { class: "text-lg font-medium text-gray-900", "篩選與搜尋" }
                            button {
                                id: "admin-users-filter-toggle",
                                r#type: "button",
                                class: "inline-flex items-center gap-2 px-3 py-1 text-sm text-gray-600 hover:text-gray-900",
                                onclick: move |_| on_toggle_filters.call(()),
                                Icon { name: IconName::Filter, class: "w-4 h-4".to_string() }
                                "{filter_toggle}"
                            }
                        }
                    }
                    div { class: "p-4",
                        div { class: "relative mb-4",
                            Icon {
                                name: IconName::Search,
                                class: "absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400".to_string(),
                            }
                            input {
                                r#type: "text",
                                id: "admin-users-search",
                                placeholder: "依姓名、電子郵件、職業搜尋...",
                                value: "{filters.search}",
                                class: "w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                oninput: move |evt| on_search.call(evt.value()),
                            }
                        }
                        if show_filters {
                            div {
                                id: "admin-users-filters",
                                class: "{filters_class}",
                                onanimationend: move |_| on_filters_animation_end.call(()),
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "角色" }
                                    select {
                                        id: "admin-users-role-filter",
                                        value: "{filters.role}",
                                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                        onchange: move |evt| on_role.call(evt.value()),
                                        option { value: "", "所有角色" }
                                        option { value: "user", "使用者" }
                                        option { value: "admin", "管理員" }
                                        option { value: "super_admin", "超級管理員" }
                                    }
                                }
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "會員等級" }
                                    select {
                                        id: "admin-users-tier-filter",
                                        value: "{filters.membership_tier}",
                                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                        onchange: move |evt| on_membership_tier.call(evt.value()),
                                        option { value: "", "所有等級" }
                                        option { value: "Platinum", "白金卡" }
                                        option { value: "Diamond", "鑽石卡" }
                                        option { value: "Black Card", "黑卡" }
                                    }
                                }
                                div {
                                    label { class: "block text-sm font-medium text-gray-700 mb-1", "驗證狀態" }
                                    select {
                                        id: "admin-users-status-filter",
                                        value: "{filters.verification_status}",
                                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                                        onchange: move |evt| on_verification_status.call(evt.value()),
                                        option { value: "", "All Statuses" }
                                        option { value: "pending", "Pending" }
                                        option { value: "approved", "Approved" }
                                        option { value: "rejected", "Rejected" }
                                    }
                                }
                            }
                        }
                        if show_clear {
                            div { class: "mt-4 flex justify-end",
                                button {
                                    id: "admin-users-clear-filters",
                                    r#type: "button",
                                    class: "px-4 py-2 text-sm text-gray-600 hover:text-gray-900 border border-gray-300 rounded-lg hover:bg-gray-50",
                                    onclick: move |_| on_clear_filters.call(()),
                                    "Clear All Filters"
                                }
                            }
                        }
                    }
                }

                div { class: "bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden",
                    div { class: "px-6 py-4 border-b border-gray-200",
                        div { class: "flex items-center justify-between",
                            h3 { id: "admin-users-heading", class: "text-lg font-medium text-gray-900",
                                "{users_heading}"
                            }
                            button {
                                id: "admin-users-export",
                                r#type: "button",
                                class: "inline-flex items-center gap-2 px-3 py-1 text-sm text-gray-600 hover:text-gray-900 border border-gray-300 rounded-lg hover:bg-gray-50",
                                Icon { name: IconName::ArrowRight, class: "w-4 h-4".to_string() }
                                "Export"
                            }
                        }
                    }
                    if loading {
                        div {
                            id: "admin-users-loading",
                            class: "flex items-center justify-center py-12",
                            Icon { name: IconName::RefreshCw, class: "w-6 h-6 text-gray-400 animate-spin".to_string() }
                            span { class: "ml-2 text-gray-600", "Loading users..." }
                        }
                    } else {
                        div { class: "overflow-x-auto",
                            table { id: "admin-users-table", class: "w-full",
                                thead { class: "bg-gray-50",
                                    tr {
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "User" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Membership" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Status" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Role" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Financial" }
                                        th { class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider", "Joined" }
                                        th { class: "px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider", "Actions" }
                                    }
                                }
                                tbody { class: "bg-white divide-y divide-gray-200",
                                    if empty {
                                        tr { id: "admin-users-empty",
                                            td {
                                                class: "px-6 py-8 text-center text-sm text-gray-500",
                                                colspan: "7",
                                            }
                                        }
                                    }
                                    for user in users.iter().cloned() {
                                        UserTableRow {
                                            user,
                                            action_loading: action_loading.clone(),
                                            can_delete,
                                            on_open_detail,
                                            on_open_edit,
                                            on_open_delete,
                                            on_verify_approve,
                                            on_verify_reject,
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if show_pagination {
                        div {
                            id: "admin-users-pagination",
                            class: "px-6 py-4 border-t border-gray-200 flex items-center justify-between",
                            div { class: "text-sm text-gray-700", "{range_copy}" }
                            div { class: "flex items-center gap-2",
                                button {
                                    id: "admin-users-prev",
                                    r#type: "button",
                                    class: "inline-flex items-center px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: prev_disabled,
                                    onclick: move |_| on_page.call(prev_page),
                                    Icon { name: IconName::ChevronLeft, class: "w-4 h-4".to_string() }
                                    "Previous"
                                }
                                span { class: "text-sm text-gray-700", "{page_copy}" }
                                button {
                                    id: "admin-users-next",
                                    r#type: "button",
                                    class: "inline-flex items-center px-3 py-2 border border-gray-300 rounded-lg text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: next_disabled,
                                    onclick: move |_| on_page.call(next_page),
                                    "Next"
                                    Icon { name: IconName::ChevronRight, class: "w-4 h-4".to_string() }
                                }
                            }
                        }
                    }
                }
            }

            if modal == UsersModal::Detail {
                if let Some(user) = selected_user.clone() {
                    UserDetailModal { user, on_close_modal }
                }
            }
            if modal == UsersModal::Edit {
                if let Some(user) = selected_user.clone() {
                    if let Some(edit_data) = edit_data.clone() {
                        UserEditModal {
                            user,
                            edit_data,
                            edit_busy,
                            on_close_modal,
                            on_edit_data,
                            on_save_edit,
                        }
                    }
                }
            }
            if modal == UsersModal::Delete {
                if let Some(user) = selected_user.clone() {
                    UserDeleteModal {
                        user,
                        delete_busy,
                        on_close_modal,
                        on_confirm_delete,
                    }
                }
            }
        }
    }
}

#[component]
fn StatsCards(stats: UserStats) -> Element {
    let pending = pending_verification_count(&stats);
    let admins = admin_count(&stats);
    rsx! {
        div { id: "admin-users-stats", class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mt-6",
            div { class: "bg-white rounded-lg p-6 shadow-sm border border-gray-200",
                div { class: "flex items-center justify-between",
                    div {
                        p { class: "text-sm font-medium text-gray-600", "總使用者數" }
                        p { id: "admin-users-stat-total", class: "text-3xl font-bold text-gray-900", "{stats.total_users}" }
                    }
                    Icon { name: IconName::Users, class: "w-8 h-8 text-blue-600".to_string() }
                }
            }
            div { class: "bg-white rounded-lg p-6 shadow-sm border border-gray-200",
                div { class: "flex items-center justify-between",
                    div {
                        p { class: "text-sm font-medium text-gray-600", "近期註冊" }
                        p { id: "admin-users-stat-recent", class: "text-3xl font-bold text-gray-900", "{stats.recent_registrations}" }
                    }
                    Icon { name: IconName::User, class: "w-8 h-8 text-green-600".to_string() }
                }
            }
            div { class: "bg-white rounded-lg p-6 shadow-sm border border-gray-200",
                div { class: "flex items-center justify-between",
                    div {
                        p { class: "text-sm font-medium text-gray-600", "待審核驗證" }
                        p { id: "admin-users-stat-pending", class: "text-3xl font-bold text-gray-900", "{pending}" }
                    }
                    Icon { name: IconName::AlertTriangle, class: "w-8 h-8 text-yellow-600".to_string() }
                }
            }
            div { class: "bg-white rounded-lg p-6 shadow-sm border border-gray-200",
                div { class: "flex items-center justify-between",
                    div {
                        p { class: "text-sm font-medium text-gray-600", "管理員" }
                        p { id: "admin-users-stat-admins", class: "text-3xl font-bold text-gray-900", "{admins}" }
                    }
                    Icon { name: IconName::Shield, class: "w-8 h-8 text-red-600".to_string() }
                }
            }
        }
    }
}

#[component]
fn UserTableRow(
    user: User,
    action_loading: Option<String>,
    can_delete: bool,
    on_open_detail: EventHandler<User>,
    on_open_edit: EventHandler<User>,
    on_open_delete: EventHandler<User>,
    on_verify_approve: EventHandler<String>,
    on_verify_reject: EventHandler<String>,
) -> Element {
    let row_id = format!("admin-users-row-{}", user.id);
    let name = display_name(&user);
    let initials = user_initials(&user.first_name, &user.last_name);
    let email = user.email.clone();
    let profession = user.profession.clone();
    let picture = user.profile_picture.clone();
    let income = format_currency(user.annual_income);
    let worth = format_currency(user.net_worth);
    let joined = format_joined_date(&user.created_at);
    let tier_class = membership_tier_badge_class(&user.membership_tier);
    let tier_label = membership_tier_label(&user.membership_tier);
    let tier_icon = match user.membership_tier.as_str() {
        "Diamond" => IconName::Crown,
        "Black Card" => IconName::Shield,
        _ => IconName::Star,
    };
    let badge = verification_badge(&user.verification_status, user.is_verified);
    let badge_class = verification_badge_class(badge);
    let badge_label = verification_badge_label(badge);
    let badge_icon = match badge {
        VerificationBadge::Approved => IconName::Check,
        VerificationBadge::Rejected => IconName::X,
        VerificationBadge::Pending => IconName::AlertTriangle,
    };
    let role_class = role_badge_class(&user.role);
    let role_text = role_label(&user.role);
    let show_verify = shows_verify_actions(&user.verification_status);
    let verify_key = verify_action_key(&user.id);
    let verify_busy = action_is(action_loading.as_deref(), &verify_key);
    let detail_user = user.clone();
    let edit_user = user.clone();
    let delete_user = user.clone();
    let approve_id = user.id.clone();
    let reject_id = user.id.clone();

    rsx! {
        tr { id: "{row_id}", class: "hover:bg-gray-50",
            td { class: "px-6 py-4 whitespace-nowrap",
                div { class: "flex items-center",
                    div { class: "flex-shrink-0 h-10 w-10",
                        if let Some(src) = picture {
                            img { class: "h-10 w-10 rounded-full", src: "{src}", alt: "" }
                        } else {
                            div { class: "h-10 w-10 rounded-full bg-gray-300 flex items-center justify-center",
                                span { class: "text-sm font-medium text-gray-700", "{initials}" }
                            }
                        }
                    }
                    div { class: "ml-4",
                        div { class: "text-sm font-medium text-gray-900", "{name}" }
                        div { class: "text-sm text-gray-500", "{email}" }
                        div { class: "text-xs text-gray-400", "{profession}" }
                    }
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap",
                span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border {tier_class}",
                    Icon { name: tier_icon, class: "w-3 h-3".to_string() }
                    "{tier_label}"
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap",
                span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium {badge_class}",
                    Icon { name: badge_icon, class: "w-3 h-3".to_string() }
                    "{badge_label}"
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap",
                span { class: "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium {role_class}",
                    "{role_text}"
                }
            }
            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                div { "Income: {income}" }
                div { class: "text-gray-500", "Net Worth: {worth}" }
            }
            td { class: "px-6 py-4 whitespace-nowrap text-sm text-gray-500", "{joined}" }
            td { class: "px-6 py-4 whitespace-nowrap text-right text-sm font-medium",
                div { class: "flex items-center justify-end gap-2",
                    button {
                        id: "admin-users-view-{user.id}",
                        r#type: "button",
                        class: "text-gray-600 hover:text-gray-900",
                        title: "View Details",
                        onclick: move |_| on_open_detail.call(detail_user.clone()),
                        Icon { name: IconName::Eye, class: "w-4 h-4".to_string() }
                    }
                    button {
                        id: "admin-users-edit-{user.id}",
                        r#type: "button",
                        class: "text-blue-600 hover:text-blue-900",
                        title: "Edit User",
                        onclick: move |_| on_open_edit.call(edit_user.clone()),
                        Icon { name: IconName::Edit, class: "w-4 h-4".to_string() }
                    }
                    if show_verify {
                        button {
                            id: "admin-users-approve-{user.id}",
                            r#type: "button",
                            class: "text-green-600 hover:text-green-900 disabled:opacity-50",
                            title: "Approve",
                            disabled: verify_busy,
                            onclick: move |_| on_verify_approve.call(approve_id.clone()),
                            Icon { name: IconName::Check, class: "w-4 h-4".to_string() }
                        }
                        button {
                            id: "admin-users-reject-{user.id}",
                            r#type: "button",
                            class: "text-red-600 hover:text-red-900 disabled:opacity-50",
                            title: "Reject",
                            disabled: verify_busy,
                            onclick: move |_| on_verify_reject.call(reject_id.clone()),
                            Icon { name: IconName::X, class: "w-4 h-4".to_string() }
                        }
                    }
                    if can_delete {
                        button {
                            id: "admin-users-delete-{user.id}",
                            r#type: "button",
                            class: "text-red-600 hover:text-red-900",
                            title: "Delete User",
                            onclick: move |_| on_open_delete.call(delete_user.clone()),
                            Icon { name: IconName::AlertTriangle, class: "w-4 h-4".to_string() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UserDetailModal(user: User, on_close_modal: EventHandler<()>) -> Element {
    let name = display_name(&user);
    let email = user.email.clone();
    let profession = user.profession.clone();
    let privacy = format!("{}/5", user.privacy_level);
    let joined = format_joined_date(&user.created_at);
    let income = format_currency(user.annual_income);
    let worth = format_currency(user.net_worth);
    let bio = user.bio.clone();
    let interests = user.interests.clone();
    let tier_class = membership_tier_badge_class(&user.membership_tier);
    let tier_label = membership_tier_label(&user.membership_tier);
    let tier_icon = match user.membership_tier.as_str() {
        "Diamond" => IconName::Crown,
        "Black Card" => IconName::Shield,
        _ => IconName::Star,
    };
    let badge = verification_badge(&user.verification_status, user.is_verified);
    let badge_class = verification_badge_class(badge);
    let badge_label = verification_badge_label(badge);
    let badge_icon = match badge {
        VerificationBadge::Approved => IconName::Check,
        VerificationBadge::Rejected => IconName::X,
        VerificationBadge::Pending => IconName::AlertTriangle,
    };
    let role_class = role_badge_class(&user.role);
    let role_text = role_label(&user.role);
    let age = user.age;

    rsx! {
        div {
            id: "admin-users-detail-modal",
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto",
                div { class: "flex items-center justify-between mb-6",
                    h3 { class: "text-xl font-bold text-gray-900", "User Details" }
                    button {
                        id: "admin-users-detail-close",
                        r#type: "button",
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close_modal.call(()),
                        Icon { name: IconName::X, class: "w-6 h-6".to_string() }
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    div {
                        h4 { class: "font-medium text-gray-900 mb-3", "Personal Information" }
                        div { class: "space-y-2 text-sm",
                            p { span { class: "font-medium", "Name:" } " {name}" }
                            p { span { class: "font-medium", "Email:" } " {email}" }
                            p { span { class: "font-medium", "Age:" } " {age}" }
                            p { span { class: "font-medium", "Profession:" } " {profession}" }
                            p { span { class: "font-medium", "Privacy Level:" } " {privacy}" }
                        }
                    }
                    div {
                        h4 { class: "font-medium text-gray-900 mb-3", "Account Information" }
                        div { class: "space-y-2 text-sm",
                            p {
                                span { class: "font-medium", "Membership:" }
                                span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium border {tier_class} ml-1",
                                    Icon { name: tier_icon, class: "w-3 h-3".to_string() }
                                    "{tier_label}"
                                }
                            }
                            p {
                                span { class: "font-medium", "Role:" }
                                span { class: "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium {role_class} ml-1",
                                    "{role_text}"
                                }
                            }
                            p {
                                span { class: "font-medium", "Status:" }
                                span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium {badge_class} ml-1",
                                    Icon { name: badge_icon, class: "w-3 h-3".to_string() }
                                    "{badge_label}"
                                }
                            }
                            p { span { class: "font-medium", "Joined:" } " {joined}" }
                        }
                    }
                    div {
                        h4 { class: "font-medium text-gray-900 mb-3", "Financial Information" }
                        div { class: "space-y-2 text-sm",
                            p { span { class: "font-medium", "Annual Income:" } " {income}" }
                            p { span { class: "font-medium", "Net Worth:" } " {worth}" }
                        }
                    }
                    div {
                        h4 { class: "font-medium text-gray-900 mb-3", "Interests" }
                        div { class: "flex flex-wrap gap-2",
                            for interest in interests.iter() {
                                span { class: "px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs", "{interest}" }
                            }
                        }
                    }
                }
                if let Some(bio) = bio {
                    if !bio.is_empty() {
                        div { class: "mt-6",
                            h4 { class: "font-medium text-gray-900 mb-3", "Bio" }
                            p { class: "text-sm text-gray-700", "{bio}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn UserEditModal(
    user: User,
    edit_data: EditUserData,
    edit_busy: bool,
    on_close_modal: EventHandler<()>,
    on_edit_data: EventHandler<EditUserData>,
    on_save_edit: EventHandler<()>,
) -> Element {
    let _ = user;
    let first_name = edit_data.first_name.clone();
    let last_name = edit_data.last_name.clone();
    let age = edit_data.age.to_string();
    let profession = edit_data.profession.clone();
    let annual_income = edit_data.annual_income.to_string();
    let net_worth = edit_data.net_worth.to_string();
    let membership_tier = edit_data.membership_tier.clone();
    let privacy_level = edit_data.privacy_level.to_string();
    let bio = edit_data.bio.clone();
    let data_first = edit_data.clone();
    let data_last = edit_data.clone();
    let data_age = edit_data.clone();
    let data_profession = edit_data.clone();
    let data_income = edit_data.clone();
    let data_worth = edit_data.clone();
    let data_tier = edit_data.clone();
    let data_privacy = edit_data.clone();
    let data_bio = edit_data.clone();

    rsx! {
        div {
            id: "admin-users-edit-modal",
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto",
                div { class: "flex items-center justify-between mb-6",
                    h3 { class: "text-xl font-bold text-gray-900", "Edit User" }
                    button {
                        id: "admin-users-edit-close",
                        r#type: "button",
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close_modal.call(()),
                        Icon { name: IconName::X, class: "w-6 h-6".to_string() }
                    }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "First Name" }
                        input {
                            r#type: "text",
                            id: "admin-users-edit-first-name",
                            value: "{first_name}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            oninput: move |evt| {
                                let mut next = data_first.clone();
                                next.first_name = evt.value();
                                on_edit_data.call(next);
                            },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Last Name" }
                        input {
                            r#type: "text",
                            id: "admin-users-edit-last-name",
                            value: "{last_name}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            oninput: move |evt| {
                                let mut next = data_last.clone();
                                next.last_name = evt.value();
                                on_edit_data.call(next);
                            },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Age" }
                        input {
                            r#type: "number",
                            id: "admin-users-edit-age",
                            value: "{age}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            oninput: move |evt| {
                                let mut next = data_age.clone();
                                if let Some(value) = parse_edit_int(&evt.value()) {
                                    next.age = value;
                                    on_edit_data.call(next);
                                }
                            },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Profession" }
                        input {
                            r#type: "text",
                            id: "admin-users-edit-profession",
                            value: "{profession}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            oninput: move |evt| {
                                let mut next = data_profession.clone();
                                next.profession = evt.value();
                                on_edit_data.call(next);
                            },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Annual Income (TWD)" }
                        input {
                            r#type: "number",
                            id: "admin-users-edit-income",
                            value: "{annual_income}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            oninput: move |evt| {
                                let mut next = data_income.clone();
                                if let Some(value) = parse_edit_int(&evt.value()) {
                                    next.annual_income = value;
                                    on_edit_data.call(next);
                                }
                            },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Net Worth (TWD)" }
                        input {
                            r#type: "number",
                            id: "admin-users-edit-net-worth",
                            value: "{net_worth}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            oninput: move |evt| {
                                let mut next = data_worth.clone();
                                if let Some(value) = parse_edit_int(&evt.value()) {
                                    next.net_worth = value;
                                    on_edit_data.call(next);
                                }
                            },
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Membership Tier" }
                        select {
                            id: "admin-users-edit-tier",
                            value: "{membership_tier}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            onchange: move |evt| {
                                let mut next = data_tier.clone();
                                next.membership_tier = evt.value();
                                on_edit_data.call(next);
                            },
                            option { value: "Platinum", "Platinum" }
                            option { value: "Diamond", "Diamond" }
                            option { value: "Black Card", "Black Card" }
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Privacy Level" }
                        select {
                            id: "admin-users-edit-privacy",
                            value: "{privacy_level}",
                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                            onchange: move |evt| {
                                let mut next = data_privacy.clone();
                                if let Some(value) = parse_edit_int(&evt.value()) {
                                    next.privacy_level = value;
                                    on_edit_data.call(next);
                                }
                            },
                            option { value: "1", "1 - Public" }
                            option { value: "2", "2 - Low" }
                            option { value: "3", "3 - Medium" }
                            option { value: "4", "4 - High" }
                            option { value: "5", "5 - Very High" }
                        }
                    }
                }
                div { class: "mt-4",
                    label { class: "block text-sm font-medium text-gray-700 mb-1", "Bio" }
                    textarea {
                        id: "admin-users-edit-bio",
                        value: "{bio}",
                        rows: 3,
                        class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-transparent",
                        oninput: move |evt| {
                            let mut next = data_bio.clone();
                            next.bio = evt.value();
                            on_edit_data.call(next);
                        },
                    }
                }
                div { class: "flex justify-end gap-3 mt-6",
                    button {
                        id: "admin-users-edit-cancel",
                        r#type: "button",
                        class: "px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50",
                        onclick: move |_| on_close_modal.call(()),
                        "Cancel"
                    }
                    button {
                        id: "admin-users-edit-save",
                        r#type: "button",
                        class: "px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:opacity-50 flex items-center gap-2",
                        disabled: edit_busy,
                        onclick: move |_| on_save_edit.call(()),
                        if edit_busy {
                            Icon { name: IconName::RefreshCw, class: "w-4 h-4 animate-spin".to_string() }
                        }
                        "Save Changes"
                    }
                }
            }
        }
    }
}

#[component]
fn UserDeleteModal(
    user: User,
    delete_busy: bool,
    on_close_modal: EventHandler<()>,
    on_confirm_delete: EventHandler<()>,
) -> Element {
    let name = display_name(&user);
    rsx! {
        div {
            id: "admin-users-delete-modal",
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg p-6 w-full max-w-md",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-lg font-bold text-gray-900", "Confirm Delete" }
                    button {
                        id: "admin-users-delete-close",
                        r#type: "button",
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| on_close_modal.call(()),
                        Icon { name: IconName::X, class: "w-5 h-5".to_string() }
                    }
                }
                p { class: "text-gray-700 mb-6",
                    "Are you sure you want to delete user "
                    strong { "{name}" }
                    "? This action cannot be undone."
                }
                div { class: "flex justify-end gap-3",
                    button {
                        id: "admin-users-delete-cancel",
                        r#type: "button",
                        class: "px-4 py-2 text-gray-700 border border-gray-300 rounded-lg hover:bg-gray-50",
                        onclick: move |_| on_close_modal.call(()),
                        "Cancel"
                    }
                    button {
                        id: "admin-users-delete-confirm",
                        r#type: "button",
                        class: "px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 flex items-center gap-2",
                        disabled: delete_busy,
                        onclick: move |_| on_confirm_delete.call(()),
                        if delete_busy {
                            Icon { name: IconName::RefreshCw, class: "w-4 h-4 animate-spin".to_string() }
                        }
                        "Delete User"
                    }
                }
            }
        }
    }
}
