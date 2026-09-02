#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::permissions::{AuthSnapshot, Role, RouteGuard};
use hesocial_frontend::users::{
    DELETE_ACTION, DELETE_FALLBACK, EDIT_ACTION, NETWORK_ERROR, NETWORK_ERROR_ZH, PAGE_SIZE,
    ROLE_FALLBACK, STATS_FETCH_FALLBACK, UPDATE_FALLBACK, USER_FETCH_FALLBACK,
    USER_MANAGEMENT_FALLBACK, USER_STATS_API_PATH, USERS_API_PATH, USERS_FETCH_FALLBACK,
    UserFilters, UserStats, VERIFY_FALLBACK, VerificationBadge, action_is, admin_count,
    display_name, edit_data_from_user, edit_payload, filters_are_active, format_currency,
    format_joined_date, is_valid_membership_tier, is_valid_privacy_level, is_valid_role,
    is_valid_verify_status, membership_tier_badge_class, membership_tier_label,
    page_after_filter_change, page_in_range, pagination_range, parse_delete_response,
    parse_edit_int, parse_role_response, parse_update_response, parse_user_response,
    parse_user_stats_response, parse_users_response, parse_verify_response,
    pending_verification_count, role_action_key, role_badge_class, role_label, role_payload,
    shows_verify_actions, total_pages, user_api_path, user_initials, user_management_guard,
    user_role_api_path, user_verify_api_path, users_list_url, users_query_string,
    verification_badge, verification_badge_class, verification_badge_label, verify_action_key,
    verify_payload,
};

fn snake_list_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": "7",
                "email": "ada@example.com",
                "first_name": "Ada",
                "last_name": "Lovelace",
                "age": 36,
                "profession": "Mathematician",
                "annual_income": 8000000,
                "net_worth": 50000000,
                "membership_tier": "Diamond",
                "privacy_level": 3,
                "is_verified": 1,
                "verification_status": "approved",
                "role": "user",
                "profile_picture": "ada.jpg",
                "bio": "Notes",
                "interests": ["math", "computing"],
                "created_at": "2024-03-05T10:00:00.000Z",
                "updated_at": "2024-03-06T10:00:00.000Z"
            }
        ],
        "pagination": {
            "page": 2,
            "limit": 20,
            "total": 21,
            "totalPages": 2
        }
    }"#
    .to_string()
}

fn pending_user_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": "pending-1",
                "email": "pending@example.com",
                "first_name": "Pat",
                "last_name": "Pending",
                "age": 40,
                "profession": "Investor",
                "annual_income": 6000000,
                "net_worth": 40000000,
                "membership_tier": "Platinum",
                "privacy_level": 2,
                "is_verified": false,
                "verification_status": "pending",
                "role": "user",
                "profile_picture": null,
                "bio": null,
                "interests": [],
                "created_at": "2024-01-15T00:00:00.000Z",
                "updated_at": "2024-01-15T00:00:00.000Z"
            }
        ],
        "pagination": {
            "page": 1,
            "limit": 20,
            "total": 1,
            "totalPages": 1
        }
    }"#
    .to_string()
}

fn stats_body() -> String {
    r#"{
        "success": true,
        "data": {
            "totalUsers": 12,
            "usersByRole": [
                {"role": "user", "count": 9},
                {"role": "admin", "count": 2},
                {"role": "super_admin", "count": 1}
            ],
            "usersByMembershipTier": [
                {"membership_tier": "Platinum", "count": 7},
                {"membership_tier": "Diamond", "count": 4},
                {"membership_tier": "Black Card", "count": 1}
            ],
            "usersByVerificationStatus": [
                {"verification_status": "pending", "count": 3},
                {"verification_status": "approved", "count": 8},
                {"verification_status": "rejected", "count": 1}
            ],
            "recentRegistrations": 4
        }
    }"#
    .to_string()
}

fn snapshot(role: Option<Role>, authenticated: bool) -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated: authenticated,
        role,
        ..AuthSnapshot::default()
    }
}

#[test]
fn page_size_matches_react() {
    assert_eq!(PAGE_SIZE, 20);
}

#[test]
fn api_paths_match_backend_worker_routes() {
    assert_eq!(USERS_API_PATH, "/api/users");
    assert_eq!(USER_STATS_API_PATH, "/api/users/stats/overview");
    assert_eq!(user_api_path("7"), "/api/users/7");
    assert_eq!(user_verify_api_path("7"), "/api/users/7/verify");
    assert_eq!(user_role_api_path("7"), "/api/users/7/role");
}

