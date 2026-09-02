use hesocial_frontend::auth::{
    LOGIN_FAILED_FALLBACK, TOKEN_STORAGE_KEY, VALIDATE_API_PATH, apply_complete_profile_redirect,
    bearer_authorization, boot_claim_oauth, display_login_error, extract_oauth_token,
    parse_login_response, parse_validate_response, password_input_type, session_after_validate,
};
use hesocial_frontend::events::{
    EventFilters, PAGE_LIMIT, PLACEHOLDER_IMAGE, collapse_on_error, detail_exclusivity_color,
    dress_code_text, events_query_string, exclusivity_color, exclusivity_label, first_image,
    format_price, gallery_image, is_full, occupancy_percent, page_after_filter_change,
    page_in_range, parse_event_detail_response, parse_events_response, price_kind_label,
    shows_diamond, spots_remaining, star_count, venue_star_count, wrap_image_index,
};
use hesocial_frontend::logic::{next_toggled, toggle_label};
use hesocial_frontend::permissions::{
    AuthSnapshot, MembershipTier, Role, RouteGuard, USER_ROUTE_FALLBACK, VerificationStatus,
    permissions, user_route_guard,
};
use hesocial_frontend::profile::{
    PROFILE_API_PATH, display_age, display_full_name, display_optional, display_optional_i64,
    display_privacy_level, membership_benefits, membership_color_class, parse_profile_response,
    profile_picture_src,
};
use hesocial_frontend::register::{
    REGISTER_AGE_RANGE, REGISTER_API_PATH, REGISTER_FAILED_FALLBACK, REGISTER_INCOME_ASSET,
    REGISTER_INCOME_MIN, REGISTER_INTEREST_REQUIRED, REGISTER_NETWORTH_MIN,
    REGISTER_PASSWORD_MISMATCH, REGISTER_PASSWORD_SHORT, REGISTER_REQUIRED, RegisterForm,
    parse_register_response, push_interest, registration_payload, remove_interest, step_title,
    validate_step,
};
use hesocial_frontend::shell::{
    Presence, SessionKind, is_active_path, presence_after_animation_end, presence_is_mounted,
    presence_toggle, primary_nav_items, session_entries,
};

#[test]
fn off_state_uses_off_label() {
    assert_eq!(toggle_label(false), "Off");
}

#[test]
fn on_state_uses_on_label() {
    assert_eq!(toggle_label(true), "On");
}

#[test]
fn toggling_flips_boolean_state() {
    assert!(next_toggled(false));
    assert!(!next_toggled(true));
}

#[test]
fn password_is_masked_until_toggled() {
    assert_eq!(password_input_type(false), "password");
    assert_eq!(password_input_type(true), "text");
}

#[test]
fn authenticated_requests_use_hesocial_token_bearer_header() {
    assert_eq!(TOKEN_STORAGE_KEY, "hesocial_token");
    assert_eq!(bearer_authorization("abc.def.ghi"), "Bearer abc.def.ghi");
}

#[test]
fn login_error_prefers_api_error_string() {
    assert_eq!(
        display_login_error(Some("Invalid email or password")),
        "Invalid email or password"
    );
}

#[test]
fn login_error_falls_back_when_api_error_missing() {
    assert_eq!(display_login_error(None), LOGIN_FAILED_FALLBACK);
    assert_eq!(display_login_error(Some("")), LOGIN_FAILED_FALLBACK);
    assert_eq!(display_login_error(Some("   ")), LOGIN_FAILED_FALLBACK);
}

#[test]
fn parse_login_success_extracts_token() {
    let body = r#"{"success":true,"data":{"token":"jwt-1","user":{"email":"a@b.c"}}}"#;
    let ok = parse_login_response(body).expect("success body");
    assert_eq!(ok.token, "jwt-1");
}

#[test]
fn parse_login_uses_backend_401_error_string() {
    let body = r#"{"success":false,"error":"Invalid email or password"}"#;
    let err = parse_login_response(body).expect_err("401 body");
    assert_eq!(err, "Invalid email or password");
}

#[test]
fn parse_login_missing_error_uses_chinese_fallback() {
    let body = r#"{"success":false}"#;
    let err = parse_login_response(body).expect_err("empty error");
    assert_eq!(err, LOGIN_FAILED_FALLBACK);
}

#[test]
fn parse_login_malformed_body_uses_chinese_fallback() {
    assert_eq!(
        parse_login_response("not-json").unwrap_err(),
        LOGIN_FAILED_FALLBACK
    );
}

