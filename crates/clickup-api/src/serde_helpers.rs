use std::fmt;

use serde::{Deserialize, Deserializer, de};

/// Deserializes a value that may be either a string or a number into a `String`.
///
/// The ClickUp API is inconsistent — some fields arrive as `"6"` in one
/// context and `6` in another.  This helper normalises both to `String`.
pub fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrNumber;

    impl<'de> de::Visitor<'de> for StringOrNumber {
        type Value = String;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or a number")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_owned())
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }
    }

    deserializer.deserialize_any(StringOrNumber)
}

/// Deserializes an optional value that may be a string, a number, or null.
pub fn deserialize_option_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptStringOrNumber;

    impl<'de> de::Visitor<'de> for OptStringOrNumber {
        type Value = Option<String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string, a number, or null")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
        where
            D2: Deserializer<'de>,
        {
            deserialize_string_or_number(deserializer).map(Some)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_owned()))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }
    }

    deserializer.deserialize_option(OptStringOrNumber)
}

/// Deserializes a string that may be absent or `null`, defaulting to `""`.
///
/// Use with `#[serde(default, deserialize_with = "...")]` on `String` fields
/// that the ClickUp API may omit or set to `null` (e.g. deactivated users).
pub fn deserialize_string_or_null<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Deserializes a value that may be a string or a number into an `i32`.
///
/// Handles cases where the API returns `"0"` (string) or `0` (integer).
pub fn deserialize_i32_or_string<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    struct I32OrString;

    impl<'de> de::Visitor<'de> for I32OrString {
        type Value = i32;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an integer or a string containing an integer")
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            i32::try_from(v).map_err(|_| E::custom(format!("integer {v} out of i32 range")))
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            i32::try_from(v).map_err(|_| E::custom(format!("integer {v} out of i32 range")))
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(v as i32)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<i32>()
                .map_err(|_| E::custom(format!("cannot parse \"{v}\" as i32")))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            self.visit_str(&v)
        }
    }

    deserializer.deserialize_any(I32OrString)
}

/// Deserializes a time value that may be a number (ms) or `{"time": ms}` object, or null.
///
/// ClickUp returns time_spent as either a raw number or a wrapped object.
pub fn deserialize_time_value<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match &value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => Ok(n.as_u64()),
        serde_json::Value::Object(obj) => Ok(obj.get("time").and_then(|v| v.as_u64())),
        serde_json::Value::String(s) => Ok(s.parse::<u64>().ok()),
        _ => Ok(None),
    }
}

/// Deserializes a value that may be an object, `false`, or `null` into `Option<T>`.
///
/// ClickUp returns `"priority": false` when no priority is set instead of `null`.
pub fn deserialize_maybe_false<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: serde::de::DeserializeOwned,
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match &value {
        serde_json::Value::Bool(false) | serde_json::Value::Null => Ok(None),
        _ => serde_json::from_value(value)
            .map(Some)
            .map_err(de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    struct TestString {
        #[serde(deserialize_with = "deserialize_string_or_number")]
        value: String,
    }

    #[derive(Deserialize)]
    struct TestOptionString {
        #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
        value: Option<String>,
    }

    #[derive(Deserialize)]
    struct TestI32 {
        #[serde(deserialize_with = "deserialize_i32_or_string")]
        value: i32,
    }

    #[test]
    fn test_string_from_string() {
        let v: TestString = serde_json::from_str(r#"{"value":"hello"}"#).unwrap();
        assert_eq!(v.value, "hello");
    }

    #[test]
    fn test_string_from_integer() {
        let v: TestString = serde_json::from_str(r#"{"value":42}"#).unwrap();
        assert_eq!(v.value, "42");
    }

    #[test]
    fn test_string_from_float() {
        let v: TestString = serde_json::from_str(r#"{"value":3.14}"#).unwrap();
        assert_eq!(v.value, "3.14");
    }

    #[test]
    fn test_option_string_from_null() {
        let v: TestOptionString = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert!(v.value.is_none());
    }

    #[test]
    fn test_option_string_from_string() {
        let v: TestOptionString = serde_json::from_str(r#"{"value":"6"}"#).unwrap();
        assert_eq!(v.value.as_deref(), Some("6"));
    }

    #[test]
    fn test_option_string_from_integer() {
        let v: TestOptionString = serde_json::from_str(r#"{"value":6}"#).unwrap();
        assert_eq!(v.value.as_deref(), Some("6"));
    }

    #[test]
    fn test_option_string_absent() {
        let v: TestOptionString = serde_json::from_str(r#"{}"#).unwrap();
        assert!(v.value.is_none());
    }

    #[test]
    fn test_i32_from_integer() {
        let v: TestI32 = serde_json::from_str(r#"{"value":6}"#).unwrap();
        assert_eq!(v.value, 6);
    }

    #[test]
    fn test_i32_from_string() {
        let v: TestI32 = serde_json::from_str(r#"{"value":"6"}"#).unwrap();
        assert_eq!(v.value, 6);
    }

    #[derive(Deserialize)]
    struct TestStringOrNull {
        #[serde(default, deserialize_with = "deserialize_string_or_null")]
        value: String,
    }

    #[test]
    fn test_string_or_null_from_string() {
        let v: TestStringOrNull = serde_json::from_str(r#"{"value":"hi"}"#).unwrap();
        assert_eq!(v.value, "hi");
    }

    #[test]
    fn test_string_or_null_from_null() {
        let v: TestStringOrNull = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(v.value, "");
    }

    #[test]
    fn test_string_or_null_absent() {
        let v: TestStringOrNull = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(v.value, "");
    }

    #[derive(Deserialize)]
    struct TestTimeValue {
        #[serde(default, deserialize_with = "deserialize_time_value")]
        value: Option<u64>,
    }

    #[test]
    fn test_time_value_from_number() {
        let v: TestTimeValue = serde_json::from_str(r#"{"value":123}"#).unwrap();
        assert_eq!(v.value, Some(123));
    }

    #[test]
    fn test_time_value_from_object() {
        let v: TestTimeValue = serde_json::from_str(r#"{"value":{"time":456}}"#).unwrap();
        assert_eq!(v.value, Some(456));
    }

    #[test]
    fn test_time_value_from_null() {
        let v: TestTimeValue = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(v.value, None);
    }

    #[test]
    fn test_time_value_from_string() {
        let v: TestTimeValue = serde_json::from_str(r#"{"value":"789"}"#).unwrap();
        assert_eq!(v.value, Some(789));
    }

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct Inner {
        name: String,
    }

    #[derive(Deserialize)]
    struct TestMaybeFalse {
        #[serde(default, deserialize_with = "deserialize_maybe_false")]
        value: Option<Inner>,
    }

    #[test]
    fn test_maybe_false_with_false() {
        let v: TestMaybeFalse = serde_json::from_str(r#"{"value":false}"#).unwrap();
        assert!(v.value.is_none(), "false should deserialize to None");
    }

    #[test]
    fn test_maybe_false_with_null() {
        let v: TestMaybeFalse = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert!(v.value.is_none(), "null should deserialize to None");
    }

    #[test]
    fn test_maybe_false_with_object() {
        let v: TestMaybeFalse = serde_json::from_str(r#"{"value":{"name":"high"}}"#).unwrap();
        assert_eq!(
            v.value,
            Some(Inner {
                name: "high".to_owned()
            }),
            "valid object should deserialize to Some"
        );
    }

    #[test]
    fn test_maybe_false_absent() {
        let v: TestMaybeFalse = serde_json::from_str(r#"{}"#).unwrap();
        assert!(v.value.is_none(), "absent field should default to None");
    }
}
