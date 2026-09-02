#![cfg(not(target_arch = "wasm32"))]

use hesocial_frontend::eventmgmt::{
    ACTION_FALLBACK, ALLOWED_DOCUMENT_MIMES, ALLOWED_IMAGE_MIMES, APPROVE_FALLBACK,
    CAPACITY_MAX_POSITIVE, CAPACITY_MIN_EXCEEDS_MAX, CATEGORIES_API_PATH, CATEGORY_REQUIRED,
    CREATE_FALLBACK, DELETE_FALLBACK, DESCRIPTION_REQUIRED, END_AFTER_START, END_REQUIRED,
    EVENT_DOCUMENTS_FIELD, EVENT_FETCH_FALLBACK, EVENT_IMAGES_FIELD, EVENT_MANAGEMENT_FALLBACK,
    EVENT_NOT_FOUND, EVENTS_API_PATH, EVENTS_FETCH_FALLBACK, EventFormData, EventMgmtFilters,
    FORM_LOAD_FALLBACK, FileCandidate, MAX_DOCUMENT_FILES, MAX_FILE_SIZE_BYTES, MAX_IMAGE_FILES,
    MAX_SIZE_MB, MEDIA_API_PATH, MEDIA_DELETE_FALLBACK, MEDIA_EVENTS_API_PATH,
    MEDIA_FETCH_FALLBACK, MEDIA_UPLOAD_DOCS_FALLBACK, MEDIA_UPLOAD_IMAGES_FALLBACK, MediaKind,
    MediaTab, NETWORK_ERROR, PAGE_LIMIT, PUBLISH_FALLBACK, START_REQUIRED, TITLE_REQUIRED,
    UPDATE_FALLBACK, UploadBytes, VENUE_REQUIRED, VENUES_API_PATH, approve_payload, can_approve,
    can_publish, datetime_local_value, datetime_to_iso, empty_form, encode_multipart,
    event_api_path, event_approve_api_path, event_form_payload, event_management_guard,
    event_media_api_path, event_media_documents_api_path, event_media_images_api_path,
    event_publish_api_path, events_list_url, events_query_string, form_from_event,
    format_file_size, format_price, is_allowed_mime, lines_to_list, media_item_api_path,
    media_kind_from_mime, media_kind_from_type_field, media_page_path,
    next_approval_after_decision, next_status_after_publish, page_after_filter_change,
    page_in_range, parse_approve_response, parse_categories_response, parse_create_response,
    parse_delete_response, parse_event_response, parse_events_response,
    parse_media_delete_response, parse_media_list_response, parse_media_upload_response,
    parse_publish_response, parse_update_response, parse_venues_response, select_valid_files,
    status_badge_class, status_label, tab_media_kind, toggle_membership_tier, validate_event_form,
    validate_file, validate_file_for_kind, venues_list_url,
};
use hesocial_frontend::permissions::{AuthSnapshot, Role, RouteGuard};

fn snapshot(role: Option<Role>, authenticated: bool) -> AuthSnapshot {
    AuthSnapshot {
        is_authenticated: authenticated,
        role,
        ..AuthSnapshot::default()
    }
}

fn list_body() -> String {
    r#"{
        "success": true,
        "data": [
            {
                "id": 2,
                "name": "Autumn Yacht Social",
                "title": "Autumn Yacht Social",
                "description": "Sunset cruise",
                "dateTime": "2026-10-10T09:00:00.000Z",
                "start_datetime": "2026-10-10T09:00:00.000Z",
                "end_datetime": "2026-10-10T12:00:00.000Z",
                "pricing": {"vip": 18000, "vvip": 28000, "general": 38000, "currency": "TWD"},
                "status": "published",
                "approval_status": "approved",
                "venue_name": "Keelung Luxury Yacht",
                "category_name": "遊艇派對",
                "venue": {"name": "Keelung Luxury Yacht", "city": "Keelung"},
                "category": {"name": "遊艇派對", "icon": "ship"}
            }
        ],
        "pagination": {"page": 1, "limit": 10, "total": 1, "totalPages": 1}
    }"#
    .to_string()
}

