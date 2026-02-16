use serde::{Deserialize, Serialize};

/// FSM states for the bot dialogue.
#[derive(Clone, Default, Serialize, Deserialize)]
pub enum BotState {
    #[default]
    Start,
    SelectGenre,
    SelectTone {
        genre: Option<String>,
    },
    SelectMoves {
        genre: Option<String>,
        tone: Option<String>,
    },
    SelectCharacterNames {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
    },
    AwaitCharacterNames {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
    },
    SelectPlace {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        character_names: Vec<String>,
    },
    AwaitPlaceText {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        character_names: Vec<String>,
    },
    SelectMaxEpisodes {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        character_names: Vec<String>,
        place: Option<String>,
    },
    SelectMaxMoments {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        character_names: Vec<String>,
        place: Option<String>,
        max_episodes: Option<usize>,
    },
    SelectSeed {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        character_names: Vec<String>,
        place: Option<String>,
        max_episodes: Option<usize>,
        max_moments_per_episode: Option<usize>,
    },
}

/// Fully resolved generation config built from dialogue steps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateConfig {
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub moves: usize,
    pub character_names: Vec<String>,
    pub place: Option<String>,
    pub max_episodes: Option<usize>,
    pub max_moments_per_episode: Option<usize>,
    pub seed: Option<u64>,
}
