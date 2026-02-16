//! Story composer — orchestrates the generation process.

use std::sync::Arc;

use fableforge_core::{Lang, Tale};
use tracing::{debug, info, instrument};

use crate::{
    client::LlmClient,
    context::{CharacterResponse, GeneratedStory, TaleContext},
    episode::{Episode, EpisodeKind},
    error::LlmError,
    prompt::{PromptBuilder, StyleConfig},
};

/// Story composer that orchestrates tale generation.
pub struct StoryComposer<C: LlmClient = crate::client::ClaudeClient> {
    client: Arc<C>,
    prompt_builder: PromptBuilder,
    /// Maximum character count for generated story (optional).
    max_characters: Option<usize>,
    /// Maximum number of episodes to generate (optional).
    max_episodes: Option<usize>,
    /// Maximum moments per episode (optional).
    max_moments_per_episode: Option<usize>,
}

impl<C: LlmClient> StoryComposer<C> {
    /// Create a new composer with client and style configuration.
    pub fn new(client: C, style: StyleConfig) -> Self {
        Self {
            client: Arc::new(client),
            prompt_builder: PromptBuilder::new(style),
            max_characters: None,
            max_episodes: None,
            max_moments_per_episode: None,
        }
    }

    /// Set the language for prompts.
    pub fn with_lang(mut self, lang: Lang) -> Self {
        self.prompt_builder = self.prompt_builder.with_lang(lang);
        self
    }

    /// Set maximum character count for generated story.
    pub fn with_max_characters(mut self, max: usize) -> Self {
        self.max_characters = Some(max);
        self
    }

    /// Set maximum number of episodes to generate.
    pub fn with_max_episodes(mut self, max: usize) -> Self {
        self.max_episodes = Some(max);
        self
    }

    /// Set maximum moments per episode.
    pub fn with_max_moments_per_episode(mut self, max: usize) -> Self {
        self.max_moments_per_episode = Some(max);
        self
    }

    /// Generate a complete story from a tale structure.
    ///
    /// This performs two-phase generation:
    /// 1. Character generation — creates names, epithets, appearances, and
    ///    setting
    /// 2. Episode generation — generates narrative for each episode
    ///    sequentially
    #[instrument(skip(self, tale), fields(moves = tale.moves.len(), personae = tale.personae.len()))]
    pub async fn compose(
        &self,
        tale: &Tale,
    ) -> Result<GeneratedStory, LlmError> {
        // Validate tale has content
        if tale.personae.is_empty()
            && tale.moves.is_empty()
            && tale.initial.is_none()
        {
            return Err(LlmError::EmptyTale);
        }

        // Segment tale into episodes
        let mut episodes = Episode::segment(tale);
        info!(
            "Segmented tale into {} episodes",
            episodes.len()
        );

        // Apply episode limit if specified
        if let Some(max_eps) = self.max_episodes
            && episodes.len() > max_eps
        {
            info!(
                "Limiting episodes from {} to {}",
                episodes.len(),
                max_eps
            );
            episodes.truncate(max_eps);
        }

        // Apply moments per episode limit if specified
        if let Some(max_moments) = self.max_moments_per_episode {
            for episode in &mut episodes {
                if episode.moments.len() > max_moments {
                    debug!(
                        "Limiting episode moments from {} to {}",
                        episode.moments.len(),
                        max_moments
                    );
                    episode.moments.truncate(max_moments);
                }
            }
        }

        let mut ctx = TaleContext::new();

        // Phase 1: Generate characters and setting
        if !tale.personae.is_empty() {
            info!("Generating characters...");
            let char_prompt = self.prompt_builder.character_prompt(tale);
            debug!(
                "Character prompt length: {} chars",
                char_prompt.len()
            );

            let characters: CharacterResponse =
                self.client.complete_json(&char_prompt).await?;
            info!(
                "Generated {} characters",
                characters.characters.len()
            );
            ctx.apply_characters(characters);
        }

        // Phase 2: Generate narrative for each episode
        for (i, episode) in episodes.iter().enumerate() {
            // Check character limit before generating next episode
            if let Some(max_chars) = self.max_characters {
                let current_length = ctx.total_text_length();
                if current_length >= max_chars {
                    info!(
                        "Character limit reached ({}/{}), stopping generation",
                        current_length, max_chars
                    );
                    break;
                }
            }

            match episode.kind {
                EpisodeKind::CharacterGeneration => {
                    // Already handled above
                    continue;
                }
                EpisodeKind::InitialSituation => {
                    info!(
                        "Generating initial situation (episode {})",
                        i
                    );
                    let initial = tale.initial.as_ref().ok_or_else(|| {
                        LlmError::MissingContext(
                            "Initial situation not found".to_string(),
                        )
                    })?;
                    let prompt = self.prompt_builder.initial_situation_prompt(
                        initial,
                        &ctx,
                        &tale.personae,
                    );
                    debug!(
                        "Initial situation prompt length: {} chars",
                        prompt.len()
                    );

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
                    let prompt = self.prompt_builder.phase_prompt(
                        episode,
                        &ctx,
                        &tale.personae,
                    );
                    debug!(
                        "Phase prompt length: {} chars",
                        prompt.len()
                    );

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
    use fableforge_core::{
        InitialSituation, Move, NarrativeFunction, Persona, Sphere,
    };

    use super::*;

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
        assert!(matches!(
            episodes[0].kind,
            EpisodeKind::CharacterGeneration
        ));
        assert!(matches!(
            episodes[1].kind,
            EpisodeKind::InitialSituation
        ));
        assert!(matches!(
            episodes[2].kind,
            EpisodeKind::Phase(_)
        ));
    }
}