#[test]
fn extract_oauth_token_from_complete_profile_query() {
    assert_eq!(
        extract_oauth_token("/complete-profile?token=jwt-from-google").as_deref(),
        Some("jwt-from-google")
    );
    assert_eq!(
        extract_oauth_token("?token=jwt-from-google").as_deref(),
        Some("jwt-from-google")
    );
    assert_eq!(extract_oauth_token("/complete-profile"), None);
    assert_eq!(extract_oauth_token("/profile"), None);
}

#[test]
fn complete_profile_redirect_drops_the_query_string() {
    assert_eq!(
        apply_complete_profile_redirect("/complete-profile?token=jwt-from-google"),
        "/profile"
    );
    assert_eq!(
        extract_oauth_token("/profile"),
        None,
        "reading the token after the redirect loses it"
    );
}

#[test]
fn boot_claims_oauth_token_before_complete_profile_redirect() {
    let claimed = boot_claim_oauth("/complete-profile?token=jwt-from-google");
    assert_eq!(
        claimed.token.as_deref(),
        Some("jwt-from-google"),
        "token must be taken from the landing URL, not from the post-redirect path"
    );
    assert_eq!(claimed.route, "/profile");
    assert_ne!(claimed.route, "/login");
}

fn default_filters() -> EventFilters {
    EventFilters {
        page: 1,
        limit: PAGE_LIMIT,
        search: String::new(),
        category: "all".into(),
        exclusivity_level: "all".into(),
    }
}

#[test]
fn events_query_omits_blank_and_all_filters() {
    assert_eq!(events_query_string(&default_filters()), "page=1&limit=9");
}

#[test]
fn events_query_appends_search_category_and_level() {
    let filters = EventFilters {
        page: 2,
        limit: 9,
        search: "Yacht".into(),
        category: "yacht".into(),
        exclusivity_level: "VIP".into(),
    };
    assert_eq!(
        events_query_string(&filters),
        "page=2&limit=9&search=Yacht&category=yacht&exclusivityLevel=VIP"
    );
}

#[test]
fn events_query_encodes_search_like_urlsearchparams() {
    let mut filters = default_filters();
    filters.search = "松露 私宴".into();
    assert_eq!(
        events_query_string(&filters),
        "page=1&limit=9&search=%E6%9D%BE%E9%9C%B2+%E7%A7%81%E5%AE%B4"
    );
}

#[test]
fn changing_any_filter_resets_to_page_one() {
    assert_eq!(page_after_filter_change(1), 1);
    assert_eq!(page_after_filter_change(4), 1);
    assert_eq!(page_after_filter_change(99), 1);
}

#[test]
fn pagination_range_matches_react_handle_page_change() {
    assert!(!page_in_range(0, 3));
    assert!(page_in_range(1, 3));
    assert!(page_in_range(3, 3));
    assert!(!page_in_range(4, 3));
    assert!(!page_in_range(1, 0));
}

#[test]
fn error_and_empty_collapse_keeps_requested_page() {
    let collapsed = collapse_on_error(4, 9);
    assert_eq!(collapsed.page, 4);
    assert_eq!(collapsed.limit, 9);
    assert_eq!(collapsed.total, 0);
    assert_eq!(collapsed.total_pages, 1);
}

#[test]
fn parse_success_uses_payload_and_stringifies_numeric_id() {
    let body = r#"{
        "success": true,
        "data": [{
            "id": 11,
            "name": "松露季私宴",
            "description": "白松露當季。",
            "dateTime": "2026-10-04T10:00:00.000Z",
            "venue": {"name": "Taipei Private Dining Room", "address": "Da'an", "rating": 5},
            "exclusivityLevel": null,
            "pricing": {"vip": 15000, "vvip": 15000, "currency": "TWD"},
            "currentAttendees": 0,
            "capacity": 12,
            "images": ["https://media.example/e11.webp"]
        }],
        "pagination": {"page": 1, "limit": 9, "total": 21, "totalPages": 3}
    }"#;
    let view = parse_events_response(body, 1, 9);
    assert_eq!(view.events.len(), 1);
    assert_eq!(view.events[0].id, "11");
    assert_eq!(view.events[0].name, "松露季私宴");
    assert_eq!(
        view.events[0].venue.as_ref().map(|v| v.name.as_str()),
        Some("Taipei Private Dining Room")
    );
    assert_eq!(view.events[0].exclusivity_level, None);
    assert_eq!(view.pagination.total, 21);
    assert_eq!(view.pagination.total_pages, 3);
}

