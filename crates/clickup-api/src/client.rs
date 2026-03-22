use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::error::{ClickUpError, Result};
use crate::pagination::PaginatedResponse;
use crate::rate_limiter::RateLimiter;

const DEFAULT_BASE_URL: &str = "https://api.clickup.com/api/v2";
const USER_AGENT: &str = "clickup-rs/0.1.0";
const TIMEOUT_SECS: u64 = 30;

/// HTTP client for the ClickUp API v2.
#[derive(Clone)]
pub struct ClickUpClient {
    http: reqwest::Client,
    base_url: String,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

/// Shape of ClickUp API error responses.
#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    err: Option<String>,
    #[serde(rename = "ECODE")]
    ecode: Option<String>,
}

impl ClickUpClient {
    /// Creates a new `ClickUpClient` with the default ClickUp API base URL.
    pub fn new(token: String) -> Self {
        Self::with_base_url(token, DEFAULT_BASE_URL.to_string())
    }

    /// Creates a new `ClickUpClient` with a custom base URL (useful for testing).
    pub fn with_base_url(token: String, base_url: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&token).unwrap_or_else(|_| HeaderValue::from_static("")),
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            base_url,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new())),
        }
    }

    /// Performs a GET request and deserializes the response.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.get_with_params(path, &[]).await
    }

    /// Performs a GET request with query parameters and deserializes the response.
    pub async fn get_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        self.rate_limiter.lock().await.check_and_wait().await;

        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(%url, ?params, "GET request");

        let response = self.http.get(&url).query(params).send().await?;

        self.handle_response(response, &url).await
    }

    /// Performs a POST request with a JSON body and deserializes the response.
    pub async fn post<T: DeserializeOwned, B: Serialize + Send>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.rate_limiter.lock().await.check_and_wait().await;

        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(%url, "POST request");

        let response = self.http.post(&url).json(body).send().await?;

        self.handle_response(response, &url).await
    }

    /// Performs a PUT request with a JSON body and deserializes the response.
    pub async fn put<T: DeserializeOwned, B: Serialize + Send>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.rate_limiter.lock().await.check_and_wait().await;

        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(%url, "PUT request");

        let response = self.http.put(&url).json(body).send().await?;

        self.handle_response(response, &url).await
    }

    /// Performs a PUT request with a JSON body that expects no usable
    /// response body.
    pub async fn put_no_body<B: Serialize + Send>(&self, path: &str, body: &B) -> Result<()> {
        self.rate_limiter.lock().await.check_and_wait().await;

        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(%url, "PUT request (no body)");

        let response = self.http.put(&url).json(body).send().await?;

        self.handle_response_no_body(response, &url).await
    }

    /// Performs a DELETE request that expects no response body.
    pub async fn delete(&self, path: &str) -> Result<()> {
        self.rate_limiter.lock().await.check_and_wait().await;

        let url = format!("{}{}", self.base_url, path);
        tracing::debug!(%url, "DELETE request");

        let response = self.http.delete(&url).send().await?;

        self.handle_response_no_body(response, &url).await
    }

    /// Fetches all pages of a paginated endpoint, collecting items into a
    /// single `Vec`.
    ///
    /// `extract` converts the raw JSON [`serde_json::Value`] from each page
    /// into a [`PaginatedResponse<T>`].
    pub async fn get_all_pages<T, F>(
        &self,
        path: &str,
        params: &[(&str, &str)],
        extract: F,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
        F: Fn(serde_json::Value) -> Result<PaginatedResponse<T>>,
    {
        let mut all_items = Vec::new();
        let mut page: u64 = 0;

        loop {
            let page_str = page.to_string();
            let mut full_params: Vec<(&str, &str)> = params.to_vec();
            full_params.push(("page", &page_str));

            let value: serde_json::Value = self.get_with_params(path, &full_params).await?;
            let paginated = extract(value)?;

            let is_last = paginated.last_page;
            all_items.extend(paginated.data);

            if is_last {
                break;
            }
            page += 1;
        }

        Ok(all_items)
    }

    /// Processes an HTTP response, mapping status codes to error variants and
    /// updating the rate limiter from response headers.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
        endpoint: &str,
    ) -> Result<T> {
        // Update rate limiter from headers.
        if let (Some(remaining), Some(reset)) = (
            response
                .headers()
                .get("X-RateLimit-Remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok()),
            response
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok()),
        ) {
            self.rate_limiter
                .lock()
                .await
                .update_from_headers(remaining, reset);
        }

        let status = response.status();
        tracing::debug!(%status, "response received");

        if status.is_success() {
            let body = response.text().await?;
            tracing::trace!(body_len = body.len(), "response body received");
            let parsed: T = serde_json::from_str(&body).map_err(|e| {
                tracing::error!(
                    body_preview = &body[..body.len().min(500)],
                    "deserialization failed"
                );
                ClickUpError::DeserializationError {
                    message: e.to_string(),
                    endpoint: endpoint.to_string(),
                    body_preview: body[..body.len().min(200)].to_string(),
                }
            })?;
            return Ok(parsed);
        }

        // 429 — rate limited
        if status.as_u16() == 429 {
            let retry_after_secs = response
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(|reset_ts| {
                    let now = chrono::Utc::now().timestamp() as u64;
                    reset_ts.saturating_sub(now)
                })
                .unwrap_or(60);

            return Err(ClickUpError::RateLimited { retry_after_secs });
        }

        // 401 — auth error
        if status.as_u16() == 401 {
            let body = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<ApiErrorBody>(&body)
                .ok()
                .and_then(|b| b.err)
                .unwrap_or_else(|| "unauthorized".to_string());
            return Err(ClickUpError::AuthError(message));
        }

        // 404 — not found
        if status.as_u16() == 404 {
            let body = response.text().await.unwrap_or_default();
            let message = serde_json::from_str::<ApiErrorBody>(&body)
                .ok()
                .and_then(|b| b.err)
                .unwrap_or_else(|| "resource not found".to_string());
            return Err(ClickUpError::NotFound(message));
        }

        // Other errors
        let status_code = status.as_u16();
        let body = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<ApiErrorBody>(&body)
            .ok()
            .map(|b| {
                let err = b.err.unwrap_or_default();
                let ecode = b.ecode.unwrap_or_default();
                if ecode.is_empty() {
                    err
                } else {
                    format!("{err} ({ecode})")
                }
            })
            .unwrap_or(body);

        Err(ClickUpError::ApiError {
            status: status_code,
            message,
        })
    }

    /// Processes an HTTP response that returns no body on success, mapping
    /// non-success status codes to error variants.
    async fn handle_response_no_body(
        &self,
        response: reqwest::Response,
        endpoint: &str,
    ) -> Result<()> {
        // Update rate limiter from headers.
        if let (Some(remaining), Some(reset)) = (
            response
                .headers()
                .get("X-RateLimit-Remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok()),
            response
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok()),
        ) {
            self.rate_limiter
                .lock()
                .await
                .update_from_headers(remaining, reset);
        }

        let status = response.status();
        tracing::debug!(%status, "response received");

        if status.is_success() {
            return Ok(());
        }

        // Reuse the same error-handling logic via handle_response with a
        // permissive type — this path only runs for non-success responses
        // where the body is always an error JSON.
        let err: std::result::Result<serde_json::Value, _> =
            self.handle_response(response, endpoint).await;
        match err {
            Err(e) => Err(e),
            // Should not happen since status is not success, but handle
            // gracefully.
            Ok(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn test_client(base_url: &str) -> ClickUpClient {
        ClickUpClient::with_base_url("pk_test_token".to_string(), base_url.to_string())
    }

    #[tokio::test]
    async fn test_get_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": 123, "name": "Test"})),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let result: serde_json::Value = client.get("/user").await.unwrap();
        assert_eq!(result["id"], 123);
        assert_eq!(result["name"], "Test");
    }

    #[tokio::test]
    async fn test_get_with_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/list/1/task"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"tasks": []})),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let result: serde_json::Value = client
            .get_with_params("/list/1/task", &[("page", "0")])
            .await
            .unwrap();
        assert_eq!(result["tasks"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn test_401_returns_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/user"))
            .respond_with(
                ResponseTemplate::new(401).set_body_json(
                    serde_json::json!({"err": "Token invalid", "ECODE": "OAUTH_025"}),
                ),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let err = client.get::<serde_json::Value>("/user").await.unwrap_err();

        match err {
            ClickUpError::AuthError(msg) => {
                assert_eq!(msg, "Token invalid");
            }
            other => panic!("expected AuthError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_404_returns_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/task/nonexistent"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(
                    serde_json::json!({"err": "Task not found", "ECODE": "ITEM_015"}),
                ),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let err = client
            .get::<serde_json::Value>("/task/nonexistent")
            .await
            .unwrap_err();

        match err {
            ClickUpError::NotFound(msg) => {
                assert_eq!(msg, "Task not found");
            }
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_429_returns_rate_limited() {
        let server = MockServer::start().await;
        let reset_ts = chrono::Utc::now().timestamp() as u64 + 30;
        Mock::given(method("GET"))
            .and(path("/api/v2/user"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("X-RateLimit-Reset", reset_ts.to_string().as_str())
                    .insert_header("X-RateLimit-Remaining", "0"),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let err = client.get::<serde_json::Value>("/user").await.unwrap_err();

        match err {
            ClickUpError::RateLimited { retry_after_secs } => {
                assert!(
                    retry_after_secs <= 31,
                    "retry_after_secs should be ~30, got {retry_after_secs}"
                );
            }
            other => panic!("expected RateLimited, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_500_returns_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/team"))
            .respond_with(
                ResponseTemplate::new(500).set_body_json(
                    serde_json::json!({"err": "Internal error", "ECODE": "SVR_001"}),
                ),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let err = client.get::<serde_json::Value>("/team").await.unwrap_err();

        match err {
            ClickUpError::ApiError { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Internal error (SVR_001)");
            }
            other => panic!("expected ApiError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_post_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v2/task/abc/comment"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"id": "c1", "comment_text": "hello", "user": {"id": 1}}),
            ))
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let body = serde_json::json!({"comment_text": "hello"});
        let result: serde_json::Value = client.post("/task/abc/comment", &body).await.unwrap();
        assert_eq!(result["id"], "c1");
        assert_eq!(result["comment_text"], "hello");
    }

    #[tokio::test]
    async fn test_post_401_returns_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v2/task/abc/comment"))
            .respond_with(
                ResponseTemplate::new(401).set_body_json(
                    serde_json::json!({"err": "Token invalid", "ECODE": "OAUTH_025"}),
                ),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let body = serde_json::json!({"comment_text": "hello"});
        let err = client
            .post::<serde_json::Value, _>("/task/abc/comment", &body)
            .await
            .unwrap_err();

        match err {
            ClickUpError::AuthError(msg) => {
                assert_eq!(msg, "Token invalid");
            }
            other => panic!("expected AuthError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_post_429_returns_rate_limited() {
        let server = MockServer::start().await;
        let reset_ts = chrono::Utc::now().timestamp() as u64 + 30;
        Mock::given(method("POST"))
            .and(path("/api/v2/task/abc/comment"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("X-RateLimit-Reset", reset_ts.to_string().as_str())
                    .insert_header("X-RateLimit-Remaining", "0"),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let body = serde_json::json!({"comment_text": "hello"});
        let err = client
            .post::<serde_json::Value, _>("/task/abc/comment", &body)
            .await
            .unwrap_err();

        match err {
            ClickUpError::RateLimited { retry_after_secs } => {
                assert!(
                    retry_after_secs <= 31,
                    "retry_after_secs should be ~30, got {retry_after_secs}"
                );
            }
            other => panic!("expected RateLimited, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_rate_limit_headers_update_limiter() {
        let server = MockServer::start().await;
        let reset_ts = chrono::Utc::now().timestamp() as u64 + 60;
        Mock::given(method("GET"))
            .and(path("/api/v2/user"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"ok": true}))
                    .insert_header("X-RateLimit-Remaining", "42")
                    .insert_header("X-RateLimit-Reset", reset_ts.to_string().as_str()),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let _: serde_json::Value = client.get("/user").await.unwrap();

        let limiter = client.rate_limiter.lock().await;
        assert_eq!(limiter.remaining(), 42);
    }

    #[tokio::test]
    async fn test_put_success() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/v2/comment/c1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"id": "c1", "comment_text": "updated"})),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let body = serde_json::json!({"comment_text": "updated"});
        let result: serde_json::Value = client.put("/comment/c1", &body).await.unwrap();
        assert_eq!(result["id"], "c1");
        assert_eq!(result["comment_text"], "updated");
    }

    #[tokio::test]
    async fn test_put_404_returns_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/v2/comment/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_json(
                serde_json::json!({"err": "Comment not found", "ECODE": "COMMENT_015"}),
            ))
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let body = serde_json::json!({"comment_text": "updated"});
        let err = client
            .put::<serde_json::Value, _>("/comment/nonexistent", &body)
            .await
            .unwrap_err();

        match err {
            ClickUpError::NotFound(msg) => {
                assert_eq!(msg, "Comment not found");
            }
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_delete_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v2/comment/c1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        client.delete("/comment/c1").await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_404_returns_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v2/comment/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_json(
                serde_json::json!({"err": "Comment not found", "ECODE": "COMMENT_015"}),
            ))
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let err = client.delete("/comment/nonexistent").await.unwrap_err();

        match err {
            ClickUpError::NotFound(msg) => {
                assert_eq!(msg, "Comment not found");
            }
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_delete_401_returns_auth_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v2/comment/c1"))
            .respond_with(
                ResponseTemplate::new(401).set_body_json(
                    serde_json::json!({"err": "Token invalid", "ECODE": "OAUTH_025"}),
                ),
            )
            .mount(&server)
            .await;

        let client = test_client(&format!("{}/api/v2", server.uri())).await;
        let err = client.delete("/comment/c1").await.unwrap_err();

        match err {
            ClickUpError::AuthError(msg) => {
                assert_eq!(msg, "Token invalid");
            }
            other => panic!("expected AuthError, got: {other:?}"),
        }
    }
}
