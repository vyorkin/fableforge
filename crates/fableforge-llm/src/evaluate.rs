//! Coherence evaluation (LLM-as-judge).
//!
//! After generating a story, sends the text + morphological structure back to
//! the LLM for quality evaluation across four dimensions.

use std::sync::Arc;

use fableforge_core::{Lang, Tale};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use crate::{
    client::LlmClient,
    context::GeneratedStory,
    error::LlmError,
    prompt::{PromptBuilder, StyleConfig},
};

/// Coherence evaluation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceReport {
    /// Overall score (0.0–1.0), recalculated as average of dimensions.
    pub score: f32,
    /// Per-dimension breakdown.
    pub dimensions: CoherenceDimensions,
    /// Free-text summary from the evaluator.
    pub summary: String,
    /// Optional per-episode notes.
    #[serde(default)]
    pub episode_notes: Vec<EpisodeNote>,
}

impl CoherenceReport {
    /// Recalculate the overall score as the average of 4 dimensions,
    /// clamping all values to [0.0, 1.0].
    pub fn recalculate(&mut self) {
        self.dimensions.clamp();
        self.score = self.dimensions.average();
    }
}

/// Four evaluation dimensions, each scored 0.0–1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceDimensions {
    /// Do characters behave consistently with their assigned spheres and
    /// personalities?
    pub character_consistency: f32,
    /// Does the narrative actually follow the morphological structure
    /// (functions, phases)?
    pub structural_fidelity: f32,
    /// Do episodes flow logically from one to the next?
    pub episode_continuity: f32,
    /// Overall writing quality (style, language, engagement).
    pub narrative_quality: f32,
}

impl CoherenceDimensions {
    /// Clamp all dimension scores to [0.0, 1.0].
    pub fn clamp(&mut self) {
        self.character_consistency = self.character_consistency.clamp(0.0, 1.0);
        self.structural_fidelity = self.structural_fidelity.clamp(0.0, 1.0);
        self.episode_continuity = self.episode_continuity.clamp(0.0, 1.0);
        self.narrative_quality = self.narrative_quality.clamp(0.0, 1.0);
    }

    /// Average of all four dimensions.
    pub fn average(&self) -> f32 {
        (self.character_consistency
            + self.structural_fidelity
            + self.episode_continuity
            + self.narrative_quality)
            / 4.0
    }
}

/// A note about a specific episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeNote {
    /// Zero-based episode index.
    pub episode_index: usize,
    /// Evaluator's note about this episode.
    pub note: String,
}

/// Evaluates generated stories using LLM-as-judge.
pub struct CoherenceEvaluator<C: LlmClient> {
    client: Arc<C>,
    prompt_builder: PromptBuilder,
}

impl<C: LlmClient> CoherenceEvaluator<C> {
    /// Create a new evaluator.
    pub fn new(client: C, style: StyleConfig) -> Self {
        Self {
            client: Arc::new(client),
            prompt_builder: PromptBuilder::new(style),
        }
    }

    /// Set the language for prompts.
    pub fn with_lang(mut self, lang: Lang) -> Self {
        self.prompt_builder = self.prompt_builder.with_lang(lang);
        self
    }

    /// Evaluate a generated story against its morphological structure.
    #[instrument(skip(self, tale, story), fields(moves = tale.moves.len()))]
    pub async fn evaluate(
        &self,
        tale: &Tale,
        story: &GeneratedStory,
    ) -> Result<CoherenceReport, LlmError> {
        info!("Evaluating story coherence...");

        let prompt = self.prompt_builder.evaluation_prompt(tale, story);
        debug!(
            "Evaluation prompt length: {} chars",
            prompt.len()
        );

        let mut report: CoherenceReport =
            self.client.complete_json(&prompt).await?;
        report.recalculate();

        info!("Coherence score: {:.2}", report.score);
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use fableforge_core::{
        Move, NarrativeFunction, Persona, PersonaId, Sphere,
    };

    use super::*;
    use crate::{
        client::MockClient,
        context::{EpisodeResult, GeneratedCharacter},
        episode::Episode,
    };

    fn sample_report_json() -> String {
        r#"{
            "score": 0.99,
            "dimensions": {
                "character_consistency": 0.8,
                "structural_fidelity": 0.7,
                "episode_continuity": 0.9,
                "narrative_quality": 0.6
            },
            "summary": "Хорошая история с небольшими недочётами.",
            "episode_notes": [
                {"episode_index": 0, "note": "Отличное начало."},
                {"episode_index": 1, "note": "Слабая связь с предыдущим эпизодом."}
            ]
        }"#
        .to_string()
    }