#[test]
fn parse_success_false_collapses_like_react() {
    let view = parse_events_response(r#"{"success":false,"error":"nope"}"#, 3, 9);
    assert!(view.events.is_empty());
    assert_eq!(view.pagination.page, 3);
    assert_eq!(view.pagination.total, 0);
    assert_eq!(view.pagination.total_pages, 1);
}

#[test]
fn parse_malformed_body_collapses_like_react() {
    let view = parse_events_response("not-json", 2, 9);
    assert!(view.events.is_empty());
    assert_eq!(view.pagination.page, 2);
    assert_eq!(view.pagination.total, 0);
    assert_eq!(view.pagination.total_pages, 1);
}

#[test]
fn null_exclusivity_uses_gray_badge_and_no_icons() {
    assert_eq!(
        exclusivity_color(None),
        "bg-gray-500/20 text-gray-400 border-gray-500/30"
    );
    assert_eq!(exclusivity_label(None), "");
    assert_eq!(star_count(None), 0);
    assert!(!shows_diamond(None));
}

#[test]
fn exclusivity_badge_and_stars_match_react_branches() {
    assert_eq!(
        exclusivity_color(Some("VIP")),
        "bg-blue-500/20 text-blue-400 border-blue-500/30"
    );
    assert_eq!(
        exclusivity_color(Some("VVIP")),
        "bg-luxury-gold/20 text-luxury-gold border-luxury-gold/30"
    );
    assert_eq!(
        exclusivity_color(Some("僅限邀請")),
        "bg-purple-500/20 text-purple-400 border-purple-500/30"
    );
    assert_eq!(
        exclusivity_color(Some("Invitation Only")),
        "bg-gray-500/20 text-gray-400 border-gray-500/30"
    );
    assert_eq!(exclusivity_label(Some("VIP")), "VIP");
    assert_eq!(star_count(Some("VIP")), 2);
    assert_eq!(star_count(Some("VVIP")), 3);
    assert_eq!(star_count(Some("Invitation Only")), 3);
    assert_eq!(star_count(Some("僅限邀請")), 0);
    assert!(shows_diamond(Some("Invitation Only")));
    assert!(!shows_diamond(Some("僅限邀請")));
    assert!(!shows_diamond(Some("VVIP")));
}

#[test]
fn first_image_falls_back_to_placeholder() {
    assert_eq!(first_image(None), "/api/placeholder/400/300");
    assert_eq!(first_image(Some(&[])), "/api/placeholder/400/300");
    assert_eq!(
        first_image(Some(&["https://a.webp".into(), "https://b.webp".into()])),
        "https://a.webp"
    );
}

#[test]
fn price_prefers_vvip_then_vip_and_treats_zero_as_missing() {
    assert_eq!(format_price(Some(15000.0), Some(12000.0)), "NT$ 15,000");
    assert_eq!(format_price(None, Some(12000.0)), "NT$ 12,000");
    assert_eq!(format_price(Some(0.0), Some(8000.0)), "NT$ 8,000");
    assert_eq!(format_price(None, None), "價格洽詢");
}

#[test]
fn active_route_is_exact_pathname_match() {
    assert!(is_active_path("/", "/"));
    assert!(is_active_path("/events", "/events"));
    assert!(is_active_path("/vvip", "/vvip"));
    assert!(!is_active_path("/events", "/"));
    assert!(!is_active_path("/", "/events"));
    assert!(
        !is_active_path("/events/11", "/events"),
        "React uses location.pathname === path, not a prefix match"
    );
    assert!(!is_active_path("/events", "/events/11"));
    assert!(!is_active_path("/login", "/"));
}

#[test]
fn primary_nav_items_match_react_navbar() {
    let items = primary_nav_items();
    let pairs: Vec<(&str, &str)> = items.iter().map(|item| (item.name, item.path)).collect();
    assert_eq!(
        pairs,
        vec![
            ("首頁", "/"),
            ("精選活動", "/events"),
            ("VVIP專區", "/vvip")
        ]
    );
    assert!(items[0].icon.is_none());
    assert!(items[1].icon.is_none());
    assert_eq!(items[2].icon.as_deref(), Some("crown"));
}

fn kinds(entries: &[hesocial_frontend::shell::SessionEntry]) -> Vec<SessionKind> {
    entries.iter().map(|entry| entry.kind).collect()
}

#[test]
fn signed_out_session_yields_login_and_register() {
    let entries = session_entries(false, false);
    assert_eq!(
        kinds(&entries),
        vec![SessionKind::Login, SessionKind::Register]
    );
    assert_eq!(entries[0].href, Some("/login"));
    assert_eq!(entries[0].label, "登入");
    assert_eq!(entries[1].href, Some("/register"));
    assert_eq!(entries[1].label, "註冊");
}

#[test]
fn signed_in_session_yields_user_links_without_admin() {
    let entries = session_entries(true, false);
    assert_eq!(
        kinds(&entries),
        vec![
            SessionKind::Profile,
            SessionKind::Registrations,
            SessionKind::Logout,
        ]
    );
    assert_eq!(entries[0].href, Some("/profile"));
    assert_eq!(entries[0].label, "個人檔案");
    assert_eq!(entries[1].href, Some("/profile/registrations"));
    assert_eq!(entries[1].label, "我的報名");
    assert_eq!(entries[2].href, None);
    assert_eq!(entries[2].label, "登出");
    assert!(
        !kinds(&entries).contains(&SessionKind::Admin),
        "viewAdmin false must not emit admin entries"
    );
}

#[test]
fn admin_session_appends_the_four_admin_entries() {
    let entries = session_entries(true, true);
    assert_eq!(
        kinds(&entries),
        vec![
            SessionKind::Profile,
            SessionKind::Registrations,
            SessionKind::Admin,
            SessionKind::EventMgmt,
            SessionKind::Sales,
            SessionKind::SystemHealth,
            SessionKind::Logout,
        ]
    );
    assert_eq!(entries[2].href, Some("/admin"));
    assert_eq!(entries[2].label, "管理後台");
    assert_eq!(entries[3].href, Some("/event-mgmt"));
    assert_eq!(entries[3].label, "活動管理");
    assert_eq!(entries[4].href, Some("/admin/sales"));
    assert_eq!(entries[4].label, "銷售管理");
    assert_eq!(entries[5].href, Some("/admin/system"));
    assert_eq!(entries[5].label, "系統健康");
}

#[test]
fn view_admin_is_admin_or_super_admin_only() {
    let none = permissions(&AuthSnapshot::default());
    assert!(!none.view_admin);
    assert!(!none.access);

    let signed_out_admin_claim = permissions(&AuthSnapshot {
        is_authenticated: false,
        role: Some(Role::Admin),
        ..AuthSnapshot::default()
    });
    assert!(
        signed_out_admin_claim.view_admin,
        "React's isAdmin is role-level >= 2, independent of isAuthenticated"
    );
    assert!(!signed_out_admin_claim.access);

    let user = permissions(&AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        ..AuthSnapshot::default()
    });
    assert!(user.access);
    assert!(!user.view_admin);

    let no_role = permissions(&AuthSnapshot {
        is_authenticated: true,
        role: None,
        ..AuthSnapshot::default()
    });
    assert!(no_role.access);
    assert!(!no_role.view_admin);

    let admin = permissions(&AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::Admin),
        ..AuthSnapshot::default()
    });
    assert!(admin.view_admin);
    assert!(!admin.manage_super_admin);

    let super_admin = permissions(&AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::SuperAdmin),
        ..AuthSnapshot::default()
    });
    assert!(super_admin.view_admin);
    assert!(super_admin.manage_super_admin);
}

