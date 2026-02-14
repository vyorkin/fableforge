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
    SelectSeed {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
    },
}

/// Fully resolved generation config built from dialogue steps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerateConfig {
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub moves: usize,
    pub seed: Option<u64>,
}
