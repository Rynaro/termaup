use std::fmt;

use serde::{Deserializer, de};

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
}
