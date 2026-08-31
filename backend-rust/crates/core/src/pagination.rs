use serde_json::{Map, Value};

const MAX_SAFE_INTEGER_F64: f64 = 9_007_199_254_740_992.0;

pub fn number_json(value: f64) -> Value {
    if value.is_finite() && value.fract() == 0.0 && value.abs() < MAX_SAFE_INTEGER_F64 {
        Value::from(value as i64)
    } else if value.is_finite() {
        serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
    } else {
        Value::Null
    }
}

pub fn total_pages(total: i64, limit: f64) -> f64 {
    (total as f64 / limit).ceil()
}

pub fn js_parse_f64(raw: &str) -> Option<f64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(0.0);
    }
    trimmed
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

pub fn pagination_json(page: f64, limit: f64, total: i64) -> Value {
    let mut map = Map::new();
    map.insert("page".to_owned(), number_json(page));
    map.insert("limit".to_owned(), number_json(limit));
    map.insert("total".to_owned(), Value::from(total));
    map.insert(
        "totalPages".to_owned(),
        number_json(total_pages(total, limit)),
    );
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn total_pages_rounds_up_like_math_ceil() {
        assert_eq!(total_pages(0, 20.0), 0.0);
        assert_eq!(total_pages(1, 20.0), 1.0);
        assert_eq!(total_pages(20, 20.0), 1.0);
        assert_eq!(total_pages(21, 20.0), 2.0);
        assert_eq!(total_pages(41, 20.0), 3.0);
        assert_eq!(total_pages(200, 20.0), 10.0);
        assert_eq!(total_pages(7, 3.0), 3.0);
    }

    #[test]
    fn pagination_serializes_integral_numbers_without_decimals() {
        let pagination = pagination_json(2.0, 20.0, 41);
        assert_eq!(
            pagination,
            json!({"page": 2, "limit": 20, "total": 41, "totalPages": 3})
        );
        assert_eq!(
            serde_json::to_string(&pagination).expect("pagination JSON"),
            r#"{"page":2,"limit":20,"total":41,"totalPages":3}"#
        );
    }

    #[test]
    fn pagination_handles_empty_result_set() {
        let pagination = pagination_json(1.0, 20.0, 0);
        assert_eq!(
            serde_json::to_string(&pagination).expect("pagination JSON"),
            r#"{"page":1,"limit":20,"total":0,"totalPages":0}"#
        );
    }

    #[test]
    fn zero_limit_turns_total_pages_into_null_like_json_stringify_infinity() {
        let pagination = pagination_json(1.0, 0.0, 5);
        assert_eq!(
            serde_json::to_string(&pagination).expect("pagination JSON"),
            r#"{"page":1,"limit":0,"total":5,"totalPages":null}"#
        );
    }

    #[test]
    fn number_json_keeps_fractional_values_as_floats() {
        assert_eq!(
            serde_json::to_string(&number_json(2.5)).expect("fractional number"),
            "2.5"
        );
        assert_eq!(
            serde_json::to_string(&number_json(f64::NAN)).expect("NaN number"),
            "null"
        );
    }

    #[test]
    fn js_parse_f64_mirrors_number_semantics_for_realistic_inputs() {
        assert_eq!(js_parse_f64(" 2 "), Some(2.0));
        assert_eq!(js_parse_f64(""), Some(0.0));
        assert_eq!(js_parse_f64("   "), Some(0.0));
        assert_eq!(js_parse_f64("2.5"), Some(2.5));
        assert_eq!(js_parse_f64("0"), Some(0.0));
        assert_eq!(js_parse_f64("abc"), None);
        assert_eq!(js_parse_f64("NaN"), None);
        assert_eq!(js_parse_f64("Infinity"), None);
    }
}
