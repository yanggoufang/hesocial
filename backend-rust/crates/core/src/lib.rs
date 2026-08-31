use std::error::Error;
use std::fmt::{Display, Formatter};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub mod auth;
pub mod event_management;
pub mod events;
pub mod oauth;
pub mod pagination;
pub mod participants;
pub mod pbkdf2;
pub mod registrations;

pub const JWT_EXPIRY_SECONDS: u64 = 7 * 24 * 60 * 60;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtPayload {
    pub user_id: String,
    pub email: String,
    pub membership_tier: String,
    pub iat: u64,
    pub exp: u64,
}

impl JwtPayload {
    pub fn with_seven_day_expiry(
        user_id: impl Into<String>,
        email: impl Into<String>,
        membership_tier: impl Into<String>,
        issued_at: u64,
    ) -> Self {
        Self::with_expiry(
            user_id,
            email,
            membership_tier,
            issued_at,
            JWT_EXPIRY_SECONDS,
        )
    }

    pub fn with_expiry(
        user_id: impl Into<String>,
        email: impl Into<String>,
        membership_tier: impl Into<String>,
        issued_at: u64,
        expiry_seconds: u64,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            email: email.into(),
            membership_tier: membership_tier.into(),
            iat: issued_at,
            exp: issued_at + expiry_seconds,
        }
    }
}

pub fn parse_jwt_expiry(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let digits = raw.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let value = raw[..digits].parse::<u64>().ok()?;
    if value == 0 {
        return None;
    }
    let multiplier = match raw[digits..].trim() {
        "" | "s" => 1,
        "m" => 60,
        "h" => 60 * 60,
        "d" => 24 * 60 * 60,
        "w" => 7 * 24 * 60 * 60,
        _ => return None,
    };
    value.checked_mul(multiplier)
}

#[derive(Debug, Eq, PartialEq)]
pub enum JwtError {
    Expired,
    InvalidFormat,
    InvalidHeader,
    InvalidSignature,
    Serialization(String),
}

impl Display for JwtError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Expired => formatter.write_str("token has expired"),
            Self::InvalidFormat => formatter.write_str("token has an invalid format"),
            Self::InvalidHeader => formatter.write_str("token is not an HS256 JWT"),
            Self::InvalidSignature => formatter.write_str("token signature is invalid"),
            Self::Serialization(message) => formatter.write_str(message),
        }
    }
}

impl Error for JwtError {}

pub fn sign_jwt(payload: &JwtPayload, secret: &str) -> Result<String, JwtError> {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload_json =
        serde_json::to_vec(payload).map_err(|error| JwtError::Serialization(error.to_string()))?;
    let payload = URL_SAFE_NO_PAD.encode(payload_json);
    let signing_input = format!("{header}.{payload}");
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| JwtError::InvalidSignature)?;
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    Ok(format!("{signing_input}.{signature}"))
}

