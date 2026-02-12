//! Connective elements (связки).
//!
//! Connectives are the elements that link functions together in a tale.
//! They include motivations, modes of transference, temporal markers,
//! and the principle of triplication.

use serde::{Deserialize, Serialize};

use crate::tale::Moment;

/// Connective element between moments (связка между моментами).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Connective {
    /// Motivation for action (мотивировка действия).
    Motivation(Motivation),
    /// Mode of spatial transference (способ перемещения).
    Transference(Transference),
    /// Temporal transition (временной переход).
    Temporal(Temporal),
}

impl Connective {
    /// Create a motivation connective.
    #[must_use]
    pub const fn motivation(m: Motivation) -> Self {
        Self::Motivation(m)
    }

    /// Create a transference connective.
    #[must_use]
    pub const fn transference(t: Transference) -> Self {
        Self::Transference(t)
    }

    /// Create a temporal connective.
    #[must_use]
    pub const fn temporal(t: Temporal) -> Self {
        Self::Temporal(t)
    }
}

/// Motivation (мотивировка).
///
/// Motivations explain why a character performs an action.
/// They are often formulaic and interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Motivation {
    /// Revenge (месть).
    Revenge,
    /// Desire (желание).
    Desire,
    /// Duty (долг).
    Duty,
    /// Curiosity (любопытство).
    Curiosity,
    /// Need (нужда).
    Need,
}

impl Motivation {
    /// All motivations.
    pub const ALL: [Motivation; 5] = [
        Motivation::Revenge,
        Motivation::Desire,
        Motivation::Duty,
        Motivation::Curiosity,
        Motivation::Need,
    ];

    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Motivation::Revenge => "Revenge",
            Motivation::Desire => "Desire",
            Motivation::Duty => "Duty",
            Motivation::Curiosity => "Curiosity",
            Motivation::Need => "Need",
        }
    }

    /// Russian name.
    #[must_use]
    pub const fn name_ru(&self) -> &'static str {
        match self {
            Motivation::Revenge => "Месть",
            Motivation::Desire => "Желание",
            Motivation::Duty => "Долг",
            Motivation::Curiosity => "Любопытство",
            Motivation::Need => "Нужда",
        }
    }
}

/// Mode of transference (способ перемещения).
///
/// How characters move through space in the tale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Transference {
    /// On foot (пешком).
    Walking,
    /// By air (по воздуху).
    Flying,
    /// On horseback (верхом).
    Riding,
    /// Instantaneous magical transport (мгновенное перемещение).
    Magical,
}

impl Transference {
    /// All modes of transference.
    pub const ALL: [Transference; 4] = [
        Transference::Walking,
        Transference::Flying,
        Transference::Riding,
        Transference::Magical,
    ];

    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Transference::Walking => "Walking",
            Transference::Flying => "Flying",
            Transference::Riding => "Riding",
            Transference::Magical => "Magical",
        }
    }
}

/// Temporal markers (временные маркеры).
///
/// Formulaic expressions of time passing between events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Temporal {
    /// Immediately (сразу).
    Immediate,
    /// Next day (на следующий день).
    NextDay,
    /// "Whether long or short" — a formulaic indefinite time (долго ли, коротко ли).
    AfterTime,
}

impl Temporal {
    /// All temporal markers.
    pub const ALL: [Temporal; 3] = [Temporal::Immediate, Temporal::NextDay, Temporal::AfterTime];

    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Temporal::Immediate => "Immediately",
            Temporal::NextDay => "Next day",
            Temporal::AfterTime => "Whether long or short",
        }
    }

    /// Russian formula.
    #[must_use]
    pub const fn formula_ru(&self) -> &'static str {
        match self {
            Temporal::Immediate => "И тотчас",
            Temporal::NextDay => "На следующий день",
            Temporal::AfterTime => "Долго ли, коротко ли",
        }
    }
}

/// Triplication (утроение) — repetition of an episode three times.
///
/// One of the most characteristic features of fairy tales.
/// The third attempt is usually successful or climactic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Triplication {
    /// The repeated moments.
    pub moments: Vec<Moment>,
    /// Variation pattern.
    pub variation: TriplicationVariant,
}

impl Triplication {
    /// Create a new triplication.
    #[must_use]
    pub fn new(moments: Vec<Moment>, variation: TriplicationVariant) -> Self {
        Self { moments, variation }
    }

    /// Create an identical triplication.
    #[must_use]
    pub fn identical(moments: Vec<Moment>) -> Self {
        Self::new(moments, TriplicationVariant::Identical)
    }

    /// Create an ascending triplication.
    #[must_use]
    pub fn ascending(moments: Vec<Moment>) -> Self {
        Self::new(moments, TriplicationVariant::Ascending)
    }

    /// Create a third-success triplication.
    #[must_use]
    pub fn third_success(moments: Vec<Moment>) -> Self {
        Self::new(moments, TriplicationVariant::ThirdSuccess)
    }

    /// The number of repetitions (always 3 in classic fairy tales).
    #[must_use]
    pub const fn count(&self) -> usize {
        3
    }
}

/// Variant of triplication (вид утроения).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriplicationVariant {
    /// Identical repetition (одинаковое повторение).
    Identical,
    /// Ascending difficulty (нарастающая сложность).
    Ascending,
    /// Third attempt succeeds (третья попытка успешна).
    ThirdSuccess,
}

impl TriplicationVariant {
    /// All variants.
    pub const ALL: [TriplicationVariant; 3] = [
        TriplicationVariant::Identical,
        TriplicationVariant::Ascending,
        TriplicationVariant::ThirdSuccess,
    ];

    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            TriplicationVariant::Identical => "Identical",
            TriplicationVariant::Ascending => "Ascending",
            TriplicationVariant::ThirdSuccess => "Third Success",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motivation() {
        assert_eq!(Motivation::ALL.len(), 5);
        assert_eq!(Motivation::Revenge.name(), "Revenge");
        assert_eq!(Motivation::Revenge.name_ru(), "Месть");
    }

    #[test]
    fn test_transference() {
        assert_eq!(Transference::ALL.len(), 4);
        assert_eq!(Transference::Flying.name(), "Flying");
    }

    #[test]
    fn test_temporal() {
        assert_eq!(Temporal::ALL.len(), 3);
        assert_eq!(Temporal::AfterTime.formula_ru(), "Долго ли, коротко ли");
    }

    #[test]
    fn test_connective_creation() {
        let c1 = Connective::motivation(Motivation::Revenge);
        assert!(matches!(c1, Connective::Motivation(Motivation::Revenge)));

        let c2 = Connective::transference(Transference::Flying);
        assert!(matches!(c2, Connective::Transference(Transference::Flying)));

        let c3 = Connective::temporal(Temporal::NextDay);
        assert!(matches!(c3, Connective::Temporal(Temporal::NextDay)));
    }

    #[test]
    fn test_triplication() {
        let trip = Triplication::third_success(vec![]);
        assert_eq!(trip.count(), 3);
        assert_eq!(trip.variation, TriplicationVariant::ThirdSuccess);
    }
}
