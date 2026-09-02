#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::participants::{
    ACCESS_FETCH_FALLBACK, NETWORK_ERROR, PAGE_SIZE, PARTICIPANTS_FETCH_FALLBACK,
    PRIVACY_FETCH_FALLBACK, PRIVACY_UPDATE_FALLBACK, PRIVACY_UPDATE_SUCCESS,
    ParticipantAccessCheck, ParticipantFilters, ParticipantList, ParticipantRow,
    ParticipantViewAccess, ParticipantsPhase, PrivacyPhase, PrivacySettings, ViewerRelationship,
    can_send_contact, contact_payload, display_initial, mask_participant,
    membership_tier_badge_class, page_after_filter_change, parse_access_check_response,
    parse_contact_response, parse_participant_detail_response, parse_participants_response,
    parse_privacy_settings_response, parse_privacy_update_response, participant_access_api_path,
    participant_contact_api_path, participant_detail_api_path, participant_view_access,
    participants_api_path, participants_phase, participants_query_string, privacy_level_card_class,
    privacy_level_description, privacy_level_indicator_class, privacy_settings_api_path,
    privacy_settings_payload, privacy_settings_phase, total_pages, viewer_relationship,
    visible_interests,
};
use hesocial_frontend::permissions::{
    AuthSnapshot, Role, RouteGuard, USER_ROUTE_FALLBACK, user_route_guard,
};

fn ada(level: i64) -> ParticipantRow {
    ParticipantRow {
        id: "participant-1".to_owned(),
        first_name: "Ada".to_owned(),
        last_name: "Lovelace".to_owned(),
        email: "ada@example.com".to_owned(),
        phone: Some("+886900000000".to_owned()),
        age: Some(36),
        profession: Some("Software Engineer".to_owned()),
        company: Some("Example Tech".to_owned()),
        city: Some("London".to_owned()),
        membership_tier: "Diamond".to_owned(),
        interests: Some("[\"math\",\"computing\",\"music\",\"travel\"]".to_owned()),
        profile_picture: Some("ada.jpg".to_owned()),
        bio: Some("A".repeat(220)),
        effective_privacy_level: level,
        can_contact: 1,
    }
}

fn paid_list_body() -> String {
    r#"{
        "success": true,
        "data": {
            "participants": [
                {
                    "id": "7",
                    "displayName": "Ada L.",
                    "profession": "Technology",
                    "membershipTier": "Diamond",
                    "interests": ["math", "computing", "music"],
                    "profilePicture": "ada.jpg",
                    "ageRange": "35-39",
                    "privacyLevel": 1,
                    "canContact": true
                }
            ],
            "totalCount": 13,
            "paidParticipantCount": 8,
            "unpaidParticipantCount": 2,
            "viewerAccess": {
                "canViewParticipants": true,
                "maxPrivacyLevelVisible": 3,
                "canSeeContactInfo": false,
                "canInitiateContact": true,
                "participantCountVisible": true,
                "accessLevel": 3
            },
            "participantCountByTier": {
                "Platinum": 3,
                "Diamond": 4,
                "Black Card": 1
            }
        }
    }"#
    .to_string()
}

#[test]
fn page_size_matches_react_grid() {
    assert_eq!(PAGE_SIZE, 12);
}

#[test]
fn api_paths_include_event_and_participant_ids() {
    assert_eq!(participants_api_path("42"), "/api/events/42/participants");
    assert_eq!(
        participant_access_api_path("42"),
        "/api/events/42/participant-access"
    );
    assert_eq!(
        participant_detail_api_path("42", "7"),
        "/api/events/42/participants/7"
    );
    assert_eq!(
        participant_contact_api_path("42", "7"),
        "/api/events/42/participants/7/contact"
    );
    assert_eq!(
        privacy_settings_api_path("42"),
        "/api/events/42/privacy-settings"
    );
}

#[test]
fn query_string_omits_empty_filters() {
    let filters = ParticipantFilters::default();
    assert_eq!(
        participants_query_string(2, PAGE_SIZE, &filters),
        "page=2&limit=12"
    );
}

#[test]
fn query_string_encodes_all_active_filters() {
    let filters = ParticipantFilters {
        search: "Ada Lovelace".to_string(),
        membership_tier: "Black Card".to_string(),
        profession: "科技".to_string(),
    };
    let query = participants_query_string(1, PAGE_SIZE, &filters);
    assert!(query.contains("page=1"));
    assert!(query.contains("limit=12"));
    assert!(query.contains("membershipTier=Black+Card"));
    assert!(query.contains("search=Ada+Lovelace"));
    assert!(query.contains("profession="));
}