#[test]
fn permission_primitive_exposes_the_rest_of_the_react_can_flags() {
    let diamond_verified = permissions(&AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        membership_tier: Some(MembershipTier::Diamond),
        is_verified: true,
        verification_status: Some(VerificationStatus::Approved),
    });
    assert!(diamond_verified.access);
    assert!(!diamond_verified.view_admin);
    assert!(diamond_verified.access_vvip);
    assert!(diamond_verified.access_premium_events);
    assert!(!diamond_verified.access_exclusive_events);
    assert!(diamond_verified.upload_media);
    assert!(diamond_verified.register_for_events);
    assert!(diamond_verified.member_features);

    let black_unverified = permissions(&AuthSnapshot {
        is_authenticated: true,
        membership_tier: Some(MembershipTier::BlackCard),
        is_verified: false,
        verification_status: Some(VerificationStatus::Pending),
        ..AuthSnapshot::default()
    });
    assert!(!black_unverified.access_vvip);
    assert!(black_unverified.access_exclusive_events);
    assert!(!black_unverified.register_for_events);
}

#[test]
fn dropdown_presence_stays_mounted_through_exit() {
    assert_eq!(presence_toggle(Presence::Hidden), Presence::Entering);
    assert!(presence_is_mounted(Presence::Entering));
    assert_eq!(
        presence_after_animation_end(Presence::Entering),
        Presence::Shown
    );

    assert_eq!(presence_toggle(Presence::Shown), Presence::Exiting);
    assert!(
        presence_is_mounted(Presence::Exiting),
        "exit animation requires the node to stay mounted"
    );
    assert!(!presence_is_mounted(Presence::Hidden));
    assert_eq!(
        presence_after_animation_end(Presence::Exiting),
        Presence::Hidden
    );
    assert_eq!(presence_toggle(Presence::Exiting), Presence::Entering);
}

