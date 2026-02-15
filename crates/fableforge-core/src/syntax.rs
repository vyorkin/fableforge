//! Syntax rules and validation (правила синтаксиса и валидация).
//!
//! Propp identified patterns in the order and combination of functions.
//! This module provides validation and an "absurdity score" measuring deviation
//! from canonical fairy tale structure.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    function::NarrativeFunction,
    tale::{Move, Tale},
};

/// Set of morphological syntax rules (правила морфологического синтаксиса).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syntax {
    rules: Vec<Rule>,
}

impl Syntax {
    /// Create empty syntax.
    #[must_use]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create syntax from rules.
    #[must_use]
    pub fn from_rules(rules: Vec<Rule>) -> Self {
        Self { rules }
    }

    /// Canonical Propp rules (канонические правила Проппа).
    #[must_use]
    pub fn canonical() -> Self {
        use NarrativeFunction::*;
        use Rule::*;

        Self::from_rules(vec![
            // Order rules: preparatory section
            Precedes {
                before: Absentation,
                after: Interdiction,
            },
            Precedes {
                before: Interdiction,
                after: Violation,
            },
            Precedes {
                before: Reconnaissance,
                after: Delivery,
            },
            Precedes {
                before: Trickery,
                after: Complicity,
            },
            // Complication comes after preparation
            Precedes {
                before: Complicity,
                after: Villainy,
            },
            Precedes {
                before: Complicity,
                after: Lack,
            },
            // Villainy/Lack triggers the quest
            Precedes {
                before: Villainy,
                after: Mediation,
            },
            Precedes {
                before: Lack,
                after: Mediation,
            },
            Precedes {
                before: Mediation,
                after: Counteraction,
            },
            Precedes {
                before: Counteraction,
                after: Departure,
            },
            // Donor sequence
            Precedes {
                before: Departure,
                after: DonorTest,
            },
            Precedes {
                before: DonorTest,
                after: HeroReaction,
            },
            Precedes {
                before: HeroReaction,
                after: Acquisition,
            },
            // Struggle sequence
            Precedes {
                before: Acquisition,
                after: Guidance,
            },
            Precedes {
                before: Guidance,
                after: Struggle,
            },
            Precedes {
                before: Struggle,
                after: Victory,
            },
            Precedes {
                before: Victory,
                after: Liquidation,
            },
            // Return
            Precedes {
                before: Liquidation,
                after: Return,
            },
            Precedes {
                before: Return,
                after: Pursuit,
            },
            Precedes {
                before: Pursuit,
                after: Rescue,
            },
            // Recognition sequence
            Precedes {
                before: UnrecognizedArrival,
                after: UnfoundedClaims,
            },
            Precedes {
                before: UnfoundedClaims,
                after: DifficultTask,
            },
            Precedes {
                before: DifficultTask,
                after: Solution,
            },
            // Resolution
            Precedes {
                before: Solution,
                after: Recognition,
            },
            Precedes {
                before: Recognition,
                after: Exposure,
            },
            Precedes {
                before: Exposure,
                after: Transfiguration,
            },
            // Paired functions
            Paired {
                first: Interdiction,
                second: Violation,
            },
            Paired {
                first: Reconnaissance,
                second: Delivery,
            },
            Paired {
                first: Trickery,
                second: Complicity,
            },
            Paired {
                first: DonorTest,
                second: HeroReaction,
            },
            Paired {
                first: Struggle,
                second: Victory,
            },
            Paired {
                first: DifficultTask,
                second: Solution,
            },
            Paired {
                first: Pursuit,
                second: Rescue,
            },
            // Mutually exclusive
            Excludes {
                a: Villainy,
                b: Lack,
            },
        ])
    }

    /// Get all rules.
    #[must_use]
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Validate a move (валидация хода).
    #[must_use]
    pub fn validate_move(&self, m: &Move) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let functions: Vec<NarrativeFunction> =
            m.moments.iter().map(|m| m.function.function).collect();

