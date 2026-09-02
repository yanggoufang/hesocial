use serde_json::{Map, Value};

use crate::permissions::{AuthSnapshot, RouteGuard, permissions};

pub const EVENTS_API_PATH: &str = "/api/events";
pub const CATEGORIES_API_PATH: &str = "/api/categories";
pub const VENUES_API_PATH: &str = "/api/venues";
pub const MEDIA_EVENTS_API_PATH: &str = "/api/media/events";
pub const MEDIA_API_PATH: &str = "/api/media";
pub const PAGE_LIMIT: u32 = 10;
pub const EVENT_MANAGEMENT_FALLBACK: &str = "/login";
pub const MEDIA_ROUTE_PREFIX: &str = "/event-mgmt/media";
pub const EVENT_MGMT_PATH: &str = "/event-mgmt";
pub const CATEGORIES_PATH: &str = "/event-mgmt/categories";
pub const VENUES_PATH: &str = "/event-mgmt/venues";

pub const EVENTS_FETCH_FALLBACK: &str = "Failed to load event data";
pub const EVENT_FETCH_FALLBACK: &str = "Failed to load event";
pub const EVENT_LOAD_ERROR: &str = "Error loading event";
pub const EVENT_NOT_FOUND: &str = "Event not found";
pub const FORM_LOAD_FALLBACK: &str = "Failed to load form data";
pub const CREATE_FALLBACK: &str = "Failed to save event";
pub const UPDATE_FALLBACK: &str = "Failed to save event";
pub const DELETE_FALLBACK: &str = "An error occurred";
pub const ACTION_FALLBACK: &str = "An error occurred";
pub const PUBLISH_FALLBACK: &str = "Failed to publish event";
pub const APPROVE_FALLBACK: &str = "Failed to approve/reject event";
pub const MEDIA_FETCH_FALLBACK: &str = "Failed to get event media";
pub const MEDIA_UPLOAD_IMAGES_FALLBACK: &str = "Failed to upload images";
pub const MEDIA_UPLOAD_DOCS_FALLBACK: &str = "Failed to upload documents";
pub const MEDIA_DELETE_FALLBACK: &str = "Failed to delete media";
pub const NETWORK_ERROR: &str = "Network error occurred";

pub const TITLE_REQUIRED: &str = "Event title is required";
pub const DESCRIPTION_REQUIRED: &str = "Event description is required";
pub const CATEGORY_REQUIRED: &str = "Event category is required";
pub const VENUE_REQUIRED: &str = "Venue is required";
pub const START_REQUIRED: &str = "Start date and time is required";
pub const END_REQUIRED: &str = "End date and time is required";
pub const END_AFTER_START: &str = "End time must be after start time";
pub const CAPACITY_MAX_POSITIVE: &str = "Maximum capacity must be greater than 0";
pub const CAPACITY_MIN_EXCEEDS_MAX: &str = "Minimum capacity cannot exceed maximum capacity";

pub const EVENT_IMAGES_FIELD: &str = "eventImages";
pub const EVENT_DOCUMENTS_FIELD: &str = "eventDocuments";
pub const MAX_IMAGE_FILES: u32 = 10;
pub const MAX_DOCUMENT_FILES: u32 = 5;
pub const MAX_FORM_IMAGE_FILES: u32 = 5;
pub const MAX_FORM_DOCUMENT_FILES: u32 = 3;
pub const MAX_SIZE_MB: u32 = 10;
pub const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;
pub const MULTIPART_BOUNDARY: &str = "----HesocialEventMediaBoundary";

pub const CREATE_ACTION: &str = "create";
pub const EDIT_ACTION: &str = "edit";
pub const DELETE_ACTION: &str = "delete";
pub const PUBLISH_ACTION: &str = "publish";
pub const APPROVE_ACTION: &str = "approve";
pub const UPLOAD_ACTION: &str = "upload";

