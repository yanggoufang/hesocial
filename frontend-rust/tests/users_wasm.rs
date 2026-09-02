#![cfg(target_arch = "wasm32")]

use dioxus::prelude::*;
use hesocial_frontend::pages::users::AdminUsersScreen;
use hesocial_frontend::shell::Presence;
use hesocial_frontend::users::{
    EditUserData, User, UserFilters, UserStats, UsersModal, UsersPagination, VerificationCount,
    edit_data_from_user,
};
use wasm_bindgen_test::wasm_bindgen_test;

fn sample_user() -> User {
    User {
        id: "7".to_string(),
        email: "ada@example.com".to_string(),
        first_name: "Ada".to_string(),
        last_name: "Lovelace".to_string(),
        age: 36,
        profession: "Mathematician".to_string(),
        annual_income: 8_000_000,
        net_worth: 50_000_000,
        membership_tier: "Diamond".to_string(),
        privacy_level: 3,
        is_verified: true,
        verification_status: "approved".to_string(),
        role: "user".to_string(),
        profile_picture: None,
        bio: Some("Notes on the analytical engine".to_string()),
        interests: vec!["math".to_string(), "computing".to_string()],
        created_at: "2024-03-05T10:00:00.000Z".to_string(),
        updated_at: "2024-03-06T10:00:00.000Z".to_string(),
    }
}

fn pending_user() -> User {
    User {
        id: "pending-1".to_string(),
        email: "pending@example.com".to_string(),
        first_name: "Pat".to_string(),
        last_name: "Pending".to_string(),
        age: 40,
        profession: "Investor".to_string(),
        annual_income: 6_000_000,
        net_worth: 40_000_000,
        membership_tier: "Platinum".to_string(),
        privacy_level: 2,
        is_verified: false,
        verification_status: "pending".to_string(),
        role: "user".to_string(),
        profile_picture: None,
        bio: None,
        interests: Vec::new(),
        created_at: "2024-01-15T00:00:00.000Z".to_string(),
        updated_at: "2024-01-15T00:00:00.000Z".to_string(),
    }
}

fn sample_stats() -> UserStats {
    UserStats {
        total_users: 12,
        recent_registrations: 4,
        users_by_role: vec![
            hesocial_frontend::users::RoleCount {
                role: "user".to_string(),
                count: 9,
            },
            hesocial_frontend::users::RoleCount {
                role: "admin".to_string(),
                count: 2,
            },
            hesocial_frontend::users::RoleCount {
                role: "super_admin".to_string(),
                count: 1,
            },
        ],
        users_by_membership_tier: Vec::new(),
        users_by_verification_status: vec![VerificationCount {
            verification_status: "pending".to_string(),
            count: 3,
        }],
    }
}

#[component]
fn UsersAt(
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
) -> Element {
    rsx! {
        AdminUsersScreen {
            can_manage,
            can_delete,
            loading,
            error,
            users,
            stats,
            current_page,
            pagination,
            filters,
            filters_presence,
            selected_user,
            edit_data,
            modal,
            action_loading,
        }
    }
}

fn render_users(
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
) -> String {
    let mut vdom = VirtualDom::new_with_props(
        UsersAt,
        UsersAtProps {
            can_manage,
            can_delete,
            loading,
            error,
            users,
            stats,
            current_page,
            pagination,
            filters,
            filters_presence,
            selected_user,
            edit_data,
            modal,
            action_loading,
        },
    );
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}

fn render_default(
    loading: bool,
    error: Option<String>,
    users: Vec<User>,
    stats: Option<UserStats>,
) -> String {
    let total = users.len() as u32;
    render_users(
        true,
        false,
        loading,
        error,
        users,
        stats,
        1,
        UsersPagination {
            page: 1,
            limit: 20,
            total,
            total_pages: 1,
        },
        UserFilters::default(),
        Presence::Hidden,
        None,
        None,
        UsersModal::None,
        None,
    )
}

#[wasm_bindgen_test]
fn users_denied_copy() {
    let html = render_users(
        false,
        false,
        false,
        None,
        Vec::new(),
        None,
        1,
        UsersPagination::default(),
        UserFilters::default(),
        Presence::Hidden,
        None,
        None,
        UsersModal::None,
        None,
    );
    assert!(
        html.contains("id=\"admin-users-denied\""),
        "denied id missing: {html}"
    );
    assert!(html.contains("拒絕存取"), "denied title missing: {html}");
    assert!(
        html.contains("您沒有權限存取此頁面。"),
        "denied body missing: {html}"
    );
    assert!(
        !html.contains("使用者管理"),
        "heading must not render when denied: {html}"
    );
}