        // Check each rule
        for rule in &self.rules {
            match rule {
                Rule::Precedes { before, after } => {
                    if let (Some(pos_before), Some(pos_after)) = (
                        functions.iter().position(|f| f == before),
                        functions.iter().position(|f| f == after),
                    ) && pos_before > pos_after
                    {
                        errors.push(SyntaxError::OrderViolation {
                            before: *before,
                            after: *after,
                        });
                    }
                }
                Rule::Paired { first, second } => {
                    let has_first = functions.contains(first);
                    let has_second = functions.contains(second);
                    if has_first != has_second {
                        warnings.push(SyntaxWarning::BrokenPair {
                            first: *first,
                            second: *second,
                            present: if has_first { *first } else { *second },
                        });
                    }
                }
                Rule::Excludes { a, b } => {
                    if functions.contains(a) && functions.contains(b) {
                        warnings.push(SyntaxWarning::MutualExclusion {
                            a: *a,
                            b: *b,
                        });
                    }
                }
            }
        }

        // Check for core functions
        let has_complication = functions.contains(&NarrativeFunction::Villainy)
            || functions.contains(&NarrativeFunction::Lack);
        if !has_complication && !functions.is_empty() {
            warnings.push(SyntaxWarning::MissingComplication);
        }

        let valid = errors.is_empty();
        let absurdity = calculate_absurdity(&functions, &errors, &warnings);

        ValidationResult {
            valid,
            errors,
            warnings,
            absurdity,
        }
    }

    /// Validate a tale (валидация сказки).
    #[must_use]
    pub fn validate_tale(&self, t: &Tale) -> ValidationResult {
        let mut all_errors = Vec::new();
        let mut all_warnings = Vec::new();

        // Validate each move and its embedded moves
        for m in &t.moves {
            let result = self.validate_move(m);
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);

            for em in &m.embedded_moves {
                let em_result = self.validate_move(em);
                all_errors.extend(em_result.errors);
                all_warnings.extend(em_result.warnings);
            }
        }

        // Check for at least one move
        if t.moves.is_empty() {
            all_warnings.push(SyntaxWarning::EmptyTale);
        }

        let valid = all_errors.is_empty();

        // Calculate overall absurdity
        let all_functions: Vec<NarrativeFunction> =
            t.all_moments().map(|m| m.function.function).collect();
        let absurdity = calculate_absurdity(
            &all_functions,
            &all_errors,
            &all_warnings,
        );

        ValidationResult {
            valid,
            errors: all_errors,
            warnings: all_warnings,
            absurdity,
        }
    }
}

impl Default for Syntax {
    fn default() -> Self {
        Self::canonical()
    }
}

/// Syntax rule (правило синтаксиса).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rule {
    /// A must precede B (A должна быть до B).
    Precedes {
        before: NarrativeFunction,
        after: NarrativeFunction,
    },
    /// A and B are paired (парные функции).
    Paired {
        first: NarrativeFunction,
        second: NarrativeFunction,
    },
    /// A excludes B in the same move (взаимоисключающие).
    Excludes {
        a: NarrativeFunction,
        b: NarrativeFunction,
    },
}

/// Result of validation (результат валидации).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the structure is valid.
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<SyntaxError>,
    /// Validation warnings.
    pub warnings: Vec<SyntaxWarning>,
    /// Absurdity coefficient: 0.0 = canonical, 1.0 = chaos.
    pub absurdity: f32,
}

/// Syntax error (ошибка синтаксиса).
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SyntaxError {
    /// Order violation.
    #[error("{before:?} must precede {after:?}")]
    OrderViolation {
        before: NarrativeFunction,
        after: NarrativeFunction,
    },
}