#[test]
fn query_string_omits_empty_filters() {
    let query = users_query_string(1, PAGE_SIZE, &UserFilters::default());
    assert_eq!(query, "page=1&limit=20");
    assert_eq!(
        users_list_url(1, &UserFilters::default()),
        "/api/users?page=1&limit=20"
    );
}

#[test]
fn query_string_encodes_every_active_filter() {
    let filters = UserFilters {
        search: "Ada Lovelace".to_string(),
        role: "admin".to_string(),
        membership_tier: "Black Card".to_string(),
        verification_status: "pending".to_string(),
    };
    let query = users_query_string(2, PAGE_SIZE, &filters);
    assert_eq!(
        query,
        "page=2&limit=20&search=Ada+Lovelace&role=admin&membershipTier=Black+Card&verificationStatus=pending"
    );
}

#[test]
fn query_string_omits_blank_optional_filters() {
    let filters = UserFilters {
        search: String::new(),
        role: "user".to_string(),
        membership_tier: String::new(),
        verification_status: "approved".to_string(),
    };
    let query = users_query_string(1, PAGE_SIZE, &filters);
    assert_eq!(
        query,
        "page=1&limit=20&role=user&verificationStatus=approved"
    );
    assert!(!query.contains("search="));
    assert!(!query.contains("membershipTier="));
}

#[test]
fn filter_change_resets_to_first_page() {
    assert_eq!(page_after_filter_change(4), 1);
}

#[test]
fn filters_are_active_when_any_field_is_set() {
    assert!(!filters_are_active(&UserFilters::default()));
    assert!(filters_are_active(&UserFilters {
        search: "ada".to_string(),
        ..UserFilters::default()
    }));
    assert!(filters_are_active(&UserFilters {
        role: "admin".to_string(),
        ..UserFilters::default()
    }));
    assert!(filters_are_active(&UserFilters {
        membership_tier: "Diamond".to_string(),
        ..UserFilters::default()
    }));
    assert!(filters_are_active(&UserFilters {
        verification_status: "pending".to_string(),
        ..UserFilters::default()
    }));
}

#[test]
fn pagination_helpers_match_react_table_footer() {
    assert_eq!(total_pages(0, PAGE_SIZE), 0);
    assert_eq!(total_pages(20, PAGE_SIZE), 1);
    assert_eq!(total_pages(21, PAGE_SIZE), 2);
    assert_eq!(pagination_range(1, 20, 45), (1, 20));
    assert_eq!(pagination_range(3, 20, 45), (41, 45));
    assert_eq!(pagination_range(1, 20, 0), (1, 0));
    assert!(page_in_range(1, 2));
    assert!(page_in_range(2, 2));
    assert!(!page_in_range(0, 2));
    assert!(!page_in_range(3, 2));
    assert!(page_in_range(1, 0));
}

#[test]
fn parse_users_snake_case_backend_row_and_pagination() {
    let page = parse_users_response(200, &snake_list_body()).expect("ok");
    assert_eq!(page.users.len(), 1);
    let user = &page.users[0];
    assert_eq!(user.id, "7");
    assert_eq!(user.email, "ada@example.com");
    assert_eq!(user.first_name, "Ada");
    assert_eq!(user.last_name, "Lovelace");
    assert_eq!(user.age, 36);
    assert_eq!(user.profession, "Mathematician");
    assert_eq!(user.annual_income, 8_000_000);
    assert_eq!(user.net_worth, 50_000_000);
    assert_eq!(user.membership_tier, "Diamond");
    assert_eq!(user.privacy_level, 3);
    assert!(user.is_verified);
    assert_eq!(user.verification_status, "approved");
    assert_eq!(user.role, "user");
    assert_eq!(user.profile_picture.as_deref(), Some("ada.jpg"));
    assert_eq!(user.bio.as_deref(), Some("Notes"));
    assert_eq!(
        user.interests,
        vec!["math".to_string(), "computing".to_string()]
    );
    assert_eq!(user.created_at, "2024-03-05T10:00:00.000Z");
    assert_eq!(page.pagination.page, 2);
    assert_eq!(page.pagination.limit, 20);
    assert_eq!(page.pagination.total, 21);
    assert_eq!(page.pagination.total_pages, 2);
}

