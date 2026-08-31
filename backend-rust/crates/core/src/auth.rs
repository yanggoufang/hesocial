use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::events::parse_json_column;

pub const MEMBERSHIP_PLATINUM: &str = "Platinum";
pub const MEMBERSHIP_DIAMOND: &str = "Diamond";
pub const MEMBERSHIP_BLACK_CARD: &str = "Black Card";

const BLACK_CARD_NET_WORTH: f64 = 100_000_000.0;
const BLACK_CARD_ANNUAL_INCOME: f64 = 20_000_000.0;
const DIAMOND_NET_WORTH: f64 = 30_000_000.0;
const DIAMOND_ANNUAL_INCOME: f64 = 5_000_000.0;

pub const USER_COLUMNS: &str = "id, email, first_name AS \"firstName\", last_name AS \"lastName\", age, profession, annual_income AS \"annualIncome\", net_worth AS \"netWorth\", membership_tier AS \"membershipTier\", privacy_level AS \"privacyLevel\", is_verified AS \"isVerified\", verification_status AS \"verificationStatus\", role, profile_picture AS \"profilePicture\", bio, interests, created_at AS \"createdAt\", updated_at AS \"updatedAt\"";

pub const USER_SELECT_BY_ID: &str = "SELECT {columns} FROM users WHERE id = ?";
pub const USER_SELECT_BY_ID_ALIVE: &str =
    "SELECT {columns} FROM users WHERE id = ? AND deleted_at IS NULL";
pub const LOGIN_USER_SELECT: &str = "SELECT password_hash, password_algo, {columns} FROM users WHERE email = ? AND deleted_at IS NULL";

pub const DEFAULT_PRIVACY_LEVEL: i64 = 3;

pub const LEGACY_PASSWORD_ALGORITHM: &str = "bcrypt";

#[derive(Clone, Debug, Deserialize)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    // Nullable because Google-OAuth-created users are inserted with NULL
    // financial/profile fields (they complete them via /complete-profile),
    // exactly like the Express passport strategy.
    pub age: Option<i64>,
    pub profession: Option<String>,
    #[serde(rename = "annualIncome")]
    pub annual_income: Option<i64>,
    #[serde(rename = "netWorth")]
    pub net_worth: Option<i64>,
    #[serde(rename = "membershipTier")]
    pub membership_tier: String,
    #[serde(rename = "privacyLevel")]
    pub privacy_level: i64,
    #[serde(default)]
    #[serde(rename = "isVerified")]
    pub is_verified: i64,
    #[serde(default)]
    #[serde(rename = "verificationStatus")]
    pub verification_status: String,
    #[serde(default)]
    pub role: String,
    #[serde(rename = "profilePicture")]
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
    pub interests: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(default)]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub password_algo: Option<String>,
}

pub fn user_select(statement: &str) -> String {
    statement.replace("{columns}", USER_COLUMNS)
}

pub fn membership_tier_for(net_worth: f64, annual_income: f64) -> &'static str {
    if net_worth >= BLACK_CARD_NET_WORTH || annual_income >= BLACK_CARD_ANNUAL_INCOME {
        MEMBERSHIP_BLACK_CARD
    } else if net_worth >= DIAMOND_NET_WORTH || annual_income >= DIAMOND_ANNUAL_INCOME {
        MEMBERSHIP_DIAMOND
    } else {
        MEMBERSHIP_PLATINUM
    }
}

pub fn new_uuid_v4() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex = |slice: &[u8]| -> String {
        slice
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };

    Ok(format!(
        "{}-{}-{}-{}-{}",
        hex(&bytes[0..4]),
        hex(&bytes[4..6]),
        hex(&bytes[6..8]),
        hex(&bytes[8..10]),
        hex(&bytes[10..16])
    ))
}