    fn sample_tale_and_story() -> (Tale, GeneratedStory) {
        let mut mov = Move::new();
        mov.add_function(NarrativeFunction::Villainy);

        let tale = Tale {
            initial: None,
            personae: vec![
                Persona::new(1u32, vec![Sphere::Hero]),
                Persona::new(2u32, vec![Sphere::Villain]),
            ],
            moves: vec![mov],
        };

        let story = GeneratedStory {
            characters: vec![
                GeneratedCharacter {
                    id: PersonaId(1),
                    name: "Иван".to_string(),
                    epithet: Some("храбрый".to_string()),
                    appearance: None,
                },
                GeneratedCharacter {
                    id: PersonaId(2),
                    name: "Кощей".to_string(),
                    epithet: Some("бессмертный".to_string()),
                    appearance: None,
                },
            ],
            setting: "Тридевятое царство".to_string(),
            text: "Жил-был Иван. Кощей похитил царевну.".to_string(),
            episodes: vec![EpisodeResult {
                episode: Episode::phase(
                    fableforge_core::Phase::Complication,
                    Vec::new(),
                ),
                text: "Кощей похитил царевну.".to_string(),
            }],
        };

        (tale, story)
    }

    #[test]
    fn test_deserialize_coherence_report() {
        let json = sample_report_json();
        let mut report: CoherenceReport = serde_json::from_str(&json).unwrap();
        report.recalculate();

        // Score is recalculated as average, not the LLM-provided 0.99
        let expected = (0.8 + 0.7 + 0.9 + 0.6) / 4.0;
        assert!((report.score - expected).abs() < 1e-6);
        assert_eq!(report.episode_notes.len(), 2);
        assert_eq!(report.episode_notes[0].episode_index, 0);
    }

    #[test]
    fn test_deserialize_without_episode_notes() {
        let json = r#"{
            "score": 0.5,
            "dimensions": {
                "character_consistency": 0.5,
                "structural_fidelity": 0.5,
                "episode_continuity": 0.5,
                "narrative_quality": 0.5
            },
            "summary": "Средне."
        }"#;
        let report: CoherenceReport = serde_json::from_str(json).unwrap();
        assert!(report.episode_notes.is_empty());
    }

    #[test]
    fn test_score_clamping() {
        let mut dims = CoherenceDimensions {
            character_consistency: 1.5,
            structural_fidelity: -0.2,
            episode_continuity: 0.8,
            narrative_quality: 2.0,
        };
        dims.clamp();

        assert_eq!(dims.character_consistency, 1.0);
        assert_eq!(dims.structural_fidelity, 0.0);
        assert_eq!(dims.episode_continuity, 0.8);
        assert_eq!(dims.narrative_quality, 1.0);
    }

    #[test]
    fn test_dimensions_average() {
        let dims = CoherenceDimensions {
            character_consistency: 0.8,
            structural_fidelity: 0.6,
            episode_continuity: 1.0,
            narrative_quality: 0.4,
        };
        let avg = dims.average();
        assert!((avg - 0.7).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_evaluator_with_mock_client() {
        let client = MockClient::new().with_json(sample_report_json());
        let evaluator = CoherenceEvaluator::new(client, StyleConfig::new());

        let (tale, story) = sample_tale_and_story();
        let report = evaluator.evaluate(&tale, &story).await.unwrap();

        let expected = (0.8 + 0.7 + 0.9 + 0.6) / 4.0;
        assert!((report.score - expected).abs() < 1e-6);
        assert_eq!(
            report.summary,
            "Хорошая история с небольшими недочётами."
        );
    }

    #[tokio::test]
    async fn test_evaluator_error_handling() {
        let client = MockClient::new().with_error("API unavailable");
        let evaluator = CoherenceEvaluator::new(client, StyleConfig::new());

        let (tale, story) = sample_tale_and_story();
        let result = evaluator.evaluate(&tale, &story).await;

        assert!(result.is_err());
    }
}