#[test]
fn parse_users_camel_case_and_interests_json_string() {
    let body = r#"{
        "success": true,
        "data": [{
            "id": "8",
            "email": "cam@example.com",
            "firstName": "Cam",
            "lastName": "El",
            "age": 41,
            "profession": "CEO",
            "annualIncome": 9000000,
            "netWorth": 80000000,
            "membershipTier": "Black Card",
            "privacyLevel": 5,
            "isVerified": true,
            "verificationStatus": "approved",
            "role": "admin",
            "profilePicture": null,
            "bio": null,
            "interests": "[\"yachts\",\"art\"]",
            "createdAt": "2024-02-01T00:00:00.000Z",
            "updatedAt": "2024-02-01T00:00:00.000Z"
        }],
        "pagination": {"page": 1, "limit": 20, "total": 1, "totalPages": 1}
    }"#;
    let page = parse_users_response(200, body).expect("ok");
    assert_eq!(page.users[0].first_name, "Cam");
    assert_eq!(page.users[0].membership_tier, "Black Card");
    assert_eq!(page.users[0].annual_income, 9_000_000);
    assert_eq!(
        page.users[0].interests,
        vec!["yachts".to_string(), "art".to_string()]
    );
}

#[test]
fn parse_users_empty_array() {
    let body = r#"{
        "success": true,
        "data": [],
        "pagination": {"page": 1, "limit": 20, "total": 0, "totalPages": 0}
    }"#;
    let page = parse_users_response(200, body).expect("ok");
    assert!(page.users.is_empty());
    assert_eq!(page.pagination.total, 0);
    assert_eq!(page.pagination.total_pages, 0);
}