#[test]
fn filter_change_resets_to_first_page() {
    assert_eq!(page_after_filter_change(4), 1);
}

#[test]
fn total_pages_uses_ceiling_and_zero_count() {
    assert_eq!(total_pages(0, PAGE_SIZE), 0);
    assert_eq!(total_pages(12, PAGE_SIZE), 1);
    assert_eq!(total_pages(13, PAGE_SIZE), 2);
}

#[test]
fn relationship_matrix_matches_paid_and_membership_gates() {
    assert_eq!(
        viewer_relationship(None, "Black Card"),
        ViewerRelationship::Unpaid
    );
    assert_eq!(
        viewer_relationship(Some("pending"), "Diamond"),
        ViewerRelationship::Unpaid
    );
    assert_eq!(
        viewer_relationship(Some("none"), "Platinum"),
        ViewerRelationship::Unpaid
    );
    assert_eq!(
        viewer_relationship(Some("paid"), "Platinum"),
        ViewerRelationship::PaidStandard
    );
    assert_eq!(
        viewer_relationship(Some("paid"), "Diamond"),
        ViewerRelationship::PaidPremium
    );
    assert_eq!(
        viewer_relationship(Some("paid"), "Black Card"),
        ViewerRelationship::PaidPremium
    );
}

#[test]
fn unpaid_access_denies_all_participant_fields() {
    let access = participant_view_access(ViewerRelationship::Unpaid);
    assert_eq!(
        access,
        ParticipantViewAccess {
            can_view_participants: false,
            max_privacy_level_visible: 0,
            can_see_contact_info: false,
            can_initiate_contact: false,
            participant_count_visible: true,
            access_level: 0,
        }
    );
}

#[test]
fn standard_paid_access_caps_at_level_three() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    assert!(access.can_view_participants);
    assert_eq!(access.max_privacy_level_visible, 3);
    assert!(!access.can_see_contact_info);
    assert!(access.can_initiate_contact);
    assert_eq!(access.access_level, 3);
}

#[test]
fn premium_paid_access_opens_levels_four_and_five() {
    let access = participant_view_access(ViewerRelationship::PaidPremium);
    assert!(access.can_view_participants);
    assert_eq!(access.max_privacy_level_visible, 5);
    assert!(access.can_see_contact_info);
    assert!(access.can_initiate_contact);
    assert_eq!(access.access_level, 4);
}

#[test]
fn unpaid_viewer_sees_none_of_levels_one_through_five() {
    let access = participant_view_access(ViewerRelationship::Unpaid);
    for level in 1..=5 {
        assert!(
            mask_participant(&ada(level), access).is_none(),
            "level {level} must be hidden from unpaid viewers"
        );
    }
}

#[test]
fn standard_paid_viewer_sees_levels_one_through_three_only() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    for level in 1..=5 {
        assert_eq!(mask_participant(&ada(level), access).is_some(), level <= 3);
    }
}

#[test]
fn level_one_masks_to_initials_category_and_no_company() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    let masked = mask_participant(&ada(1), access).expect("visible");
    assert_eq!(masked.display_name, "Ada L.");
    assert_eq!(masked.profession.as_deref(), Some("Technology"));
    assert_eq!(masked.age_range.as_deref(), Some("35-39"));
    assert_eq!(masked.membership_tier, "Diamond");
    assert_eq!(masked.privacy_level, 1);
    assert!(masked.can_contact);
    assert_eq!(
        masked.interests,
        vec![
            "math".to_string(),
            "computing".to_string(),
            "music".to_string()
        ]
    );
    assert_eq!(masked.profile_picture.as_deref(), Some("ada.jpg"));
    assert!(masked.company.is_none());
    assert!(masked.city.is_none());
    assert!(masked.bio.is_none());
    assert!(masked.contact_info.is_none());
}

#[test]
fn level_two_adds_company_category_and_city() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    let masked = mask_participant(&ada(2), access).expect("visible");
    assert_eq!(masked.display_name, "Ada L.");
    assert_eq!(masked.company.as_deref(), Some("Technology Company"));
    assert_eq!(masked.city.as_deref(), Some("London"));
    assert!(masked.bio.is_none());
    assert!(masked.contact_info.is_none());
}