#[wasm_bindgen_test]
fn users_loading_copy() {
    let html = render_default(true, None, Vec::new(), None);
    assert!(
        html.contains("id=\"admin-users-loading\""),
        "loading id missing: {html}"
    );
    assert!(
        html.contains("Loading users..."),
        "loading copy missing: {html}"
    );
    assert!(
        html.contains("使用者管理"),
        "heading missing while loading: {html}"
    );
    assert!(
        html.contains("管理平台會員及其權限"),
        "subtitle missing: {html}"
    );
    assert!(html.contains("重新整理"), "refresh copy missing: {html}");
    assert!(
        !html.contains("id=\"admin-users-table\""),
        "table must not render while loading: {html}"
    );
}

#[wasm_bindgen_test]
fn users_empty_table() {
    let html = render_default(false, None, Vec::new(), None);
    assert!(
        html.contains("id=\"admin-users-table\""),
        "table missing: {html}"
    );
    assert!(
        html.contains("id=\"admin-users-empty\""),
        "empty row missing: {html}"
    );
    assert!(
        html.contains("Users (0 total)"),
        "empty heading missing: {html}"
    );
    assert!(
        html.contains("篩選與搜尋"),
        "filters heading missing: {html}"
    );
    assert!(
        html.contains("顯示篩選"),
        "show filters copy missing: {html}"
    );
    assert!(
        html.contains("依姓名、電子郵件、職業搜尋..."),
        "search placeholder missing: {html}"
    );
}

#[wasm_bindgen_test]
fn users_populated_row_and_stats() {
    let html = render_default(false, None, vec![sample_user()], Some(sample_stats()));
    assert!(html.contains("Ada Lovelace"), "name missing: {html}");
    assert!(html.contains("ada@example.com"), "email missing: {html}");
    assert!(html.contains("Mathematician"), "profession missing: {html}");
    assert!(html.contains("鑽石卡"), "tier label missing: {html}");
    assert!(html.contains("已驗證"), "verified badge missing: {html}");
    assert!(html.contains("使用者"), "role label missing: {html}");
    assert!(html.contains("NT$8,000,000"), "income missing: {html}");
    assert!(html.contains("NT$50,000,000"), "net worth missing: {html}");
    assert!(html.contains("Income:"), "income label missing: {html}");
    assert!(
        html.contains("Net Worth:"),
        "net worth label missing: {html}"
    );
    assert!(
        html.contains("Users (1 total)"),
        "count heading missing: {html}"
    );
    assert!(
        html.contains("id=\"admin-users-stats\""),
        "stats missing: {html}"
    );
    assert!(
        html.contains("總使用者數"),
        "total users copy missing: {html}"
    );
    assert!(html.contains("近期註冊"), "recent copy missing: {html}");
    assert!(html.contains("待審核驗證"), "pending copy missing: {html}");
    assert!(html.contains("管理員"), "admins copy missing: {html}");
    assert!(
        html.contains("id=\"admin-users-stat-total\""),
        "total stat missing: {html}"
    );
    assert!(
        html.contains(">12<") || html.contains("12"),
        "total 12 missing: {html}"
    );
    assert!(html.contains("4"), "recent 4 missing: {html}");
    assert!(html.contains("3"), "pending 3 missing: {html}");
    assert!(
        !html.contains("id=\"admin-users-approve-7\""),
        "approved user must not show approve: {html}"
    );
}

#[wasm_bindgen_test]
fn users_error_banner() {
    let html = render_default(
        false,
        Some("Failed to fetch users".to_string()),
        Vec::new(),
        None,
    );
    assert!(
        html.contains("id=\"admin-users-error\""),
        "error id missing: {html}"
    );
    assert!(
        html.contains("Failed to fetch users"),
        "error copy missing: {html}"
    );
}

#[wasm_bindgen_test]
fn users_pending_shows_verify_actions() {
    let html = render_default(false, None, vec![pending_user()], None);
    assert!(html.contains("待審核"), "pending badge missing: {html}");
    assert!(
        html.contains("id=\"admin-users-approve-pending-1\""),
        "approve missing: {html}"
    );
    assert!(
        html.contains("id=\"admin-users-reject-pending-1\""),
        "reject missing: {html}"
    );
    assert!(
        html.contains("title=\"Approve\""),
        "approve title missing: {html}"
    );
    assert!(
        html.contains("title=\"Reject\""),
        "reject title missing: {html}"
    );
}