pub fn verify_jwt(token: &str, secret: &str, now: u64) -> Result<JwtPayload, JwtError> {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(JwtError::InvalidFormat);
    };

    let header_json = URL_SAFE_NO_PAD
        .decode(header)
        .map_err(|_| JwtError::InvalidHeader)?;
    let header_value: serde_json::Value =
        serde_json::from_slice(&header_json).map_err(|_| JwtError::InvalidHeader)?;
    if header_value.get("alg").and_then(serde_json::Value::as_str) != Some("HS256") {
        return Err(JwtError::InvalidHeader);
    }

    let signature = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| JwtError::InvalidSignature)?;
    let signing_input = format!("{header}.{payload}");
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| JwtError::InvalidSignature)?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&signature)
        .map_err(|_| JwtError::InvalidSignature)?;

    let payload_json = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| JwtError::Serialization(error.to_string()))?;
    let payload: JwtPayload = serde_json::from_slice(&payload_json)
        .map_err(|error| JwtError::Serialization(error.to_string()))?;
    if payload.exp <= now {
        return Err(JwtError::Expired);
    }

    Ok(payload)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ApiEnvelope<T> {
    Success {
        success: bool,
        data: T,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Error {
        success: bool,
        error: String,
    },
}

impl<T> ApiEnvelope<T> {
    pub fn success(data: T) -> Self {
        Self::Success {
            success: true,
            data,
            message: None,
        }
    }

    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self::Success {
            success: true,
            data,
            message: Some(message.into()),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self::Error {
            success: false,
            error: error.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HealthResponse {
    pub success: bool,
    pub message: &'static str,
    pub timestamp: String,
    pub version: &'static str,
    pub database: &'static str,
}

impl HealthResponse {
    pub fn healthy(timestamp: String) -> Self {
        Self {
            success: true,
            message: "API health check passed (D1)",
            timestamp,
            version: "1.0.0",
            database: "d1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ADMIN_HASH: &str = "$2a$10$TC8bYbpDQYjwyi66LiZMYuaX6XAKcZMjQXtfoGV/8u6rQ7T.jj2N6";

    #[test]
    fn hs256_jwt_round_trip_uses_express_payload_names_and_expiry() {
        let issued_at = 1_700_000_000;
        let payload = JwtPayload::with_seven_day_expiry(
            "1000",
            "admin@hesocial.com",
            "Black Card",
            issued_at,
        );
        let token = sign_jwt(&payload, "test-secret").expect("JWT should serialize");
        let verified = verify_jwt(&token, "test-secret", issued_at).expect("JWT should verify");

        assert_eq!(verified, payload);
        assert_eq!(verified.iat, issued_at);
        assert_eq!(verified.exp, issued_at + 604_800);

        let encoded_payload = token.split('.').nth(1).expect("JWT payload segment");
        let decoded_payload = URL_SAFE_NO_PAD
            .decode(encoded_payload)
            .expect("base64url payload");
        let json_payload: serde_json::Value =
            serde_json::from_slice(&decoded_payload).expect("JSON payload");
        assert_eq!(
            json_payload,
            json!({
                "userId": "1000",
                "email": "admin@hesocial.com",
                "membershipTier": "Black Card",
                "iat": issued_at,
                "exp": issued_at + 604_800,
            })
        );
    }

    #[test]
    fn seeded_admin_bcrypt_hash_matches_only_the_expected_password() {
        assert!(verify_password("admin123", ADMIN_HASH));
        assert!(!verify_password("wrong-password", ADMIN_HASH));
    }

    #[test]
    fn api_envelopes_match_the_express_shapes_exactly() {
        let success = ApiEnvelope::success(json!({ "value": 42 }));
        let error = ApiEnvelope::<serde_json::Value>::error("Not found");

        assert_eq!(
            serde_json::to_string(&success).expect("success envelope"),
            r#"{"success":true,"data":{"value":42}}"#
        );
        assert_eq!(
            serde_json::to_string(&error).expect("error envelope"),
            r#"{"success":false,"error":"Not found"}"#
        );
    }

    #[test]
    fn auth_envelopes_carry_the_express_message_field() {
        let with_message =
            ApiEnvelope::success_with_message(json!({ "token": "jwt" }), "Login successful");
        let error = ApiEnvelope::<serde_json::Value>::error("Invalid email or password");

        assert_eq!(
            serde_json::to_string(&with_message).expect("message envelope"),
            r#"{"success":true,"data":{"token":"jwt"},"message":"Login successful"}"#
        );
        assert_eq!(
            serde_json::to_string(&error).expect("error envelope"),
            r#"{"success":false,"error":"Invalid email or password"}"#
        );
    }

    #[test]
    fn jsonwebtoken_signed_fixture_verifies_and_round_trips_bit_for_bit() {
        let express_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiJmNDdhYzEwYi01OGNjLTQzNzItYTU2Ny0wZTAyYjJjM2Q0NzkiLCJlbWFpbCI6ImFkbWluQGhlc29jaWFsLmNvbSIsIm1lbWJlcnNoaXBUaWVyIjoiQmxhY2sgQ2FyZCIsImlhdCI6MTcwMDAwMDAwMCwiZXhwIjoxNzAwMDYwNDgwfQ.7zCFaXQqWIEMI8-_k8Mh2rWKfGHUnIHP6bhMUohOqjc";
        let issued_at = 1_700_000_000;

        let claims =
            verify_jwt(express_token, "test-secret", issued_at).expect("Express token verifies");
        assert_eq!(
            claims,
            JwtPayload::with_expiry(
                "f47ac10b-58cc-4372-a567-0e02b2c3d479",
                "admin@hesocial.com",
                "Black Card",
                issued_at,
                60_480
            )
        );

        let rust_signed = sign_jwt(
            &JwtPayload::with_expiry(
                "1000",
                "admin@hesocial.com",
                "Black Card",
                issued_at,
                60_480,
            ),
            "test-secret",
        )
        .expect("JWT should serialize");
        assert_eq!(
            rust_signed,
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiIxMDAwIiwiZW1haWwiOiJhZG1pbkBoZXNvY2lhbC5jb20iLCJtZW1iZXJzaGlwVGllciI6IkJsYWNrIENhcmQiLCJpYXQiOjE3MDAwMDAwMDAsImV4cCI6MTcwMDA2MDQ4MH0.jS3uqD_FS5pUnpFhajJRfgcBPy1luxtB5vkyFeTDP1Q"
        );
    }

    #[test]
    fn jwt_expiry_accepts_the_jsonwebtoken_duration_spellings() {
        assert_eq!(parse_jwt_expiry("7d"), Some(7 * 24 * 60 * 60));
        assert_eq!(parse_jwt_expiry("12h"), Some(12 * 60 * 60));
        assert_eq!(parse_jwt_expiry("900s"), Some(900));
        assert_eq!(parse_jwt_expiry("30m"), Some(30 * 60));
        assert_eq!(parse_jwt_expiry("1w"), Some(7 * 24 * 60 * 60));
        assert_eq!(parse_jwt_expiry(" 7d "), Some(7 * 24 * 60 * 60));
        assert_eq!(parse_jwt_expiry("604800"), Some(604_800));
    }

    #[test]
    fn jwt_expiry_rejects_unsupported_or_degenerate_values() {
        assert_eq!(parse_jwt_expiry(""), None);
        assert_eq!(parse_jwt_expiry("d"), None);
        assert_eq!(parse_jwt_expiry("0d"), None);
        assert_eq!(parse_jwt_expiry("-5m"), None);
        assert_eq!(parse_jwt_expiry("1h30m"), None);
        assert_eq!(parse_jwt_expiry("fortnight"), None);
        assert_eq!(parse_jwt_expiry("1.5h"), None);
    }

    #[test]
    fn configured_expiry_changes_only_the_expiration_claim() {
        let issued_at = 1_700_000_000;
        let payload = JwtPayload::with_expiry("1000", "a@b.c", "Diamond", issued_at, 12 * 60 * 60);

        assert_eq!(payload.iat, issued_at);
        assert_eq!(payload.exp, issued_at + 12 * 60 * 60);

        let token = sign_jwt(&payload, "test-secret").expect("JWT should serialize");
        let claims = verify_jwt(&token, "test-secret", issued_at).expect("JWT should verify");
        assert_eq!(claims.exp, issued_at + 12 * 60 * 60);
        assert!(verify_jwt(&token, "test-secret", issued_at + 12 * 60 * 60).is_err());
    }
}