pub fn user_json(row: &UserRow) -> Value {
    let mut user = Map::new();
    user.insert("id".to_owned(), json!(row.id));
    user.insert("email".to_owned(), json!(row.email));
    user.insert("firstName".to_owned(), json!(row.first_name));
    user.insert("lastName".to_owned(), json!(row.last_name));
    user.insert("age".to_owned(), json!(row.age));
    user.insert("profession".to_owned(), json!(row.profession));
    user.insert("annualIncome".to_owned(), json!(row.annual_income));
    user.insert("netWorth".to_owned(), json!(row.net_worth));
    user.insert("membershipTier".to_owned(), json!(row.membership_tier));
    user.insert("privacyLevel".to_owned(), json!(row.privacy_level));
    user.insert("isVerified".to_owned(), json!(row.is_verified != 0));
    user.insert(
        "verificationStatus".to_owned(),
        json!(row.verification_status),
    );
    user.insert("role".to_owned(), json!(row.role));
    user.insert("profilePicture".to_owned(), json!(row.profile_picture));
    user.insert("bio".to_owned(), json!(row.bio));
    user.insert(
        "interests".to_owned(),
        row.interests.as_ref().map_or(Value::Null, |raw| {
            parse_json_column(Value::String(raw.clone()))
        }),
    );
    user.insert("createdAt".to_owned(), json!(row.created_at));
    user.insert("updatedAt".to_owned(), json!(row.updated_at));
    Value::Object(user)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_admin() -> UserRow {
        serde_json::from_value(json!({
            "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "email": "admin@hesocial.com",
            "firstName": "Admin",
            "lastName": "User",
            "age": 40,
            "profession": "System Administrator",
            "annualIncome": 5_000_000,
            "netWorth": 30_000_000,
            "membershipTier": "Black Card",
            "privacyLevel": 5,
            "isVerified": 1,
            "verificationStatus": "approved",
            "role": "super_admin",
            "profilePicture": null,
            "bio": "Platform super administrator for development and system maintenance.",
            "interests": "[\"system administration\"]",
            "createdAt": "2026-08-30T00:00:00.000Z",
            "updatedAt": "2026-08-30T00:00:00.000Z"
        }))
        .expect("seeded admin row should deserialize")
    }

    #[test]
    fn membership_tier_uses_the_express_thresholds() {
        assert_eq!(
            membership_tier_for(100_000_000.0, 0.0),
            MEMBERSHIP_BLACK_CARD
        );
        assert_eq!(
            membership_tier_for(99_999_999.0, 20_000_000.0),
            MEMBERSHIP_BLACK_CARD
        );
        assert_eq!(
            membership_tier_for(500_000_000.0, 50_000_000.0),
            MEMBERSHIP_BLACK_CARD
        );
        assert_eq!(membership_tier_for(30_000_000.0, 0.0), MEMBERSHIP_DIAMOND);
        assert_eq!(
            membership_tier_for(29_999_999.0, 5_000_000.0),
            MEMBERSHIP_DIAMOND
        );
        assert_eq!(membership_tier_for(0.0, 0.0), MEMBERSHIP_PLATINUM);
        assert_eq!(
            membership_tier_for(29_999_999.0, 4_999_999.0),
            MEMBERSHIP_PLATINUM
        );
    }

    #[test]
    fn missing_financial_fields_stay_platinum_like_nan_comparisons() {
        assert_eq!(membership_tier_for(f64::NAN, f64::NAN), MEMBERSHIP_PLATINUM);
        assert_eq!(
            membership_tier_for(f64::NEG_INFINITY, 0.0),
            MEMBERSHIP_PLATINUM
        );
    }

    #[test]
    fn user_json_matches_the_express_select_order_and_types() {
        let json = user_json(&seeded_admin());
        assert_eq!(
            serde_json::to_string(&json).expect("user JSON"),
            r#"{"id":"f47ac10b-58cc-4372-a567-0e02b2c3d479","email":"admin@hesocial.com","firstName":"Admin","lastName":"User","age":40,"profession":"System Administrator","annualIncome":5000000,"netWorth":30000000,"membershipTier":"Black Card","privacyLevel":5,"isVerified":true,"verificationStatus":"approved","role":"super_admin","profilePicture":null,"bio":"Platform super administrator for development and system maintenance.","interests":["system administration"],"createdAt":"2026-08-30T00:00:00.000Z","updatedAt":"2026-08-30T00:00:00.000Z"}"#
        );
    }

    #[test]
    fn user_json_hides_password_columns_and_keeps_interests_null_when_absent() {
        let mut row = seeded_admin();
        row.password_hash = Some("pbkdf2$100000$abc$def".to_owned());
        row.password_algo = Some("pbkdf2".to_owned());
        row.interests = None;
        row.is_verified = 0;

        let json = user_json(&row);
        assert!(json.get("passwordHash").is_none());
        assert!(json.get("password_hash").is_none());
        assert_eq!(json.get("isVerified"), Some(&json!(false)));
        assert_eq!(json.get("interests"), Some(&Value::Null));
    }

    #[test]
    fn user_json_serializes_null_google_oauth_fields_as_null() {
        let row: UserRow = serde_json::from_value(json!({
            "id": "11111111-2222-4333-8444-555555555555",
            "email": "oauth@example.com",
            "firstName": "OAuth",
            "lastName": "User",
            "age": null,
            "profession": null,
            "annualIncome": null,
            "netWorth": null,
            "membershipTier": "Platinum",
            "privacyLevel": 3,
            "isVerified": 0,
            "verificationStatus": "pending",
            "role": "user",
            "profilePicture": "https://photo.example.com/p.jpg",
            "bio": null,
            "interests": "[]",
            "createdAt": "2026-08-31T00:00:00.000Z",
            "updatedAt": "2026-08-31T00:00:00.000Z"
        }))
        .expect("google-oauth user row should deserialize");

        let json = user_json(&row);
        assert_eq!(json.get("age"), Some(&Value::Null));
        assert_eq!(json.get("profession"), Some(&Value::Null));
        assert_eq!(json.get("annualIncome"), Some(&Value::Null));
        assert_eq!(json.get("netWorth"), Some(&Value::Null));
        assert_eq!(json.get("interests"), Some(&json!([])));
    }

    #[test]
    fn user_select_expands_the_shared_column_aliases() {
        let by_id = user_select(USER_SELECT_BY_ID);
        assert_eq!(
            by_id,
            "SELECT id, email, first_name AS \"firstName\", last_name AS \"lastName\", age, profession, annual_income AS \"annualIncome\", net_worth AS \"netWorth\", membership_tier AS \"membershipTier\", privacy_level AS \"privacyLevel\", is_verified AS \"isVerified\", verification_status AS \"verificationStatus\", role, profile_picture AS \"profilePicture\", bio, interests, created_at AS \"createdAt\", updated_at AS \"updatedAt\" FROM users WHERE id = ?"
        );
        assert!(user_select(LOGIN_USER_SELECT).contains("password_hash, password_algo,"));
        assert!(user_select(USER_SELECT_BY_ID_ALIVE).ends_with("AND deleted_at IS NULL"));
    }

    #[test]
    fn generated_uuids_are_version_four() {
        for _ in 0..32 {
            let uuid = new_uuid_v4().expect("randomness should be available");
            let parts: Vec<&str> = uuid.split('-').collect();
            assert_eq!(parts.len(), 5);
            assert_eq!(
                parts.iter().map(|part| part.len()).collect::<Vec<_>>(),
                vec![8, 4, 4, 4, 12]
            );
            assert!(uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
            assert_eq!(&uuid[14..15], "4");
        }

        let first = new_uuid_v4().expect("randomness should be available");
        let second = new_uuid_v4().expect("randomness should be available");
        assert_ne!(first, second);
    }
}