fn admin_detail_body() -> String {
    r#"{
        "success": true,
        "data": {
            "id": 2,
            "name": "Autumn Yacht Social",
            "title": "Autumn Yacht Social",
            "slug": "autumn-yacht-social",
            "description": "Sunset cruise",
            "detailed_description": "Full agenda",
            "dateTime": "2026-10-10T09:00:00.000Z",
            "start_datetime": "2026-10-10T09:00:00.000Z",
            "end_datetime": "2026-10-10T12:00:00.000Z",
            "category_id": 4,
            "venue_id": 8,
            "organizer_id": 1,
            "timezone": "Asia/Taipei",
            "capacity_min": 2,
            "capacity_max": 30,
            "current_registrations": 1,
            "price_platinum": 18000,
            "price_diamond": 28000,
            "price_black_card": 38000,
            "currency": "TWD",
            "status": "draft",
            "approval_status": "pending",
            "required_membership_tiers": ["Platinum", "Diamond"],
            "required_verification": true,
            "dress_code": "Resort Casual",
            "language": "Traditional Chinese",
            "inclusions": ["Welcome drink"],
            "exclusions": ["Transport"],
            "waitlist_enabled": true,
            "auto_approval": false,
            "venue_name": "Keelung Luxury Yacht",
            "category_name": "遊艇派對"
        }
    }"#
    .to_string()
}

fn valid_form() -> EventFormData {
    EventFormData {
        title: "Autumn Yacht Social".to_string(),
        description: "Sunset cruise".to_string(),
        category_id: "4".to_string(),
        venue_id: "8".to_string(),
        start_datetime: "2026-10-10T09:00".to_string(),
        end_datetime: "2026-10-10T12:00".to_string(),
        capacity_min: 1,
        capacity_max: 20,
        ..EventFormData::default()
    }
}

#[test]
fn page_limit_and_paths_match_backend_and_react() {
    assert_eq!(PAGE_LIMIT, 10);
    assert_eq!(EVENTS_API_PATH, "/api/events");
    assert_eq!(CATEGORIES_API_PATH, "/api/categories");
    assert_eq!(VENUES_API_PATH, "/api/venues");
    assert_eq!(event_api_path("2"), "/api/events/2");
    assert_eq!(event_publish_api_path("2"), "/api/events/2/publish");
    assert_eq!(event_approve_api_path("2"), "/api/events/2/approve");
    assert_eq!(event_media_api_path("2", None), "/api/media/events/2");
    assert_eq!(
        event_media_api_path("2", Some(MediaKind::Image)),
        "/api/media/events/2?type=image"
    );
    assert_eq!(
        event_media_api_path("2", Some(MediaKind::Document)),
        "/api/media/events/2?type=document"
    );
    assert_eq!(
        event_media_images_api_path("2"),
        "/api/media/events/2/images"
    );
    assert_eq!(
        event_media_documents_api_path("2"),
        "/api/media/events/2/documents"
    );
    assert_eq!(media_item_api_path("m1"), "/api/media/m1");
    assert_eq!(media_page_path("2"), "/event-mgmt/media/2");
    assert_eq!(EVENT_IMAGES_FIELD, "eventImages");
    assert_eq!(EVENT_DOCUMENTS_FIELD, "eventDocuments");
    assert_eq!(venues_list_url(), "/api/venues?limit=100");
    let _ = (MEDIA_API_PATH, MEDIA_EVENTS_API_PATH);
}

#[test]
fn query_string_omits_empty_filters() {
    let filters = EventMgmtFilters {
        page: 1,
        limit: PAGE_LIMIT,
        ..EventMgmtFilters::default()
    };
    assert_eq!(events_query_string(&filters), "page=1&limit=10");
    assert_eq!(events_list_url(&filters), "/api/events?page=1&limit=10");
}

