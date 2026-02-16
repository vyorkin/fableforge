mod format;
mod handlers;
mod state;

use std::sync::Arc;

use handlers::{Deps, LlmProvider};
use teloxide::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .init();

    info!("Starting FableForge Telegram bot...");

    let bot = Bot::from_env();

    // Resolve LLM provider from environment
    let llm_provider = resolve_llm_provider();
    if llm_provider.is_none() {
        info!("No API key found. Bot will only support /structure command.");
    }

    let deps = Arc::new(Deps { llm_provider });

    handlers::run(bot, deps).await;
}

fn resolve_llm_provider() -> Option<LlmProvider> {
    // Try OpenRouter first
    if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY")
        && !api_key.is_empty()
    {
        let model = std::env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "deepseek/deepseek-v3.2".to_string());
        return Some(LlmProvider::OpenRouter { api_key, model });
    }

    // Try Claude
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY")
        && !api_key.is_empty()
    {
        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
        return Some(LlmProvider::Claude { api_key, model });
    }

    None
}
