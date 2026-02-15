//! LLM client trait and Claude AI implementation.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{debug, instrument, warn};

use crate::error::LlmError;

/// Trait for LLM clients.
///
/// This trait abstracts over different LLM providers (Claude, OpenAI, Ollama,
/// etc.) and allows for easy mocking in tests.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a prompt and get a text response.
    async fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// Send a prompt and parse response as JSON.
    async fn complete_json<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: DeserializeOwned + Send;
}

/// Default Claude model.
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Claude API base URL.
const API_BASE_URL: &str = "https://api.anthropic.com/v1";

/// Claude API version header.
const API_VERSION: &str = "2023-06-01";

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_factor: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retries.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    /// Set initial delay.
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Disable retries.
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            ..Self::default()
        }
    }

    /// Calculate delay for a given attempt (0-indexed).
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_secs = self.initial_delay.as_secs_f64()
            * self.backoff_factor.powi(attempt as i32);
        let delay = Duration::from_secs_f64(delay_secs);
        delay.min(self.max_delay)
    }
}

/// Client for Claude AI API.
pub struct ClaudeClient {
    api_key: String,
    model: String,
    http: reqwest::Client,
    max_tokens: u32,
    retry: RetryConfig,
}

impl ClaudeClient {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
            http: reqwest::Client::new(),
            max_tokens: 4096,
            retry: RetryConfig::default(),
        }
    }

    /// Set the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set max tokens for responses.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set retry configuration.
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry = config;
        self
    }

    /// Send a prompt and get a text response.
    #[instrument(skip(self, prompt), fields(model = %self.model))]
    pub async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        let request = MessagesRequest {
            model: &self.model,
            max_tokens: self.max_tokens,
            messages: vec![Message {
                role: "user",
                content: prompt,
            }],
        };

        let response = self.send_request(&request).await?;
        self.extract_text(&response)
    }

    /// Send a prompt and parse response as JSON.
    #[instrument(skip(self, prompt), fields(model = %self.model))]
    pub async fn complete_json<T: DeserializeOwned>(
        &self,
        prompt: &str,
    ) -> Result<T, LlmError> {
        let text = self.complete(prompt).await?;

        // Try to extract JSON from the response
        let json_text = extract_json(&text).unwrap_or(&text);

        serde_json::from_str(json_text).map_err(|e| {
            debug!(
                "Failed to parse JSON: {}, text: {}",
                e, json_text
            );
            LlmError::InvalidJsonResponse { text: text.clone() }
        })
    }

    /// Send request to Claude API with retry logic.
    async fn send_request(
        &self,
        request: &MessagesRequest<'_>,
    ) -> Result<MessagesResponse, LlmError> {
        let mut last_error: Option<LlmError> = None;

        for attempt in 0..=self.retry.max_retries {
            if attempt > 0 {
                let delay = last_error
                    .as_ref()
                    .and_then(|e| {
                        if let LlmError::RateLimit {
                            retry_after: Some(secs),
                        } = e
                        {
                            Some(Duration::from_secs(*secs))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        self.retry.delay_for_attempt(attempt - 1)
                    });

                warn!(
                    attempt = attempt,
                    delay_secs = delay.as_secs_f64(),
                    "Retrying request after error"
                );
                tokio::time::sleep(delay).await;
            }

            match self.send_request_once(request).await {
                Ok(response) => return Ok(response),
                Err(e)
                    if e.is_retryable() && attempt < self.retry.max_retries =>
                {
                    debug!(attempt = attempt, error = %e, "Request failed, will retry");
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(
            last_error.unwrap_or_else(|| LlmError::Api {
                message: "Unknown error after retries".to_string(),
                status: None,
            }),
        )
    }

    /// Send a single request without retry.
    async fn send_request_once(
        &self,
        request: &MessagesRequest<'_>,
    ) -> Result<MessagesResponse, LlmError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|_| LlmError::InvalidApiKey)?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(API_VERSION),
        );

        let url = format!("{}/messages", API_BASE_URL);

        debug!("Sending request to Claude API");

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(request)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let body = response.json::<MessagesResponse>().await?;
            Ok(body)
        } else {
            // Try to extract retry-after header
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            let error_body = response.text().await.unwrap_or_default();
            let mut error = self.parse_error(status.as_u16(), &error_body);

            // Inject retry-after if we got it from header
            if let LlmError::RateLimit {
                retry_after: ref mut ra,
            } = error
                && ra.is_none()
            {
                *ra = retry_after;
            }

            Err(error)
        }
    }

    /// Extract text content from response.
    fn extract_text(
        &self,
        response: &MessagesResponse,
    ) -> Result<String, LlmError> {
        let text = response
            .content
            .iter()
            .filter_map(|block| {
                if block.content_type == "text" {
                    Some(block.text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            Err(LlmError::Api {
                message: "Empty response from API".to_string(),
                status: None,
            })
        } else {
            Ok(text)
        }
    }

    /// Parse error response from API.
    fn parse_error(&self, status: u16, body: &str) -> LlmError {
        // Try to parse as API error
        if let Ok(error) = serde_json::from_str::<ApiError>(body) {
            match error.error.error_type.as_str() {
                "authentication_error" => return LlmError::InvalidApiKey,
                "rate_limit_error" => {
                    return LlmError::RateLimit { retry_after: None };
                }
                "invalid_request_error"
                    if error.error.message.contains("model") =>
                {
                    return LlmError::ModelNotAvailable {
                        model: self.model.clone(),
                    };
                }
                _ => {}
            }
            return LlmError::api(status, error.error.message);
        }

        // Fallback to generic error
        LlmError::api(status, body.to_string())
    }
}

#[async_trait]
impl LlmClient for ClaudeClient {
    async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        ClaudeClient::complete(self, prompt).await
    }

    async fn complete_json<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: DeserializeOwned + Send,
    {
        ClaudeClient::complete_json(self, prompt).await
    }
}

/// Extract JSON from text that may contain markdown code blocks.
fn extract_json(text: &str) -> Option<&str> {
    // Look for ```json ... ``` blocks
    if let Some(start) = text.find("```json") {
        let json_start = start + 7;
        if let Some(end) = text[json_start..].find("```") {
            return Some(text[json_start..json_start + end].trim());
        }
    }

    // Look for ``` ... ``` blocks
    if let Some(start) = text.find("```") {
        let json_start = start + 3;
        // Skip language identifier if present
        let json_start = text[json_start..]
            .find('\n')
            .map(|i| json_start + i + 1)
            .unwrap_or(json_start);
        if let Some(end) = text[json_start..].find("```") {
            return Some(text[json_start..json_start + end].trim());
        }
    }

    // Look for { ... } or [ ... ]
    let text = text.trim();
    if (text.starts_with('{') && text.ends_with('}'))
        || (text.starts_with('[') && text.ends_with(']'))
    {
        return Some(text);
    }

    None
}

/// Request to the messages API.
#[derive(Debug, Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

/// A message in the conversation.
#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// Response from the messages API.
#[derive(Debug, Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

/// Content block in response.
#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

/// API error response.
#[derive(Debug, Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

/// Error detail.
#[derive(Debug, Deserialize)]
struct ApiErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

// ── OpenRouter ──────────────────────────────────────────────

/// Default OpenRouter model.
const OPENROUTER_DEFAULT_MODEL: &str = "anthropic/claude-sonnet-4";

/// OpenRouter API base URL.
const OPENROUTER_API_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Client for OpenRouter API (OpenAI-compatible).
pub struct OpenRouterClient {
    api_key: String,
    model: String,
    http: reqwest::Client,
    max_tokens: u32,
    retry: RetryConfig,
    app_name: Option<String>,
}

impl OpenRouterClient {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: OPENROUTER_DEFAULT_MODEL.to_string(),
            http: reqwest::Client::new(),
            max_tokens: 4096,
            retry: RetryConfig::default(),
            app_name: None,
        }
    }

    /// Set the model to use (OpenRouter format: `"provider/model"`).
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set max tokens for responses.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set retry configuration.
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry = config;
        self
    }

    /// Set application name (sent as `X-Title` header for OpenRouter ranking).
    pub fn with_app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Send a prompt and get a text response.
    #[instrument(skip(self, prompt), fields(model = %self.model))]
    async fn do_complete(&self, prompt: &str) -> Result<String, LlmError> {
        let request = ChatCompletionRequest {
            model: &self.model,
            max_tokens: self.max_tokens,
            messages: vec![ChatMessage {
                role: "user",
                content: prompt,
            }],
        };

        let response = self.send_request(&request).await?;

        let text = response
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        if text.is_empty() {
            Err(LlmError::Api {
                message: "Empty response from API".to_string(),
                status: None,
            })
        } else {
            Ok(text)
        }
    }

    /// Send request with retry logic.
    async fn send_request(
        &self,
        request: &ChatCompletionRequest<'_>,
    ) -> Result<ChatCompletionResponse, LlmError> {
        let mut last_error: Option<LlmError> = None;

        for attempt in 0..=self.retry.max_retries {
            if attempt > 0 {
                let delay = last_error
                    .as_ref()
                    .and_then(|e| {
                        if let LlmError::RateLimit {
                            retry_after: Some(secs),
                        } = e
                        {
                            Some(Duration::from_secs(*secs))
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| {
                        self.retry.delay_for_attempt(attempt - 1)
                    });

                warn!(
                    attempt = attempt,
                    delay_secs = delay.as_secs_f64(),
                    "Retrying request after error"
                );
                tokio::time::sleep(delay).await;
            }

            match self.send_request_once(request).await {
                Ok(response) => return Ok(response),
                Err(e)
                    if e.is_retryable() && attempt < self.retry.max_retries =>
                {
                    debug!(attempt = attempt, error = %e, "Request failed, will retry");
                    last_error = Some(e);
                }
                Err(e) => return Err(e),
            }
        }

        Err(
            last_error.unwrap_or_else(|| LlmError::Api {
                message: "Unknown error after retries".to_string(),
                status: None,
            }),
        )
    }

    /// Send a single request without retry.
    async fn send_request_once(
        &self,
        request: &ChatCompletionRequest<'_>,
    ) -> Result<ChatCompletionResponse, LlmError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|_| LlmError::InvalidApiKey)?,
        );
        if let Some(ref name) = self.app_name
            && let Ok(val) = HeaderValue::from_str(name)
        {
            headers.insert("X-Title", val);
        }

        let url = format!(
            "{}/chat/completions",
            OPENROUTER_API_BASE_URL
        );

        debug!("Sending request to OpenRouter API");

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(request)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let body = response.json::<ChatCompletionResponse>().await?;
            Ok(body)
        } else {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            let error_body = response.text().await.unwrap_or_default();
            let mut error = self.parse_error(status.as_u16(), &error_body);

            if let LlmError::RateLimit {
                retry_after: ref mut ra,
            } = error
                && ra.is_none()
            {
                *ra = retry_after;
            }

            Err(error)
        }
    }

    /// Parse error response from API (OpenAI-compatible format).
    fn parse_error(&self, status: u16, body: &str) -> LlmError {
        // OpenRouter/OpenAI error format: {"error": {"message": "...", "type":
        // "..."}}
        if let Ok(error) = serde_json::from_str::<ApiError>(body) {
            match error.error.error_type.as_str() {
                "authentication_error" => return LlmError::InvalidApiKey,
                "rate_limit_error" => {
                    return LlmError::RateLimit { retry_after: None };
                }
                "invalid_request_error"
                    if error.error.message.contains("model") =>
                {
                    return LlmError::ModelNotAvailable {
                        model: self.model.clone(),
                    };
                }
                _ => {}
            }
            return LlmError::api(status, error.error.message);
        }

        // Also try OpenAI-style: {"error": {"message": "...", "type": "..."}}
        // where "type" may differ — fall through to generic
        if status == 401 {
            return LlmError::InvalidApiKey;
        }
        if status == 429 {
            return LlmError::RateLimit { retry_after: None };
        }

        LlmError::api(status, body.to_string())
    }
}