#[test]
fn level_three_reveals_full_name_company_and_truncated_bio() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    let masked = mask_participant(&ada(3), access).expect("visible");
    assert_eq!(masked.display_name, "Ada Lovelace");
    assert_eq!(masked.company.as_deref(), Some("Example Tech"));
    assert_eq!(masked.bio.as_ref().map(String::len), Some(200));
    assert!(masked.contact_info.is_none());
}

#[test]
fn premium_level_four_adds_email_and_full_bio() {
    let access = participant_view_access(ViewerRelationship::PaidPremium);
    let masked = mask_participant(&ada(4), access).expect("visible");
    assert_eq!(masked.display_name, "Ada Lovelace");
    assert_eq!(masked.bio.as_ref().map(String::len), Some(220));
    let contact = masked.contact_info.expect("email at level 4");
    assert_eq!(contact.email.as_deref(), Some("ada@example.com"));
    assert!(contact.phone.is_none());
}

#[test]
fn premium_level_five_adds_phone() {
    let access = participant_view_access(ViewerRelationship::PaidPremium);
    let masked = mask_participant(&ada(5), access).expect("visible");
    let contact = masked.contact_info.expect("contact at level 5");
    assert_eq!(contact.email.as_deref(), Some("ada@example.com"));
    assert_eq!(contact.phone.as_deref(), Some("+886900000000"));
}

#[test]
fn premium_levels_one_through_three_still_omit_contact() {
    let access = participant_view_access(ViewerRelationship::PaidPremium);
    for level in 1..=3 {
        let masked = mask_participant(&ada(level), access).expect("visible");
        assert!(
            masked.contact_info.is_none(),
            "level {level} must not leak contactInfo"
        );
    }
}

#[test]
fn can_contact_requires_both_subject_opt_in_and_viewer_right() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    let mut row = ada(2);
    row.can_contact = 0;
    let masked = mask_participant(&row, access).expect("visible");
    assert!(!masked.can_contact);

    let unpaid = participant_view_access(ViewerRelationship::Unpaid);
    assert!(mask_participant(&ada(2), unpaid).is_none());
}

#[test]
fn profession_and_age_fallbacks_match_backend_categories() {
    let access = participant_view_access(ViewerRelationship::PaidStandard);
    let mut row = ada(1);
    row.profession = None;
    row.age = None;
    let masked = mask_participant(&row, access).expect("visible");
    assert_eq!(masked.profession.as_deref(), Some("Professional"));
    assert_eq!(masked.age_range.as_deref(), Some("18-24"));
}

#[test]
fn parse_access_success_with_pending_paywall() {
    let body = r#"{
        "success": true,
        "data": {
            "hasAccess": false,
            "accessLevel": {
                "canViewParticipants": false,
                "maxPrivacyLevelVisible": 0,
                "canSeeContactInfo": false,
                "canInitiateContact": false,
                "participantCountVisible": true,
                "accessLevel": 0
            },
            "paymentRequired": true,
            "paymentStatus": "pending",
            "registrationStatus": "pending"
        }
    }"#;
    let check = parse_access_check_response(200, body).expect("ok");
    assert!(!check.has_access);
    assert!(check.payment_required);
    assert_eq!(check.payment_status, "pending");
    assert_eq!(check.registration_status.as_deref(), Some("pending"));
    assert!(!check.access_level.can_view_participants);
}

#[test]
fn parse_access_success_with_paid_standard() {
    let body = r#"{
        "success": true,
        "data": {
            "hasAccess": true,
            "accessLevel": {
                "canViewParticipants": true,
                "maxPrivacyLevelVisible": 3,
                "canSeeContactInfo": false,
                "canInitiateContact": true,
                "participantCountVisible": true,
                "accessLevel": 3
            },
            "paymentRequired": false,
            "paymentStatus": "paid",
            "registrationStatus": "approved"
        }
    }"#;
    let check = parse_access_check_response(200, body).expect("ok");
    assert!(check.has_access);
    assert_eq!(check.payment_status, "paid");
    assert!(check.access_level.can_initiate_contact);
    assert!(!check.access_level.can_see_contact_info);
}