const ADMIN_VALIDATE_BODY: &str = r#"{
    "success": true,
    "data": {
        "user": {
            "id": "9",
            "email": "admin@hesocial.com",
            "firstName": "Admin",
            "lastName": "User",
            "role": "admin",
            "membershipTier": "Black Card",
            "isVerified": true,
            "verificationStatus": "approved"
        },
        "valid": true
    }
}"#;

const GOOGLE_PROFILE_BODY: &str = r#"{
    "success": true,
    "data": {
        "user": {
            "id": "g-1",
            "email": "google@example.com",
            "firstName": "Ada",
            "lastName": "Li",
            "age": null,
            "profession": null,
            "annualIncome": null,
            "netWorth": null,
            "membershipTier": "Platinum",
            "privacyLevel": 3,
            "isVerified": false,
            "verificationStatus": "pending",
            "role": "user",
            "profilePicture": null,
            "bio": null,
            "interests": null
        }
    }
}"#;

const COMPLETE_PROFILE_BODY: &str = r#"{
    "success": true,
    "data": {
        "user": {
            "id": "1",
            "email": "ok@example.com",
            "firstName": "Wei",
            "lastName": "Chen",
            "age": 42,
            "profession": "投資人",
            "annualIncome": 8000000,
            "netWorth": 50000000,
            "membershipTier": "Diamond",
            "privacyLevel": 4,
            "isVerified": true,
            "verificationStatus": "approved",
            "role": "user",
            "profilePicture": "https://media.example/p.jpg",
            "bio": "喜歡藝術與航海",
            "interests": ["藝術", "遊艇"]
        }
    }
}"#;

#[test]
fn validate_path_and_bearer_header_match_react() {
    assert_eq!(VALIDATE_API_PATH, "/api/auth/validate");
    assert_eq!(PROFILE_API_PATH, "/api/auth/profile");
    assert_eq!(TOKEN_STORAGE_KEY, "hesocial_token");
    assert_eq!(bearer_authorization("stored-jwt"), "Bearer stored-jwt");
}

#[test]
fn parse_validate_success_extracts_user_and_feeds_auth_snapshot() {
    let user = parse_validate_response(200, ADMIN_VALIDATE_BODY).expect("valid body");
    assert_eq!(user.email.as_deref(), Some("admin@hesocial.com"));
    assert_eq!(user.role, Some(Role::Admin));
    assert_eq!(user.membership_tier, Some(MembershipTier::BlackCard));
    assert!(user.is_verified);
    assert_eq!(user.verification_status, Some(VerificationStatus::Approved));

    let session = session_after_validate("stored-jwt", Ok(user));
    assert_eq!(session.token.as_deref(), Some("stored-jwt"));
    assert!(
        session.view_admin(),
        "restored admin role must light up view_admin"
    );
    assert!(permissions(&session.snapshot()).view_admin);
    assert!(!session.restoring);
}

