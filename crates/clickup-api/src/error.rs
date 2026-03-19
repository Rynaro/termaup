use thiserror::Error;

/// Errors that can occur when interacting with the ClickUp API.
#[derive(Debug, Error)]
pub enum ClickUpError {
    /// Authentication failed (invalid or expired token).
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// The API returned an error response.
    #[error("API error (status {status}): {message}")]
    ApiError {
        /// HTTP status code.
        status: u16,
        /// Error message from the API.
        message: String,
    },

    /// Rate limit exceeded.
    #[error("Rate limited. Retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds until the rate limit resets.
        retry_after_secs: u64,
    },

    /// A network-level error occurred.
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// Failed to deserialize an API response.
    #[error("Deserialization error at {endpoint}: {message}")]
    DeserializationError {
        /// The serde error message.
        message: String,
        /// The endpoint path that produced the error.
        endpoint: String,
        /// A preview of the response body (first 200 chars).
        body_preview: String,
    },

    /// A configuration or credential error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// The requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),
}

impl ClickUpError {
    /// Creates a `DeserializationError` from a `serde_json::Error` without endpoint context.
    ///
    /// Use this when deserializing values that have already been fetched
    /// (e.g., extracting nested fields from a raw `serde_json::Value`).
    pub fn deserialization(err: serde_json::Error) -> Self {
        Self::DeserializationError {
            message: err.to_string(),
            endpoint: String::new(),
            body_preview: String::new(),
        }
    }
}

/// A convenience `Result` type that uses [`ClickUpError`].
pub type Result<T> = std::result::Result<T, ClickUpError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = ClickUpError::AuthError("invalid token".into());
        assert_eq!(err.to_string(), "Authentication failed: invalid token");
    }

    #[test]
    fn test_api_error_display() {
        let err = ClickUpError::ApiError {
            status: 400,
            message: "bad request".into(),
        };
        assert_eq!(err.to_string(), "API error (status 400): bad request");
    }

    #[test]
    fn test_rate_limited_display() {
        let err = ClickUpError::RateLimited {
            retry_after_secs: 42,
        };
        assert_eq!(err.to_string(), "Rate limited. Retry after 42 seconds");
    }

    #[test]
    fn test_config_error_display() {
        let err = ClickUpError::ConfigError("missing file".into());
        assert_eq!(err.to_string(), "Configuration error: missing file");
    }

    #[test]
    fn test_not_found_display() {
        let err = ClickUpError::NotFound("task abc123".into());
        assert_eq!(err.to_string(), "Not found: task abc123");
    }

    #[test]
    fn test_deserialization_error_display() {
        let err = ClickUpError::DeserializationError {
            message: "missing field `id`".into(),
            endpoint: "/api/v2/task/abc".into(),
            body_preview: r#"{"name":"test"}"#.into(),
        };
        assert_eq!(
            err.to_string(),
            "Deserialization error at /api/v2/task/abc: missing field `id`"
        );
    }

    #[test]
    fn test_deserialization_helper_creates_error() {
        let serde_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let err = ClickUpError::deserialization(serde_err);
        match err {
            ClickUpError::DeserializationError {
                message,
                endpoint,
                body_preview,
            } => {
                assert!(!message.is_empty(), "message should contain serde error");
                assert!(endpoint.is_empty(), "endpoint should be empty");
                assert!(body_preview.is_empty(), "body_preview should be empty");
            }
            other => panic!("expected DeserializationError, got: {other:?}"),
        }
    }
}
