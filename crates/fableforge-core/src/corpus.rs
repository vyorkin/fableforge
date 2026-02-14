//! Corpus of connective elements and initial situation phrases.
//!
//! Loaded from an embedded TOML file at compile time.

use rand::prelude::*;
use serde::Deserialize;

const CORPUS_TOML: &str = include_str!("../data/connectives.toml");

/// Parsed corpus data from the embedded TOML dictionary.
#[derive(Debug, Clone)]
pub struct Corpus {
    data: CorpusData,
}

#[derive(Debug, Clone, Deserialize)]
struct CorpusData {
    motivations: StringList,
    transferences: StringList,
    temporals: StringList,
    initial_situation: InitialSituationData,
}

#[derive(Debug, Clone, Deserialize)]
struct StringList {
    items: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct InitialSituationData {
    times: StringList,
    family_contexts: StringList,
    prosperity_states: StringList,
}

impl Corpus {
    /// Load corpus from embedded TOML data.
    #[must_use]
    pub fn load() -> Self {
        let data: CorpusData =
            toml::from_str(CORPUS_TOML).expect("embedded connectives.toml is invalid");
        Self { data }
    }

    /// Pick a random motivation.
    pub fn pick_motivation(&self, rng: &mut impl Rng) -> &str {
        &self.data.motivations.items[rng.gen_range(0..self.data.motivations.items.len())]
    }

    /// Pick a random transference.
    pub fn pick_transference(&self, rng: &mut impl Rng) -> &str {
        &self.data.transferences.items[rng.gen_range(0..self.data.transferences.items.len())]
    }

    /// Pick a random temporal.
    pub fn pick_temporal(&self, rng: &mut impl Rng) -> &str {
        &self.data.temporals.items[rng.gen_range(0..self.data.temporals.items.len())]
    }

    /// Pick a random time phrase for initial situation.
    pub fn pick_time(&self, rng: &mut impl Rng) -> &str {
        &self.data.initial_situation.times.items
            [rng.gen_range(0..self.data.initial_situation.times.items.len())]
    }

    /// Pick a random family context for initial situation.
    pub fn pick_family_context(&self, rng: &mut impl Rng) -> &str {
        &self.data.initial_situation.family_contexts.items
            [rng.gen_range(0..self.data.initial_situation.family_contexts.items.len())]
    }

    /// Pick a random prosperity state for initial situation.
    pub fn pick_prosperity_state(&self, rng: &mut impl Rng) -> &str {
        &self.data.initial_situation.prosperity_states.items
            [rng.gen_range(0..self.data.initial_situation.prosperity_states.items.len())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;

    #[test]
    fn test_corpus_loads() {
        let corpus = Corpus::load();
        assert!(corpus.data.motivations.items.len() >= 20);
        assert!(corpus.data.transferences.items.len() >= 18);
        assert!(corpus.data.temporals.items.len() >= 16);
        assert!(corpus.data.initial_situation.times.items.len() >= 8);
        assert!(corpus.data.initial_situation.family_contexts.items.len() >= 10);
        assert!(corpus.data.initial_situation.prosperity_states.items.len() >= 8);
    }

    #[test]
    fn test_corpus_pick_motivation() {
        let corpus = Corpus::load();
        let mut rng = StdRng::seed_from_u64(42);
        let m = corpus.pick_motivation(&mut rng);
        assert!(!m.is_empty());
    }

    #[test]
    fn test_corpus_pick_transference() {
        let corpus = Corpus::load();
        let mut rng = StdRng::seed_from_u64(42);
        let t = corpus.pick_transference(&mut rng);
        assert!(!t.is_empty());
    }

    #[test]
    fn test_corpus_pick_temporal() {
        let corpus = Corpus::load();
        let mut rng = StdRng::seed_from_u64(42);
        let t = corpus.pick_temporal(&mut rng);
        assert!(!t.is_empty());
    }

    #[test]
    fn test_corpus_pick_initial_situation() {
        let corpus = Corpus::load();
        let mut rng = StdRng::seed_from_u64(42);
        let time = corpus.pick_time(&mut rng);
        let family = corpus.pick_family_context(&mut rng);
        let prosperity = corpus.pick_prosperity_state(&mut rng);
        assert!(!time.is_empty());
        assert!(!family.is_empty());
        assert!(!prosperity.is_empty());
    }

    #[test]
    fn test_corpus_deterministic_with_seed() {
        let corpus = Corpus::load();
        let mut rng1 = StdRng::seed_from_u64(123);
        let mut rng2 = StdRng::seed_from_u64(123);
        assert_eq!(
            corpus.pick_motivation(&mut rng1),
            corpus.pick_motivation(&mut rng2)
        );
    }
}
