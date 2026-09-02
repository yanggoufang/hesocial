#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::permissions::{
    AuthSnapshot, MembershipTier, RouteGuard, VerificationStatus,
};
use hesocial_frontend::vvip::{
    ParticipantsFetch, PreviewStatus, VvipSurface, classify_category, filter_events,
    parse_participant_access_response, parse_participants_response, parse_vvip_events_response,
    preview_attendee, preview_bundle, vvip_stats, vvip_surface,
};

fn snapshot(
    is_authenticated: bool,
    tier: Option<MembershipTier>,
    is_verified: bool,
) -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated,
        membership_tier: tier,
        is_verified,
        verification_status: if is_verified {
            Some(VerificationStatus::Approved)
        } else if is_authenticated {
            Some(VerificationStatus::Pending)
        } else {
            None
        },
        ..AuthSnapshot::default()
    }
}

fn all_auth_states() -> Vec<(bool, AuthSnapshot)> {
    let tiers = [
        None,
        Some(MembershipTier::Platinum),
        Some(MembershipTier::Diamond),
        Some(MembershipTier::BlackCard),
    ];
    let mut out = Vec::new();
    for restoring in [false, true] {
        out.push((restoring, snapshot(false, None, false)));
        for tier in tiers {
            for verified in [false, true] {
                out.push((restoring, snapshot(true, tier, verified)));
            }
        }
    }
    out
}

#[test]
fn signed_out_renders_recruitment_not_a_redirect() {
    let surface = vvip_surface(false, &snapshot(false, None, false));
    assert_eq!(surface, VvipSurface::Recruitment);
    assert!(!matches!(
        surface_as_guard(surface),
        RouteGuard::Redirect(_)
    ));
}

#[test]
fn signed_in_below_diamond_renders_recruitment() {
    for verified in [false, true] {
        let surface = vvip_surface(
            false,
            &snapshot(true, Some(MembershipTier::Platinum), verified),
        );
        assert_eq!(surface, VvipSurface::Recruitment);
    }
    let no_tier = vvip_surface(false, &snapshot(true, None, true));
    assert_eq!(no_tier, VvipSurface::Recruitment);
}

#[test]
fn diamond_unverified_renders_recruitment() {
    let surface = vvip_surface(false, &snapshot(true, Some(MembershipTier::Diamond), false));
    assert_eq!(surface, VvipSurface::Recruitment);
}

#[test]
fn diamond_verified_renders_content() {
    let surface = vvip_surface(false, &snapshot(true, Some(MembershipTier::Diamond), true));
    assert_eq!(surface, VvipSurface::Content);
}

#[test]
fn black_card_verified_renders_content() {
    let surface = vvip_surface(
        false,
        &snapshot(true, Some(MembershipTier::BlackCard), true),
    );
    assert_eq!(surface, VvipSurface::Content);
}

#[test]
fn black_card_unverified_renders_recruitment() {
    let surface = vvip_surface(
        false,
        &snapshot(true, Some(MembershipTier::BlackCard), false),
    );
    assert_eq!(surface, VvipSurface::Recruitment);
}

#[test]
fn restoring_is_loading_never_redirect() {
    let surface = vvip_surface(true, &snapshot(false, None, false));
    assert_eq!(surface, VvipSurface::Loading);
    let diamond = vvip_surface(true, &snapshot(true, Some(MembershipTier::Diamond), true));
    assert_eq!(diamond, VvipSurface::Loading);
}

#[test]
fn no_auth_state_produces_a_redirect() {
    for (restoring, snap) in all_auth_states() {
        let surface = vvip_surface(restoring, &snap);
        assert!(
            !matches!(surface_as_guard(surface), RouteGuard::Redirect(_)),
            "restoring={restoring} snap={snap:?} surface={surface:?} must not redirect"
        );
        assert!(matches!(
            surface,
            VvipSurface::Loading | VvipSurface::Recruitment | VvipSurface::Content
        ));
    }
}

fn surface_as_guard(surface: VvipSurface) -> RouteGuard {
    match surface {
        VvipSurface::Loading => RouteGuard::Loading,
        VvipSurface::Recruitment | VvipSurface::Content => RouteGuard::Allow,
    }
}

#[test]
fn parse_events_success_stringifies_numeric_id_and_reads_category() {
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
            "currentAttendees": 4,
            "capacity": 12,
            "images": ["https://media.example/e11.webp"],
            "category": {"name": "私人晚宴", "icon": "utensils"},
            "tags": ["私宴", "松露"]
        }],
        "pagination": {"page": 1, "limit": 50, "total": 1, "totalPages": 1}
    }"#;
    let events = parse_vvip_events_response(body);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "11");
    assert_eq!(events[0].name, "松露季私宴");
    assert_eq!(events[0].location, "Taipei Private Dining Room");
    assert_eq!(events[0].current_attendees, 4);
    assert_eq!(events[0].capacity, 12);
    assert_eq!(events[0].pricing_vvip, Some(15000.0));
    assert_eq!(events[0].category_id, "dining");
    assert_eq!(events[0].tags, vec!["私宴", "松露"]);
}