#[async_trait]
impl LlmClient for OpenRouterClient {
    async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        self.do_complete(prompt).await
    }

    async fn complete_json<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: DeserializeOwned + Send,
    {
        let text = self.do_complete(prompt).await?;
        let json_text = extract_json(&text).unwrap_or(&text);

        serde_json::from_str(json_text).map_err(|e| {
            debug!(
                "Failed to parse JSON: {}, text: {}",
                e, json_text
            );
            LlmError::InvalidJsonResponse { text: text.clone() }
        })
    }
}

// ── AnyClient enum dispatch ─────────────────────────────────

/// A provider-agnostic LLM client that dispatches to the concrete
/// implementation.
pub enum AnyClient {
    /// Claude API client.
    Claude(ClaudeClient),
    /// OpenRouter API client.
    OpenRouter(OpenRouterClient),
}

#[async_trait]
impl LlmClient for AnyClient {
    async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        match self {
            AnyClient::Claude(c) => c.complete(prompt).await,
            AnyClient::OpenRouter(c) => c.complete(prompt).await,
        }
    }

    async fn complete_json<T>(&self, prompt: &str) -> Result<T, LlmError>
    where
        T: DeserializeOwned + Send,
    {
        match self {
            AnyClient::Claude(c) => c.complete_json(prompt).await,
            AnyClient::OpenRouter(c) => c.complete_json(prompt).await,
        }
    }
}

