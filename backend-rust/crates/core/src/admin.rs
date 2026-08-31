//! Admin / user-management domain logic (Phase 7 port of
//! `backend/src/routes/userManagement.ts` and the `GET /api/admin/database/stats`
//! route of `backend/src/routes/admin.ts`).
//!
//! Everything here is host-testable pure logic: SQL constants, the list
//! filter builder, the user-row JSON shaper, the PUT update whitelist, and
//! the response envelopes. The D1 boundary lives in
//! `worker/src/admin_handlers.rs`.
//!
//! ## Documented deviations (vs Express/DuckDB)
//!
//! - **ILIKE → LIKE**: SQLite has no ILIKE; LIKE is case-insensitive for
//!   ASCII only (DuckDB's ILIKE folds Unicode). Same deviation class as the
//!   public `/api/events` search, already recorded in the ROADMAP.
//! - **`is_verified`**: DuckDB returns a real BOOLEAN; D1 stores INTEGER 0/1.
//!   [`user_json`] converts back to `true`/`false` so the response matches
//!   Express (the sales phase does the same).
//! - **`interests`**: DuckDB's `VARCHAR[]` serializes to a JS array; D1 stores
//!   TEXT JSON. [`user_json`] parses the string back into an array; a value
//!   that fails to parse is passed through raw.
//! - **Timestamps**: DuckDB TIMESTAMPs serialize as ISO strings; D1 TEXT is
//!   already ISO-8601 UTC — passed through unchanged.
//! - **`serverStats` pinned quirk**: Express `getServerStats()` returns the
//!   DuckDB result *wrapper* (`{rows: [...]}`), and `admin.ts` then reads
//!   `serverStats.start_count` etc. off the wrapper — every key is
//!   `undefined`, so `JSON.stringify` drops them and the live response is
//!   `serverStats: {}`. Reproduced verbatim (query success → `{}`, query
//!   failure → 500 for the whole endpoint, exactly like Express).
//! - **`schemaVersion`**: D1 has no `schema_migrations` table (locked decision:
//!   wrangler d1 migrations instead). The probe query fails and the endpoint
//!   reports `'unknown'`, which is precisely the Express fallback path.
//! - **Negative page/limit**: Express passes `parseInt` output straight into
//!   `LIMIT ? OFFSET ?`; DuckDB rejects a negative LIMIT (500), SQLite treats
//!   it as unlimited (200). Edge case, not contract-pinned.

use serde_json::{Map, Value, json};

/// Verbatim column list from `userManagement.ts` (rows are returned raw,
/// snake_case keys and all).
pub const USER_LIST_SELECT: &str = "SELECT id, email, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, role, profile_picture, bio, interests, created_at, updated_at FROM users";

pub const USER_BY_ID_SQL: &str = "SELECT id, email, first_name, last_name, age, profession, annual_income, net_worth, membership_tier, privacy_level, is_verified, verification_status, role, profile_picture, bio, interests, created_at, updated_at FROM users WHERE id = ?";

pub const USER_COUNT_SQL: &str = "SELECT COUNT(*) as total FROM users";

pub const USER_EXISTS_SQL: &str = "SELECT id FROM users WHERE id = ?";

pub const DELETE_USER_SQL: &str = "DELETE FROM users WHERE id = ?";

pub const VERIFY_USER_SQL: &str =
    "UPDATE users SET verification_status = ?, is_verified = ?, updated_at = ? WHERE id = ?";

pub const UPDATE_ROLE_SQL: &str = "UPDATE users SET role = ?, updated_at = ? WHERE id = ?";

pub const USERS_BY_ROLE_SQL: &str = "SELECT role, COUNT(*) as count FROM users GROUP BY role";

pub const USERS_BY_TIER_SQL: &str =
    "SELECT membership_tier, COUNT(*) as count FROM users GROUP BY membership_tier";

pub const USERS_BY_VERIFICATION_SQL: &str =
    "SELECT verification_status, COUNT(*) as count FROM users GROUP BY verification_status";

pub const RECENT_REGISTRATIONS_SQL: &str =
    "SELECT COUNT(*) as recent FROM users WHERE created_at >= date('now', '-30 days')";

/// D1 replacement for Express's DuckDB `information_schema` query.
pub const TABLES_SQL: &str = "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_cf_%' ORDER BY name";

