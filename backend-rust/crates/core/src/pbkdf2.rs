use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha256;

pub const PBKDF2_ALGORITHM: &str = "pbkdf2";
pub const PBKDF2_ITERATIONS: u32 = 100_000;
const PBKDF2_SALT_LENGTH: usize = 16;
const PBKDF2_HASH_LENGTH: usize = 32;

#[derive(Debug, Eq, PartialEq)]
pub enum Pbkdf2Error {
    Randomness(String),
}

impl std::fmt::Display for Pbkdf2Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Randomness(message) => {
                formatter.write_str("secure randomness is unavailable: ")?;
                formatter.write_str(message)
            }
        }
    }
}

impl std::error::Error for Pbkdf2Error {}

pub fn encode(iterations: u32, salt: &[u8], hash: &[u8]) -> String {
    let salt = STANDARD_NO_PAD.encode(salt);
    let hash = STANDARD_NO_PAD.encode(hash);
    format!("{PBKDF2_ALGORITHM}${iterations}${salt}${hash}")
}

pub fn parse(encoded: &str) -> Option<(u32, Vec<u8>, Vec<u8>)> {
    let mut parts = encoded.split('$');
    let (Some(algorithm), Some(iterations), Some(salt), Some(hash), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return None;
    };
    if algorithm != PBKDF2_ALGORITHM {
        return None;
    }
    let iterations = iterations.parse::<u32>().ok()?;
    if iterations == 0 {
        return None;
    }
    let salt = STANDARD_NO_PAD.decode(salt).ok()?;
    let hash = STANDARD_NO_PAD.decode(hash).ok()?;
    Some((iterations, salt, hash))
}

pub fn hash_password(password: &str) -> Result<String, Pbkdf2Error> {
    let mut salt = [0u8; PBKDF2_SALT_LENGTH];
    getrandom::getrandom(&mut salt).map_err(|error| Pbkdf2Error::Randomness(error.to_string()))?;
    let hash = derive_array(password, &salt, PBKDF2_ITERATIONS);
    Ok(encode(PBKDF2_ITERATIONS, &salt, &hash))
}

pub fn verify_password(password: &str, encoded: &str) -> bool {
    let Some((iterations, salt, expected)) = parse(encoded) else {
        return false;
    };
    let derived =
        pbkdf2_hmac_array::<Sha256, PBKDF2_HASH_LENGTH>(password.as_bytes(), &salt, iterations);
    constant_time_eq(&derived, &expected)
}

fn derive_array(password: &str, salt: &[u8], iterations: u32) -> [u8; PBKDF2_HASH_LENGTH] {
    pbkdf2_hmac_array::<Sha256, PBKDF2_HASH_LENGTH>(password.as_bytes(), salt, iterations)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .fold(0u8, |accumulator, (left, right)| {
            accumulator | (left ^ right)
        })
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;
    use pbkdf2::pbkdf2_hmac as derive_raw;

    const RFC_7914_PASSWORD: &str = "password";
    const RFC_7914_SALT: &[u8] = b"salt";

    #[test]
    fn matches_the_pbkdf2_hmac_sha256_known_answer_vectors() {
        let mut one_round = [0u8; 32];
        derive_raw::<Sha256>(
            RFC_7914_PASSWORD.as_bytes(),
            RFC_7914_SALT,
            1,
            &mut one_round,
        );
        assert_eq!(
            STANDARD.encode(one_round),
            "Eg+2z/z4syxD5yJSVsT4N6hlSMkszDVICAWYfLcL4Xs="
        );

        let mut two_rounds = [0u8; 32];
        derive_raw::<Sha256>(
            RFC_7914_PASSWORD.as_bytes(),
            RFC_7914_SALT,
            2,
            &mut two_rounds,
        );
        assert_eq!(
            STANDARD.encode(two_rounds),
            "rk0Mla9rRtMtCt/5KPBt0CowP47zwlHf1uLYWpVHTEM="
        );

        assert_eq!(
            derive_array(RFC_7914_PASSWORD, RFC_7914_SALT, 1).to_vec(),
            one_round.to_vec()
        );
    }

    #[test]
    fn hashed_passwords_round_trip_and_reject_other_passwords() {
        let encoded = hash_password("phase-one-password").expect("hashing should succeed");

        assert!(encoded.starts_with("pbkdf2$100000$"));
        assert!(verify_password("phase-one-password", &encoded));
        assert!(!verify_password("not-the-password", &encoded));
        assert!(!verify_password("", &encoded));
    }

    #[test]
    fn wire_format_is_the_documented_four_part_string() {
        let encoded = encode(100_000, b"0123456789abcdef", &[0u8; 32]);
        assert_eq!(
            encoded,
            "pbkdf2$100000$MDEyMzQ1Njc4OWFiY2RlZg$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );

        let (iterations, salt, hash) = parse(&encoded).expect("wire format should parse");
        assert_eq!(iterations, 100_000);
        assert_eq!(salt, b"0123456789abcdef");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn parser_rejects_malformed_and_foreign_encodings() {
        assert!(parse("").is_none());
        assert!(parse("pbkdf2").is_none());
        assert!(parse("pbkdf2$100000$only-salt").is_none());
        assert!(parse("pbkdf2$abc$aaaa$bbbb").is_none());
        assert!(parse("pbkdf2$0$aaaa$bbbb").is_none());
        assert!(parse("pbkdf2$100000$not base64!$bbbb").is_none());
        assert!(parse("bcrypt$100000$aaaa$bbbb").is_none());
        assert!(parse("pbkdf2$100000$aaaa$bbbb$extra").is_none());
    }

    #[test]
    fn tampered_hashes_fail_verification() {
        let encoded = hash_password("phase-one-password").expect("hashing should succeed");
        let (iterations, salt, hash) = parse(&encoded).expect("encoding should parse");

        let mut flipped = hash.clone();
        flipped[0] ^= 0x01;
        assert!(!verify_password(
            "phase-one-password",
            &encode(iterations, &salt, &flipped)
        ));

        let mut truncated = hash.clone();
        truncated.truncate(31);
        assert!(!verify_password(
            "phase-one-password",
            &encode(iterations, &salt, &truncated)
        ));

        assert!(!verify_password(
            "phase-one-password",
            "not-a-pbkdf2-string"
        ));
    }

    #[test]
    fn every_hash_uses_a_fresh_salt() {
        let first = hash_password("same-password").expect("hashing should succeed");
        let second = hash_password("same-password").expect("hashing should succeed");

        assert_ne!(first, second);
        assert!(verify_password("same-password", &first));
        assert!(verify_password("same-password", &second));
    }
}