#[test]
fn query_string_encodes_every_active_filter() {
    let filters = EventMgmtFilters {
        page: 2,
        limit: 10,
        search: "Yacht Party".to_string(),
        status: "draft".to_string(),
        category: "yacht".to_string(),
        venue: "8".to_string(),
    };
    assert_eq!(
        events_query_string(&filters),
        "page=2&limit=10&search=Yacht+Party&status=draft&category=yacht&venue=8"
    );
}

#[test]
fn filter_change_resets_to_first_page() {
    assert_eq!(page_after_filter_change(4), 1);
    assert!(page_in_range(1, 2));
    assert!(page_in_range(2, 2));
    assert!(!page_in_range(0, 2));
    assert!(!page_in_range(3, 2));
}

#[test]
fn parse_events_public_list_shape_and_pagination() {
    let page = parse_events_response(200, &list_body()).expect("ok");
    assert_eq!(page.events.len(), 1);
    let event = &page.events[0];
    assert_eq!(event.id, "2");
    assert_eq!(event.title, "Autumn Yacht Social");
    assert_eq!(event.start_datetime, "2026-10-10T09:00:00.000Z");
    assert_eq!(event.venue_name, "Keelung Luxury Yacht");
    assert_eq!(event.price_platinum, 18000.0);
    assert_eq!(event.price_diamond, 28000.0);
    assert_eq!(event.currency, "TWD");
    assert_eq!(event.status, "published");
    assert_eq!(page.pagination.page, 1);
    assert_eq!(page.pagination.limit, 10);
    assert_eq!(page.pagination.total, 1);
    assert_eq!(page.pagination.total_pages, 1);
}

#[test]
fn parse_events_empty_array() {
    let body = r#"{
        "success": true,
        "data": [],
        "pagination": {"page": 1, "limit": 10, "total": 0, "totalPages": 0}
    }"#;
    let page = parse_events_response(200, body).expect("ok");
    assert!(page.events.is_empty());
    assert_eq!(page.pagination.total, 0);
}