/// Bound table-valued pragma: one column-count per table name.
pub const COLUMN_COUNT_SQL: &str = "SELECT COUNT(*) AS column_count FROM pragma_table_info(?)";

/// D1 has no `schema_migrations`; the query is expected to fail and the
/// caller falls back to `'unknown'`, mirroring the Express try/catch.
pub const SCHEMA_VERSION_SQL: &str =
    "SELECT version FROM schema_migrations ORDER BY id DESC LIMIT 1";

/// Express runs `SELECT * FROM server_state WHERE id = 1` (see the pinned
/// `serverStats` quirk above: the row itself is never actually read).
pub const SERVER_STATE_SQL: &str = "SELECT * FROM server_state WHERE id = 1";

pub const VERIFY_STATUSES: [&str; 2] = ["approved", "rejected"];

pub const ROLES: [&str; 3] = ["user", "admin", "super_admin"];

/// JS `parseInt(raw) || fallback`: leading-integer prefix parse (sign and
/// leading whitespace allowed, `"20abc"` → 20, `"3.9"` → 3); empty, garbage,
/// zero, and overflow all fall back.
pub fn js_parse_int_or(raw: Option<&str>, fallback: i64) -> i64 {
    let Some(raw) = raw else { return fallback };
    let trimmed = raw.trim_start();
    let digits: String = trimmed
        .chars()
        .skip_while(|c| *c == '+' || *c == '-')
        .take_while(char::is_ascii_digit)
        .collect();
    if digits.is_empty() {
        return fallback;
    }
    let negative = trimmed.starts_with('-');
    let Ok(magnitude) = digits.parse::<i64>() else {
        return fallback;
    };
    let value = if negative { -magnitude } else { magnitude };
    if value == 0 { fallback } else { value }
}

/// Query-string filters for `GET /api/users`. Express tests each with a bare
/// `if (value)`, so empty strings are skipped — the worker filters those out
/// before calling [`list_where`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListFilters<'a> {
    pub search: Option<&'a str>,
    pub role: Option<&'a str>,
    pub membership_tier: Option<&'a str>,
    pub verification_status: Option<&'a str>,
}

/// Builds the `WHERE ...` clause (empty string when no filters) and the bind
/// parameters in order. LIKE stands in for DuckDB's ILIKE (see module docs).
pub fn list_where(filters: &ListFilters) -> (String, Vec<String>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(search) = filters.search {
        conditions.push("(first_name LIKE ? OR last_name LIKE ? OR email LIKE ?)".to_owned());
        let pattern = format!("%{search}%");
        for _ in 0..3 {
            params.push(pattern.clone());
        }
    }
    if let Some(role) = filters.role {
        conditions.push("role = ?".to_owned());
        params.push(role.to_owned());
    }
    if let Some(tier) = filters.membership_tier {
        conditions.push("membership_tier = ?".to_owned());
        params.push(tier.to_owned());
    }
    if let Some(status) = filters.verification_status {
        conditions.push("verification_status = ?".to_owned());
        params.push(status.to_owned());
    }

    let clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };
    (clause, params)
}

/// Shapes one raw D1 user row into the Express response row: `is_verified`
/// back to a boolean, `interests` TEXT parsed back into an array (raw string
/// passthrough when it does not parse), everything else unchanged.
pub fn user_json(row: &Value) -> Value {
    let Value::Object(map) = row else {
        return row.clone();
    };
    let mut out = map.clone();

    if let Some(Value::Number(number)) = out.get("is_verified") {
        let flag = number
            .as_i64()
            .map(|value| value != 0)
            .or_else(|| number.as_f64().map(|value| value != 0.0))
            .unwrap_or(false);
        out.insert("is_verified".to_owned(), Value::Bool(flag));
    }

    if let Some(Value::String(raw)) = out.get("interests") {
        if let Ok(parsed @ Value::Array(_)) = serde_json::from_str::<Value>(raw) {
            out.insert("interests".to_owned(), parsed);
        }
    }

    Value::Object(out)
}

/// Express's fixed camelCase → column whitelist, in the exact order the
/// Express handler checks them (the SET clause order is observable to a
/// characterization test diffing SQL logs, and keeping it costs nothing).
pub const UPDATE_FIELDS: [(&str, &str); 13] = [
    ("firstName", "first_name"),
    ("lastName", "last_name"),
    ("age", "age"),
    ("profession", "profession"),
    ("annualIncome", "annual_income"),
    ("netWorth", "net_worth"),
    ("membershipTier", "membership_tier"),
    ("privacyLevel", "privacy_level"),
    ("isVerified", "is_verified"),
    ("verificationStatus", "verification_status"),
    ("role", "role"),
    ("bio", "bio"),
    ("interests", "interests"),
];

