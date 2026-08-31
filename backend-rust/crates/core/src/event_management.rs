use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::events::parse_json_column;
use crate::pagination::number_json;

/// JS `String.prototype.trim` ignores its argument, so the Express slug
/// builder's `.trim('-')` (backend/src/routes/eventManagement.ts:292) only
/// trims whitespace — leading/trailing dashes survive. Pinned deliberately.
pub fn slugify_title(title: &str, epoch_ms: u64) -> String {
    // Mirrors eventManagement.ts:288-292 step by step:
    //   title.toLowerCase()
    //     .replace(/[^a-z0-9\s-]/g, '')
    //     .replace(/\s+/g, '-')
    //     .replace(/-+/g, '-')
    //     .trim('-') + '-' + Date.now()
    let lowered = title.to_lowercase();
    let stripped: String = lowered
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace() || *c == '-')
        .collect();

    let mut dashed = String::with_capacity(stripped.len());
    let mut in_whitespace = false;
    for c in stripped.chars() {
        if c.is_whitespace() {
            if !in_whitespace {
                dashed.push('-');
                in_whitespace = true;
            }
        } else {
            dashed.push(c);
            in_whitespace = false;
        }
    }

    let mut collapsed = String::with_capacity(dashed.len());
    let mut in_dash = false;
    for c in dashed.chars() {
        if c == '-' {
            if !in_dash {
                collapsed.push('-');
                in_dash = true;
            }
        } else {
            collapsed.push(c);
            in_dash = false;
        }
    }

    // JS `.trim('-')` ignores its argument and trims whitespace only — by this
    // point all whitespace has become '-', so this is a no-op and any
    // leading/trailing dash survives (pinned deliberate quirk).
    let slug = collapsed.trim();
    format!("{slug}-{epoch_ms}")
}

fn optional_text_json(value: &Option<String>) -> Value {
    match value {
        Some(text) => parse_json_column(Value::String(text.clone())),
        None => Value::Null,
    }
}

fn bool_column(value: Option<i64>) -> Value {
    match value {
        Some(flag) => json!(flag != 0),
        None => Value::Null,
    }
}

fn number_column(value: Option<f64>) -> Value {
    value.map_or(Value::Null, number_json)
}

/// `PUT /api/events/:id` whitelist. The Express route only accepts the
/// snake_case columns (allowedFields at eventManagement.ts:373-385); the
/// camelCase entries are the EventForm (`CreateEventData`) keys the frontend
/// actually submits — the Rust port accepts both so the admin UI keeps
/// working (declared deviation: Express 400s on camelCase-only bodies).
pub const UPDATE_FIELD_MAP: &[(&str, &str)] = &[
    ("title", "title"),
    ("description", "description"),
    ("detailedDescription", "detailed_description"),
    ("categoryId", "category_id"),
    ("venueId", "venue_id"),
    ("startDatetime", "start_datetime"),
    ("endDatetime", "end_datetime"),
    ("timezone", "timezone"),
    ("capacityMin", "capacity_min"),
    ("capacityMax", "capacity_max"),
    ("pricePlatinum", "price_platinum"),
    ("priceDiamond", "price_diamond"),
    ("priceBlackCard", "price_black_card"),
    ("currency", "currency"),
    ("requiredMembershipTiers", "required_membership_tiers"),
    ("requiredVerification", "required_verification"),
    ("ageRestriction", "age_restriction"),
    ("dressCode", "dress_code"),
    ("language", "language"),
    ("specialRequirements", "special_requirements"),
    ("inclusions", "inclusions"),
    ("exclusions", "exclusions"),
    ("registrationOpensAt", "registration_opens_at"),
    ("registrationClosesAt", "registration_closes_at"),
    ("cancellationDeadline", "cancellation_deadline"),
    ("waitlistEnabled", "waitlist_enabled"),
    ("autoApproval", "auto_approval"),
    ("metaTitle", "meta_title"),
    ("metaDescription", "meta_description"),
    ("featuredImage", "featured_image"),
    ("internalNotes", "internal_notes"),
    ("costBreakdown", "cost_breakdown"),
    ("profitMargin", "profit_margin"),
    ("detailed_description", "detailed_description"),
    ("category_id", "category_id"),
    ("venue_id", "venue_id"),
    ("start_datetime", "start_datetime"),
    ("end_datetime", "end_datetime"),
    ("capacity_min", "capacity_min"),
    ("capacity_max", "capacity_max"),
    ("price_platinum", "price_platinum"),
    ("price_diamond", "price_diamond"),
    ("price_black_card", "price_black_card"),
    ("required_membership_tiers", "required_membership_tiers"),
    ("required_verification", "required_verification"),
    ("age_restriction", "age_restriction"),
    ("dress_code", "dress_code"),
    ("special_requirements", "special_requirements"),
    ("registration_opens_at", "registration_opens_at"),
    ("registration_closes_at", "registration_closes_at"),
    ("waitlist_enabled", "waitlist_enabled"),
    ("auto_approval", "auto_approval"),
    ("meta_title", "meta_title"),
    ("meta_description", "meta_description"),
    ("featured_image", "featured_image"),
    ("gallery_images", "gallery_images"),
    ("internal_notes", "internal_notes"),
    ("cost_breakdown", "cost_breakdown"),
    ("profit_margin", "profit_margin"),
];