#[test]
fn parse_users_error_empty_and_malformed() {
    assert_eq!(
        parse_users_response(200, "not-json").unwrap_err(),
        USERS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_users_response(200, r#"{"success":false}"#).unwrap_err(),
        USERS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_users_response(200, r#"{"success":false,"error":"nope"}"#).unwrap_err(),
        "nope"
    );
    assert_eq!(
        parse_users_response(200, r#"{"success":true}"#).unwrap_err(),
        USERS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_users_response(200, r#"{"success":true,"data":null}"#).unwrap_err(),
        USERS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_users_response(0, "").unwrap_err(),
        USERS_FETCH_FALLBACK
    );
}

#[test]
fn parse_users_401_and_403_use_backend_error_strings() {
    assert_eq!(
        parse_users_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_users_response(401, "").unwrap_err(),
        USERS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_users_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
    assert_eq!(
        parse_users_response(403, r#"{"success":false}"#).unwrap_err(),
        USERS_FETCH_FALLBACK
    );
}

#[test]
fn parse_user_stats_overview_and_derived_counts() {
    let stats = parse_user_stats_response(200, &stats_body()).expect("ok");
    assert_eq!(stats.total_users, 12);
    assert_eq!(stats.recent_registrations, 4);
    assert_eq!(pending_verification_count(&stats), 3);
    assert_eq!(admin_count(&stats), 3);
    assert_eq!(
        stats.users_by_membership_tier[0].membership_tier,
        "Platinum"
    );
}

#[test]
fn parse_user_stats_empty_breakdowns_and_errors() {
    let empty = parse_user_stats_response(
        200,
        r#"{
            "success": true,
            "data": {
                "totalUsers": 0,
                "usersByRole": [],
                "usersByMembershipTier": [],
                "usersByVerificationStatus": [],
                "recentRegistrations": 0
            }
        }"#,
    )
    .expect("ok");
    assert_eq!(empty.total_users, 0);
    assert_eq!(pending_verification_count(&empty), 0);
    assert_eq!(admin_count(&empty), 0);
    assert_eq!(
        parse_user_stats_response(200, "bad").unwrap_err(),
        STATS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_user_stats_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_user_stats_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
}

#[test]
fn parse_single_user_and_not_found() {
    let body = r#"{
        "success": true,
        "data": {
            "id": "7",
            "email": "ada@example.com",
            "first_name": "Ada",
            "last_name": "Lovelace",
            "age": 36,
            "profession": "Mathematician",
            "annual_income": 1,
            "net_worth": 2,
            "membership_tier": "Diamond",
            "privacy_level": 3,
            "is_verified": true,
            "verification_status": "approved",
            "role": "user",
            "interests": [],
            "created_at": "2024-03-05T10:00:00.000Z",
            "updated_at": "2024-03-05T10:00:00.000Z"
        }
    }"#;
    let user = parse_user_response(200, body).expect("ok");
    assert_eq!(user.id, "7");
    assert_eq!(
        parse_user_response(404, r#"{"success":false,"error":"User not found"}"#).unwrap_err(),
        "User not found"
    );
    assert_eq!(
        parse_user_response(200, r#"{"success":true,"data":null}"#).unwrap_err(),
        USER_FETCH_FALLBACK
    );
}

#[test]
fn parse_update_verify_role_delete_envelopes() {
    parse_update_response(
        200,
        r#"{"success":true,"message":"User updated successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_update_response(
            400,
            r#"{"success":false,"error":"No valid fields to update"}"#
        )
        .unwrap_err(),
        "No valid fields to update"
    );
    assert_eq!(
        parse_update_response(200, "bad").unwrap_err(),
        UPDATE_FALLBACK
    );

    parse_verify_response(
        200,
        r#"{"success":true,"message":"User verified successfully"}"#,
    )
    .expect("ok");
    parse_verify_response(
        200,
        r#"{"success":true,"message":"User rejected successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_verify_response(
            400,
            r#"{"success":false,"error":"Invalid verification status"}"#
        )
        .unwrap_err(),
        "Invalid verification status"
    );
    assert_eq!(
        parse_verify_response(404, r#"{"success":false,"error":"User not found"}"#).unwrap_err(),
        "User not found"
    );
    assert_eq!(
        parse_verify_response(200, "bad").unwrap_err(),
        VERIFY_FALLBACK
    );

    parse_role_response(
        200,
        r#"{"success":true,"message":"User role updated successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_role_response(400, r#"{"success":false,"error":"Invalid role"}"#).unwrap_err(),
        "Invalid role"
    );
    assert_eq!(
        parse_role_response(
            403,
            r#"{"success":false,"error":"Super admin access required"}"#
        )
        .unwrap_err(),
        "Super admin access required"
    );
    assert_eq!(parse_role_response(200, "bad").unwrap_err(), ROLE_FALLBACK);

    parse_delete_response(
        200,
        r#"{"success":true,"message":"User deleted successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_delete_response(
            403,
            r#"{"success":false,"error":"Super admin access required"}"#
        )
        .unwrap_err(),
        "Super admin access required"
    );
    assert_eq!(
        parse_delete_response(200, "bad").unwrap_err(),
        DELETE_FALLBACK
    );
}

#[test]
fn verification_approval_flow_states() {
    assert!(shows_verify_actions("pending"));
    assert!(!shows_verify_actions("approved"));
    assert!(!shows_verify_actions("rejected"));
    assert_eq!(
        verification_badge("approved", true),
        VerificationBadge::Approved
    );
    assert_eq!(
        verification_badge("approved", false),
        VerificationBadge::Pending
    );
    assert_eq!(
        verification_badge("rejected", false),
        VerificationBadge::Rejected
    );
    assert_eq!(
        verification_badge("pending", false),
        VerificationBadge::Pending
    );
    assert_eq!(
        verification_badge_label(VerificationBadge::Approved),
        "已驗證"
    );
    assert_eq!(
        verification_badge_label(VerificationBadge::Rejected),
        "已拒絕"
    );
    assert_eq!(
        verification_badge_label(VerificationBadge::Pending),
        "待審核"
    );
    assert!(verification_badge_class(VerificationBadge::Approved).contains("bg-green-100"));
    assert!(verification_badge_class(VerificationBadge::Rejected).contains("bg-red-100"));
    assert!(verification_badge_class(VerificationBadge::Pending).contains("bg-yellow-100"));

    let pending = parse_users_response(200, &pending_user_body()).expect("ok");
    assert!(shows_verify_actions(&pending.users[0].verification_status));
    let key = verify_action_key(&pending.users[0].id);
    assert_eq!(key, "verify-pending-1");
    assert!(action_is(Some(&key), &key));
    assert!(!action_is(None, &key));
    assert!(!action_is(Some(EDIT_ACTION), &key));
    assert_eq!(
        verify_payload("approved"),
        serde_json::json!({"status":"approved"})
    );
    assert_eq!(
        verify_payload("rejected"),
        serde_json::json!({"status":"rejected"})
    );
}

#[test]
fn edit_payload_is_camel_case_whitelist() {
    let page = parse_users_response(200, &snake_list_body()).expect("ok");
    let data = edit_data_from_user(&page.users[0]);
    assert_eq!(data.first_name, "Ada");
    assert_eq!(data.bio, "Notes");
    assert_eq!(
        edit_payload(&data),
        serde_json::json!({
            "firstName": "Ada",
            "lastName": "Lovelace",
            "age": 36,
            "profession": "Mathematician",
            "annualIncome": 8000000,
            "netWorth": 50000000,
            "membershipTier": "Diamond",
            "privacyLevel": 3,
            "bio": "Notes",
            "interests": ["math", "computing"]
        })
    );
    assert_eq!(parse_edit_int("42"), Some(42));
    assert_eq!(parse_edit_int(" 7 "), Some(7));
    assert_eq!(parse_edit_int(""), None);
    assert_eq!(parse_edit_int("abc"), None);
}

#[test]
fn role_and_field_validation() {
    assert!(is_valid_verify_status("approved"));
    assert!(is_valid_verify_status("rejected"));
    assert!(!is_valid_verify_status("pending"));
    assert!(!is_valid_verify_status(""));
    assert!(is_valid_role("user"));
    assert!(is_valid_role("admin"));
    assert!(is_valid_role("super_admin"));
    assert!(!is_valid_role("moderator"));
    assert!(is_valid_membership_tier("Platinum"));
    assert!(is_valid_membership_tier("Diamond"));
    assert!(is_valid_membership_tier("Black Card"));
    assert!(!is_valid_membership_tier("Gold"));
    assert!(is_valid_privacy_level(1));
    assert!(is_valid_privacy_level(5));
    assert!(!is_valid_privacy_level(0));
    assert!(!is_valid_privacy_level(6));
    assert_eq!(role_payload("admin"), serde_json::json!({"role":"admin"}));
    assert_eq!(role_action_key("7"), "role-7");
    assert_eq!(EDIT_ACTION, "edit");
    assert_eq!(DELETE_ACTION, "delete");
}

#[test]
fn labels_currency_date_and_initials() {
    assert_eq!(membership_tier_label("Platinum"), "白金卡");
    assert_eq!(membership_tier_label("Diamond"), "鑽石卡");
    assert_eq!(membership_tier_label("Black Card"), "黑卡");
    assert_eq!(membership_tier_label("Other"), "Other");
    assert!(membership_tier_badge_class("Platinum").contains("bg-gray-100"));
    assert!(membership_tier_badge_class("Diamond").contains("bg-blue-100"));
    assert!(membership_tier_badge_class("Black Card").contains("bg-black"));
    assert_eq!(role_label("super_admin"), "超級管理員");
    assert_eq!(role_label("admin"), "管理員");
    assert_eq!(role_label("user"), "使用者");
    assert!(role_badge_class("admin").contains("bg-purple-100"));
    assert!(role_badge_class("super_admin").contains("bg-red-100"));
    assert_eq!(format_currency(8_000_000), "NT$8,000,000");
    assert_eq!(format_currency(0), "NT$0");
    assert_eq!(format_currency(-1200), "NT$-1,200");
    assert_eq!(format_joined_date("2024-03-05T10:00:00.000Z"), "2024/3/5");
    assert_eq!(format_joined_date("2024-12-15"), "2024/12/15");
    assert_eq!(user_initials("Ada", "Lovelace"), "AL");
    assert_eq!(user_initials("", ""), "");
    let page = parse_users_response(200, &snake_list_body()).expect("ok");
    assert_eq!(display_name(&page.users[0]), "Ada Lovelace");
}

#[test]
fn user_management_guard_three_states_pin_admin_fallback() {
    assert_eq!(USER_MANAGEMENT_FALLBACK, "/admin");
    assert_ne!(USER_MANAGEMENT_FALLBACK, "/login");
    assert_eq!(
        user_management_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading
    );
    assert_eq!(
        user_management_guard(false, &AuthSnapshot::default()),
        RouteGuard::Redirect("/admin")
    );
    assert_eq!(
        user_management_guard(false, &snapshot(Some(Role::User), true)),
        RouteGuard::Redirect("/admin")
    );
    assert_eq!(
        user_management_guard(false, &snapshot(Some(Role::Admin), true)),
        RouteGuard::Allow
    );
    assert_eq!(
        user_management_guard(false, &snapshot(Some(Role::SuperAdmin), true)),
        RouteGuard::Allow
    );
    match user_management_guard(false, &snapshot(None, false)) {
        RouteGuard::Redirect(path) => assert_eq!(path, USER_MANAGEMENT_FALLBACK),
        other => panic!("expected redirect to /admin, got {other:?}"),
    }
}

#[test]
fn network_error_copy_matches_react() {
    assert_eq!(NETWORK_ERROR, "Network error occurred");
    assert_eq!(NETWORK_ERROR_ZH, "發生網路錯誤");
    assert_eq!(USERS_FETCH_FALLBACK, "Failed to fetch users");
    assert_eq!(UPDATE_FALLBACK, "Failed to update user");
    assert_eq!(VERIFY_FALLBACK, "Failed to verify user");
    assert_eq!(ROLE_FALLBACK, "更新使用者角色失敗");
    assert_eq!(DELETE_FALLBACK, "刪除使用者失敗");
}

#[test]
fn unused_stats_struct_keeps_zero_defaults() {
    let stats = UserStats::default();
    assert_eq!(stats.total_users, 0);
    assert_eq!(pending_verification_count(&stats), 0);
    assert_eq!(admin_count(&stats), 0);
}
