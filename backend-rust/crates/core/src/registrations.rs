use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::events::parse_json_column;

#[derive(Clone, Debug, Deserialize)]
pub struct RegistrationEventRow {
    pub id: i64,
    pub registration_closes_at: Option<String>,
    pub start_datetime: String,
    pub capacity_max: i64,
    pub current_registrations: i64,
    pub required_membership_tiers: Option<String>,
    pub required_verification: i64,
    pub waitlist_enabled: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExistingRegistrationRow {
    pub id: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegistrationOwnerRow {
    pub id: i64,
    pub event_id: i64,
    pub user_id: String,
    pub status: String,
    pub event_date_time: String,
    pub registration_deadline: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RegistrationViewRow {
    pub id: i64,
    pub user_id: String,
    pub event_id: i64,
    pub status: String,
    pub payment_status: String,
    pub payment_intent_id: Option<String>,
    pub special_requests: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub event_name: String,
    pub event_description: Option<String>,
    pub event_date_time: String,
    pub registration_deadline: Option<String>,
    pub dress_code: Option<String>,
    pub capacity: i64,
    pub current_attendees: i64,
    pub price_platinum: Option<f64>,
    pub price_diamond: Option<f64>,
    pub price_black_card: Option<f64>,
    pub pricing_currency: String,
    pub venue_name: String,
    pub venue_address: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub venue_amenities: Option<String>,
    pub venue_images: Option<String>,
    pub category_name: String,
    pub category_description: Option<String>,
}

pub fn allowed_membership_tiers(raw: Option<&str>) -> Vec<String> {
    raw.and_then(|text| serde_json::from_str::<Vec<String>>(text).ok())
        .unwrap_or_default()
}

pub fn registration_eligibility_error(
    event: &RegistrationEventRow,
    membership_tier: &str,
    is_verified: bool,
    verification_status: &str,
    now_iso: &str,
) -> Option<String> {
    if event
        .registration_closes_at
        .as_deref()
        .is_some_and(|deadline| now_iso > deadline)
    {
        return Some("Registration deadline has passed".to_owned());
    }

    let allowed = allowed_membership_tiers(event.required_membership_tiers.as_deref());
    if !allowed.is_empty() && !allowed.iter().any(|tier| tier == membership_tier) {
        return Some(format!(
            "This event requires {} membership",
            allowed.join(" or ")
        ));
    }

    if event.required_verification != 0 && (!is_verified || verification_status != "approved") {
        return Some("This event requires verified membership status".to_owned());
    }

    None
}

pub fn registration_json(row: &RegistrationViewRow) -> Value {
    let mut registration = Map::new();
    registration.insert("id".to_owned(), json!(row.id));
    registration.insert("userId".to_owned(), json!(row.user_id));
    registration.insert("eventId".to_owned(), json!(row.event_id));
    registration.insert("status".to_owned(), json!(row.status));
    registration.insert("paymentStatus".to_owned(), json!(row.payment_status));
    registration.insert("paymentIntentId".to_owned(), json!(row.payment_intent_id));
    registration.insert("specialRequests".to_owned(), json!(row.special_requests));
    registration.insert("createdAt".to_owned(), json!(row.created_at));
    registration.insert("updatedAt".to_owned(), json!(row.updated_at));
    registration.insert("eventName".to_owned(), json!(row.event_name));
    registration.insert("eventDescription".to_owned(), json!(row.event_description));
    registration.insert("eventDateTime".to_owned(), json!(row.event_date_time));
    registration.insert(
        "registrationDeadline".to_owned(),
        json!(row.registration_deadline),
    );
    registration.insert("exclusivityLevel".to_owned(), Value::Null);
    registration.insert("dressCode".to_owned(), json!(row.dress_code));
    registration.insert("capacity".to_owned(), json!(row.capacity));
    registration.insert("currentAttendees".to_owned(), json!(row.current_attendees));
    registration.insert(
        "pricing".to_owned(),
        json!({
            "vip": row.price_platinum,
            "vvip": row.price_diamond,
            "general": row.price_black_card,
            "currency": row.pricing_currency,
        }),
    );
    registration.insert("amenities".to_owned(), Value::Null);
    registration.insert("privacyGuarantees".to_owned(), Value::Null);
    registration.insert("requirements".to_owned(), json!([]));
    registration.insert("eventImages".to_owned(), Value::Null);
    registration.insert("venueName".to_owned(), json!(row.venue_name));
    registration.insert("venueAddress".to_owned(), json!(row.venue_address));
    registration.insert("latitude".to_owned(), json!(row.latitude));
    registration.insert("longitude".to_owned(), json!(row.longitude));
    registration.insert(
        "venueAmenities".to_owned(),
        row.venue_amenities
            .as_ref()
            .map_or(Value::Null, |raw| parse_json_column(json!(raw))),
    );
    registration.insert(
        "venueImages".to_owned(),
        row.venue_images
            .as_ref()
            .map_or(Value::Null, |raw| parse_json_column(json!(raw))),
    );
    registration.insert("categoryName".to_owned(), json!(row.category_name));
    registration.insert(
        "categoryDescription".to_owned(),
        json!(row.category_description),
    );
    Value::Object(registration)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> RegistrationEventRow {
        RegistrationEventRow {
            id: 7,
            registration_closes_at: Some("2026-10-01T00:00:00.000Z".to_owned()),
            start_datetime: "2026-10-02T00:00:00.000Z".to_owned(),
            capacity_max: 10,
            current_registrations: 2,
            required_membership_tiers: Some("[\"Diamond\",\"Black Card\"]".to_owned()),
            required_verification: 1,
            waitlist_enabled: 1,
        }
    }

    #[test]
    fn eligibility_preserves_express_error_order_and_wording() {
        let mut event = event();
        assert_eq!(
            registration_eligibility_error(
                &event,
                "Platinum",
                false,
                "pending",
                "2026-08-31T00:00:00.000Z"
            ),
            Some("This event requires Diamond or Black Card membership".to_owned())
        );

        event.required_membership_tiers = Some("[\"Platinum\"]".to_owned());
        assert_eq!(
            registration_eligibility_error(
                &event,
                "Platinum",
                false,
                "pending",
                "2026-08-31T00:00:00.000Z"
            ),
            Some("This event requires verified membership status".to_owned())
        );

        assert_eq!(
            registration_eligibility_error(
                &event,
                "Platinum",
                true,
                "approved",
                "2026-10-01T00:00:00.001Z"
            ),
            Some("Registration deadline has passed".to_owned())
        );
    }

    #[test]
    fn registration_json_uses_the_frontend_field_names() {
        let row = RegistrationViewRow {
            id: 9,
            user_id: "user-1".to_owned(),
            event_id: 7,
            status: "pending".to_owned(),
            payment_status: "pending".to_owned(),
            payment_intent_id: None,
            special_requests: Some("Vegetarian".to_owned()),
            created_at: "2026-08-31T00:00:00.000Z".to_owned(),
            updated_at: "2026-08-31T00:00:00.000Z".to_owned(),
            event_name: "Gala".to_owned(),
            event_description: None,
            event_date_time: "2026-12-01T18:00:00.000Z".to_owned(),
            registration_deadline: None,
            dress_code: Some("Black Tie".to_owned()),
            capacity: 20,
            current_attendees: 1,
            price_platinum: Some(100.0),
            price_diamond: Some(90.0),
            price_black_card: Some(80.0),
            pricing_currency: "TWD".to_owned(),
            venue_name: "Room".to_owned(),
            venue_address: "Taipei".to_owned(),
            latitude: None,
            longitude: None,
            venue_amenities: Some("[\"valet\"]".to_owned()),
            venue_images: None,
            category_name: "Dinner".to_owned(),
            category_description: None,
        };

        let json = registration_json(&row);
        assert_eq!(json.get("eventName"), Some(&json!("Gala")));
        assert_eq!(json.get("eventDateTime"), Some(&json!(row.event_date_time)));
        assert_eq!(json.get("specialRequests"), Some(&json!("Vegetarian")));
        assert_eq!(json.get("venueAmenities"), Some(&json!(["valet"])));
        assert!(json.get("event_name").is_none());
    }
}