/// Columns whose bound value is `JSON.stringify`'d before write
/// (eventManagement.ts:392).
pub fn is_json_column(column: &str) -> bool {
    matches!(
        column,
        "required_membership_tiers"
            | "age_restriction"
            | "inclusions"
            | "exclusions"
            | "gallery_images"
            | "cost_breakdown"
    )
}

/// Boolean-ish columns stored as INTEGER 0/1 in D1 (Express binds JS booleans
/// to DuckDB BOOLEAN).
pub fn is_bool_column(column: &str) -> bool {
    matches!(
        column,
        "required_verification" | "waitlist_enabled" | "auto_approval"
    )
}

pub fn update_column(key: &str) -> Option<&'static str> {
    UPDATE_FIELD_MAP
        .iter()
        .find(|(field, _)| *field == key)
        .map(|(_, column)| *column)
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegistrationStatsRow {
    pub total_registrations: i64,
    pub confirmed_registrations: i64,
    pub waitlisted_registrations: i64,
    pub pending_registrations: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventDetailRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "dateTime")]
    pub date_time: String,
    #[serde(rename = "registrationDeadline")]
    pub registration_deadline: Option<String>,
    pub price_platinum: Option<f64>,
    pub price_diamond: Option<f64>,
    pub price_black_card: Option<f64>,
    pub pricing_currency: String,
    #[serde(rename = "dressCode")]
    pub dress_code_alias: Option<String>,
    pub capacity: i64,
    #[serde(rename = "currentAttendees")]
    pub current_attendees: i64,
    pub images: Option<String>,
    pub requirements: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "venueId")]
    pub venue_id_alias: i64,
    #[serde(rename = "venueName")]
    pub venue_name: String,
    #[serde(rename = "venueAddress")]
    pub venue_address: String,
    #[serde(rename = "venueCity")]
    pub venue_city: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[serde(rename = "venueRating")]
    pub venue_rating: Option<i64>,
    #[serde(rename = "venueAmenities")]
    pub venue_amenities: Option<String>,
    #[serde(rename = "venueImages")]
    pub venue_images: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id_alias: i64,
    #[serde(rename = "categoryName")]
    pub category_name: String,
    #[serde(rename = "categoryDescription")]
    pub category_description: Option<String>,
    #[serde(rename = "categoryIcon")]
    pub category_icon: Option<String>,
    #[serde(rename = "organizerName")]
    pub organizer_name: String,
    // Raw management columns (merged into the response for admins only).
    pub slug: String,
    pub title: String,
    pub detailed_description: Option<String>,
    pub category_id: i64,
    pub venue_id: i64,
    pub organizer_id: String,
    pub start_datetime: String,
    pub end_datetime: String,
    pub timezone: Option<String>,
    pub capacity_min: Option<i64>,
    pub capacity_max: i64,
    pub current_registrations: i64,
    pub currency: String,
    pub status: Option<String>,
    pub approval_status: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub required_membership_tiers: Option<String>,
    pub required_verification: Option<i64>,
    pub age_restriction: Option<String>,
    pub dress_code: Option<String>,
    pub language: Option<String>,
    pub special_requirements: Option<String>,
    pub inclusions: Option<String>,
    pub exclusions: Option<String>,
    pub registration_opens_at: Option<String>,
    pub registration_closes_at: Option<String>,
    pub cancellation_deadline: Option<String>,
    pub waitlist_enabled: Option<i64>,
    pub auto_approval: Option<i64>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub featured_image: Option<String>,
    pub gallery_images: Option<String>,
    pub internal_notes: Option<String>,
    pub cost_breakdown: Option<String>,
    pub profit_margin: Option<f64>,
    pub published_at: Option<String>,
}

