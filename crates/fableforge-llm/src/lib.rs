//! FableForge LLM — Claude AI integration for story generation.
//!
//! This crate transforms the formal morphological structure of a tale
//! into a sequence of prompts for Claude AI, collects responses,
//! and assembles a coherent narrative.
//!
//! # Two-Phase Generation
//!
//! 1. **Character Generation**: A single prompt generates names, epithets,
//!    appearances, and setting for all characters. This ensures consistency
//!    across the narrative.
//!
//! 2. **Episode Generation**: Sequential prompts generate narrative for each
//!    episode (grouped by Propp's phases). Each prompt includes context from
//!    previous episodes for coherence.
//!
//! # Example
//!
//! ```ignore
//! use fableforge_core::{Tale, GenConfig, RandomGen, Generator};
//! use fableforge_llm::{ClaudeClient, StoryComposer, StyleConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Generate morphological structure
//!     let mut gen = RandomGen::with_seed(42);
//!     let config = GenConfig::new().with_move_count(1..2);
//!     let tale = gen.generate(&config)?;
//!
//!     // Configure style
//!     let style = StyleConfig::new()
//!         .genre("психологический триллер")
//!         .setting_hint("современная Москва")
//!         .tone("напряжённый, тревожный");
//!
//!     // Create client and generate
//!     let client = ClaudeClient::new(std::env::var("ANTHROPIC_API_KEY")?)
//!         .with_model("claude-sonnet-4-20250514");
//!
//!     let composer = StoryComposer::new(client, style);
//!     let result = composer.compose(&tale).await?;
//!
//!     println!("{}", result.text);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod composer;
pub mod context;
pub mod episode;
pub mod error;
pub mod prompt;

// Re-exports for convenience
pub use client::ClaudeClient;
pub use composer::StoryComposer;
pub use context::{CharacterResponse, EpisodeResult, GeneratedCharacter, GeneratedStory, TaleContext};
pub use episode::{Episode, EpisodeKind};
pub use error::LlmError;
pub use prompt::{PromptBuilder, StyleConfig};