#[test]
fn parse_validate_rejects_success_false() {
    let err = parse_validate_response(200, r#"{"success":false,"error":"Invalid token"}"#);
    assert!(err.is_err());
    let session = session_after_validate("stored-jwt", err);
    assert_eq!(session, hesocial_frontend::permissions::Session::default());
    assert!(!session.view_admin());
}

#[test]
fn parse_validate_401_is_failure() {
    let err = parse_validate_response(401, r#"{"success":false,"error":"Access token required"}"#);
    assert!(err.is_err());
    let session = session_after_validate("stored-jwt", err);
    assert_eq!(session.token, None);
    assert_eq!(session.user, None);
    assert!(!session.snapshot().is_authenticated);
}

#[test]
fn parse_validate_malformed_body_is_failure() {
    let err = parse_validate_response(200, "not-json");
    assert!(err.is_err());
    let session = session_after_validate("stored-jwt", err);
    assert_eq!(session, hesocial_frontend::permissions::Session::default());
}

#[test]
fn parse_validate_missing_user_is_failure() {
    let err = parse_validate_response(200, r#"{"success":true,"data":{"valid":true}}"#);
    assert!(err.is_err());
    let session = session_after_validate("stored-jwt", err);
    assert_eq!(session.token, None);
}

#[test]
fn any_validate_failure_logs_out() {
    for result in [
        parse_validate_response(401, r#"{"success":false}"#),
        parse_validate_response(500, "oops"),
        parse_validate_response(200, r#"{"success":false}"#),
        parse_validate_response(200, "{"),
        parse_validate_response(0, ""),
        Err(hesocial_frontend::auth::ValidateFailure::Transport),
    ] {
        let session = session_after_validate("stored-jwt", result);
        assert_eq!(
            session,
            hesocial_frontend::permissions::Session::default(),
            "React logs out on any validate failure"
        );
        assert!(!permissions(&session.snapshot()).access);
        assert!(!session.view_admin());
    }
}

#[test]
fn user_route_guard_redirects_signed_out_to_login() {
    assert_eq!(USER_ROUTE_FALLBACK, "/login");
    let signed_out = AuthSnapshot::default();
    assert_eq!(
        user_route_guard(false, &signed_out),
        RouteGuard::Redirect("/login")
    );
    assert_eq!(
        user_route_guard(false, &signed_out),
        RouteGuard::Redirect(USER_ROUTE_FALLBACK)
    );
}

#[test]
fn user_route_guard_waits_while_restoring() {
    let pending = AuthSnapshot {
        is_authenticated: true,
        ..AuthSnapshot::default()
    };
    assert_eq!(user_route_guard(true, &pending), RouteGuard::Loading);
    assert_eq!(
        user_route_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading,
        "restoring must not bounce to /login before validate returns"
    );
}

#[test]
fn user_route_guard_allows_authenticated_snapshot() {
    let signed_in = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        ..AuthSnapshot::default()
    };
    assert_eq!(user_route_guard(false, &signed_in), RouteGuard::Allow);
}

#[test]
fn parse_profile_complete_user() {
    let profile = parse_profile_response(COMPLETE_PROFILE_BODY).expect("complete body");
    assert_eq!(profile.email.as_deref(), Some("ok@example.com"));
    assert_eq!(
        display_full_name(profile.first_name.as_deref(), profile.last_name.as_deref()),
        "Wei Chen"
    );
    assert_eq!(display_age(profile.age), "42 歲");
    assert_eq!(display_optional(profile.profession.as_deref()), "投資人");
    assert_eq!(display_privacy_level(profile.privacy_level), "Level 4");
    assert_eq!(
        profile.interests,
        vec!["藝術".to_string(), "遊艇".to_string()]
    );
    assert_eq!(
        profile_picture_src(profile.profile_picture.as_deref()),
        "https://media.example/p.jpg"
    );
    assert_eq!(
        membership_color_class(profile.membership_tier_label()),
        "text-blue-400"
    );
    assert_eq!(
        membership_benefits(profile.membership_tier_label()),
        &[
            "VIP活動優先預訂",
            "專屬社交顧問",
            "私人活動邀請",
            "高端場地折扣"
        ]
    );
}

#[test]
fn parse_profile_google_null_fields_match_react_interpolation() {
    let profile = parse_profile_response(GOOGLE_PROFILE_BODY).expect("google body");
    assert_eq!(profile.age, None);
    assert_eq!(profile.profession, None);
    assert_eq!(profile.annual_income, None);
    assert_eq!(profile.net_worth, None);
    assert_eq!(profile.bio, None);
    assert!(profile.interests.is_empty());
    assert_eq!(display_age(None), " 歲");
    assert_eq!(display_optional(None), "");
    assert_eq!(display_optional_i64(None), "");
    assert_eq!(display_optional_i64(Some(0)), "0");
    assert_eq!(
        display_full_name(profile.first_name.as_deref(), profile.last_name.as_deref()),
        "Ada Li"
    );
    assert_eq!(
        profile_picture_src(profile.profile_picture.as_deref()),
        "/api/placeholder/150/150"
    );
    assert_eq!(display_privacy_level(profile.privacy_level), "Level 3");
    assert_eq!(membership_color_class(Some("Platinum")), "text-gray-400");
    assert_eq!(
        membership_color_class(Some("Black Card")),
        "text-luxury-gold"
    );
    assert_eq!(membership_color_class(None), "text-luxury-platinum");
}

#[test]
fn parse_profile_success_false_is_error() {
    assert!(
        parse_profile_response(r#"{"success":false,"error":"Authentication required"}"#).is_err()
    );
    assert!(parse_profile_response("not-json").is_err());
}

#[test]
fn dress_code_and_detail_badge_match_event_detail_page() {
    assert_eq!(dress_code_text(1), "休閒");
    assert_eq!(dress_code_text(2), "商務休閒");
    assert_eq!(dress_code_text(3), "正式");
    assert_eq!(dress_code_text(4), "晚宴正裝");
    assert_eq!(dress_code_text(5), "黑領結/長禮服");
    assert_eq!(dress_code_text(0), "未指定");
    assert_eq!(dress_code_text(9), "未指定");
    assert_eq!(
        detail_exclusivity_color(Some("Invitation Only")),
        "bg-purple-500/20 text-purple-400 border-purple-500/30"
    );
    assert_eq!(
        exclusivity_color(Some("Invitation Only")),
        "bg-gray-500/20 text-gray-400 border-gray-500/30",
        "list helper must stay unchanged"
    );
}

#[test]
fn occupancy_and_gallery_helpers() {
    assert_eq!(occupancy_percent(3, 12), 25);
    assert_eq!(occupancy_percent(0, 0), 0);
    assert_eq!(spots_remaining(3, 12), 9);
    assert_eq!(spots_remaining(12, 12), 0);
    assert!(is_full(12, 12));
    assert!(!is_full(11, 12));
    assert_eq!(venue_star_count(5.0), 5);
    assert_eq!(venue_star_count(4.7), 4);
    assert_eq!(venue_star_count(0.0), 0);
    assert_eq!(wrap_image_index(-1, 3), 2);
    assert_eq!(wrap_image_index(3, 3), 0);
    assert_eq!(wrap_image_index(1, 3), 1);
    assert_eq!(wrap_image_index(0, 0), 0);
    assert_eq!(gallery_image(&[], 0), PLACEHOLDER_IMAGE);
    assert_eq!(
        gallery_image(&["https://a.webp".into()], 0),
        "https://a.webp"
    );
    assert_eq!(price_kind_label(Some(15000.0), Some(12000.0)), "VVIP 價格");
    assert_eq!(price_kind_label(None, Some(12000.0)), "VIP 價格");
}

#[test]
fn parse_event_detail_success_and_null_lists() {
    let body = r#"{
        "success": true,
        "data": {
            "id": 11,
            "name": "松露季私宴",
            "description": "白松露當季。",
            "dateTime": "2026-10-04T12:00:00.000Z",
            "registrationDeadline": "2026-09-20T12:00:00.000Z",
            "venue": {
                "name": "Taipei Private Dining Room",
                "address": "Da'an",
                "rating": 5,
                "amenities": ["Valet", "Wine cellar"]
            },
            "exclusivityLevel": "VVIP",
            "pricing": {"vip": 15000, "vvip": 18000, "currency": "TWD"},
            "currentAttendees": 3,
            "capacity": 12,
            "images": ["https://media.example/e11.webp", "https://media.example/e11b.webp"],
            "organizer": "Wei Chen",
            "tags": ["晚宴", "松露"],
            "dressCode": 4,
            "amenities": ["專車接送"],
            "privacyGuarantees": ["匿名參與"],
            "requirements": [{"description": "需通過身份審核"}]
        }
    }"#;
    let detail = parse_event_detail_response(200, body).expect("detail");
    assert_eq!(detail.id, "11");
    assert_eq!(detail.name, "松露季私宴");
    assert_eq!(detail.organizer, "Wei Chen");
    assert_eq!(detail.dress_code, 4);
    assert_eq!(detail.tags, vec!["晚宴", "松露"]);
    assert_eq!(detail.amenities, vec!["專車接送"]);
    assert_eq!(detail.privacy_guarantees, vec!["匿名參與"]);
    assert_eq!(detail.requirements, vec!["需通過身份審核"]);
    assert_eq!(detail.venue_amenities, vec!["Valet", "Wine cellar"]);
    assert_eq!(detail.images.len(), 2);
    assert_eq!(detail.exclusivity_level.as_deref(), Some("VVIP"));
}

#[test]
fn parse_event_detail_404_and_encoded_json_columns() {
    assert!(
        parse_event_detail_response(404, r#"{"success":false,"error":"Event not found"}"#)
            .is_none()
    );
    assert!(parse_event_detail_response(200, r#"{"success":false}"#).is_none());
    assert!(parse_event_detail_response(200, "not-json").is_none());
    let encoded = r#"{
        "success": true,
        "data": {
            "id": "11",
            "name": "松露季私宴",
            "description": "x",
            "dateTime": "2026-10-04T12:00:00.000Z",
            "venue": {"name": "Room", "address": "Da'an", "rating": 4, "amenities": "[\"Valet\"]"},
            "exclusivityLevel": null,
            "pricing": {"vip": 15000, "currency": "TWD"},
            "currentAttendees": 0,
            "capacity": 12,
            "images": "[\"https://media.example/e11.webp\"]",
            "organizer": "Wei Chen",
            "amenities": null,
            "privacyGuarantees": null,
            "requirements": "[{\"description\":\"需年滿18歲\"}]",
            "dressCode": "3"
        }
    }"#;
    let detail = parse_event_detail_response(200, encoded).expect("encoded columns");
    assert_eq!(detail.images, vec!["https://media.example/e11.webp"]);
    assert!(detail.amenities.is_empty());
    assert!(detail.privacy_guarantees.is_empty());
    assert_eq!(detail.requirements, vec!["需年滿18歲"]);
    assert_eq!(detail.venue_amenities, vec!["Valet"]);
    assert_eq!(detail.dress_code, 3);
    assert_eq!(detail.tags.len(), 0);
}

#[test]
fn register_step_validation_matches_react() {
    let mut form = RegisterForm::default();
    assert_eq!(validate_step(&form), Err(REGISTER_REQUIRED));
    form.email = "a@b.c".into();
    form.password = "short".into();
    form.confirm_password = "other".into();
    assert_eq!(validate_step(&form), Err(REGISTER_PASSWORD_MISMATCH));
    form.confirm_password = "short".into();
    assert_eq!(validate_step(&form), Err(REGISTER_PASSWORD_SHORT));
    form.password = "longenough".into();
    form.confirm_password = "longenough".into();
    assert!(validate_step(&form).is_ok());

    form.step = 2;
    assert_eq!(validate_step(&form), Err(REGISTER_REQUIRED));
    form.first_name = "Wei".into();
    form.last_name = "Chen".into();
    form.age = "17".into();
    form.profession = "投資人".into();
    assert_eq!(validate_step(&form), Err(REGISTER_AGE_RANGE));
    form.age = "42".into();
    assert!(validate_step(&form).is_ok());

    form.step = 3;
    assert_eq!(validate_step(&form), Err(REGISTER_INCOME_ASSET));
    form.annual_income = "400".into();
    form.net_worth = "3000".into();
    assert_eq!(validate_step(&form), Err(REGISTER_INCOME_MIN));
    form.annual_income = "500".into();
    form.net_worth = "2000".into();
    assert_eq!(validate_step(&form), Err(REGISTER_NETWORTH_MIN));
    form.net_worth = "3000".into();
    assert_eq!(validate_step(&form), Err(REGISTER_INTEREST_REQUIRED));
    form.interests = vec!["藝術".into()];
    assert!(validate_step(&form).is_ok());
}

#[test]
fn register_interests_and_wan_conversion() {
    let added = push_interest(&[], " 藝術 ").expect("trim");
    assert_eq!(added, vec!["藝術"]);
    assert!(push_interest(&added, "藝術").is_none());
    assert!(push_interest(&added, "   ").is_none());
    let ten: Vec<String> = (0..10).map(|i| i.to_string()).collect();
    assert!(push_interest(&ten, "extra").is_none());
    assert_eq!(remove_interest(&["a".into(), "b".into()], "a"), vec!["b"]);
    let mut form = RegisterForm::default();
    form.email = "a@b.c".into();
    form.password = "longenough".into();
    form.first_name = "Wei".into();
    form.last_name = "Chen".into();
    form.age = "42".into();
    form.profession = "投資人".into();
    form.annual_income = "500".into();
    form.net_worth = "3000".into();
    form.interests = vec!["藝術".into()];
    let payload = registration_payload(&form);
    assert_eq!(payload.annual_income, 5_000_000);
    assert_eq!(payload.net_worth, 30_000_000);
    assert_eq!(payload.age, 42);
    assert_eq!(step_title(1), "步驟 1: 帳戶設定");
    assert_eq!(step_title(2), "步驟 2: 個人資訊");
    assert_eq!(step_title(3), "步驟 3: 會員資格");
}

#[test]
fn parse_register_success_and_error_paths() {
    assert_eq!(REGISTER_API_PATH, "/api/auth/register");
    let ok = parse_register_response(
        r#"{"success":true,"data":{"token":"jwt-reg","user":{"email":"a@b.c"}}}"#,
    )
    .expect("success");
    assert_eq!(ok.token, "jwt-reg");
    let err = parse_register_response(
        r#"{"success":false,"error":"User with this email already exists"}"#,
    )
    .expect_err("duplicate");
    assert_eq!(err, "User with this email already exists");
    assert_eq!(
        parse_register_response(r#"{"success":false}"#).unwrap_err(),
        REGISTER_FAILED_FALLBACK
    );
    assert_eq!(
        parse_register_response("not-json").unwrap_err(),
        REGISTER_FAILED_FALLBACK
    );
}