/// Ordered `(column, value)` pairs for the recognized body keys. Express
/// tests `field !== undefined`, so a present-but-null key IS bound (stores
/// NULL) and unknown keys are ignored. `interests` is pre-serialized with
/// `JSON.stringify` semantics (compact JSON text). Returns `None` when no
/// recognized key is present — Express's 400 'No valid fields to update'.
pub fn update_assignments(body: &Value) -> Option<Vec<(&'static str, Value)>> {
    let fields = body.as_object()?;
    let mut assignments = Vec::new();
    for (key, column) in UPDATE_FIELDS {
        if let Some(value) = fields.get(key) {
            let value = if column == "interests" {
                Value::String(value.to_string())
            } else {
                value.clone()
            };
            assignments.push((column, value));
        }
    }
    if assignments.is_empty() {
        None
    } else {
        Some(assignments)
    }
}

/// `User ${status === 'approved' ? 'verified' : 'rejected'} successfully`.
pub fn verify_message(status: &str) -> String {
    let verb = if status == "approved" {
        "verified"
    } else {
        "rejected"
    };
    format!("User {verb} successfully")
}

/// `GET /api/users/stats/overview` envelope. The three breakdowns are raw
/// `GROUP BY` rows (`{role, count}` etc.), exactly as Express returns them.
pub fn stats_envelope(
    total_users: i64,
    users_by_role: Vec<Value>,
    users_by_tier: Vec<Value>,
    users_by_verification: Vec<Value>,
    recent_registrations: i64,
) -> Value {
    json!({
        "success": true,
        "data": {
            "totalUsers": total_users,
            "usersByRole": users_by_role,
            "usersByMembershipTier": users_by_tier,
            "usersByVerificationStatus": users_by_verification,
            "recentRegistrations": recent_registrations,
        }
    })
}

/// Pinned Express quirk: the `serverStats` object whose keys were all read
/// off the DuckDB result wrapper and therefore serialize to nothing.
pub fn server_stats_json() -> Value {
    json!({})
}

/// One `{name, columnCount}` entry of the `tables` array.
pub fn table_json(name: &str, column_count: i64) -> Value {
    json!({ "name": name, "columnCount": column_count })
}

