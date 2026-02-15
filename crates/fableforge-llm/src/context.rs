//! Generation context for maintaining state across prompts.

use std::collections::HashMap;

use fableforge_core::PersonaId;
use serde::{Deserialize, Serialize};

use crate::episode::Episode;

/// Context passed between prompts during generation.
#[derive(Debug, Clone, Default)]
pub struct TaleContext {
    /// Generated characters indexed by PersonaId.
    pub characters: HashMap<PersonaId, GeneratedCharacter>,
    /// Generated setting description.
    pub setting: Option<String>,
    /// Results from previous episodes.
    pub episodes: Vec<EpisodeResult>,
}

impl TaleContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply character generation response to context.
    pub fn apply_characters(&mut self, response: CharacterResponse) {
        for char in response.characters {
            self.characters.insert(char.id, char);
        }
        self.setting = Some(response.setting);
    }

    /// Add an episode result to context.
    pub fn add_episode_result(&mut self, episode: Episode, text: String) {
        self.episodes.push(EpisodeResult { episode, text });
    }

    /// Get summary of previous episodes for context.
    pub fn summary(&self) -> String {
        if self.episodes.is_empty() {
            return String::new();
        }

        self.episodes
            .iter()
            .enumerate()
            .map(|(i, ep)| {
                let preview = truncate_text(&ep.text, 200);
                format!("{}. {}", i + 1, preview)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get text of the last episode.
    pub fn last_text(&self) -> Option<&str> {
        self.episodes.last().map(|ep| ep.text.as_str())
    }

    /// Get a character by ID.
    pub fn character(&self, id: PersonaId) -> Option<&GeneratedCharacter> {
        self.characters.get(&id)
    }

    /// Get character name by ID, with fallback.
    pub fn character_name(&self, id: PersonaId) -> String {
        self.characters
            .get(&id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("Персонаж {}", id.0))
    }

    /// Convert context into final generated story.
    pub fn into_story(self) -> GeneratedStory {
        let text = self
            .episodes
            .iter()
            .map(|ep| ep.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        GeneratedStory {
            characters: self.characters.into_values().collect(),
            setting: self.setting.unwrap_or_default(),
            text,
            episodes: self.episodes,
        }
    }
}

/// A generated character with name and attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCharacter {
    /// Original persona ID from the tale structure.
    pub id: PersonaId,
    /// Generated name.
    pub name: String,
    /// Short epithet (e.g., "хитрый", "мудрый").
    pub epithet: Option<String>,
    /// Appearance description.
    pub appearance: Option<String>,
}

/// Response from character generation prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterResponse {
    /// List of generated characters.
    pub characters: Vec<GeneratedCharacter>,
    /// Generated setting description.
    pub setting: String,
}

/// Result of generating a single episode.
#[derive(Debug, Clone)]
pub struct EpisodeResult {
    /// The episode that was generated.
    pub episode: Episode,
    /// The generated text.
    pub text: String,
}

/// Complete generated story.
#[derive(Debug, Clone)]
pub struct GeneratedStory {
    /// All generated characters.
    pub characters: Vec<GeneratedCharacter>,
    /// Setting description.
    pub setting: String,
    /// Complete narrative text.
    pub text: String,
    /// Individual episode results.
    pub episodes: Vec<EpisodeResult>,
}

/// Truncate text to a maximum length, adding ellipsis if truncated.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(
            truncate_text("this is a longer text", 10),
            "this is..."
        );
    }

    #[test]
    fn test_context_character_name_fallback() {
        let ctx = TaleContext::new();
        assert_eq!(
            ctx.character_name(PersonaId(1)),
            "Персонаж 1"
        );
    }
}
