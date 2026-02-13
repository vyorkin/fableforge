//! Story composer — orchestrates the generation process.

use std::sync::Arc;

use tracing::{debug, info, instrument};

use fableforge_core::Tale;

use crate::client::LlmClient;
use crate::context::{CharacterResponse, GeneratedStory, TaleContext};
use crate::episode::{Episode, EpisodeKind};
use crate::error::LlmError;
use crate::prompt::{PromptBuilder, StyleConfig};

/// Story composer that orchestrates tale generation.
pub struct StoryComposer<C: LlmClient = crate::client::ClaudeClient> {
    client: Arc<C>,
    prompt_builder: PromptBuilder,
}

impl<C: LlmClient> StoryComposer<C> {
    /// Create a new composer with client and style configuration.
    pub fn new(client: C, style: StyleConfig) -> Self {
        Self {
            client: Arc::new(client),
            prompt_builder: PromptBuilder::new(style),
        }
    }

    /// Generate a complete story from a tale structure.
    ///
    /// This performs two-phase generation:
    /// 1. Character generation — creates names, epithets, appearances, and setting
    /// 2. Episode generation — generates narrative for each episode sequentially
    #[instrument(skip(self, tale), fields(moves = tale.moves.len(), personae = tale.personae.len()))]
    pub async fn compose(&self, tale: &Tale) -> Result<GeneratedStory, LlmError> {
        // Validate tale has content
        if tale.personae.is_empty() && tale.moves.is_empty() && tale.initial.is_none() {
            return Err(LlmError::EmptyTale);
        }

        // Segment tale into episodes
        let episodes = Episode::segment(tale);
        info!("Segmented tale into {} episodes", episodes.len());

        let mut ctx = TaleContext::new();

        // Phase 1: Generate characters and setting
        if !tale.personae.is_empty() {
            info!("Generating characters...");
            let char_prompt = self.prompt_builder.character_prompt(tale);
            debug!("Character prompt length: {} chars", char_prompt.len());

            let characters: CharacterResponse = self.client.complete_json(&char_prompt).await?;
            info!(
                "Generated {} characters",
                characters.characters.len()
            );
            ctx.apply_characters(characters);
        }

        // Phase 2: Generate narrative for each episode
        for (i, episode) in episodes.iter().enumerate() {
            match episode.kind {
                EpisodeKind::CharacterGeneration => {
                    // Already handled above
                    continue;
                }
                EpisodeKind::InitialSituation => {
                    info!("Generating initial situation (episode {})", i);
                    let initial = tale.initial.as_ref().ok_or_else(|| {
                        LlmError::MissingContext("Initial situation not found".to_string())
                    })?;
                    let prompt = self.prompt_builder.initial_situation_prompt(initial, &ctx);
                    debug!("Initial situation prompt length: {} chars", prompt.len());

                    let text = self.client.complete(&prompt).await?;
                    ctx.add_episode_result(episode.clone(), text);
                }
                EpisodeKind::Phase(phase) => {
                    info!(
                        "Generating phase {:?} (episode {}, {} moments)",
                        phase,
                        i,
                        episode.moments.len()
                    );
                    let prompt = self.prompt_builder.phase_prompt(episode, &ctx);
                    debug!("Phase prompt length: {} chars", prompt.len());

                    let text = self.client.complete(&prompt).await?;
                    ctx.add_episode_result(episode.clone(), text);
                }
            }
        }

        info!("Story generation complete");
        Ok(ctx.into_story())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fableforge_core::{InitialSituation, Move, NarrativeFunction, Persona, Sphere};

    fn create_test_tale() -> Tale {
        let mut mov = Move::new();
        mov.add_function(NarrativeFunction::Villainy);
        mov.add_function(NarrativeFunction::Departure);

        Tale {
            initial: Some(InitialSituation::default()),
            personae: vec![
                Persona::new(1u32, vec![Sphere::Hero]),
                Persona::new(2u32, vec![Sphere::Villain]),
            ],
            moves: vec![mov],
        }
    }

    #[test]
    fn test_segment_test_tale() {
        let tale = create_test_tale();
        let episodes = Episode::segment(&tale);

        // CharGen + InitialSituation + Complication phase
        assert_eq!(episodes.len(), 3);
        assert!(matches!(episodes[0].kind, EpisodeKind::CharacterGeneration));
        assert!(matches!(episodes[1].kind, EpisodeKind::InitialSituation));
        assert!(matches!(episodes[2].kind, EpisodeKind::Phase(_)));
    }
}