// ── Chat completion types (OpenAI-compatible) ───────────────

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

/// Mock LLM client for testing.
///
/// Returns predefined responses for prompts.
#[derive(Debug, Clone, Default)]
pub struct MockClient {
    /// Response to return for text completions.
    pub text_response: Option<String>,
    /// Response to return for JSON completions.
    pub json_response: Option<String>,
    /// Error message to return instead of response.
    pub error_message: Option<String>,
}

impl MockClient {
    /// Create a new mock client.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the text response.
    pub fn with_text(mut self, response: impl Into<String>) -> Self {
        self.text_response = Some(response.into());
        self
    }

    /// Set the JSON response.
    pub fn with_json(mut self, response: impl Into<String>) -> Self {
        self.json_response = Some(response.into());
        self
    }

    /// Set an error message to return.
    pub fn with_error(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }
}

#[async_trait]
impl LlmClient for MockClient {
    async fn complete(&self, _prompt: &str) -> Result<String, LlmError> {
        if let Some(ref msg) = self.error_message {
            return Err(LlmError::Api {
                message: msg.clone(),
                status: None,
            });
        }
        self.text_response.clone().ok_or_else(|| LlmError::Api {
            message: "MockClient: no text response configured".to_string(),
            status: None,
        })
    }

    async fn complete_json<T>(&self, _prompt: &str) -> Result<T, LlmError>
    where
        T: DeserializeOwned + Send,
    {
        if let Some(ref msg) = self.error_message {
            return Err(LlmError::Api {
                message: msg.clone(),
                status: None,
            });
        }
        let json =
            self.json_response.as_ref().ok_or_else(|| LlmError::Api {
                message: "MockClient: no JSON response configured".to_string(),
                status: None,
            })?;
        serde_json::from_str(json)
            .map_err(|_| LlmError::InvalidJsonResponse { text: json.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_code_block() {
        let text = r#"Here is the JSON:
```json
{"name": "test"}
```"#;
        assert_eq!(
            extract_json(text),
            Some(r#"{"name": "test"}"#)
        );
    }

    #[test]
    fn test_extract_json_plain() {
        let text = r#"{"name": "test"}"#;
        assert_eq!(
            extract_json(text),
            Some(r#"{"name": "test"}"#)
        );
    }

    #[test]
    fn test_extract_json_array() {
        let text = r#"[1, 2, 3]"#;
        assert_eq!(extract_json(text), Some(r#"[1, 2, 3]"#));
    }

    #[test]
    fn test_extract_json_with_whitespace() {
        let text = r#"
        {"name": "test"}
        "#;
        assert_eq!(
            extract_json(text),
            Some(r#"{"name": "test"}"#)
        );
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(
            config.initial_delay,
            Duration::from_secs(1)
        );
        assert_eq!(config.backoff_factor, 2.0);
    }

    #[test]
    fn test_retry_config_no_retry() {
        let config = RetryConfig::no_retry();
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_retry_delay_exponential_backoff() {
        let config = RetryConfig::new()
            .initial_delay(Duration::from_secs(1))
            .max_delay(Duration::from_secs(60));

        // attempt 0: 1s * 2^0 = 1s
        assert_eq!(
            config.delay_for_attempt(0),
            Duration::from_secs(1)
        );
        // attempt 1: 1s * 2^1 = 2s
        assert_eq!(
            config.delay_for_attempt(1),
            Duration::from_secs(2)
        );
        // attempt 2: 1s * 2^2 = 4s
        assert_eq!(
            config.delay_for_attempt(2),
            Duration::from_secs(4)
        );
        // attempt 3: 1s * 2^3 = 8s
        assert_eq!(
            config.delay_for_attempt(3),
            Duration::from_secs(8)
        );
    }

    #[test]
    fn test_retry_delay_respects_max() {
        let config = RetryConfig::new()
            .initial_delay(Duration::from_secs(10))
            .max_delay(Duration::from_secs(30));

        // attempt 2: 10s * 2^2 = 40s, but capped at 30s
        assert_eq!(
            config.delay_for_attempt(2),
            Duration::from_secs(30)
        );
    }

    #[tokio::test]
    async fn test_mock_client_text_response() {
        let client = MockClient::new().with_text("Hello, world!");
        let response = client.complete("test prompt").await.unwrap();
        assert_eq!(response, "Hello, world!");
    }

    #[tokio::test]
    async fn test_mock_client_json_response() {
        let client = MockClient::new().with_json(r#"{"name": "test"}"#);

        #[derive(Debug, Deserialize, PartialEq)]
        struct Response {
            name: String,
        }

        let response: Response =
            client.complete_json("test prompt").await.unwrap();
        assert_eq!(response.name, "test");
    }

    #[tokio::test]
    async fn test_mock_client_error() {
        let client = MockClient::new().with_error("simulated error");
        let result = client.complete("test").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_openrouter_request_format() {
        let request = ChatCompletionRequest {
            model: "anthropic/claude-sonnet-4",
            max_tokens: 4096,
            messages: vec![ChatMessage {
                role: "user",
                content: "Hello",
            }],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json["model"],
            "anthropic/claude-sonnet-4"
        );
        assert_eq!(json["max_tokens"], 4096);
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "Hello");
    }

    #[test]
    fn test_openrouter_response_parse() {
        let json = r#"{
            "choices": [{
                "message": {
                    "content": "Hello, world!"
                }
            }]
        }"#;

        let response: ChatCompletionResponse =
            serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(
            response.choices[0].message.content,
            "Hello, world!"
        );
    }

    #[test]
    fn test_openrouter_extract_json() {
        let text = r#"Here is the JSON:
```json
{"characters": [{"name": "Ivan"}]}
```"#;
        let extracted = extract_json(text).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(extracted).unwrap();
        assert_eq!(parsed["characters"][0]["name"], "Ivan");
    }
}
