use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::pagination::number_json;

pub fn parse_json_column(value: Value) -> Value {
    match value {
        Value::String(text) => match serde_json::from_str(&text) {
            Ok(parsed) => parsed,
            Err(_) => Value::String(text),
        },
        other => other,
    }
}

fn optional_text_json(value: &Option<String>) -> Value {
    match value {
        Some(text) => parse_json_column(Value::String(text.clone())),
        None => Value::Null,
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventListRow {
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
    pub dress_code: Option<String>,
    pub capacity: i64,
    #[serde(rename = "currentAttendees")]
    pub current_attendees: i64,
    pub images: Option<String>,
    pub requirements: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "venueName")]
    pub venue_name: String,
    #[serde(rename = "venueAddress")]
    pub venue_address: String,
    #[serde(rename = "venueCity")]
    pub venue_city: String,
    #[serde(rename = "venueRating")]
    pub venue_rating: Option<i64>,
    #[serde(rename = "venueAmenities")]
    pub venue_amenities: Option<String>,
    #[serde(rename = "categoryName")]
    pub category_name: String,
    #[serde(rename = "categoryIcon")]
    pub category_icon: Option<String>,
    #[serde(rename = "organizerName")]
    pub organizer_name: String,
}

pub fn event_list_item_json(row: &EventListRow) -> Value {
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

    let venue = json!({
        "name": row.venue_name,
        "address": row.venue_address,
        "city": row.venue_city,
        "rating": row.venue_rating,
        "amenities": optional_text_json(&row.venue_amenities),
    });

    let category = json!({
        "name": row.category_name,
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
    event.insert("pricing".to_owned(), Value::Object(pricing));
    event.insert("exclusivityLevel".to_owned(), Value::Null);
    event.insert("dressCode".to_owned(), json!(row.dress_code));
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
    event.insert("title".to_owned(), json!(row.name));
    event.insert("start_datetime".to_owned(), json!(row.date_time));
    event.insert("end_datetime".to_owned(), json!(row.date_time));
    event.insert(
        "current_registrations".to_owned(),
        json!(row.current_attendees),
    );
    event.insert("status".to_owned(), json!("published"));
    event.insert("approval_status".to_owned(), json!("approved"));
    event.insert("timezone".to_owned(), json!("Asia/Taipei"));
    event.insert("currency".to_owned(), json!("TWD"));
    event.insert("language".to_owned(), json!("zh-TW"));
    event.insert(
        "required_membership_tiers".to_owned(),
        json!(["Platinum", "Diamond", "Black Card"]),
    );
    event.insert("required_verification".to_owned(), json!(true));
    event.insert("venue".to_owned(), venue);
    event.insert("category".to_owned(), category);
    event.insert("organizer".to_owned(), json!(row.organizer_name));
    event.insert("venue_name".to_owned(), json!(row.venue_name));
    event.insert("category_name".to_owned(), json!(row.category_name));
    Value::Object(event)
}

#[derive(Clone, Debug, Deserialize)]
pub struct VenueListRow {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub city: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub rating: Option<i64>,
    pub amenities: Option<String>,
    pub images: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

pub fn venue_list_item_json(row: &VenueListRow) -> Value {
    let coordinates = json!({
        "lat": row.latitude,
        "lng": row.longitude,
    });

    let mut venue = Map::new();
    venue.insert("id".to_owned(), json!(row.id));
    venue.insert("name".to_owned(), json!(row.name));
    venue.insert("address".to_owned(), json!(row.address));
    venue.insert("city".to_owned(), json!(row.city));
    venue.insert("rating".to_owned(), json!(row.rating));
    venue.insert("amenities".to_owned(), optional_text_json(&row.amenities));
    venue.insert("images".to_owned(), optional_text_json(&row.images));
    venue.insert("createdAt".to_owned(), json!(row.created_at));
    venue.insert("coordinates".to_owned(), coordinates);
    Value::Object(venue)
}

#[derive(Clone, Debug, Deserialize)]
pub struct CategoryRow {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

pub fn category_json(row: &CategoryRow) -> Value {
    let mut category = Map::new();
    category.insert("id".to_owned(), json!(row.id));
    category.insert("name".to_owned(), json!(row.name));
    category.insert("description".to_owned(), json!(row.description));
    category.insert("icon".to_owned(), json!(row.icon));
    category.insert("createdAt".to_owned(), json!(row.created_at));
    Value::Object(category)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event_row() -> EventListRow {
        EventListRow {
            id: 2,
            name: "Autumn Yacht Social".to_owned(),
            description: Some("Sunset cruise around Keelung Harbor.".to_owned()),
            date_time: "2026-10-10T09:00:00.000Z".to_owned(),
            registration_deadline: Some("2026-10-05T23:59:59.000Z".to_owned()),
            price_platinum: Some(18000.0),
            price_diamond: Some(18000.0),
            price_black_card: Some(18000.0),
            pricing_currency: "TWD".to_owned(),
            dress_code: Some("Resort Casual".to_owned()),
            capacity: 30,
            current_attendees: 1,
            images: Some("[\"/images/yacht.jpg\"]".to_owned()),
            requirements: Some("[{\"type\":\"membership\"}]".to_owned()),
            created_at: "2026-08-30T00:00:00.000Z".to_owned(),
            updated_at: "2026-08-30T01:00:00.000Z".to_owned(),
            venue_name: "Keelung Luxury Yacht".to_owned(),
            venue_address: "Keelung Harbor Pier 8".to_owned(),
            venue_city: "Keelung".to_owned(),
            venue_rating: Some(4),
            venue_amenities: Some("[\"parking\", \"security\"]".to_owned()),
            category_name: "遊艇派對".to_owned(),
            category_icon: Some("anchor".to_owned()),
            organizer_name: "Admin User".to_owned(),
        }
    }

    #[test]
    fn event_list_item_matches_the_express_output_exactly() {
        let json = event_list_item_json(&sample_event_row());
        assert_eq!(
            serde_json::to_string(&json).expect("event JSON"),
            r#"{"id":2,"name":"Autumn Yacht Social","description":"Sunset cruise around Keelung Harbor.","dateTime":"2026-10-10T09:00:00.000Z","registrationDeadline":"2026-10-05T23:59:59.000Z","pricing":{"vip":18000,"vvip":18000,"general":18000,"currency":"TWD"},"exclusivityLevel":null,"dressCode":"Resort Casual","capacity":30,"currentAttendees":1,"amenities":null,"privacyGuarantees":null,"images":["/images/yacht.jpg"],"videoUrl":null,"requirements":[{"type":"membership"}],"createdAt":"2026-08-30T00:00:00.000Z","updatedAt":"2026-08-30T01:00:00.000Z","title":"Autumn Yacht Social","start_datetime":"2026-10-10T09:00:00.000Z","end_datetime":"2026-10-10T09:00:00.000Z","current_registrations":1,"status":"published","approval_status":"approved","timezone":"Asia/Taipei","currency":"TWD","language":"zh-TW","required_membership_tiers":["Platinum","Diamond","Black Card"],"required_verification":true,"venue":{"name":"Keelung Luxury Yacht","address":"Keelung Harbor Pier 8","city":"Keelung","rating":4,"amenities":["parking","security"]},"category":{"name":"遊艇派對","icon":"anchor"},"organizer":"Admin User","venue_name":"Keelung Luxury Yacht","category_name":"遊艇派對"}"#
        );
    }

    #[test]
    fn event_list_item_omits_null_prices_but_keeps_currency() {
        let mut row = sample_event_row();
        row.price_platinum = None;
        row.price_diamond = None;
        row.price_black_card = None;
        let json = event_list_item_json(&row);
        assert_eq!(
            json.get("pricing")
                .and_then(Value::as_object)
                .map(|p| { p.keys().cloned().collect::<Vec<_>>() }),
            Some(vec!["currency".to_owned()])
        );
    }

    #[test]
    fn venue_list_item_matches_the_express_output_exactly() {
        let row = VenueListRow {
            id: 1,
            name: "Taipei Private Dining Room".to_owned(),
            address: "No. 101, Dunhua S. Rd".to_owned(),
            city: "Taipei".to_owned(),
            latitude: Some(25.033),
            longitude: Some(121.5654),
            rating: Some(5),
            amenities: Some("[\"valet\", \"wine_cellar\"]".to_owned()),
            images: None,
            created_at: "2026-08-30T00:00:00.000Z".to_owned(),
        };
        let json = venue_list_item_json(&row);
        assert_eq!(
            serde_json::to_string(&json).expect("venue JSON"),
            r#"{"id":1,"name":"Taipei Private Dining Room","address":"No. 101, Dunhua S. Rd","city":"Taipei","rating":5,"amenities":["valet","wine_cellar"],"images":null,"createdAt":"2026-08-30T00:00:00.000Z","coordinates":{"lat":25.033,"lng":121.5654}}"#
        );
    }

    #[test]
    fn category_json_matches_the_express_output_exactly() {
        let row = CategoryRow {
            id: 1,
            name: "私人晚宴".to_owned(),
            description: Some("獨家私人晚宴體驗".to_owned()),
            icon: Some("chef-hat".to_owned()),
            created_at: "2026-08-30T00:00:00.000Z".to_owned(),
        };
        let json = category_json(&row);
        assert_eq!(
            serde_json::to_string(&json).expect("category JSON"),
            r#"{"id":1,"name":"私人晚宴","description":"獨家私人晚宴體驗","icon":"chef-hat","createdAt":"2026-08-30T00:00:00.000Z"}"#
        );
    }

    #[test]
    fn parse_json_column_is_defensive_about_non_json_text() {
        assert_eq!(
            parse_json_column(Value::String("[1, 2]".to_owned())),
            json!([1, 2])
        );
        assert_eq!(
            parse_json_column(Value::String("not json".to_owned())),
            json!("not json")
        );
        assert_eq!(parse_json_column(Value::Null), Value::Null);
    }
}
