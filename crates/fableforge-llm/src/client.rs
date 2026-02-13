//! Claude AI API client.

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::error::LlmError;

/// Default Claude model.
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// Claude API base URL.
const API_BASE_URL: &str = "https://api.anthropic.com/v1";

/// Claude API version header.
const API_VERSION: &str = "2023-06-01";

/// Client for Claude AI API.
pub struct ClaudeClient {
    api_key: String,
    model: String,
    http: reqwest::Client,
    max_tokens: u32,
}

impl ClaudeClient {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
            http: reqwest::Client::new(),
            max_tokens: 4096,
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
    pub async fn complete_json<T: DeserializeOwned>(&self, prompt: &str) -> Result<T, LlmError> {
        let text = self.complete(prompt).await?;

        // Try to extract JSON from the response
        let json_text = extract_json(&text).unwrap_or(&text);

        serde_json::from_str(json_text).map_err(|e| {
            debug!("Failed to parse JSON: {}, text: {}", e, json_text);
            LlmError::InvalidJsonResponse {
                text: text.clone(),
            }
        })
    }

    /// Send request to Claude API.
    async fn send_request(&self, request: &MessagesRequest<'_>) -> Result<MessagesResponse, LlmError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).map_err(|_| LlmError::InvalidApiKey)?,
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
            let error_body = response.text().await.unwrap_or_default();
            Err(self.parse_error(status.as_u16(), &error_body))
        }
    }

    /// Extract text content from response.
    fn extract_text(&self, response: &MessagesResponse) -> Result<String, LlmError> {
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
                "invalid_request_error" if error.error.message.contains("model") => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_code_block() {
        let text = r#"Here is the JSON:
```json
{"name": "test"}
```"#;
        assert_eq!(extract_json(text), Some(r#"{"name": "test"}"#));
    }

    #[test]
    fn test_extract_json_plain() {
        let text = r#"{"name": "test"}"#;
        assert_eq!(extract_json(text), Some(r#"{"name": "test"}"#));
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
        assert_eq!(extract_json(text), Some(r#"{"name": "test"}"#));
    }
}