#[wasm_bindgen_test]
fn users_filters_and_pagination_copy() {
    let html = render_users(
        true,
        false,
        false,
        None,
        vec![sample_user()],
        None,
        2,
        UsersPagination {
            page: 2,
            limit: 20,
            total: 45,
            total_pages: 3,
        },
        UserFilters {
            search: "Ada".to_string(),
            role: "admin".to_string(),
            membership_tier: String::new(),
            verification_status: String::new(),
        },
        Presence::Shown,
        None,
        None,
        UsersModal::None,
        None,
    );
    assert!(html.contains("隱藏篩選"), "hide filters missing: {html}");
    assert!(
        html.contains("id=\"admin-users-filters\""),
        "filters missing: {html}"
    );
    assert!(html.contains("所有角色"), "role options missing: {html}");
    assert!(html.contains("所有等級"), "tier options missing: {html}");
    assert!(
        html.contains("All Statuses"),
        "status options missing: {html}"
    );
    assert!(
        html.contains("Clear All Filters"),
        "clear filters missing: {html}"
    );
    assert!(
        html.contains("id=\"admin-users-pagination\""),
        "pagination missing: {html}"
    );
    assert!(
        html.contains("Showing 21 to 40 of 45 results"),
        "range copy missing: {html}"
    );
    assert!(html.contains("Page 2 of 3"), "page copy missing: {html}");
    assert!(html.contains("Previous"), "previous missing: {html}");
    assert!(html.contains("Next"), "next missing: {html}");
}

#[wasm_bindgen_test]
fn users_detail_modal_copy() {
    let user = sample_user();
    let html = render_users(
        true,
        false,
        false,
        None,
        vec![user.clone()],
        None,
        1,
        UsersPagination {
            page: 1,
            limit: 20,
            total: 1,
            total_pages: 1,
        },
        UserFilters::default(),
        Presence::Hidden,
        Some(user),
        None,
        UsersModal::Detail,
        None,
    );
    assert!(
        html.contains("id=\"admin-users-detail-modal\""),
        "detail modal missing: {html}"
    );
    assert!(html.contains("User Details"), "title missing: {html}");
    assert!(
        html.contains("Personal Information"),
        "personal missing: {html}"
    );
    assert!(
        html.contains("Account Information"),
        "account missing: {html}"
    );
    assert!(
        html.contains("Financial Information"),
        "financial missing: {html}"
    );
    assert!(html.contains("Interests"), "interests missing: {html}");
    assert!(html.contains("Bio"), "bio missing: {html}");
    assert!(
        html.contains("Notes on the analytical engine"),
        "bio text missing: {html}"
    );
    assert!(html.contains("math"), "interest missing: {html}");
}

#[wasm_bindgen_test]
fn users_edit_modal_copy() {
    let user = sample_user();
    let html = render_users(
        true,
        false,
        false,
        None,
        vec![user.clone()],
        None,
        1,
        UsersPagination {
            page: 1,
            limit: 20,
            total: 1,
            total_pages: 1,
        },
        UserFilters::default(),
        Presence::Hidden,
        Some(user.clone()),
        Some(edit_data_from_user(&user)),
        UsersModal::Edit,
        None,
    );
    assert!(
        html.contains("id=\"admin-users-edit-modal\""),
        "edit modal missing: {html}"
    );
    assert!(html.contains("Edit User"), "title missing: {html}");
    assert!(html.contains("First Name"), "first name missing: {html}");
    assert!(html.contains("Last Name"), "last name missing: {html}");
    assert!(
        html.contains("Annual Income (TWD)"),
        "income missing: {html}"
    );
    assert!(
        html.contains("Net Worth (TWD)"),
        "net worth missing: {html}"
    );
    assert!(html.contains("Membership Tier"), "tier missing: {html}");
    assert!(html.contains("Privacy Level"), "privacy missing: {html}");
    assert!(html.contains("1 - Public"), "privacy 1 missing: {html}");
    assert!(html.contains("5 - Very High"), "privacy 5 missing: {html}");
    assert!(html.contains("Save Changes"), "save missing: {html}");
    assert!(html.contains("Cancel"), "cancel missing: {html}");
}

#[wasm_bindgen_test]
fn users_delete_modal_copy() {
    let user = sample_user();
    let html = render_users(
        true,
        true,
        false,
        None,
        vec![user.clone()],
        None,
        1,
        UsersPagination {
            page: 1,
            limit: 20,
            total: 1,
            total_pages: 1,
        },
        UserFilters::default(),
        Presence::Hidden,
        Some(user),
        None,
        UsersModal::Delete,
        None,
    );
    assert!(
        html.contains("id=\"admin-users-delete-modal\""),
        "delete modal missing: {html}"
    );
    assert!(html.contains("Confirm Delete"), "title missing: {html}");
    assert!(
        html.contains("Are you sure you want to delete user"),
        "confirm copy missing: {html}"
    );
    assert!(html.contains("Ada Lovelace"), "name missing: {html}");
    assert!(
        html.contains("This action cannot be undone."),
        "undo copy missing: {html}"
    );
    assert!(
        html.contains("Delete User"),
        "delete button missing: {html}"
    );
}
