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
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),

    /// A configuration or credential error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// The requested resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),
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
}