pub const ALLOWED_IMAGE_MIMES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];
pub const ALLOWED_DOCUMENT_MIMES: &[&str] = &[
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EventMgmtFilters {
    pub page: u32,
    pub limit: u32,
    pub search: String,
    pub status: String,
    pub category: String,
    pub venue: String,
}

impl EventMgmtFilters {
    pub fn list_default() -> Self {
        Self {
            page: 1,
            limit: PAGE_LIMIT,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventsPagination {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}

impl Default for EventsPagination {
    fn default() -> Self {
        Self {
            page: 1,
            limit: PAGE_LIMIT,
            total: 0,
            total_pages: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ManagedEvent {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub detailed_description: String,
    pub category_id: String,
    pub venue_id: String,
    pub organizer_id: String,
    pub start_datetime: String,
    pub end_datetime: String,
    pub timezone: String,
    pub capacity_min: u32,
    pub capacity_max: u32,
    pub current_registrations: u32,
    pub price_platinum: f64,
    pub price_diamond: f64,
    pub price_black_card: f64,
    pub currency: String,
    pub status: String,
    pub approval_status: String,
    pub required_membership_tiers: Vec<String>,
    pub required_verification: bool,
    pub dress_code: String,
    pub language: String,
    pub special_requirements: String,
    pub inclusions: Vec<String>,
    pub exclusions: Vec<String>,
    pub registration_opens_at: String,
    pub registration_closes_at: String,
    pub cancellation_deadline: String,
    pub waitlist_enabled: bool,
    pub auto_approval: bool,
    pub meta_title: String,
    pub meta_description: String,
    pub featured_image: String,
    pub internal_notes: String,
    pub profit_margin: f64,
    pub venue_name: String,
    pub category_name: String,
    pub category_slug: String,
}

impl Default for ManagedEvent {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            slug: String::new(),
            description: String::new(),
            detailed_description: String::new(),
            category_id: String::new(),
            venue_id: String::new(),
            organizer_id: String::new(),
            start_datetime: String::new(),
            end_datetime: String::new(),
            timezone: "Asia/Taipei".to_string(),
            capacity_min: 1,
            capacity_max: 20,
            current_registrations: 0,
            price_platinum: 0.0,
            price_diamond: 0.0,
            price_black_card: 0.0,
            currency: "TWD".to_string(),
            status: String::new(),
            approval_status: String::new(),
            required_membership_tiers: Vec::new(),
            required_verification: true,
            dress_code: String::new(),
            language: "Traditional Chinese".to_string(),
            special_requirements: String::new(),
            inclusions: Vec::new(),
            exclusions: Vec::new(),
            registration_opens_at: String::new(),
            registration_closes_at: String::new(),
            cancellation_deadline: String::new(),
            waitlist_enabled: true,
            auto_approval: false,
            meta_title: String::new(),
            meta_description: String::new(),
            featured_image: String::new(),
            internal_notes: String::new(),
            profit_margin: 0.0,
            venue_name: String::new(),
            category_name: String::new(),
            category_slug: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EventCategory {
    pub id: String,
    pub name: String,
    pub slug: String,
}

impl EventCategory {
    pub fn filter_value(&self) -> &str {
        if !self.slug.is_empty() {
            &self.slug
        } else if !self.name.is_empty() {
            &self.name
        } else {
            &self.id
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EventVenue {
    pub id: String,
    pub name: String,
    pub city: String,
    pub capacity_max: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventsPage {
    pub events: Vec<ManagedEvent>,
    pub pagination: EventsPagination,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventFormData {
    pub title: String,
    pub description: String,
    pub detailed_description: String,
    pub category_id: String,
    pub venue_id: String,
    pub start_datetime: String,
    pub end_datetime: String,
    pub timezone: String,
    pub capacity_min: u32,
    pub capacity_max: u32,
    pub price_platinum: f64,
    pub price_diamond: f64,
    pub price_black_card: f64,
    pub currency: String,
    pub required_membership_tiers: Vec<String>,
    pub required_verification: bool,
    pub dress_code: String,
    pub language: String,
    pub special_requirements: String,
    pub inclusions: Vec<String>,
    pub exclusions: Vec<String>,
    pub registration_opens_at: String,
    pub registration_closes_at: String,
    pub cancellation_deadline: String,
    pub waitlist_enabled: bool,
    pub auto_approval: bool,
    pub meta_title: String,
    pub meta_description: String,
    pub featured_image: String,
    pub internal_notes: String,
    pub profit_margin: f64,
}

impl Default for EventFormData {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            detailed_description: String::new(),
            category_id: String::new(),
            venue_id: String::new(),
            start_datetime: String::new(),
            end_datetime: String::new(),
            timezone: "Asia/Taipei".to_string(),
            capacity_min: 1,
            capacity_max: 20,
            price_platinum: 0.0,
            price_diamond: 0.0,
            price_black_card: 0.0,
            currency: "TWD".to_string(),
            required_membership_tiers: Vec::new(),
            required_verification: true,
            dress_code: String::new(),
            language: "Traditional Chinese".to_string(),
            special_requirements: String::new(),
            inclusions: Vec::new(),
            exclusions: Vec::new(),
            registration_opens_at: String::new(),
            registration_closes_at: String::new(),
            cancellation_deadline: String::new(),
            waitlist_enabled: true,
            auto_approval: false,
            meta_title: String::new(),
            meta_description: String::new(),
            featured_image: String::new(),
            internal_notes: String::new(),
            profit_margin: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EventMgmtModal {
    #[default]
    None,
    Form,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MediaTab {
    #[default]
    All,
    Images,
    Documents,
}

impl MediaTab {
    pub fn as_id(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Images => "images",
            Self::Documents => "documents",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All Media",
            Self::Images => "Images",
            Self::Documents => "Documents",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Document,
}

impl MediaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Document => "document",
        }
    }

    pub fn field_name(self) -> &'static str {
        match self {
            Self::Image => EVENT_IMAGES_FIELD,
            Self::Document => EVENT_DOCUMENTS_FIELD,
        }
    }

    pub fn max_files_page(self) -> u32 {
        match self {
            Self::Image => MAX_IMAGE_FILES,
            Self::Document => MAX_DOCUMENT_FILES,
        }
    }

    pub fn upload_fallback(self) -> &'static str {
        match self {
            Self::Image => MEDIA_UPLOAD_IMAGES_FALLBACK,
            Self::Document => MEDIA_UPLOAD_DOCS_FALLBACK,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaAsset {
    pub id: String,
    pub kind: MediaKind,
    pub file_path: String,
    pub preview_url: String,
    pub original_filename: String,
    pub file_size: u64,
    pub mime_type: String,
    pub uploaded_by: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingFile {
    pub name: String,
    pub size: u64,
    pub mime: String,
    pub kind: MediaKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadBytes {
    pub filename: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileCandidate {
    pub name: String,
    pub mime: String,
    pub size: u64,
}

pub fn event_api_path(id: &str) -> String {
    format!("{EVENTS_API_PATH}/{id}")
}

pub fn event_publish_api_path(id: &str) -> String {
    format!("{EVENTS_API_PATH}/{id}/publish")
}

pub fn event_approve_api_path(id: &str) -> String {
    format!("{EVENTS_API_PATH}/{id}/approve")
}

pub fn event_media_api_path(event_id: &str, kind: Option<MediaKind>) -> String {
    let base = format!("{MEDIA_EVENTS_API_PATH}/{event_id}");
    match kind {
        Some(MediaKind::Image) => format!("{base}?type=image"),
        Some(MediaKind::Document) => format!("{base}?type=document"),
        None => base,
    }
}

pub fn event_media_images_api_path(event_id: &str) -> String {
    format!("{MEDIA_EVENTS_API_PATH}/{event_id}/images")
}

pub fn event_media_documents_api_path(event_id: &str) -> String {
    format!("{MEDIA_EVENTS_API_PATH}/{event_id}/documents")
}

pub fn media_item_api_path(media_id: &str) -> String {
    format!("{MEDIA_API_PATH}/{media_id}")
}

pub fn media_page_path(event_id: &str) -> String {
    format!("{MEDIA_ROUTE_PREFIX}/{event_id}")
}

pub fn events_list_url(filters: &EventMgmtFilters) -> String {
    let query = events_query_string(filters);
    if query.is_empty() {
        EVENTS_API_PATH.to_string()
    } else {
        format!("{EVENTS_API_PATH}?{query}")
    }
}

pub fn venues_list_url() -> String {
    format!("{VENUES_API_PATH}?limit=100")
}

pub fn events_query_string(filters: &EventMgmtFilters) -> String {
    let mut out = String::new();
    let page = if filters.page == 0 { 1 } else { filters.page };
    let limit = if filters.limit == 0 {
        PAGE_LIMIT
    } else {
        filters.limit
    };
    push_param(&mut out, "page", &page.to_string());
    push_param(&mut out, "limit", &limit.to_string());
    if !filters.search.is_empty() {
        push_param(&mut out, "search", &filters.search);
    }
    if !filters.status.is_empty() {
        push_param(&mut out, "status", &filters.status);
    }
    if !filters.category.is_empty() {
        push_param(&mut out, "category", &filters.category);
    }
    if !filters.venue.is_empty() {
        push_param(&mut out, "venue", &filters.venue);
    }
    out
}

pub fn page_after_filter_change(_current_page: u32) -> u32 {
    1
}

pub fn page_in_range(new_page: u32, total_pages: u32) -> bool {
    new_page > 0 && new_page <= total_pages
}

pub fn event_management_guard(restoring: bool, snapshot: &AuthSnapshot) -> RouteGuard {
    if restoring {
        RouteGuard::Loading
    } else if !permissions(snapshot).event_management {
        RouteGuard::Redirect(EVENT_MANAGEMENT_FALLBACK)
    } else {
        RouteGuard::Allow
    }
}

pub fn action_is(action_loading: Option<&str>, key: &str) -> bool {
    action_loading == Some(key)
}

pub fn publish_action_key(event_id: &str) -> String {
    format!("{PUBLISH_ACTION}-{event_id}")
}

pub fn approve_action_key(event_id: &str) -> String {
    format!("{APPROVE_ACTION}-{event_id}")
}

pub fn can_approve(status: &str, approval_status: &str) -> bool {
    approval_status == "pending" || status == "pending_review" || status == "draft"
}

pub fn can_publish(status: &str, approval_status: &str) -> bool {
    approval_status == "approved"
        && status != "published"
        && status != "cancelled"
        && status != "archived"
        && status != "completed"
}

pub fn next_status_after_publish(status: &str, approval_status: &str) -> Option<&'static str> {
    can_publish(status, approval_status).then_some("published")
}

pub fn next_approval_after_decision(approved: bool) -> &'static str {
    if approved { "approved" } else { "rejected" }
}

pub fn approve_payload(approved: bool) -> Value {
    serde_json::json!({ "approved": approved })
}

pub fn status_label(status: &str) -> String {
    status.replacen('_', " ", 1)
}

pub fn status_badge_class(status: &str) -> &'static str {
    match status {
        "draft" => "bg-gray-500/20 text-gray-300",
        "pending_review" => "bg-yellow-500/20 text-yellow-300",
        "approved" => "bg-blue-500/20 text-blue-300",
        "published" => "bg-green-500/20 text-green-300",
        "full" => "bg-purple-500/20 text-purple-300",
        "completed" => "bg-gray-600/30 text-gray-400",
        "cancelled" => "bg-red-500/20 text-red-300",
        "archived" => "bg-gray-700/30 text-gray-500",
        _ => "bg-gray-500/20 text-gray-300",
    }
}

pub fn media_status_badge_class(status: &str) -> &'static str {
    match status {
        "published" => "bg-green-100 text-green-800",
        "draft" => "bg-gray-100 text-gray-800",
        _ => "bg-yellow-100 text-yellow-800",
    }
}

pub fn datetime_local_value(iso: &str) -> String {
    if iso.len() >= 16 {
        iso[..16].to_string()
    } else {
        iso.to_string()
    }
}

pub fn lines_to_list(value: &str) -> Vec<String> {
    value
        .split('\n')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn list_to_lines(items: &[String]) -> String {
    items.join("\n")
}

pub fn toggle_membership_tier(tiers: &[String], tier: &str, checked: bool) -> Vec<String> {
    if checked {
        let mut next = tiers.to_vec();
        if !next.iter().any(|item| item == tier) {
            next.push(tier.to_string());
        }
        next
    } else {
        tiers
            .iter()
            .filter(|item| item.as_str() != tier)
            .cloned()
            .collect()
    }
}

pub fn empty_form() -> EventFormData {
    EventFormData::default()
}

pub fn form_from_event(event: &ManagedEvent) -> EventFormData {
    EventFormData {
        title: event.title.clone(),
        description: event.description.clone(),
        detailed_description: event.detailed_description.clone(),
        category_id: event.category_id.clone(),
        venue_id: event.venue_id.clone(),
        start_datetime: datetime_local_value(&event.start_datetime),
        end_datetime: datetime_local_value(&event.end_datetime),
        timezone: if event.timezone.is_empty() {
            "Asia/Taipei".to_string()
        } else {
            event.timezone.clone()
        },
        capacity_min: event.capacity_min,
        capacity_max: event.capacity_max,
        price_platinum: event.price_platinum,
        price_diamond: event.price_diamond,
        price_black_card: event.price_black_card,
        currency: if event.currency.is_empty() {
            "TWD".to_string()
        } else {
            event.currency.clone()
        },
        required_membership_tiers: event.required_membership_tiers.clone(),
        required_verification: event.required_verification,
        dress_code: event.dress_code.clone(),
        language: if event.language.is_empty() {
            "Traditional Chinese".to_string()
        } else {
            event.language.clone()
        },
        special_requirements: event.special_requirements.clone(),
        inclusions: event.inclusions.clone(),
        exclusions: event.exclusions.clone(),
        registration_opens_at: datetime_local_value(&event.registration_opens_at),
        registration_closes_at: datetime_local_value(&event.registration_closes_at),
        cancellation_deadline: datetime_local_value(&event.cancellation_deadline),
        waitlist_enabled: event.waitlist_enabled,
        auto_approval: event.auto_approval,
        meta_title: event.meta_title.clone(),
        meta_description: event.meta_description.clone(),
        featured_image: event.featured_image.clone(),
        internal_notes: event.internal_notes.clone(),
        profit_margin: event.profit_margin,
    }
}

pub fn validate_event_form(data: &EventFormData) -> Result<(), String> {
    if data.title.trim().is_empty() {
        return Err(TITLE_REQUIRED.to_string());
    }
    if data.description.trim().is_empty() {
        return Err(DESCRIPTION_REQUIRED.to_string());
    }
    if data.category_id.is_empty() {
        return Err(CATEGORY_REQUIRED.to_string());
    }
    if data.venue_id.is_empty() {
        return Err(VENUE_REQUIRED.to_string());
    }
    if data.start_datetime.is_empty() {
        return Err(START_REQUIRED.to_string());
    }
    if data.end_datetime.is_empty() {
        return Err(END_REQUIRED.to_string());
    }
    if data.start_datetime >= data.end_datetime {
        return Err(END_AFTER_START.to_string());
    }
    if data.capacity_max == 0 {
        return Err(CAPACITY_MAX_POSITIVE.to_string());
    }
    if data.capacity_min > data.capacity_max {
        return Err(CAPACITY_MIN_EXCEEDS_MAX.to_string());
    }
    Ok(())
}

pub fn event_form_payload(data: &EventFormData) -> Value {
    let mut map = Map::new();
    map.insert("title".to_string(), Value::String(data.title.clone()));
    map.insert(
        "description".to_string(),
        Value::String(data.description.clone()),
    );
    insert_optional_string(&mut map, "detailedDescription", &data.detailed_description);
    map.insert(
        "categoryId".to_string(),
        Value::String(data.category_id.clone()),
    );
    map.insert("venueId".to_string(), Value::String(data.venue_id.clone()));
    map.insert(
        "startDatetime".to_string(),
        Value::String(datetime_to_iso(&data.start_datetime)),
    );
    map.insert(
        "endDatetime".to_string(),
        Value::String(datetime_to_iso(&data.end_datetime)),
    );
    map.insert("timezone".to_string(), Value::String(data.timezone.clone()));
    map.insert(
        "capacityMin".to_string(),
        json_number(data.capacity_min as f64),
    );
    map.insert(
        "capacityMax".to_string(),
        json_number(data.capacity_max as f64),
    );
    map.insert(
        "pricePlatinum".to_string(),
        json_number(data.price_platinum),
    );
    map.insert("priceDiamond".to_string(), json_number(data.price_diamond));
    map.insert(
        "priceBlackCard".to_string(),
        json_number(data.price_black_card),
    );
    map.insert("currency".to_string(), Value::String(data.currency.clone()));
    map.insert(
        "requiredMembershipTiers".to_string(),
        Value::Array(
            data.required_membership_tiers
                .iter()
                .map(|tier| Value::String(tier.clone()))
                .collect(),
        ),
    );
    map.insert(
        "requiredVerification".to_string(),
        Value::Bool(data.required_verification),
    );
    insert_optional_string(&mut map, "dressCode", &data.dress_code);
    map.insert("language".to_string(), Value::String(data.language.clone()));
    insert_optional_string(&mut map, "specialRequirements", &data.special_requirements);
    map.insert(
        "inclusions".to_string(),
        Value::Array(
            data.inclusions
                .iter()
                .map(|item| Value::String(item.clone()))
                .collect(),
        ),
    );
    map.insert(
        "exclusions".to_string(),
        Value::Array(
            data.exclusions
                .iter()
                .map(|item| Value::String(item.clone()))
                .collect(),
        ),
    );
    insert_optional_iso(&mut map, "registrationOpensAt", &data.registration_opens_at);
    insert_optional_iso(
        &mut map,
        "registrationClosesAt",
        &data.registration_closes_at,
    );
    insert_optional_iso(
        &mut map,
        "cancellationDeadline",
        &data.cancellation_deadline,
    );
    map.insert(
        "waitlistEnabled".to_string(),
        Value::Bool(data.waitlist_enabled),
    );
    map.insert("autoApproval".to_string(), Value::Bool(data.auto_approval));
    insert_optional_string(&mut map, "metaTitle", &data.meta_title);
    insert_optional_string(&mut map, "metaDescription", &data.meta_description);
    insert_optional_string(&mut map, "featuredImage", &data.featured_image);
    insert_optional_string(&mut map, "internalNotes", &data.internal_notes);
    map.insert("profitMargin".to_string(), json_number(data.profit_margin));
    Value::Object(map)
}

pub fn datetime_to_iso(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains('Z') || trimmed.contains('+') || trimmed.rfind('-').is_some_and(|i| i > 10)
    {
        return trimmed.to_string();
    }
    if trimmed.len() == 16 {
        format!("{trimmed}:00.000Z")
    } else if trimmed.len() == 19 {
        format!("{trimmed}.000Z")
    } else {
        trimmed.to_string()
    }
}

pub fn parse_u32_input(raw: &str) -> u32 {
    raw.trim().parse().unwrap_or(0)
}

pub fn parse_f64_input(raw: &str) -> f64 {
    raw.trim().parse().unwrap_or(0.0)
}

pub fn media_kind_from_mime(mime: &str) -> Option<MediaKind> {
    if mime.starts_with("image/") {
        Some(MediaKind::Image)
    } else if ALLOWED_DOCUMENT_MIMES.contains(&mime) {
        Some(MediaKind::Document)
    } else {
        None
    }
}

pub fn media_kind_from_type_field(value: &str) -> Option<MediaKind> {
    match value {
        "image" => Some(MediaKind::Image),
        "document" => Some(MediaKind::Document),
        _ => None,
    }
}

pub fn is_allowed_mime(mime: &str) -> bool {
    ALLOWED_IMAGE_MIMES.contains(&mime) || ALLOWED_DOCUMENT_MIMES.contains(&mime)
}

pub fn validate_file(candidate: &FileCandidate, max_size_mb: u32) -> Result<(), String> {
    let max_bytes = u64::from(max_size_mb) * 1024 * 1024;
    if candidate.size > max_bytes {
        return Err(format!("File size must be less than {max_size_mb}MB"));
    }
    if !is_allowed_mime(&candidate.mime) {
        return Err("File type not supported. Please upload images or documents only.".to_string());
    }
    Ok(())
}

pub fn validate_file_for_kind(
    candidate: &FileCandidate,
    kind: MediaKind,
    max_size_mb: u32,
) -> Result<(), String> {
    validate_file(candidate, max_size_mb)?;
    match kind {
        MediaKind::Image if !candidate.mime.starts_with("image/") => {
            Err(format!("{}: Only images are allowed", candidate.name))
        }
        MediaKind::Document if candidate.mime.starts_with("image/") => {
            Err(format!("{}: Only documents are allowed", candidate.name))
        }
        _ => Ok(()),
    }
}

pub fn select_valid_files(
    candidates: &[FileCandidate],
    existing_count: u32,
    max_files: u32,
    kind: MediaKind,
    max_size_mb: u32,
) -> (Vec<FileCandidate>, Vec<String>) {
    let mut accepted = Vec::new();
    let mut errors = Vec::new();
    for candidate in candidates {
        match validate_file_for_kind(candidate, kind, max_size_mb) {
            Ok(()) => accepted.push(candidate.clone()),
            Err(message) => {
                if message.starts_with(&candidate.name) {
                    errors.push(message);
                } else {
                    errors.push(format!("{}: {message}", candidate.name));
                }
            }
        }
    }
    let allowed = max_files.saturating_sub(existing_count) as usize;
    if existing_count as usize + accepted.len() > max_files as usize {
        errors.push(format!("Maximum {max_files} files allowed"));
        accepted.truncate(allowed);
    }
    (accepted, errors)
}

pub fn format_file_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024.0;
    let sizes = ["B", "KB", "MB", "GB"];
    let mut index = (bytes as f64).log(k).floor() as usize;
    if index >= sizes.len() {
        index = sizes.len() - 1;
    }
    let value = bytes as f64 / k.powi(index as i32);
    format!("{} {}", trim_float(value), sizes[index])
}

pub fn encode_multipart(field: &str, files: &[UploadBytes]) -> (String, Vec<u8>) {
    let content_type = format!("multipart/form-data; boundary={MULTIPART_BOUNDARY}");
    let mut body = Vec::new();
    let marker = format!("--{MULTIPART_BOUNDARY}");
    for file in files {
        body.extend_from_slice(marker.as_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"{field}\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",
                file.filename, file.mime_type
            )
            .as_bytes(),
        );
        body.extend_from_slice(&file.bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(marker.as_bytes());
    body.extend_from_slice(b"--\r\n");
    (content_type, body)
}

pub fn tab_media_kind(tab: MediaTab) -> Option<MediaKind> {
    match tab {
        MediaTab::All => None,
        MediaTab::Images => Some(MediaKind::Image),
        MediaTab::Documents => Some(MediaKind::Document),
    }
}

pub fn format_price(price: f64, currency: &str) -> String {
    let symbol = match currency {
        "TWD" | "" => "NT$",
        other => other,
    };
    format!("{symbol}{}", format_amount(price))
}

pub fn format_event_datetime(iso: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        return format_event_datetime_js(iso);
    }
    #[cfg(not(target_arch = "wasm32"))]
    format_event_datetime_native(iso)
}

pub fn format_event_datetime_native(iso: &str) -> String {
    let date = iso.split('T').next().unwrap_or(iso);
    let time = iso
        .split('T')
        .nth(1)
        .map(|part| part.trim_end_matches('Z'))
        .unwrap_or("");
    let time = if time.len() >= 5 { &time[..5] } else { time };
    if time.is_empty() {
        date.to_string()
    } else {
        format!("{date} {time}")
    }
}

pub fn format_media_date(iso: &str) -> String {
    iso.split('T').next().unwrap_or(iso).to_string()
}

pub fn venue_option_label(venue: &EventVenue) -> String {
    format!(
        "{} - {} (Max: {})",
        venue.name, venue.city, venue.capacity_max
    )
}

pub fn parse_events_response(status: u16, body: &str) -> Result<EventsPage, String> {
    let value = parse_success_root(status, body, EVENTS_FETCH_FALLBACK)?;
    let events = match value.get("data") {
        Some(Value::Array(rows)) => rows.iter().filter_map(parse_managed_event).collect(),
        _ => return Err(EVENTS_FETCH_FALLBACK.to_string()),
    };
    Ok(EventsPage {
        events,
        pagination: parse_pagination(value.get("pagination")),
    })
}

pub fn parse_event_response(status: u16, body: &str) -> Result<ManagedEvent, String> {
    let value = parse_success_data(status, body, EVENT_FETCH_FALLBACK)?;
    parse_managed_event(&value).ok_or_else(|| EVENT_FETCH_FALLBACK.to_string())
}

pub fn parse_categories_response(status: u16, body: &str) -> Result<Vec<EventCategory>, String> {
    let value = parse_success_data(status, body, FORM_LOAD_FALLBACK)?;
    match value {
        Value::Array(rows) => Ok(rows.iter().filter_map(parse_category).collect()),
        _ => Err(FORM_LOAD_FALLBACK.to_string()),
    }
}

pub fn parse_venues_response(status: u16, body: &str) -> Result<Vec<EventVenue>, String> {
    let value = parse_success_root(status, body, FORM_LOAD_FALLBACK)?;
    let rows = match value.get("data") {
        Some(Value::Array(rows)) => rows,
        _ => return Err(FORM_LOAD_FALLBACK.to_string()),
    };
    Ok(rows.iter().filter_map(parse_venue).collect())
}

pub fn parse_create_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, CREATE_FALLBACK)?;
    Ok(())
}

pub fn parse_update_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, UPDATE_FALLBACK)?;
    Ok(())
}

pub fn parse_delete_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, DELETE_FALLBACK)?;
    Ok(())
}

pub fn parse_publish_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, PUBLISH_FALLBACK)?;
    Ok(())
}

pub fn parse_approve_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, APPROVE_FALLBACK)?;
    Ok(())
}

pub fn parse_media_list_response(status: u16, body: &str) -> Result<Vec<MediaAsset>, String> {
    let value = parse_success_data(status, body, MEDIA_FETCH_FALLBACK)?;
    match value {
        Value::Array(rows) => Ok(rows.iter().filter_map(parse_media_asset).collect()),
        _ => Err(MEDIA_FETCH_FALLBACK.to_string()),
    }
}

pub fn parse_image_upload_response(status: u16, body: &str) -> Result<Vec<MediaAsset>, String> {
    parse_media_upload_response(status, body, MediaKind::Image)
}

pub fn parse_document_upload_response(status: u16, body: &str) -> Result<Vec<MediaAsset>, String> {
    parse_media_upload_response(status, body, MediaKind::Document)
}

pub fn parse_media_upload_response(
    status: u16,
    body: &str,
    kind: MediaKind,
) -> Result<Vec<MediaAsset>, String> {
    let fallback = kind.upload_fallback();
    let value = parse_success_data(status, body, fallback)?;
    let key = match kind {
        MediaKind::Image => "uploadedImages",
        MediaKind::Document => "uploadedDocuments",
    };
    match value.get(key) {
        Some(Value::Array(rows)) => Ok(rows.iter().filter_map(parse_media_asset).collect()),
        _ => Ok(Vec::new()),
    }
}

pub fn parse_media_delete_response(status: u16, body: &str) -> Result<(), String> {
    let _ = parse_success_root(status, body, MEDIA_DELETE_FALLBACK)?;
    Ok(())
}

pub async fn fetch_managed_events(filters: &EventMgmtFilters) -> Result<EventsPage, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = events_list_url(filters);
        return authorized_get(&url, EVENTS_FETCH_FALLBACK, parse_events_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = filters;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_managed_event(id: &str) -> Result<ManagedEvent, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = event_api_path(id);
        return authorized_get(&url, EVENT_FETCH_FALLBACK, parse_event_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_categories() -> Result<Vec<EventCategory>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_get(
            CATEGORIES_API_PATH,
            FORM_LOAD_FALLBACK,
            parse_categories_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn fetch_venues() -> Result<Vec<EventVenue>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = venues_list_url();
        return authorized_get(&url, FORM_LOAD_FALLBACK, parse_venues_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err(NETWORK_ERROR.to_string())
}

pub async fn create_event(data: &EventFormData) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        return authorized_send(
            gloo_net::http::Request::post(EVENTS_API_PATH),
            Some(event_form_payload(data)),
            CREATE_FALLBACK,
            parse_create_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = data;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn update_event(id: &str, data: &EventFormData) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = event_api_path(id);
        return authorized_send(
            gloo_net::http::Request::put(&url),
            Some(event_form_payload(data)),
            UPDATE_FALLBACK,
            parse_update_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, data);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn delete_event(id: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = event_api_path(id);
        return authorized_send(
            gloo_net::http::Request::delete(&url),
            None,
            DELETE_FALLBACK,
            parse_delete_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn publish_event(id: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = event_publish_api_path(id);
        return authorized_send(
            gloo_net::http::Request::post(&url),
            None,
            PUBLISH_FALLBACK,
            parse_publish_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn approve_event(id: &str, approved: bool) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = event_approve_api_path(id);
        return authorized_send(
            gloo_net::http::Request::post(&url),
            Some(approve_payload(approved)),
            APPROVE_FALLBACK,
            parse_approve_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (id, approved);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn fetch_event_media(
    event_id: &str,
    kind: Option<MediaKind>,
) -> Result<Vec<MediaAsset>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = event_media_api_path(event_id, kind);
        return authorized_get(&url, MEDIA_FETCH_FALLBACK, parse_media_list_response).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, kind);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn upload_event_media(
    event_id: &str,
    kind: MediaKind,
    files: &[UploadBytes],
) -> Result<Vec<MediaAsset>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let url = match kind {
            MediaKind::Image => event_media_images_api_path(event_id),
            MediaKind::Document => event_media_documents_api_path(event_id),
        };
        let (content_type, body) = encode_multipart(kind.field_name(), files);
        let parse = match kind {
            MediaKind::Image => parse_image_upload_response,
            MediaKind::Document => parse_document_upload_response,
        };
        return authorized_multipart(&url, &content_type, body, kind.upload_fallback(), parse)
            .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (event_id, kind, files);
        Err(NETWORK_ERROR.to_string())
    }
}

pub async fn delete_media(media_id: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let url = media_item_api_path(media_id);
        return authorized_send(
            gloo_net::http::Request::delete(&url),
            None,
            MEDIA_DELETE_FALLBACK,
            parse_media_delete_response,
        )
        .await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = media_id;
        Err(NETWORK_ERROR.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
fn format_event_datetime_js(iso: &str) -> String {
    use wasm_bindgen::JsValue;
    let date = js_sys::Date::new(&JsValue::from_str(iso));
    if date.get_time().is_nan() {
        return iso.to_string();
    }
    let opts = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&opts, &"dateStyle".into(), &"medium".into());
    let _ = js_sys::Reflect::set(&opts, &"timeStyle".into(), &"short".into());
    date.to_locale_string("zh-TW", &opts).into()
}

#[cfg(target_arch = "wasm32")]
async fn authorized_get<T>(
    url: &str,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let builder = authorized_builder(gloo_net::http::Request::get(url))?;
    let response = builder
        .send()
        .await
        .map_err(|_| NETWORK_ERROR.to_string())?;
    finish_response(response, fallback, parse).await
}

#[cfg(target_arch = "wasm32")]
async fn authorized_send<T>(
    builder: gloo_net::http::RequestBuilder,
    body: Option<Value>,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let builder = authorized_builder(builder)?;
    let response = if let Some(payload) = body {
        let request = builder
            .header("Content-Type", "application/json")
            .json(&payload)
            .map_err(|_| NETWORK_ERROR.to_string())?;
        request
            .send()
            .await
            .map_err(|_| NETWORK_ERROR.to_string())?
    } else {
        builder
            .send()
            .await
            .map_err(|_| NETWORK_ERROR.to_string())?
    };
    finish_response(response, fallback, parse).await
}

#[cfg(target_arch = "wasm32")]
async fn authorized_multipart<T>(
    url: &str,
    content_type: &str,
    body: Vec<u8>,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let builder = authorized_builder(gloo_net::http::Request::post(url))?;
    let array = js_sys::Uint8Array::new_with_length(body.len() as u32);
    array.copy_from(&body);
    let request = builder
        .header("Content-Type", content_type)
        .body(array)
        .map_err(|_| NETWORK_ERROR.to_string())?;
    let response = request
        .send()
        .await
        .map_err(|_| NETWORK_ERROR.to_string())?;
    finish_response(response, fallback, parse).await
}

#[cfg(target_arch = "wasm32")]
fn authorized_builder(
    builder: gloo_net::http::RequestBuilder,
) -> Result<gloo_net::http::RequestBuilder, String> {
    let token = crate::auth::read_stored_token().ok_or_else(|| NETWORK_ERROR.to_string())?;
    Ok(builder.header("Authorization", &crate::auth::bearer_authorization(&token)))
}

#[cfg(target_arch = "wasm32")]
async fn finish_response<T>(
    response: gloo_net::http::Response,
    fallback: &'static str,
    parse: fn(u16, &str) -> Result<T, String>,
) -> Result<T, String> {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    parse(status, &text).map_err(|err| {
        if err.is_empty() {
            fallback.to_string()
        } else {
            err
        }
    })
}

fn parse_success_root(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    if status == 401 || status == 403 {
        let value = serde_json::from_str::<Value>(body).unwrap_or(Value::Null);
        return Err(json_error_message(&value, fallback));
    }
    let value: Value = serde_json::from_str(body).map_err(|_| fallback.to_string())?;
    if value.get("success").and_then(Value::as_bool) == Some(true) {
        Ok(value)
    } else {
        Err(json_error_message(&value, fallback))
    }
}

fn parse_success_data(status: u16, body: &str, fallback: &'static str) -> Result<Value, String> {
    let value = parse_success_root(status, body, fallback)?;
    match value.get("data") {
        Some(Value::Null) | None => Err(fallback.to_string()),
        Some(data) => Ok(data.clone()),
    }
}

fn json_error_message(value: &Value, fallback: &'static str) -> String {
    value
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|error| !error.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn parse_pagination(value: Option<&Value>) -> EventsPagination {
    let Some(value) = value else {
        return EventsPagination::default();
    };
    EventsPagination {
        page: json_u32(value.get("page")).unwrap_or(1),
        limit: json_u32(value.get("limit")).unwrap_or(PAGE_LIMIT),
        total: json_u32(value.get("total")).unwrap_or(0),
        total_pages: json_u32(value.get("totalPages")).unwrap_or(0),
    }
}

fn parse_managed_event(value: &Value) -> Option<ManagedEvent> {
    let id = json_id(value.get("id"))?;
    let title = json_string(value.get("title"))
        .or_else(|| json_string(value.get("name")))
        .unwrap_or_default();
    let venue_name = json_string(value.get("venue_name"))
        .or_else(|| {
            value
                .get("venue")
                .and_then(|venue| json_string(venue.get("name")))
        })
        .unwrap_or_default();
    let start_datetime = json_string(value.get("start_datetime"))
        .or_else(|| json_string(value.get("dateTime")))
        .unwrap_or_default();
    let pricing = value.get("pricing");
    Some(ManagedEvent {
        id,
        title,
        slug: json_string(value.get("slug")).unwrap_or_default(),
        description: json_string(value.get("description")).unwrap_or_default(),
        detailed_description: json_string(value.get("detailed_description")).unwrap_or_default(),
        category_id: json_id(value.get("category_id"))
            .or_else(|| {
                value
                    .get("category")
                    .and_then(|category| json_id(category.get("id")))
            })
            .unwrap_or_default(),
        venue_id: json_id(value.get("venue_id"))
            .or_else(|| {
                value
                    .get("venue")
                    .and_then(|venue| json_id(venue.get("id")))
            })
            .unwrap_or_default(),
        organizer_id: json_id(value.get("organizer_id")).unwrap_or_default(),
        start_datetime,
        end_datetime: json_string(value.get("end_datetime")).unwrap_or_default(),
        timezone: json_string(value.get("timezone")).unwrap_or_else(|| "Asia/Taipei".to_string()),
        capacity_min: json_u32(value.get("capacity_min")).unwrap_or(1),
        capacity_max: json_u32(value.get("capacity_max"))
            .or_else(|| json_u32(value.get("capacity")))
            .unwrap_or(0),
        current_registrations: json_u32(value.get("current_registrations"))
            .or_else(|| json_u32(value.get("currentAttendees")))
            .unwrap_or(0),
        price_platinum: json_f64(value.get("price_platinum"))
            .or_else(|| pricing.and_then(|p| json_f64(p.get("vip"))))
            .unwrap_or(0.0),
        price_diamond: json_f64(value.get("price_diamond"))
            .or_else(|| pricing.and_then(|p| json_f64(p.get("vvip"))))
            .unwrap_or(0.0),
        price_black_card: json_f64(value.get("price_black_card"))
            .or_else(|| pricing.and_then(|p| json_f64(p.get("general"))))
            .unwrap_or(0.0),
        currency: json_string(value.get("currency"))
            .or_else(|| pricing.and_then(|p| json_string(p.get("currency"))))
            .unwrap_or_else(|| "TWD".to_string()),
        status: json_string(value.get("status")).unwrap_or_default(),
        approval_status: json_string(value.get("approval_status")).unwrap_or_default(),
        required_membership_tiers: parse_string_list(value.get("required_membership_tiers")),
        required_verification: json_bool(value.get("required_verification")).unwrap_or(true),
        dress_code: json_string(value.get("dress_code"))
            .or_else(|| json_string(value.get("dressCode")))
            .unwrap_or_default(),
        language: json_string(value.get("language"))
            .unwrap_or_else(|| "Traditional Chinese".to_string()),
        special_requirements: json_string(value.get("special_requirements")).unwrap_or_default(),
        inclusions: parse_string_list(value.get("inclusions")),
        exclusions: parse_string_list(value.get("exclusions")),
        registration_opens_at: json_string(value.get("registration_opens_at")).unwrap_or_default(),
        registration_closes_at: json_string(value.get("registration_closes_at"))
            .or_else(|| json_string(value.get("registrationDeadline")))
            .unwrap_or_default(),
        cancellation_deadline: json_string(value.get("cancellation_deadline")).unwrap_or_default(),
        waitlist_enabled: json_bool(value.get("waitlist_enabled")).unwrap_or(true),
        auto_approval: json_bool(value.get("auto_approval")).unwrap_or(false),
        meta_title: json_string(value.get("meta_title")).unwrap_or_default(),
        meta_description: json_string(value.get("meta_description")).unwrap_or_default(),
        featured_image: json_string(value.get("featured_image")).unwrap_or_default(),
        internal_notes: json_string(value.get("internal_notes")).unwrap_or_default(),
        profit_margin: json_f64(value.get("profit_margin")).unwrap_or(0.0),
        venue_name,
        category_name: json_string(value.get("category_name"))
            .or_else(|| {
                value
                    .get("category")
                    .and_then(|category| json_string(category.get("name")))
            })
            .unwrap_or_default(),
        category_slug: json_string(value.get("category_slug"))
            .or_else(|| {
                value
                    .get("category")
                    .and_then(|category| json_string(category.get("slug")))
            })
            .unwrap_or_default(),
    })
}

fn parse_category(value: &Value) -> Option<EventCategory> {
    let id = json_id(value.get("id"))?;
    Some(EventCategory {
        id,
        name: json_string(value.get("name")).unwrap_or_default(),
        slug: json_string(value.get("slug")).unwrap_or_default(),
    })
}

fn parse_venue(value: &Value) -> Option<EventVenue> {
    let id = json_id(value.get("id"))?;
    Some(EventVenue {
        id,
        name: json_string(value.get("name")).unwrap_or_default(),
        city: json_string(value.get("city")).unwrap_or_default(),
        capacity_max: json_u32(value.get("capacity_max"))
            .or_else(|| json_u32(value.get("capacityMax")))
            .or_else(|| json_u32(value.get("capacity")))
            .unwrap_or(0),
    })
}

fn parse_media_asset(value: &Value) -> Option<MediaAsset> {
    let id = json_id(value.get("id"))?;
    let type_field = json_string(value.get("type")).unwrap_or_default();
    let mime = json_string(value.get("mimeType"))
        .or_else(|| json_string(value.get("mime_type")))
        .unwrap_or_default();
    let kind = media_kind_from_type_field(&type_field)
        .or_else(|| media_kind_from_mime(&mime))
        .unwrap_or(MediaKind::Document);
    let file_path = json_string(value.get("filePath"))
        .or_else(|| json_string(value.get("file_path")))
        .unwrap_or_default();
    let preview = thumbnail_url(value).unwrap_or_else(|| file_path.clone());
    Some(MediaAsset {
        id,
        kind,
        file_path,
        preview_url: preview,
        original_filename: json_string(value.get("originalFilename"))
            .or_else(|| json_string(value.get("original_filename")))
            .unwrap_or_default(),
        file_size: json_u64(value.get("fileSize"))
            .or_else(|| json_u64(value.get("file_size")))
            .unwrap_or(0),
        mime_type: mime,
        uploaded_by: json_string(value.get("uploadedBy"))
            .or_else(|| json_string(value.get("uploaded_by")))
            .unwrap_or_default(),
        created_at: json_string(value.get("createdAt"))
            .or_else(|| json_string(value.get("created_at")))
            .unwrap_or_default(),
    })
}

fn thumbnail_url(value: &Value) -> Option<String> {
    for key in ["thumbnails", "thumbnailPath", "thumbnail_path"] {
        if let Some(medium) = value
            .get(key)
            .and_then(|thumbnails| json_string(thumbnails.get("medium")))
            .filter(|url| !url.is_empty())
        {
            return Some(medium);
        }
    }
    None
}

fn parse_string_list(value: Option<&Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };
    match value {
        Value::Null => Vec::new(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().filter(|s| !s.is_empty()).map(str::to_string))
            .collect(),
        Value::String(text) => {
            if text.is_empty() {
                Vec::new()
            } else if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                parse_string_list(Some(&parsed))
            } else {
                vec![text.clone()]
            }
        }
        _ => Vec::new(),
    }
}

fn json_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn json_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn json_u32(value: Option<&Value>) -> Option<u32> {
    match value? {
        Value::Number(n) => n.as_u64().map(|v| v as u32),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_u64(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_f64(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn json_bool(value: Option<&Value>) -> Option<bool> {
    match value? {
        Value::Bool(flag) => Some(*flag),
        Value::Number(n) => Some(n.as_i64().unwrap_or(0) != 0),
        Value::String(s) => match s.as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn json_number(value: f64) -> Value {
    if value.is_finite() && value.fract() == 0.0 && value.abs() < 9_007_199_254_740_991.0 {
        Value::from(value as i64)
    } else if value.is_finite() {
        serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
    } else {
        Value::Null
    }
}

fn insert_optional_string(map: &mut Map<String, Value>, key: &str, value: &str) {
    if !value.is_empty() {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }
}

fn insert_optional_iso(map: &mut Map<String, Value>, key: &str, value: &str) {
    if !value.is_empty() {
        map.insert(key.to_string(), Value::String(datetime_to_iso(value)));
    }
}

fn format_amount(amount: f64) -> String {
    let negative = amount < 0.0;
    let abs = amount.abs();
    let whole = abs.trunc() as i64;
    let frac = ((abs.fract() * 100.0).round() as i64).clamp(0, 99);
    let mut digits = whole.to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        grouped.insert_str(0, &format!(",{}", &digits[split..]));
        digits.truncate(split);
    }
    grouped.insert_str(0, &digits);
    let formatted = format!("{grouped}.{frac:02}");
    if negative {
        format!("-{formatted}")
    } else {
        formatted
    }
}

fn trim_float(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded - rounded.trunc()).abs() < 1e-9 {
        format!("{}", rounded.trunc() as i64)
    } else {
        let text = format!("{rounded:.2}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn push_param(out: &mut String, key: &str, value: &str) {
    if !out.is_empty() {
        out.push('&');
    }
    out.push_str(key);
    out.push('=');
    out.push_str(&form_encode(value));
}

fn form_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