#[test]
fn parse_events_success_false_and_malformed_are_empty() {
    assert!(parse_vvip_events_response(r#"{"success":false,"error":"nope"}"#).is_empty());
    assert!(parse_vvip_events_response("not-json").is_empty());
    assert!(parse_vvip_events_response("{}").is_empty());
}

#[test]
fn classify_category_maps_api_names_to_vvip_filters() {
    assert_eq!(classify_category("私人晚宴", None), "dining");
    assert_eq!(classify_category("品酒會", Some("wine")), "dining");
    assert_eq!(classify_category("遊艇派對", Some("yacht")), "travel");
    assert_eq!(classify_category("藝術沙龍", Some("art")), "art");
    assert_eq!(classify_category("商務社交", Some("business")), "business");
    assert_eq!(classify_category("未知分類", None), "other");
}

#[test]
fn filter_events_by_category_keeps_all_and_matches_mapped_ids() {
    let body = r#"{
        "success": true,
        "data": [
            {"id": 1, "name": "晚宴", "dateTime": "2026-10-01T19:00:00.000Z", "category": {"name": "私人晚宴"}, "capacity": 8, "currentAttendees": 2, "pricing": {}},
            {"id": 2, "name": "遊艇", "dateTime": "2026-10-02T19:00:00.000Z", "category": {"name": "遊艇派對"}, "capacity": 20, "currentAttendees": 5, "pricing": {}},
            {"id": 3, "name": "沙龍", "dateTime": "2026-10-03T19:00:00.000Z", "category": {"name": "藝術沙龍"}, "capacity": 12, "currentAttendees": 1, "pricing": {}}
        ]
    }"#;
    let events = parse_vvip_events_response(body);
    assert_eq!(filter_events(&events, "all").len(), 3);
    assert_eq!(filter_events(&events, "dining").len(), 1);
    assert_eq!(filter_events(&events, "travel")[0].name, "遊艇");
    assert!(filter_events(&events, "business").is_empty());
}

#[test]
fn parse_participants_200_keeps_api_masked_rows() {
    let body = r#"{
        "success": true,
        "data": {
            "participants": [{
                "id": "p1",
                "displayName": "Ada L.",
                "membershipTier": "Diamond",
                "privacyLevel": 1,
                "profession": "Technology",
                "company": "Technology Company",
                "interests": ["math"],
                "profilePicture": "ada.jpg",
                "ageRange": "35-39"
            }],
            "totalCount": 1,
            "paidParticipantCount": 3,
            "unpaidParticipantCount": 1,
            "viewerAccess": {
                "canViewParticipants": true,
                "maxPrivacyLevelVisible": 3,
                "canSeeContactInfo": false,
                "canInitiateContact": true,
                "participantCountVisible": true,
                "accessLevel": 3
            },
            "participantCountByTier": {"Diamond": 1}
        }
    }"#;
    match parse_participants_response(200, body) {
        ParticipantsFetch::Ok(view) => {
            assert_eq!(view.participants.len(), 1);
            assert_eq!(view.total_count, 1);
            assert_eq!(view.paid_participant_count, 3);
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn parse_participants_401_and_403_degrade_without_panic() {
    assert_eq!(
        parse_participants_response(401, r#"{"success":false,"error":"Unauthorized"}"#),
        ParticipantsFetch::Unauthorized
    );
    assert_eq!(
        parse_participants_response(
            403,
            r#"{"success":false,"error":"Access denied - payment required to view participants"}"#
        ),
        ParticipantsFetch::Forbidden
    );
    assert_eq!(
        parse_participants_response(500, "oops"),
        ParticipantsFetch::Empty
    );
    assert_eq!(
        parse_participants_response(200, "not-json"),
        ParticipantsFetch::Empty
    );
}

#[test]
fn parse_access_401_is_none_and_200_reads_has_access() {
    assert!(parse_participant_access_response(401, "{}").is_none());
    let granted = parse_participant_access_response(
        200,
        r#"{"success":true,"data":{"hasAccess":true,"paymentRequired":false,"paymentStatus":"paid","registrationStatus":"confirmed","accessLevel":{"canViewParticipants":true,"maxPrivacyLevelVisible":5,"canSeeContactInfo":true,"canInitiateContact":true,"participantCountVisible":true,"accessLevel":4}}}"#,
    )
    .expect("granted");
    assert!(granted.has_access);
    assert!(!granted.payment_required);
    let denied = parse_participant_access_response(
        200,
        r#"{"success":true,"data":{"hasAccess":false,"paymentRequired":true,"paymentStatus":"none","registrationStatus":"none"}}"#,
    )
    .expect("denied");
    assert!(!denied.has_access);
    assert!(denied.payment_required);
}

#[test]
fn preview_strips_identity_and_keeps_profession_industry_tier() {
    let value = serde_json::json!({
        "id": "p1",
        "displayName": "Ada Lovelace",
        "firstName": "Ada",
        "lastName": "Lovelace",
        "email": "ada@example.com",
        "contactInfo": {"email": "ada@example.com", "phone": "+886900000000"},
        "profilePicture": "ada.jpg",
        "profession": "Technology",
        "company": "Technology Company",
        "membershipTier": "Diamond",
        "privacyLevel": 5
    });
    let preview = preview_attendee(&value, "11").expect("preview");
    assert_eq!(preview.profession.as_deref(), Some("Technology"));
    assert_eq!(preview.industry.as_deref(), Some("Technology Company"));
    assert_eq!(preview.membership_tier.as_deref(), Some("Diamond"));
    assert_eq!(preview.event_id, "11");
    assert!(preview.display_name.is_none());
    assert!(preview.email.is_none());
    assert!(preview.phone.is_none());
}

#[test]
fn preview_survives_level_one_payload_without_identity_fields() {
    let value = serde_json::json!({
        "id": "p2",
        "displayName": "Ada L.",
        "membershipTier": "Platinum",
        "privacyLevel": 1,
        "profession": "Finance"
    });
    let preview = preview_attendee(&value, "7").expect("preview");
    assert_eq!(preview.profession.as_deref(), Some("Finance"));
    assert_eq!(preview.industry, None);
    assert_eq!(preview.membership_tier.as_deref(), Some("Platinum"));
    assert!(preview.display_name.is_none());
}

#[test]
fn signed_out_preview_bundle_is_degraded_and_empty() {
    let bundle = preview_bundle(false, &[]);
    assert_eq!(bundle.status, PreviewStatus::SignedOut);
    assert!(bundle.attendees.is_empty());
}

#[test]
fn unauthorized_preview_bundle_is_degraded_and_empty() {
    let bundle = preview_bundle(true, &[ParticipantsFetch::Unauthorized]);
    assert_eq!(bundle.status, PreviewStatus::Unauthorized);
    assert!(bundle.attendees.is_empty());
}

#[test]
fn forbidden_preview_bundle_is_restricted_and_empty() {
    let bundle = preview_bundle(true, &[ParticipantsFetch::Forbidden]);
    assert_eq!(bundle.status, PreviewStatus::Restricted);
    assert!(bundle.attendees.is_empty());
}

#[test]
fn ready_preview_bundle_maps_masked_rows() {
    let body = r#"{
        "success": true,
        "data": {
            "participants": [{
                "id": "p1",
                "displayName": "Ada Lovelace",
                "membershipTier": "Black Card",
                "privacyLevel": 3,
                "profession": "Business",
                "company": "Private Company"
            }],
            "totalCount": 1,
            "paidParticipantCount": 1,
            "unpaidParticipantCount": 0
        }
    }"#;
    let fetch = parse_participants_response(200, body);
    let bundle = preview_bundle(true, &[fetch]);
    assert_eq!(bundle.status, PreviewStatus::Ready);
    assert_eq!(bundle.attendees.len(), 1);
    assert_eq!(bundle.attendees[0].profession.as_deref(), Some("Business"));
    assert_eq!(
        bundle.attendees[0].industry.as_deref(),
        Some("Private Company")
    );
    assert_eq!(
        bundle.attendees[0].membership_tier.as_deref(),
        Some("Black Card")
    );
    assert!(bundle.attendees[0].display_name.is_none());
}

#[test]
fn empty_api_data_does_not_invent_sample_stats() {
    let stats = vvip_stats(&[], &[]);
    assert_eq!(stats.event_count, 0);
    assert_eq!(stats.venue_count, 0);
    assert_eq!(stats.member_count, None);
}

#[test]
fn stats_count_events_unique_venues_and_preview_members() {
    let events = parse_vvip_events_response(
        r#"{
        "success": true,
        "data": [
            {"id": 1, "name": "A", "venue": {"name": "Hall One"}, "capacity": 8, "currentAttendees": 2, "pricing": {}},
            {"id": 2, "name": "B", "venue": {"name": "Hall One"}, "capacity": 8, "currentAttendees": 1, "pricing": {}},
            {"id": 3, "name": "C", "venue": {"name": "Hall Two"}, "capacity": 8, "currentAttendees": 1, "pricing": {}}
        ]
    }"#,
    );
    let value = serde_json::json!({
        "profession": "Legal",
        "membershipTier": "Diamond"
    });
    let attendees = vec![
        preview_attendee(&value, "1").unwrap(),
        preview_attendee(&value, "2").unwrap(),
    ];
    let stats = vvip_stats(&events, &attendees);
    assert_eq!(stats.event_count, 3);
    assert_eq!(stats.venue_count, 2);
    assert_eq!(stats.member_count, Some(2));
}