#[test]
fn parse_events_error_empty_and_malformed() {
    assert_eq!(
        parse_events_response(200, "not-json").unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_events_response(200, r#"{"success":false}"#).unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_events_response(200, r#"{"success":false,"error":"nope"}"#).unwrap_err(),
        "nope"
    );
    assert_eq!(
        parse_events_response(200, r#"{"success":true}"#).unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_events_response(200, r#"{"success":true,"data":null}"#).unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_events_response(0, "").unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
}

#[test]
fn parse_events_401_and_403_use_backend_error_strings() {
    assert_eq!(
        parse_events_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_events_response(401, "").unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
    assert_eq!(
        parse_events_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
    assert_eq!(
        parse_events_response(403, r#"{"success":false}"#).unwrap_err(),
        EVENTS_FETCH_FALLBACK
    );
}

#[test]
fn parse_event_admin_detail_and_not_found() {
    let event = parse_event_response(200, &admin_detail_body()).expect("ok");
    assert_eq!(event.id, "2");
    assert_eq!(event.title, "Autumn Yacht Social");
    assert_eq!(event.category_id, "4");
    assert_eq!(event.venue_id, "8");
    assert_eq!(event.status, "draft");
    assert_eq!(event.approval_status, "pending");
    assert_eq!(event.capacity_max, 30);
    assert_eq!(
        event.required_membership_tiers,
        vec!["Platinum".to_string(), "Diamond".to_string()]
    );
    assert_eq!(
        parse_event_response(404, r#"{"success":false,"error":"Event not found"}"#).unwrap_err(),
        "Event not found"
    );
    assert_eq!(
        parse_event_response(200, r#"{"success":true,"data":null}"#).unwrap_err(),
        EVENT_FETCH_FALLBACK
    );
}

#[test]
fn parse_categories_and_venues() {
    let categories = parse_categories_response(
        200,
        r#"{"success":true,"data":[{"id":4,"name":"遊艇派對","slug":"yacht"}]}"#,
    )
    .expect("ok");
    assert_eq!(categories[0].filter_value(), "yacht");
    let venues = parse_venues_response(
        200,
        r#"{"success":true,"data":[{"id":8,"name":"Keelung Luxury Yacht","city":"Keelung","capacity_max":40}]}"#,
    )
    .expect("ok");
    assert_eq!(venues[0].id, "8");
    assert_eq!(venues[0].capacity_max, 40);
    assert_eq!(
        parse_categories_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_venues_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
    let _ = FORM_LOAD_FALLBACK;
}

#[test]
fn parse_mutation_envelopes_including_401_403() {
    parse_create_response(
        201,
        r#"{"success":true,"message":"Event created successfully","data":{"eventId":2,"slug":"x"}}"#,
    )
    .expect("ok");
    parse_update_response(
        200,
        r#"{"success":true,"message":"Event updated successfully"}"#,
    )
    .expect("ok");
    parse_delete_response(
        200,
        r#"{"success":true,"message":"Event deleted successfully"}"#,
    )
    .expect("ok");
    parse_publish_response(
        200,
        r#"{"success":true,"message":"Event published successfully"}"#,
    )
    .expect("ok");
    parse_approve_response(
        200,
        r#"{"success":true,"message":"Event approved successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_publish_response(
            400,
            r#"{"success":false,"error":"Event not found or not approved for publishing"}"#
        )
        .unwrap_err(),
        "Event not found or not approved for publishing"
    );
    assert_eq!(
        parse_create_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_update_response(403, r#"{"success":false,"error":"Admin access required"}"#)
            .unwrap_err(),
        "Admin access required"
    );
    assert_eq!(
        parse_delete_response(
            403,
            r#"{"success":false,"error":"Super admin access required"}"#
        )
        .unwrap_err(),
        "Super admin access required"
    );
    let _ = (
        CREATE_FALLBACK,
        UPDATE_FALLBACK,
        DELETE_FALLBACK,
        PUBLISH_FALLBACK,
        APPROVE_FALLBACK,
        ACTION_FALLBACK,
        NETWORK_ERROR,
        EVENT_NOT_FOUND,
    );
}

#[test]
fn create_and_edit_form_validation_rules() {
    let mut form = valid_form();
    assert!(validate_event_form(&form).is_ok());

    form.title = "   ".to_string();
    assert_eq!(validate_event_form(&form).unwrap_err(), TITLE_REQUIRED);
    form = valid_form();
    form.description = String::new();
    assert_eq!(
        validate_event_form(&form).unwrap_err(),
        DESCRIPTION_REQUIRED
    );
    form = valid_form();
    form.category_id.clear();
    assert_eq!(validate_event_form(&form).unwrap_err(), CATEGORY_REQUIRED);
    form = valid_form();
    form.venue_id.clear();
    assert_eq!(validate_event_form(&form).unwrap_err(), VENUE_REQUIRED);
    form = valid_form();
    form.start_datetime.clear();
    assert_eq!(validate_event_form(&form).unwrap_err(), START_REQUIRED);
    form = valid_form();
    form.end_datetime.clear();
    assert_eq!(validate_event_form(&form).unwrap_err(), END_REQUIRED);
    form = valid_form();
    form.end_datetime = form.start_datetime.clone();
    assert_eq!(validate_event_form(&form).unwrap_err(), END_AFTER_START);
    form = valid_form();
    form.end_datetime = "2026-10-10T08:00".to_string();
    assert_eq!(validate_event_form(&form).unwrap_err(), END_AFTER_START);
    form = valid_form();
    form.capacity_max = 0;
    assert_eq!(
        validate_event_form(&form).unwrap_err(),
        CAPACITY_MAX_POSITIVE
    );
    form = valid_form();
    form.capacity_min = 21;
    form.capacity_max = 20;
    assert_eq!(
        validate_event_form(&form).unwrap_err(),
        CAPACITY_MIN_EXCEEDS_MAX
    );
}

#[test]
fn empty_form_defaults_match_react_and_payload_uses_camel_case() {
    let form = empty_form();
    assert_eq!(form.timezone, "Asia/Taipei");
    assert_eq!(form.capacity_min, 1);
    assert_eq!(form.capacity_max, 20);
    assert_eq!(form.currency, "TWD");
    assert!(form.required_verification);
    assert!(form.waitlist_enabled);
    assert!(!form.auto_approval);
    assert_eq!(form.language, "Traditional Chinese");

    let payload = event_form_payload(&valid_form());
    assert_eq!(payload["title"], "Autumn Yacht Social");
    assert_eq!(payload["categoryId"], "4");
    assert_eq!(payload["venueId"], "8");
    assert_eq!(payload["startDatetime"], "2026-10-10T09:00:00.000Z");
    assert_eq!(payload["endDatetime"], "2026-10-10T12:00:00.000Z");
    assert_eq!(
        datetime_to_iso("2026-10-10T09:00:00.000Z"),
        "2026-10-10T09:00:00.000Z"
    );
    assert_eq!(
        datetime_local_value("2026-10-10T09:00:00.000Z"),
        "2026-10-10T09:00"
    );
    assert_eq!(approve_payload(true)["approved"], true);
    assert_eq!(approve_payload(false)["approved"], false);
    assert_eq!(
        lines_to_list("a\n\nb\n"),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(
        toggle_membership_tier(&[], "Platinum", true),
        vec!["Platinum".to_string()]
    );
    assert!(toggle_membership_tier(&["Platinum".to_string()], "Platinum", false).is_empty());
}

#[test]
fn form_from_event_slices_datetime_local() {
    let event = parse_event_response(200, &admin_detail_body()).expect("ok");
    let form = form_from_event(&event);
    assert_eq!(form.title, "Autumn Yacht Social");
    assert_eq!(form.start_datetime, "2026-10-10T09:00");
    assert_eq!(form.category_id, "4");
    assert_eq!(form.price_diamond, 28000.0);
}

#[test]
fn publish_and_approve_state_transitions() {
    assert!(can_approve("draft", "pending"));
    assert!(can_approve("pending_review", "pending"));
    assert!(!can_approve("published", "approved"));
    assert!(can_publish("approved", "approved"));
    assert!(can_publish("draft", "approved"));
    assert!(!can_publish("published", "approved"));
    assert!(!can_publish("draft", "pending"));
    assert!(!can_publish("cancelled", "approved"));
    assert_eq!(
        next_status_after_publish("approved", "approved"),
        Some("published")
    );
    assert_eq!(next_status_after_publish("published", "approved"), None);
    assert_eq!(next_approval_after_decision(true), "approved");
    assert_eq!(next_approval_after_decision(false), "rejected");
    assert_eq!(status_label("pending_review"), "pending review");
    assert!(status_badge_class("published").contains("green"));
}

#[test]
fn event_management_guard_three_states_pin_login_fallback() {
    assert_eq!(EVENT_MANAGEMENT_FALLBACK, "/login");
    assert_eq!(
        event_management_guard(true, &AuthSnapshot::default()),
        RouteGuard::Loading
    );
    assert_eq!(
        event_management_guard(false, &AuthSnapshot::default()),
        RouteGuard::Redirect("/login")
    );
    assert_eq!(
        event_management_guard(false, &snapshot(Some(Role::User), true)),
        RouteGuard::Redirect("/login")
    );
    assert_eq!(
        event_management_guard(false, &snapshot(Some(Role::Admin), true)),
        RouteGuard::Allow
    );
    assert_eq!(
        event_management_guard(false, &snapshot(Some(Role::SuperAdmin), true)),
        RouteGuard::Allow
    );
    match event_management_guard(false, &snapshot(None, false)) {
        RouteGuard::Redirect(path) => assert_eq!(path, EVENT_MANAGEMENT_FALLBACK),
        other => panic!("expected redirect to /login, got {other:?}"),
    }
}

#[test]
fn media_kind_and_size_formatting_at_boundaries() {
    assert_eq!(media_kind_from_mime("image/jpeg"), Some(MediaKind::Image));
    assert_eq!(media_kind_from_mime("image/png"), Some(MediaKind::Image));
    assert_eq!(media_kind_from_mime("image/webp"), Some(MediaKind::Image));
    assert_eq!(media_kind_from_mime("image/gif"), Some(MediaKind::Image));
    assert_eq!(
        media_kind_from_mime("application/pdf"),
        Some(MediaKind::Document)
    );
    assert_eq!(media_kind_from_type_field("image"), Some(MediaKind::Image));
    assert_eq!(
        media_kind_from_type_field("document"),
        Some(MediaKind::Document)
    );
    assert!(is_allowed_mime("image/jpeg"));
    assert!(ALLOWED_IMAGE_MIMES.contains(&"image/gif"));
    assert!(ALLOWED_DOCUMENT_MIMES.contains(&"application/pdf"));
    assert_eq!(format_file_size(0), "0 B");
    assert_eq!(format_file_size(1023), "1023 B");
    assert_eq!(format_file_size(1024), "1 KB");
    assert_eq!(format_file_size(1536), "1.5 KB");
    assert_eq!(format_file_size(1024 * 1024), "1 MB");
    assert_eq!(format_file_size(MAX_FILE_SIZE_BYTES), "10 MB");
    assert_eq!(MAX_FILE_SIZE_BYTES, 10 * 1024 * 1024);
    assert_eq!(MAX_SIZE_MB, 10);
    assert_eq!(MAX_IMAGE_FILES, 10);
    assert_eq!(MAX_DOCUMENT_FILES, 5);
    assert_eq!(format_price(18000.0, "TWD"), "NT$18,000.00");
    assert_eq!(format_price(0.0, "TWD"), "NT$0.00");
}

#[test]
fn validate_file_size_and_mime_boundaries() {
    let ok_image = FileCandidate {
        name: "hero.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        size: MAX_FILE_SIZE_BYTES,
    };
    assert!(validate_file(&ok_image, MAX_SIZE_MB).is_ok());
    let too_big = FileCandidate {
        name: "hero.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        size: MAX_FILE_SIZE_BYTES + 1,
    };
    assert_eq!(
        validate_file(&too_big, MAX_SIZE_MB).unwrap_err(),
        "File size must be less than 10MB"
    );
    let bad_type = FileCandidate {
        name: "note.txt".to_string(),
        mime: "text/plain".to_string(),
        size: 10,
    };
    assert_eq!(
        validate_file(&bad_type, MAX_SIZE_MB).unwrap_err(),
        "File type not supported. Please upload images or documents only."
    );
    let image_as_doc = FileCandidate {
        name: "hero.jpg".to_string(),
        mime: "image/jpeg".to_string(),
        size: 10,
    };
    assert_eq!(
        validate_file_for_kind(&image_as_doc, MediaKind::Document, MAX_SIZE_MB).unwrap_err(),
        "hero.jpg: Only documents are allowed"
    );
    let pdf_as_image = FileCandidate {
        name: "brief.pdf".to_string(),
        mime: "application/pdf".to_string(),
        size: 10,
    };
    assert_eq!(
        validate_file_for_kind(&pdf_as_image, MediaKind::Image, MAX_SIZE_MB).unwrap_err(),
        "brief.pdf: Only images are allowed"
    );
}

#[test]
fn select_valid_files_enforces_max_count() {
    let candidates: Vec<FileCandidate> = (0..3)
        .map(|i| FileCandidate {
            name: format!("img{i}.png"),
            mime: "image/png".to_string(),
            size: 10,
        })
        .collect();
    let (accepted, errors) = select_valid_files(&candidates, 9, 10, MediaKind::Image, MAX_SIZE_MB);
    assert_eq!(accepted.len(), 1);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("Maximum 10 files allowed"))
    );
}

#[test]
fn encode_multipart_uses_backend_field_names() {
    let files = [UploadBytes {
        filename: "Party Photo.PNG".to_string(),
        mime_type: "image/png".to_string(),
        bytes: b"\0\r\nimage".to_vec(),
    }];
    let (content_type, body) = encode_multipart(EVENT_IMAGES_FIELD, &files);
    assert!(content_type.starts_with("multipart/form-data; boundary="));
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("name=\"eventImages\""));
    assert!(text.contains("filename=\"Party Photo.PNG\""));
    assert!(text.contains("Content-Type: image/png"));
    assert!(body.windows(8).any(|w| w == b"\0\r\nimage"));
}

#[test]
fn parse_media_list_upload_and_delete() {
    let list = parse_media_list_response(
        200,
        r#"{
            "success": true,
            "data": [{
                "id": "m1",
                "type": "image",
                "filePath": "https://media.ahexagram.com/events/hero-original.jpg",
                "thumbnails": {"medium": "https://media.ahexagram.com/events/hero-medium.jpg"},
                "originalFilename": "hero.jpg",
                "fileSize": 2048,
                "mimeType": "image/jpeg",
                "uploadedBy": "1",
                "createdAt": "2026-08-01T00:00:00.000Z"
            }]
        }"#,
    )
    .expect("ok");
    assert_eq!(list[0].id, "m1");
    assert_eq!(list[0].kind, MediaKind::Image);
    assert_eq!(
        list[0].preview_url,
        "https://media.ahexagram.com/events/hero-medium.jpg"
    );
    assert_eq!(
        list[0].file_path,
        "https://media.ahexagram.com/events/hero-original.jpg"
    );
    assert_eq!(format_file_size(list[0].file_size), "2 KB");

    let uploaded = parse_media_upload_response(
        200,
        r#"{
            "success": true,
            "data": {
                "eventId": "2",
                "uploadedImages": [{
                    "id": "m2",
                    "type": "image",
                    "filePath": "https://media.ahexagram.com/events/new.jpg",
                    "originalFilename": "new.jpg",
                    "fileSize": 10,
                    "mimeType": "image/jpeg"
                }],
                "count": 1
            }
        }"#,
        MediaKind::Image,
    )
    .expect("ok");
    assert_eq!(uploaded[0].id, "m2");
    parse_media_delete_response(
        200,
        r#"{"success":true,"message":"Media deleted successfully"}"#,
    )
    .expect("ok");
    assert_eq!(
        parse_media_list_response(401, r#"{"success":false,"error":"Access token required"}"#)
            .unwrap_err(),
        "Access token required"
    );
    assert_eq!(
        parse_media_upload_response(
            400,
            r#"{"success":false,"error":"No images provided"}"#,
            MediaKind::Image
        )
        .unwrap_err(),
        "No images provided"
    );
    assert_eq!(
        parse_media_delete_response(403, r#"{"success":false,"error":"Permission denied"}"#)
            .unwrap_err(),
        "Permission denied"
    );
    assert_eq!(tab_media_kind(MediaTab::All), None);
    assert_eq!(tab_media_kind(MediaTab::Images), Some(MediaKind::Image));
    let _ = (
        MEDIA_FETCH_FALLBACK,
        MEDIA_UPLOAD_IMAGES_FALLBACK,
        MEDIA_UPLOAD_DOCS_FALLBACK,
        MEDIA_DELETE_FALLBACK,
    );
}