fn pricing_json(row: &EventDetailRow) -> Value {
    let mut pricing = Map::new();
    if let Some(vip) = row.price_platinum {
        pricing.insert("vip".to_owned(), number_json(vip));
    }
    if let Some(vvip) = row.price_diamond {
        pricing.insert("vvip".to_owned(), number_json(vvip));
    }
    if let Some(general) = row.price_black_card {
        pricing.insert("general".to_owned(), number_json(general));
    }
    pricing.insert(
        "currency".to_owned(),
        Value::String(row.pricing_currency.clone()),
    );
    Value::Object(pricing)
}

/// Public `GET /api/events/:id` (eventController.getEventById) shape, mapped
/// from the unified D1 schema like the list endpoint. `admin_stats` is
/// `Some((stats, waitlist_count))` only for admin callers; they additionally
/// receive the raw management columns (Phase 4 必辦: raw price_* fields).
pub fn event_detail_json(
    row: &EventDetailRow,
    admin_stats: Option<(&RegistrationStatsRow, i64)>,
) -> Value {
    let venue = json!({
        "id": row.venue_id_alias,
        "name": row.venue_name,
        "address": row.venue_address,
        "city": row.venue_city,
        "coordinates": {
            "lat": number_column(row.latitude),
            "lng": number_column(row.longitude),
        },
        "rating": row.venue_rating,
        "amenities": optional_text_json(&row.venue_amenities),
        "images": optional_text_json(&row.venue_images),
    });

    let category = json!({
        "id": row.category_id_alias,
        "name": row.category_name,
        "description": row.category_description,
        "icon": row.category_icon,
    });

    let mut event = Map::new();
    event.insert("id".to_owned(), json!(row.id));
    event.insert("name".to_owned(), json!(row.name));
    event.insert("description".to_owned(), json!(row.description));
    event.insert("dateTime".to_owned(), json!(row.date_time));
    event.insert(
        "registrationDeadline".to_owned(),
        json!(row.registration_deadline),
    );
    event.insert("pricing".to_owned(), pricing_json(row));
    event.insert("exclusivityLevel".to_owned(), Value::Null);
    event.insert("dressCode".to_owned(), json!(row.dress_code_alias));
    event.insert("capacity".to_owned(), json!(row.capacity));
    event.insert("currentAttendees".to_owned(), json!(row.current_attendees));
    event.insert("amenities".to_owned(), Value::Null);
    event.insert("privacyGuarantees".to_owned(), Value::Null);
    event.insert("images".to_owned(), optional_text_json(&row.images));
    event.insert("videoUrl".to_owned(), Value::Null);
    event.insert(
        "requirements".to_owned(),
        optional_text_json(&row.requirements),
    );
    event.insert("createdAt".to_owned(), json!(row.created_at));
    event.insert("updatedAt".to_owned(), json!(row.updated_at));
    event.insert("venue".to_owned(), venue);
    event.insert("category".to_owned(), category);
    event.insert("organizer".to_owned(), json!(row.organizer_name));

    if let Some((stats, waitlist_count)) = admin_stats {
        event.insert("slug".to_owned(), json!(row.slug));
        event.insert("title".to_owned(), json!(row.title));
        event.insert(
            "detailed_description".to_owned(),
            json!(row.detailed_description),
        );
        event.insert("category_id".to_owned(), json!(row.category_id));
        event.insert("venue_id".to_owned(), json!(row.venue_id));
        event.insert("organizer_id".to_owned(), json!(row.organizer_id));
        event.insert("start_datetime".to_owned(), json!(row.start_datetime));
        event.insert("end_datetime".to_owned(), json!(row.end_datetime));
        event.insert("timezone".to_owned(), json!(row.timezone));
        event.insert("capacity_min".to_owned(), json!(row.capacity_min));
        event.insert("capacity_max".to_owned(), json!(row.capacity_max));
        event.insert(
            "current_registrations".to_owned(),
            json!(row.current_registrations),
        );
        event.insert(
            "price_platinum".to_owned(),
            number_column(row.price_platinum),
        );
        event.insert("price_diamond".to_owned(), number_column(row.price_diamond));
        event.insert(
            "price_black_card".to_owned(),
            number_column(row.price_black_card),
        );
        event.insert("currency".to_owned(), json!(row.currency));
        event.insert("status".to_owned(), json!(row.status));
        event.insert("approval_status".to_owned(), json!(row.approval_status));
        event.insert("approved_by".to_owned(), json!(row.approved_by));
        event.insert("approved_at".to_owned(), json!(row.approved_at));
        event.insert(
            "required_membership_tiers".to_owned(),
            optional_text_json(&row.required_membership_tiers),
        );
        event.insert(
            "required_verification".to_owned(),
            bool_column(row.required_verification),
        );
        event.insert(
            "age_restriction".to_owned(),
            optional_text_json(&row.age_restriction),
        );
        event.insert("dress_code".to_owned(), json!(row.dress_code));
        event.insert("language".to_owned(), json!(row.language));
        event.insert(
            "special_requirements".to_owned(),
            json!(row.special_requirements),
        );
        event.insert("inclusions".to_owned(), optional_text_json(&row.inclusions));
        event.insert("exclusions".to_owned(), optional_text_json(&row.exclusions));
        event.insert(
            "registration_opens_at".to_owned(),
            json!(row.registration_opens_at),
        );
        event.insert(
            "registration_closes_at".to_owned(),
            json!(row.registration_closes_at),
        );
        event.insert(
            "cancellation_deadline".to_owned(),
            json!(row.cancellation_deadline),
        );
        event.insert(
            "waitlist_enabled".to_owned(),
            bool_column(row.waitlist_enabled),
        );
        event.insert("auto_approval".to_owned(), bool_column(row.auto_approval));
        event.insert("meta_title".to_owned(), json!(row.meta_title));
        event.insert("meta_description".to_owned(), json!(row.meta_description));
        event.insert("featured_image".to_owned(), json!(row.featured_image));
        event.insert(
            "gallery_images".to_owned(),
            optional_text_json(&row.gallery_images),
        );
        event.insert("internal_notes".to_owned(), json!(row.internal_notes));
        event.insert(
            "cost_breakdown".to_owned(),
            optional_text_json(&row.cost_breakdown),
        );
        event.insert("profit_margin".to_owned(), number_column(row.profit_margin));
        event.insert("published_at".to_owned(), json!(row.published_at));
        event.insert(
            "registration_stats".to_owned(),
            json!({
                "total_registrations": stats.total_registrations,
                "confirmed_registrations": stats.confirmed_registrations,
                "waitlisted_registrations": stats.waitlisted_registrations,
                "pending_registrations": stats.pending_registrations,
            }),
        );
        event.insert("waitlist_count".to_owned(), json!(waitlist_count));
    }

    Value::Object(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_detail_row() -> EventDetailRow {
        serde_json::from_str(r#"{
            "id": 2,
            "name": "Autumn Yacht Social",
            "description": "Sunset cruise around Keelung Harbor with a curated guest list.",
            "dateTime": "2026-10-10T09:00:00.000Z",
            "registrationDeadline": "2026-10-05T23:59:59.000Z",
            "price_platinum": 18000.0,
            "price_diamond": 18000.0,
            "price_black_card": 18000.0,
            "pricing_currency": "TWD",
            "dressCode": "Resort Casual",
            "capacity": 30,
            "currentAttendees": 1,
            "images": null,
            "requirements": null,
            "createdAt": "2026-08-30T00:00:00.000Z",
            "updatedAt": "2026-08-30T01:00:00.000Z",
            "venueId": 2,
            "venueName": "Keelung Luxury Yacht",
            "venueAddress": "Keelung Harbor Pier 8",
            "venueCity": "Keelung",
            "latitude": 25.13,
            "longitude": 121.739,
            "venueRating": 4,
            "venueAmenities": "[\"parking\", \"security\"]",
            "venueImages": null,
            "categoryId": 2,
            "categoryName": "遊艇派對",
            "categoryDescription": "豪華遊艇上的頂級社交聚會",
            "categoryIcon": "anchor",
            "organizerName": "Admin User",
            "slug": "autumn-yacht-social-2026-10",
            "title": "Autumn Yacht Social",
            "detailed_description": "Four hours on a chartered yacht, catering and sommelier included.",
            "category_id": 2,
            "venue_id": 2,
            "organizer_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "start_datetime": "2026-10-10T09:00:00.000Z",
            "end_datetime": "2026-10-10T15:00:00.000Z",
            "timezone": "Asia/Taipei",
            "capacity_min": 10,
            "capacity_max": 30,
            "current_registrations": 1,
            "currency": "TWD",
            "status": "published",
            "approval_status": "approved",
            "approved_by": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "approved_at": "2026-08-30T01:00:00.000Z",
            "required_membership_tiers": "[\"Diamond\", \"Black Card\"]",
            "required_verification": 1,
            "age_restriction": "{\"min\": 30, \"max\": null}",
            "dress_code": "Resort Casual",
            "language": "Traditional Chinese",
            "special_requirements": null,
            "inclusions": "[\"transportation\", \"meals\", \"beverages\", \"gifts\"]",
            "exclusions": "[\"photography\"]",
            "registration_opens_at": "2026-08-30T00:00:00.000Z",
            "registration_closes_at": "2026-10-05T23:59:59.000Z",
            "cancellation_deadline": "2026-10-07T23:59:59.000Z",
            "waitlist_enabled": 1,
            "auto_approval": 0,
            "meta_title": "Autumn Yacht Social",
            "meta_description": "Sunset cruise around Keelung Harbor.",
            "featured_image": null,
            "gallery_images": null,
            "internal_notes": null,
            "cost_breakdown": null,
            "profit_margin": 12.5,
            "published_at": "2026-08-30T01:00:00.000Z"
        }"#)
        .expect("detail row should deserialize")
    }

    #[test]
    fn slug_matches_the_express_generator() {
        assert_eq!(
            slugify_title("Michelin Private Dinner", 1_756_600_000_000),
            "michelin-private-dinner-1756600000000"
        );
        assert_eq!(
            slugify_title("Art & Wine: Vol. 2!", 1_756_600_000_000),
            "art-wine-vol-2-1756600000000"
        );
    }

    #[test]
    fn slug_pins_the_js_trim_dash_noop() {
        // JS `.trim('-')` ignores the '-' argument and trims whitespace only,
        // so leading/trailing dashes produced by the collapse survive.
        assert_eq!(
            slugify_title("  Hello World!!  ", 1_756_600_000_000),
            "-hello-world--1756600000000"
        );
    }

    #[test]
    fn update_whitelist_covers_express_snake_case_and_event_form_camel_case() {
        assert_eq!(update_column("price_platinum"), Some("price_platinum"));
        assert_eq!(update_column("pricePlatinum"), Some("price_platinum"));
        assert_eq!(update_column("gallery_images"), Some("gallery_images"));
        assert_eq!(
            update_column("detailedDescription"),
            Some("detailed_description")
        );
        assert_eq!(update_column("status"), None);
        assert_eq!(update_column("approval_status"), None);
        assert_eq!(update_column("organizer_id"), None);
        assert_eq!(update_column("id"), None);
        assert!(is_json_column("cost_breakdown"));
        assert!(!is_json_column("title"));
        assert!(is_bool_column("waitlist_enabled"));
        assert!(!is_bool_column("capacity_max"));
    }

    #[test]
    fn public_event_detail_matches_the_express_output_exactly() {
        let json = event_detail_json(&sample_detail_row(), None);
        assert_eq!(
            serde_json::to_string(&json).expect("detail JSON"),
            r#"{"id":2,"name":"Autumn Yacht Social","description":"Sunset cruise around Keelung Harbor with a curated guest list.","dateTime":"2026-10-10T09:00:00.000Z","registrationDeadline":"2026-10-05T23:59:59.000Z","pricing":{"vip":18000,"vvip":18000,"general":18000,"currency":"TWD"},"exclusivityLevel":null,"dressCode":"Resort Casual","capacity":30,"currentAttendees":1,"amenities":null,"privacyGuarantees":null,"images":null,"videoUrl":null,"requirements":null,"createdAt":"2026-08-30T00:00:00.000Z","updatedAt":"2026-08-30T01:00:00.000Z","venue":{"id":2,"name":"Keelung Luxury Yacht","address":"Keelung Harbor Pier 8","city":"Keelung","coordinates":{"lat":25.13,"lng":121.739},"rating":4,"amenities":["parking","security"],"images":null},"category":{"id":2,"name":"遊艇派對","description":"豪華遊艇上的頂級社交聚會","icon":"anchor"},"organizer":"Admin User"}"#
        );
    }

    #[test]
    fn admin_event_detail_adds_raw_management_columns_and_stats() {
        let stats = RegistrationStatsRow {
            total_registrations: 1,
            confirmed_registrations: 0,
            waitlisted_registrations: 0,
            pending_registrations: 1,
        };
        let json = event_detail_json(&sample_detail_row(), Some((&stats, 0)));

        assert_eq!(json.get("price_platinum"), Some(&json!(18000)));
        assert_eq!(json.get("price_diamond"), Some(&json!(18000)));
        assert_eq!(json.get("price_black_card"), Some(&json!(18000)));
        assert_eq!(json.get("status"), Some(&json!("published")));
        assert_eq!(json.get("approval_status"), Some(&json!("approved")));
        assert_eq!(json.get("waitlist_enabled"), Some(&json!(true)));
        assert_eq!(json.get("auto_approval"), Some(&json!(false)));
        assert_eq!(json.get("required_verification"), Some(&json!(true)));
        assert_eq!(
            json.get("required_membership_tiers"),
            Some(&json!(["Diamond", "Black Card"]))
        );
        assert_eq!(
            json.get("inclusions"),
            Some(&json!(["transportation", "meals", "beverages", "gifts"]))
        );
        assert_eq!(json.get("profit_margin"), Some(&json!(12.5)));
        assert_eq!(
            json.get("registration_stats"),
            Some(&json!({
                "total_registrations": 1,
                "confirmed_registrations": 0,
                "waitlisted_registrations": 0,
                "pending_registrations": 1,
            }))
        );
        assert_eq!(json.get("waitlist_count"), Some(&json!(0)));
        // Public keys stay present for admins.
        assert_eq!(json.get("name"), Some(&json!("Autumn Yacht Social")));
        assert!(json.get("venue").is_some());
    }
}