/// `GET /api/admin/database/stats` envelope.
pub fn database_stats_envelope(
    schema_version: &str,
    server_stats: Value,
    tables: Vec<Value>,
    timestamp: &str,
) -> Value {
    let total_tables = tables.len() as i64;
    let mut data = Map::new();
    data.insert(
        "schemaVersion".to_owned(),
        Value::String(schema_version.to_owned()),
    );
    data.insert("serverStats".to_owned(), server_stats);
    data.insert("tables".to_owned(), Value::Array(tables));
    data.insert(
        "meta".to_owned(),
        json!({ "totalTables": total_tables, "timestamp": timestamp }),
    );

    let mut body = Map::new();
    body.insert("success".to_owned(), json!(true));
    body.insert("data".to_owned(), Value::Object(data));
    Value::Object(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_parse_int_or_mirrors_parse_int_or_fallback() {
        assert_eq!(js_parse_int_or(None, 1), 1);
        assert_eq!(js_parse_int_or(Some(""), 20), 20);
        assert_eq!(js_parse_int_or(Some("  2"), 1), 2);
        assert_eq!(js_parse_int_or(Some("20abc"), 1), 20);
        assert_eq!(js_parse_int_or(Some("3.9"), 1), 3);
        assert_eq!(js_parse_int_or(Some("abc"), 20), 20);
        assert_eq!(js_parse_int_or(Some("0"), 1), 1); // 0 is falsy in JS
        assert_eq!(js_parse_int_or(Some("-5"), 20), -5); // negative passes through
        assert_eq!(js_parse_int_or(Some("99999999999999999999"), 20), 20);
    }

    #[test]
    fn list_where_with_no_filters_is_empty() {
        let (clause, params) = list_where(&ListFilters::default());
        assert_eq!(clause, "");
        assert!(params.is_empty());
    }

    #[test]
    fn list_where_combines_filters_in_express_order() {
        let (clause, params) = list_where(&ListFilters {
            search: Some("admin"),
            role: Some("super_admin"),
            membership_tier: Some("Black Card"),
            verification_status: Some("approved"),
        });
        assert_eq!(
            clause,
            "WHERE (first_name LIKE ? OR last_name LIKE ? OR email LIKE ?) AND role = ? AND membership_tier = ? AND verification_status = ?"
        );
        assert_eq!(
            params,
            vec![
                "%admin%".to_owned(),
                "%admin%".to_owned(),
                "%admin%".to_owned(),
                "super_admin".to_owned(),
                "Black Card".to_owned(),
                "approved".to_owned(),
            ]
        );
    }

    #[test]
    fn list_where_skips_absent_filters() {
        let (clause, params) = list_where(&ListFilters {
            role: Some("user"),
            ..ListFilters::default()
        });
        assert_eq!(clause, "WHERE role = ?");
        assert_eq!(params, vec!["user".to_owned()]);
    }

    #[test]
    fn user_json_converts_is_verified_to_boolean() {
        let row = json!({"id": "1", "is_verified": 1, "interests": null});
        assert_eq!(user_json(&row)["is_verified"], json!(true));
        let row = json!({"id": "1", "is_verified": 0});
        assert_eq!(user_json(&row)["is_verified"], json!(false));
    }

    #[test]
    fn user_json_parses_interests_text_back_into_an_array() {
        let row = json!({"interests": "[\"art\", \"yachting\"]"});
        assert_eq!(user_json(&row)["interests"], json!(["art", "yachting"]));
        // Unparseable values pass through raw, as the frontend tolerates.
        let row = json!({"interests": "not json"});
        assert_eq!(user_json(&row)["interests"], json!("not json"));
    }

    #[test]
    fn update_assignments_follow_the_fixed_express_field_order() {
        let body = json!({"role": "admin", "firstName": "Ada", "unknown": "ignored"});
        let assignments = update_assignments(&body).expect("recognized fields");
        let columns: Vec<&str> = assignments.iter().map(|(column, _)| *column).collect();
        assert_eq!(columns, vec!["first_name", "role"]);
    }

    #[test]
    fn update_assignments_binds_present_null_and_stringifies_interests() {
        let body = json!({"bio": null, "interests": ["art"]});
        let assignments = update_assignments(&body).expect("recognized fields");
        assert_eq!(assignments[0], ("bio", Value::Null));
        assert_eq!(
            assignments[1],
            ("interests", Value::String("[\"art\"]".to_owned()))
        );
    }

    #[test]
    fn update_assignments_returns_none_without_recognized_fields() {
        assert_eq!(update_assignments(&json!({})), None);
        assert_eq!(update_assignments(&json!({"email": "x@y.z"})), None);
        assert_eq!(update_assignments(&json!(42)), None);
    }

    #[test]
    fn verify_message_matches_express_wording() {
        assert_eq!(verify_message("approved"), "User verified successfully");
        assert_eq!(verify_message("rejected"), "User rejected successfully");
    }

    #[test]
    fn stats_envelope_uses_express_key_names() {
        let envelope = stats_envelope(
            7,
            vec![json!({"role": "user", "count": 5})],
            vec![json!({"membership_tier": "Platinum", "count": 4})],
            vec![json!({"verification_status": "approved", "count": 6})],
            2,
        );
        assert_eq!(
            envelope,
            json!({
                "success": true,
                "data": {
                    "totalUsers": 7,
                    "usersByRole": [{"role": "user", "count": 5}],
                    "usersByMembershipTier": [{"membership_tier": "Platinum", "count": 4}],
                    "usersByVerificationStatus": [{"verification_status": "approved", "count": 6}],
                    "recentRegistrations": 2,
                }
            })
        );
    }

    #[test]
    fn database_stats_envelope_counts_tables_in_meta() {
        let envelope = database_stats_envelope(
            "unknown",
            server_stats_json(),
            vec![table_json("users", 30)],
            "2026-09-01T00:00:00.000Z",
        );
        assert_eq!(
            envelope,
            json!({
                "success": true,
                "data": {
                    "schemaVersion": "unknown",
                    "serverStats": {},
                    "tables": [{"name": "users", "columnCount": 30}],
                    "meta": {
                        "totalTables": 1,
                        "timestamp": "2026-09-01T00:00:00.000Z",
                    }
                }
            })
        );
    }
}