#[test]
fn parse_access_malformed_and_false_success() {
    assert_eq!(
        parse_access_check_response(200, "not-json").unwrap_err(),
        ACCESS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_access_check_response(200, r#"{"success":false}"#).unwrap_err(),
        ACCESS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_access_check_response(200, r#"{"success":false,"error":"nope"}"#).unwrap_err(),
        "nope"
    );
    assert_eq!(
        parse_access_check_response(500, r#"{"success":false,"error":"boom"}"#).unwrap_err(),
        "boom"
    );
    assert_eq!(
        parse_access_check_response(0, "").unwrap_err(),
        ACCESS_FETCH_FALLBACK
    );
}

#[test]
fn parse_participants_populated_and_tier_counts() {
    let list = parse_participants_response(200, &paid_list_body()).expect("ok");
    assert_eq!(list.participants.len(), 1);
    assert_eq!(list.participants[0].display_name, "Ada L.");
    assert_eq!(
        list.participants[0].profession.as_deref(),
        Some("Technology")
    );
    assert_eq!(list.total_count, 13);
    assert_eq!(list.paid_participant_count, 8);
    assert_eq!(list.unpaid_participant_count, 2);
    assert_eq!(list.tier_count("Diamond"), 4);
    assert_eq!(list.tier_count("Black Card"), 1);
    assert_eq!(list.tier_count("Platinum"), 3);
    assert_eq!(list.tier_count("missing"), 0);
    assert!(list.viewer_access.can_initiate_contact);
}

#[test]
fn parse_participants_empty_array() {
    let body = r#"{
        "success": true,
        "data": {
            "participants": [],
            "totalCount": 0,
            "paidParticipantCount": 0,
            "unpaidParticipantCount": 0,
            "viewerAccess": {
                "canViewParticipants": true,
                "maxPrivacyLevelVisible": 5,
                "canSeeContactInfo": true,
                "canInitiateContact": true,
                "participantCountVisible": true,
                "accessLevel": 4
            },
            "participantCountByTier": {}
        }
    }"#;
    let list = parse_participants_response(200, body).expect("ok");
    assert!(list.participants.is_empty());
    assert_eq!(list.total_count, 0);
}

#[test]
fn parse_participants_error_and_empty_cases() {
    assert_eq!(
        parse_participants_response(200, "nope").unwrap_err(),
        PARTICIPANTS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_participants_response(
            403,
            r#"{"success":false,"error":"Access denied - payment required to view participants"}"#
        )
        .unwrap_err(),
        "Access denied - payment required to view participants"
    );
    assert_eq!(
        parse_participants_response(200, r#"{"success":true}"#).unwrap_err(),
        PARTICIPANTS_FETCH_FALLBACK
    );
}

#[test]
fn parse_does_not_remask_api_payload() {
    let body = r#"{
        "success": true,
        "data": {
            "participants": [{
                "id": "7",
                "displayName": "Full Name From Api",
                "membershipTier": "Platinum",
                "privacyLevel": 1,
                "canContact": false,
                "company": "Should Stay",
                "contactInfo": { "email": "leaked@example.com" }
            }],
            "totalCount": 1,
            "paidParticipantCount": 1,
            "unpaidParticipantCount": 0,
            "viewerAccess": {
                "canViewParticipants": true,
                "maxPrivacyLevelVisible": 3,
                "canSeeContactInfo": false,
                "canInitiateContact": true,
                "participantCountVisible": true,
                "accessLevel": 3
            },
            "participantCountByTier": {}
        }
    }"#;
    let list = parse_participants_response(200, body).expect("ok");
    assert_eq!(list.participants[0].display_name, "Full Name From Api");
    assert_eq!(list.participants[0].company.as_deref(), Some("Should Stay"));
    assert_eq!(
        list.participants[0]
            .contact_info
            .as_ref()
            .and_then(|c| c.email.as_deref()),
        Some("leaked@example.com")
    );
}

#[test]
fn parse_participant_detail_uses_masked_object() {
    let body = r#"{
        "success": true,
        "data": {
            "participant": {
                "id": "7",
                "displayName": "Ada Lovelace",
                "membershipTier": "Diamond",
                "privacyLevel": 3,
                "canContact": true,
                "profession": "Technology"
            },
            "viewerAccess": 3
        }
    }"#;
    let detail = parse_participant_detail_response(200, body).expect("ok");
    assert_eq!(detail.participant.display_name, "Ada Lovelace");
    assert_eq!(detail.viewer_access_level, 3);
}

#[test]
fn parse_participant_detail_errors() {
    assert_eq!(
        parse_participant_detail_response(
            404,
            r#"{"success":false,"error":"Participant not found or not visible"}"#
        )
        .unwrap_err(),
        "Participant not found or not visible"
    );
}

#[test]
fn parse_contact_success_and_validation_errors() {
    parse_contact_response(
        200,
        r#"{"success":true,"message":"Contact request sent successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_contact_response(400, r#"{"success":false,"error":"Message is required"}"#)
            .unwrap_err(),
        "Message is required"
    );
    assert!(!can_send_contact(""));
    assert!(!can_send_contact("   "));
    assert!(can_send_contact("你好"));
    assert_eq!(
        contact_payload("hello"),
        serde_json::json!({ "message": "hello" })
    );
}

#[test]
fn parse_privacy_settings_snake_case_payload() {
    let body = r#"{
        "success": true,
        "data": {
            "privacy_level": 3,
            "allow_contact": true,
            "show_in_list": false
        }
    }"#;
    let settings = parse_privacy_settings_response(200, body).expect("ok");
    assert_eq!(settings.privacy_level, 3);
    assert!(settings.allow_contact);
    assert!(!settings.show_in_list);
}

