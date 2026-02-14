//! Episode segmentation for tale generation.
//!
//! An episode is the minimal semantic unit for a single LLM prompt.
//! Tales are segmented into episodes based on narrative phases.

use std::collections::BTreeMap;

use fableforge_core::{Moment, Phase, Tale};

/// An episode — semantic unit for one prompt.
#[derive(Debug, Clone)]
pub struct Episode {
    /// Type of episode.
    pub kind: EpisodeKind,
    /// Moments included in this episode.
    pub moments: Vec<Moment>,
    /// Whether this episode uses triplication (утроение).
    pub triplication: bool,
    /// Whether this is an embedded (side quest) episode.
    pub is_embedded: bool,
}

/// Type of episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeKind {
    /// Character and setting generation (expects JSON response).
    CharacterGeneration,
    /// Initial situation / exposition.
    InitialSituation,
    /// Narrative phase with functions.
    Phase(Phase),
}

impl Episode {
    /// Create a character generation episode.
    pub fn character_generation() -> Self {
        Self {
            kind: EpisodeKind::CharacterGeneration,
            moments: Vec::new(),
            triplication: false,
            is_embedded: false,
        }
    }

    /// Create an initial situation episode.
    pub fn initial_situation() -> Self {
        Self {
            kind: EpisodeKind::InitialSituation,
            moments: Vec::new(),
            triplication: false,
            is_embedded: false,
        }
    }

    /// Create a phase episode with moments.
    pub fn phase(phase: Phase, moments: Vec<Moment>) -> Self {
        Self {
            kind: EpisodeKind::Phase(phase),
            moments,
            triplication: false,
            is_embedded: false,
        }
    }

    /// Segment a tale into episodes.
    ///
    /// The segmentation strategy:
    /// 1. Character generation episode (always first)
    /// 2. Initial situation episode (if tale has initial situation)
    /// 3. Phase episodes for each move, grouped by phase
    pub fn segment(tale: &Tale) -> Vec<Episode> {
        let mut episodes = Vec::new();

        // 1. Character generation episode (always first)
        episodes.push(Episode::character_generation());

        // 2. Initial situation (if present)
        if tale.initial.is_some() {
            episodes.push(Episode::initial_situation());
        }

        // 3. Group moments by phase for each move
        for mov in &tale.moves {
            let mut phase_moments: BTreeMap<PhaseOrd, Vec<Moment>> = BTreeMap::new();

            for moment in &mov.moments {
                let phase = moment.function.function.phase();
                phase_moments
                    .entry(PhaseOrd(phase))
                    .or_default()
                    .push(moment.clone());
            }

            // Create episodes in phase order
            for (PhaseOrd(phase), moments) in phase_moments {
                if !moments.is_empty() {
                    let mut ep = Episode::phase(phase, moments);
                    // Pass triplication hint to donor-phase episodes
                    if phase == Phase::Donor {
                        ep.triplication = mov.triplication;
                    }
                    episodes.push(ep);
                }
            }

            // Process embedded moves
            for em in &mov.embedded_moves {
                let mut em_phase_moments: BTreeMap<PhaseOrd, Vec<Moment>> = BTreeMap::new();
                for moment in &em.moments {
                    let phase = moment.function.function.phase();
                    em_phase_moments
                        .entry(PhaseOrd(phase))
                        .or_default()
                        .push(moment.clone());
                }
                for (PhaseOrd(phase), moments) in em_phase_moments {
                    if !moments.is_empty() {
                        let mut ep = Episode::phase(phase, moments);
                        ep.is_embedded = true;
                        episodes.push(ep);
                    }
                }
            }
        }

        episodes
    }

    /// Check if this is the character generation episode.
    pub fn is_character_generation(&self) -> bool {
        matches!(self.kind, EpisodeKind::CharacterGeneration)
    }

    /// Check if this is a narrative episode (initial situation or phase).
    pub fn is_narrative(&self) -> bool {
        matches!(
            self.kind,
            EpisodeKind::InitialSituation | EpisodeKind::Phase(_)
        )
    }
}

/// Wrapper for Phase that implements Ord for BTreeMap usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhaseOrd(Phase);

impl PartialOrd for PhaseOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PhaseOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        phase_index(self.0).cmp(&phase_index(other.0))
    }
}

/// Get canonical index for phase ordering.
fn phase_index(phase: Phase) -> u8 {
    match phase {
        Phase::Preparation => 0,
        Phase::Complication => 1,
        Phase::Donor => 2,
        Phase::Struggle => 3,
        Phase::Return => 4,
        Phase::Recognition => 5,
        Phase::Resolution => 6,
    }
}

#[cfg(test)]
mod tests {
    use fableforge_core::{Move, NarrativeFunction};

    use super::*;

    #[test]
    fn test_segment_empty_tale() {
        let tale = Tale::default();
        let episodes = Episode::segment(&tale);
        // Only character generation for empty tale
        assert_eq!(episodes.len(), 1);
        assert!(episodes[0].is_character_generation());
    }

    #[test]
    fn test_segment_with_initial_situation() {
        let tale = Tale {
            initial: Some(fableforge_core::InitialSituation::default()),
            ..Default::default()
        };
        let episodes = Episode::segment(&tale);
        assert_eq!(episodes.len(), 2);
        assert!(episodes[0].is_character_generation());
        assert_eq!(episodes[1].kind, EpisodeKind::InitialSituation);
    }

    #[test]
    fn test_segment_groups_by_phase() {
        let mut tale = Tale::default();
        let mut mov = Move::new();
        mov.add_function(NarrativeFunction::Villainy); // Complication
        mov.add_function(NarrativeFunction::Departure); // Complication
        mov.add_function(NarrativeFunction::DonorTest); // Donor
        mov.add_function(NarrativeFunction::Acquisition); // Donor
        tale.moves.push(mov);

        let episodes = Episode::segment(&tale);
        // CharGen + 2 phases (Complication, Donor)
        assert_eq!(episodes.len(), 3);
        assert!(episodes[0].is_character_generation());

        if let EpisodeKind::Phase(phase) = episodes[1].kind {
            assert_eq!(phase, Phase::Complication);
            assert_eq!(episodes[1].moments.len(), 2);
        } else {
            panic!("Expected Phase episode");
        }

        if let EpisodeKind::Phase(phase) = episodes[2].kind {
            assert_eq!(phase, Phase::Donor);
            assert_eq!(episodes[2].moments.len(), 2);
        } else {
            panic!("Expected Phase episode");
        }
    }

    #[test]
    fn test_phase_ordering() {
        assert!(PhaseOrd(Phase::Preparation) < PhaseOrd(Phase::Complication));
        assert!(PhaseOrd(Phase::Complication) < PhaseOrd(Phase::Donor));
        assert!(PhaseOrd(Phase::Resolution) > PhaseOrd(Phase::Return));
    }
}
