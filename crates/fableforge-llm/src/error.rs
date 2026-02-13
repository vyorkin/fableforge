//! Error types for LLM operations.

use thiserror::Error;

/// Errors that can occur during LLM operations.
#[derive(Debug, Error)]
pub enum LlmError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing failed.
    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response.
    #[error("API error: {message}")]
    Api {
        /// Error message from API.
        message: String,
        /// HTTP status code.
        status: Option<u16>,
    },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded, retry after {retry_after:?} seconds")]
    RateLimit {
        /// Seconds until retry is allowed.
        retry_after: Option<u64>,
    },

    /// Invalid API key.
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Model not found or not available.
    #[error("Model not available: {model}")]
    ModelNotAvailable {
        /// The requested model.
        model: String,
    },

    /// Content was blocked by safety filters.
    #[error("Content blocked by safety filters")]
    ContentBlocked,

    /// Context length exceeded.
    #[error("Context length exceeded: max {max} tokens, got {actual}")]
    ContextLengthExceeded {
        /// Maximum allowed tokens.
        max: usize,
        /// Actual tokens in request.
        actual: usize,
    },

    /// Expected JSON response but got something else.
    #[error("Expected JSON response but got: {text}")]
    InvalidJsonResponse {
        /// The actual response text.
        text: String,
    },

    /// Tale has no content to generate from.
    #[error("Tale has no content to generate from")]
    EmptyTale,

    /// Missing required context.
    #[error("Missing required context: {0}")]
    MissingContext(String),
}

impl LlmError {
    /// Create an API error from status and message.
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            message: message.into(),
            status: Some(status),
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlmError::Http(_) | LlmError::RateLimit { .. } | LlmError::Api { status: Some(500..=599), .. }
        )
    }
}