#[test]
fn parse_privacy_settings_null_data_and_errors() {
    assert!(parse_privacy_settings_response(200, r#"{"success":true,"data":null}"#).is_err());
    assert_eq!(
        parse_privacy_settings_response(200, r#"{"success":false}"#).unwrap_err(),
        PRIVACY_FETCH_FALLBACK
    );
    assert_eq!(
        parse_privacy_settings_response(500, "nope").unwrap_err(),
        PRIVACY_FETCH_FALLBACK
    );
}

#[test]
fn privacy_update_payload_is_camel_case() {
    let settings = PrivacySettings {
        privacy_level: 4,
        allow_contact: false,
        show_in_list: true,
    };
    assert_eq!(
        privacy_settings_payload(&settings),
        serde_json::json!({
            "privacyLevel": 4,
            "allowContact": false,
            "showInList": true
        })
    );
    parse_privacy_update_response(
        200,
        r#"{"success":true,"message":"Privacy settings updated successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_privacy_update_response(
            400,
            r#"{"success":false,"error":"Privacy level must be between 1 and 5"}"#
        )
        .unwrap_err(),
        "Privacy level must be between 1 and 5"
    );
    assert_eq!(
        parse_privacy_update_response(200, "bad").unwrap_err(),
        PRIVACY_UPDATE_FALLBACK
    );
    assert_eq!(PRIVACY_UPDATE_SUCCESS, "隱私設定已成功更新");
}

#[test]
fn participants_phase_loading_paywall_error_empty_ready() {
    let pending = ParticipantAccessCheck {
        has_access: false,
        payment_required: true,
        payment_status: "pending".to_string(),
        registration_status: Some("pending".to_string()),
        access_level: ParticipantViewAccess::denied(),
    };
    assert_eq!(
        participants_phase(true, None, None, None),
        ParticipantsPhase::Loading
    );
    assert_eq!(
        participants_phase(false, None, None, Some("ignored")),
        ParticipantsPhase::Paywall {
            payment_pending: false
        }
    );
    assert_eq!(
        participants_phase(false, Some(&pending), None, None),
        ParticipantsPhase::Paywall {
            payment_pending: true
        }
    );

    let granted = ParticipantAccessCheck {
        has_access: true,
        payment_required: false,
        payment_status: "paid".to_string(),
        registration_status: Some("approved".to_string()),
        access_level: participant_view_access(ViewerRelationship::PaidStandard),
    };
    assert_eq!(
        participants_phase(
            false,
            Some(&granted),
            None,
            Some("Failed to fetch participants")
        ),
        ParticipantsPhase::Error("Failed to fetch participants".to_string())
    );

    let empty = ParticipantList {
        participants: Vec::new(),
        total_count: 0,
        paid_participant_count: 0,
        unpaid_participant_count: 0,
        viewer_access: granted.access_level,
        participant_count_by_tier: Default::default(),
    };
    assert_eq!(
        participants_phase(false, Some(&granted), Some(&empty), None),
        ParticipantsPhase::Empty
    );

    let ready = parse_participants_response(200, &paid_list_body()).expect("ok");
    assert_eq!(
        participants_phase(false, Some(&granted), Some(&ready), None),
        ParticipantsPhase::Ready
    );
    assert_eq!(
        participants_phase(true, Some(&granted), Some(&ready), None),
        ParticipantsPhase::Ready
    );
}

#[test]
fn privacy_phase_loading_error_ready() {
    assert_eq!(
        privacy_settings_phase(true, None, None),
        PrivacyPhase::Loading
    );
    assert_eq!(
        privacy_settings_phase(false, None, Some("Failed to fetch privacy settings")),
        PrivacyPhase::Error("Failed to fetch privacy settings".to_string())
    );
    assert_eq!(
        privacy_settings_phase(false, None, None),
        PrivacyPhase::Error(PRIVACY_FETCH_FALLBACK.to_string())
    );
    let settings = PrivacySettings {
        privacy_level: 2,
        allow_contact: true,
        show_in_list: true,
    };
    assert_eq!(
        privacy_settings_phase(false, Some(&settings), None),
        PrivacyPhase::Ready
    );
}

#[test]
fn user_route_guard_three_states() {
    assert_eq!(USER_ROUTE_FALLBACK, "/login");
    assert_eq!(
        user_route_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading
    );
    assert_eq!(
        user_route_guard(false, &AuthSnapshot::default()),
        RouteGuard::Redirect("/login")
    );
    let signed_in = AuthSnapshot {
        is_authenticated: true,
        role: Some(Role::User),
        ..AuthSnapshot::default()
    };
    assert_eq!(user_route_guard(false, &signed_in), RouteGuard::Allow);
}

#[test]
fn privacy_level_copy_and_colors_cover_one_through_five() {
    let one = privacy_level_description(1).expect("1");
    assert_eq!(one.title, "公開資料");
    assert_eq!(
        one.description,
        "顯示基本資訊：姓名縮寫、年齡範圍、職業類別、會員等級"
    );
    assert_eq!(one.visibility, "所有付費參與者可見");

    let two = privacy_level_description(2).expect("2");
    assert_eq!(two.title, "半私人資料");
    assert_eq!(two.description, "顯示完整名字、公司行業、經驗範圍、城市");
    assert_eq!(two.visibility, "所有付費參與者可見");

    let three = privacy_level_description(3).expect("3");
    assert_eq!(three.title, "選擇性分享");
    assert_eq!(three.description, "顯示全名、公司名稱、具體興趣、專業成就");
    assert_eq!(three.visibility, "所有付費參與者可見");

    let four = privacy_level_description(4).expect("4");
    assert_eq!(four.title, "增強資料");
    assert_eq!(four.description, "顯示聯絡資訊、社交連結、詳細履歷");
    assert_eq!(four.visibility, "Diamond 和 Black Card 會員可見");

    let five = privacy_level_description(5).expect("5");
    assert_eq!(five.title, "完全公開");
    assert_eq!(five.description, "顯示直接聯絡方式、個人興趣、網絡連接");
    assert_eq!(five.visibility, "Diamond 和 Black Card 會員可見");

    assert!(privacy_level_description(0).is_none());
    assert!(privacy_level_description(6).is_none());

    assert!(privacy_level_indicator_class(1).contains("text-green-400"));
    assert!(privacy_level_indicator_class(2).contains("text-blue-400"));
    assert!(privacy_level_indicator_class(3).contains("text-yellow-400"));
    assert!(privacy_level_indicator_class(4).contains("text-orange-400"));
    assert!(privacy_level_indicator_class(5).contains("text-red-400"));
    assert!(privacy_level_card_class(5).contains("border-red-500/30"));
}

#[test]
fn membership_badge_classes_and_display_helpers() {
    assert!(membership_tier_badge_class("Platinum").contains("text-gray-300"));
    assert!(membership_tier_badge_class("Diamond").contains("text-blue-400"));
    assert!(membership_tier_badge_class("Black Card").contains("text-luxury-gold"));
    assert_eq!(display_initial("Ada L."), "A");
    assert_eq!(display_initial(""), "");
    assert_eq!(
        visible_interests(&["a".into(), "b".into(), "c".into(), "d".into()]),
        &["a".to_string(), "b".to_string(), "c".to_string()][..]
    );
}

#[test]
fn network_error_copy_matches_react() {
    assert_eq!(NETWORK_ERROR, "Network error occurred");
}