/// Syntax warning (предупреждение синтаксиса).
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SyntaxWarning {
    /// Broken function pair.
    #[error("broken pair: {first:?}/{second:?}, only {present:?} present")]
    BrokenPair {
        first: NarrativeFunction,
        second: NarrativeFunction,
        present: NarrativeFunction,
    },
    /// Mutually exclusive functions present.
    #[error("{a:?} and {b:?} are mutually exclusive")]
    MutualExclusion {
        a: NarrativeFunction,
        b: NarrativeFunction,
    },
    /// Missing complication (A or a).
    #[error("missing complication (Villainy or Lack)")]
    MissingComplication,
    /// Empty tale.
    #[error("empty tale (no moves)")]
    EmptyTale,
}

/// Calculate absurdity score (коэффициент абсурда).
fn calculate_absurdity(
    functions: &[NarrativeFunction],
    errors: &[SyntaxError],
    warnings: &[SyntaxWarning],
) -> f32 {
    if functions.is_empty() {
        return 1.0;
    }

    let mut score = 0.0;

    // Errors contribute more to absurdity
    score += errors.len() as f32 * 0.15;

    // Warnings contribute less
    for warning in warnings {
        match warning {
            SyntaxWarning::MissingComplication => score += 0.2,
            SyntaxWarning::BrokenPair { .. } => score += 0.1,
            SyntaxWarning::MutualExclusion { .. } => score += 0.12,
            SyntaxWarning::EmptyTale => score += 0.3,
        }
    }

    // Check canonical order
    let order_score = canonical_order_score(functions);
    score += (1.0 - order_score) * 0.3;

    // Clamp to [0, 1]
    score.clamp(0.0, 1.0)
}

/// Score how well the functions follow canonical order (0.0 = chaos, 1.0 =
/// perfect).
fn canonical_order_score(functions: &[NarrativeFunction]) -> f32 {
    if functions.len() <= 1 {
        return 1.0;
    }

    let mut inversions = 0;
    let total_pairs = functions.len() * (functions.len() - 1) / 2;

    for i in 0..functions.len() {
        for j in (i + 1)..functions.len() {
            if functions[i].index() > functions[j].index() {
                inversions += 1;
            }
        }
    }

    if total_pairs == 0 {
        1.0
    } else {
        1.0 - (inversions as f32 / total_pairs as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{function::NarrativeFunctionInstance, tale::Moment};

    fn make_move(functions: &[NarrativeFunction]) -> Move {
        let mut m = Move::new();
        for f in functions {
            m.moments.push(Moment::new(
                NarrativeFunctionInstance::new(*f),
            ));
        }
        m
    }

    #[test]
    fn test_valid_sequence() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            NarrativeFunction::Villainy,
            NarrativeFunction::Mediation,
            NarrativeFunction::Counteraction,
            NarrativeFunction::Departure,
        ]);

        let result = syntax.validate_move(&m);
        assert!(result.valid);
    }

    #[test]
    fn test_order_violation() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            NarrativeFunction::Departure, // Should come after Counteraction
            NarrativeFunction::Counteraction,
        ]);

        let result = syntax.validate_move(&m);
        assert!(!result.valid);
    }

    #[test]
    fn test_mutual_exclusion() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            NarrativeFunction::Villainy,
            NarrativeFunction::Lack, // Both Villainy and Lack
        ]);

        let result = syntax.validate_move(&m);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| matches!(w, SyntaxWarning::MutualExclusion { .. }))
        );
    }

    #[test]
    fn test_absurdity_low_for_canonical() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            NarrativeFunction::Absentation,
            NarrativeFunction::Interdiction,
            NarrativeFunction::Violation,
            NarrativeFunction::Villainy,
            NarrativeFunction::Mediation,
            NarrativeFunction::Counteraction,
            NarrativeFunction::Departure,
            NarrativeFunction::Liquidation,
            NarrativeFunction::Return,
            NarrativeFunction::Wedding,
        ]);

        let result = syntax.validate_move(&m);
        assert!(result.absurdity < 0.3);
    }
}
