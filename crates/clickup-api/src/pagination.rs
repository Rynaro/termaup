use serde::de::DeserializeOwned;

use crate::error::Result;

/// A single page of results from a paginated ClickUp API endpoint.
#[derive(Debug, Clone)]
pub struct PaginatedResponse<T> {
    /// The items returned on this page.
    pub data: Vec<T>,
    /// Whether this is the last page of results.
    pub last_page: bool,
}

impl<T: DeserializeOwned> PaginatedResponse<T> {
    /// Creates a new `PaginatedResponse`.
    pub fn new(data: Vec<T>, last_page: bool) -> Self {
        Self { data, last_page }
    }
}

/// Helper to build an extractor for the common ClickUp pagination pattern.
///
/// Many ClickUp endpoints return `{ "<key>": [...], "last_page": bool }`.
/// This function creates a closure that extracts that shape into a
/// [`PaginatedResponse<T>`].
pub fn standard_extractor<T: DeserializeOwned>(
    key: &str,
) -> impl Fn(serde_json::Value) -> Result<PaginatedResponse<T>> + '_ {
    move |value: serde_json::Value| {
        let last_page = value
            .get("last_page")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let items_value = value
            .get(key)
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));

        let data: Vec<T> = serde_json::from_value(items_value)
            .map_err(crate::error::ClickUpError::deserialization)?;
        Ok(PaginatedResponse { data, last_page })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_extractor_parses_items() {
        let json = serde_json::json!({
            "tasks": [{"id": "a"}, {"id": "b"}],
            "last_page": false
        });

        let extract = standard_extractor::<serde_json::Value>("tasks");
        let page = extract(json).unwrap();

        assert_eq!(page.data.len(), 2);
        assert!(!page.last_page);
    }

    #[test]
    fn test_standard_extractor_defaults_to_last_page() {
        let json = serde_json::json!({"tasks": []});
        let extract = standard_extractor::<serde_json::Value>("tasks");
        let page = extract(json).unwrap();

        assert!(page.data.is_empty());
        assert!(page.last_page);
    }

    #[test]
    fn test_standard_extractor_missing_key() {
        let json = serde_json::json!({"last_page": true});
        let extract = standard_extractor::<serde_json::Value>("tasks");
        let page = extract(json).unwrap();

        assert!(page.data.is_empty());
        assert!(page.last_page);
    }
}
