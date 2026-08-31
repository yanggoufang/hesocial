use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewerRelationship {
    Unpaid,
    PaidStandard,
    PaidPremium,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantViewAccess {
    pub can_view_participants: bool,
    pub max_privacy_level_visible: i64,
    pub can_see_contact_info: bool,
    pub can_initiate_contact: bool,
    pub participant_count_visible: bool,
    pub access_level: i64,
}

impl ParticipantViewAccess {
    pub const fn denied() -> Self {
        Self {
            can_view_participants: false,
            max_privacy_level_visible: 0,
            can_see_contact_info: false,
            can_initiate_contact: false,
            participant_count_visible: true,
            access_level: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParticipantRow {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub age: Option<i64>,
    pub profession: Option<String>,
    pub company: Option<String>,
    pub city: Option<String>,
    pub membership_tier: String,
    pub interests: Option<String>,
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
    pub effective_privacy_level: i64,
    pub can_contact: i64,
}

pub fn viewer_relationship(
    payment_status: Option<&str>,
    membership_tier: &str,
) -> ViewerRelationship {
    if payment_status != Some("paid") {
        ViewerRelationship::Unpaid
    } else if matches!(membership_tier, "Diamond" | "Black Card") {
        ViewerRelationship::PaidPremium
    } else {
        ViewerRelationship::PaidStandard
    }
}

pub const fn participant_view_access(relationship: ViewerRelationship) -> ParticipantViewAccess {
    match relationship {
        ViewerRelationship::Unpaid => ParticipantViewAccess::denied(),
        ViewerRelationship::PaidStandard => ParticipantViewAccess {
            can_view_participants: true,
            max_privacy_level_visible: 3,
            can_see_contact_info: false,
            can_initiate_contact: true,
            participant_count_visible: true,
            access_level: 3,
        },
        ViewerRelationship::PaidPremium => ParticipantViewAccess {
            can_view_participants: true,
            max_privacy_level_visible: 5,
            can_see_contact_info: true,
            can_initiate_contact: true,
            participant_count_visible: true,
            access_level: 4,
        },
    }
}

fn profession_category(profession: Option<&str>) -> &'static str {
    let Some(profession) = profession else {
        return "Professional";
    };
    let profession = profession.to_lowercase();
    let categories: [(&str, &[&str]); 9] = [
        (
            "Technology",
            &[
                "software",
                "engineer",
                "developer",
                "tech",
                "it",
                "data",
                "ai",
                "machine learning",
            ],
        ),
        (
            "Finance",
            &[
                "finance",
                "banking",
                "investment",
                "fund",
                "trading",
                "analyst",
                "wealth",
            ],
        ),
        (
            "Healthcare",
            &[
                "doctor",
                "physician",
                "medical",
                "health",
                "nurse",
                "surgeon",
            ],
        ),
        (
            "Business",
            &[
                "ceo",
                "manager",
                "director",
                "executive",
                "business",
                "entrepreneur",
            ],
        ),
        (
            "Legal",
            &["lawyer", "attorney", "legal", "counsel", "judge"],
        ),
        (
            "Real Estate",
            &["real estate", "property", "development", "construction"],
        ),
        ("Consulting", &["consultant", "consulting", "advisory"]),
        (
            "Education",
            &["professor", "teacher", "education", "academic"],
        ),
        (
            "Media",
            &["media", "journalist", "marketing", "advertising", "pr"],
        ),
    ];

    categories
        .iter()
        .find(|(_, keywords)| keywords.iter().any(|keyword| profession.contains(keyword)))
        .map_or("Professional", |(category, _)| category)
}

fn company_category(company: &str) -> &'static str {
    let company = company.to_lowercase();
    if ["tech", "software", "microsoft", "google", "apple"]
        .iter()
        .any(|keyword| company.contains(keyword))
    {
        "Technology Company"
    } else if ["bank", "financial", "investment"]
        .iter()
        .any(|keyword| company.contains(keyword))
    {
        "Financial Services"
    } else {
        "Private Company"
    }
}

fn age_range(age: Option<i64>) -> &'static str {
    match age.unwrap_or(0) {
        ..=24 => "18-24",
        25..=29 => "25-29",
        30..=34 => "30-34",
        35..=39 => "35-39",
        40..=44 => "40-44",
        45..=49 => "45-49",
        50..=54 => "50-54",
        55..=59 => "55-59",
        60..=64 => "60-64",
        _ => "65+",
    }
}

fn interests(raw: Option<&str>) -> Vec<String> {
    raw.and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .unwrap_or_default()
        .into_iter()
        .take(3)
        .collect()
}

fn abbreviated_name(participant: &ParticipantRow) -> String {
    let initial = participant.last_name.chars().next().unwrap_or_default();
    format!("{} {initial}.", participant.first_name)
}

pub fn mask_participant(
    participant: &ParticipantRow,
    access: ParticipantViewAccess,
) -> Option<Value> {
    let level = participant.effective_privacy_level;
    if !access.can_view_participants || level > access.max_privacy_level_visible {
        return None;
    }

    let mut filtered = Map::new();
    filtered.insert("id".to_owned(), json!(participant.id));
    filtered.insert(
        "displayName".to_owned(),
        json!(abbreviated_name(participant)),
    );
    filtered.insert(
        "membershipTier".to_owned(),
        json!(participant.membership_tier),
    );
    filtered.insert("privacyLevel".to_owned(), json!(level));
    filtered.insert(
        "canContact".to_owned(),
        json!(participant.can_contact != 0 && access.can_initiate_contact),
    );

    if level >= 1 {
        filtered.insert(
            "profession".to_owned(),
            json!(profession_category(participant.profession.as_deref())),
        );
        filtered.insert(
            "interests".to_owned(),
            json!(interests(participant.interests.as_deref())),
        );
        filtered.insert(
            "profilePicture".to_owned(),
            json!(participant.profile_picture),
        );
        filtered.insert("ageRange".to_owned(), json!(age_range(participant.age)));
    }

    if level >= 2
        && let Some(company) = participant.company.as_deref()
    {
        filtered.insert("company".to_owned(), json!(company_category(company)));
    }
    if level >= 2
        && let Some(city) = participant.city.as_deref()
    {
        filtered.insert("city".to_owned(), json!(city));
    }

    if level >= 3 {
        filtered.insert(
            "displayName".to_owned(),
            json!(format!(
                "{} {}",
                participant.first_name, participant.last_name
            )),
        );
        if let Some(company) = participant.company.as_deref() {
            filtered.insert("company".to_owned(), json!(company));
        }
        if let Some(bio) = participant.bio.as_deref() {
            filtered.insert(
                "bio".to_owned(),
                json!(bio.chars().take(200).collect::<String>()),
            );
        }
    }

    if level >= 4 && access.can_see_contact_info {
        if let Some(bio) = participant.bio.as_deref() {
            filtered.insert("bio".to_owned(), json!(bio));
        }
        filtered.insert(
            "contactInfo".to_owned(),
            json!({ "email": participant.email }),
        );
    }

    if level >= 5 && access.can_see_contact_info {
        let mut contact = Map::new();
        contact.insert("email".to_owned(), json!(participant.email));
        if let Some(phone) = participant.phone.as_deref() {
            contact.insert("phone".to_owned(), json!(phone));
        }
        filtered.insert("contactInfo".to_owned(), Value::Object(contact));
    }

    Some(Value::Object(filtered))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant(level: i64) -> ParticipantRow {
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
    fn privacy_levels_one_through_five_are_hidden_from_unpaid_viewers() {
        let access = participant_view_access(ViewerRelationship::Unpaid);
        for level in 1..=5 {
            assert!(mask_participant(&participant(level), access).is_none());
        }
    }

    #[test]
    fn standard_paid_viewer_sees_levels_one_through_three_only() {
        let access = participant_view_access(ViewerRelationship::PaidStandard);
        for level in 1..=5 {
            assert_eq!(
                mask_participant(&participant(level), access).is_some(),
                level <= 3
            );
        }

        let level_one = mask_participant(&participant(1), access).expect("visible level one");
        assert_eq!(level_one["displayName"], "Ada L.");
        assert_eq!(level_one["profession"], "Technology");
        assert!(level_one.get("company").is_none());
        assert!(level_one.get("contactInfo").is_none());

        let level_two = mask_participant(&participant(2), access).expect("visible level two");
        assert_eq!(level_two["company"], "Technology Company");
        assert_eq!(level_two["city"], "London");

        let level_three = mask_participant(&participant(3), access).expect("visible level three");
        assert_eq!(level_three["displayName"], "Ada Lovelace");
        assert_eq!(level_three["company"], "Example Tech");
        assert_eq!(level_three["bio"].as_str().map(str::len), Some(200));
    }

    #[test]
    fn premium_paid_viewer_sees_all_five_levels_with_progressive_contact_fields() {
        let access = participant_view_access(ViewerRelationship::PaidPremium);
        for level in 1..=5 {
            let masked = mask_participant(&participant(level), access).expect("visible to premium");
            if level < 4 {
                assert!(masked.get("contactInfo").is_none());
            } else {
                assert_eq!(masked["contactInfo"]["email"], "ada@example.com");
                assert_eq!(masked["bio"].as_str().map(str::len), Some(220));
            }
            if level < 5 {
                assert!(
                    masked
                        .get("contactInfo")
                        .and_then(|value| value.get("phone"))
                        .is_none()
                );
            } else {
                assert_eq!(masked["contactInfo"]["phone"], "+886900000000");
            }
        }
    }
}
