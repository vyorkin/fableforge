//! Syntax rules and validation (правила синтаксиса и валидация).
//!
//! Propp identified strict rules about the order and combination of functions.
//! This module provides validation and an "absurdity score" measuring deviation
//! from canonical fairy tale structure.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::function::Function;
use crate::tale::{Move, Tale};

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
        use Function::*;
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
            // Requirements
            Requires {
                func: Liquidation,
                prereq: Villainy,
            },
            Requires {
                func: Victory,
                prereq: Struggle,
            },
            Requires {
                func: Acquisition,
                prereq: DonorTest,
            },
        ])
    }

    /// Add a rule.
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
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

        let functions: Vec<Function> = m.moments.iter().map(|m| m.function.function).collect();

        // Check each rule
        for rule in &self.rules {
            match rule {
                Rule::Precedes { before, after } => {
                    if let (Some(pos_before), Some(pos_after)) = (
                        functions.iter().position(|f| f == before),
                        functions.iter().position(|f| f == after),
                    )
                        && pos_before > pos_after
                    {
                        errors.push(SyntaxError::OrderViolation {
                            before: *before,
                            after: *after,
                        });
                    }
                }
                Rule::Requires { func, prereq } => {
                    if functions.contains(func) && !functions.contains(prereq) {
                        warnings.push(SyntaxWarning::MissingPrerequisite {
                            func: *func,
                            prereq: *prereq,
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
                Rule::Mandatory(func) => {
                    if !functions.contains(func) {
                        warnings.push(SyntaxWarning::MissingMandatory(*func));
                    }
                }
                Rule::Excludes { a, b } => {
                    if functions.contains(a) && functions.contains(b) {
                        warnings.push(SyntaxWarning::MutualExclusion { a: *a, b: *b });
                    }
                }
            }
        }

        // Check for core functions
        let has_complication = functions.contains(&Function::Villainy)
            || functions.contains(&Function::Lack);
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

        // Validate each move
        for m in &t.moves {
            let result = self.validate_move(m);
            all_errors.extend(result.errors);
            all_warnings.extend(result.warnings);
        }

        // Check for at least one move
        if t.moves.is_empty() {
            all_warnings.push(SyntaxWarning::EmptyTale);
        }

        let valid = all_errors.is_empty();

        // Calculate overall absurdity
        let all_functions: Vec<Function> = t
            .all_moments()
            .map(|m| m.function.function)
            .collect();
        let absurdity = calculate_absurdity(&all_functions, &all_errors, &all_warnings);

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
    Precedes { before: Function, after: Function },
    /// A requires B somewhere earlier (A требует наличия B).
    Requires { func: Function, prereq: Function },
    /// A and B are paired (парные функции).
    Paired { first: Function, second: Function },
    /// A is mandatory for a complete move (обязательная функция).
    Mandatory(Function),
    /// A excludes B in the same move (взаимоисключающие).
    Excludes { a: Function, b: Function },
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

impl ValidationResult {
    /// Check if there are any issues.
    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }

    /// Get total issue count.
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }
}

/// Syntax error (ошибка синтаксиса).
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SyntaxError {
    /// Order violation.
    #[error("{before:?} must precede {after:?}")]
    OrderViolation { before: Function, after: Function },
}

/// Syntax warning (предупреждение синтаксиса).
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SyntaxWarning {
    /// Missing prerequisite function.
    #[error("{func:?} requires {prereq:?}")]
    MissingPrerequisite { func: Function, prereq: Function },
    /// Broken function pair.
    #[error("broken pair: {first:?}/{second:?}, only {present:?} present")]
    BrokenPair {
        first: Function,
        second: Function,
        present: Function,
    },
    /// Missing mandatory function.
    #[error("missing mandatory function: {0:?}")]
    MissingMandatory(Function),
    /// Mutually exclusive functions present.
    #[error("{a:?} and {b:?} are mutually exclusive")]
    MutualExclusion { a: Function, b: Function },
    /// Missing complication (A or a).
    #[error("missing complication (Villainy or Lack)")]
    MissingComplication,
    /// Empty tale.
    #[error("empty tale (no moves)")]
    EmptyTale,
}

/// Calculate absurdity score (коэффициент абсурда).
///
/// Considers:
/// - Order violations (нарушения порядка функций)
/// - Broken pairs (разрыв парных функций)
/// - Missing mandatory functions (пропуск обязательных)
/// - Unusual function combinations
fn calculate_absurdity(
    functions: &[Function],
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
            SyntaxWarning::MissingPrerequisite { .. } => score += 0.08,
            SyntaxWarning::MutualExclusion { .. } => score += 0.12,
            SyntaxWarning::MissingMandatory(_) => score += 0.05,
            SyntaxWarning::EmptyTale => score += 0.3,
        }
    }

    // Check canonical order
    let order_score = canonical_order_score(functions);
    score += (1.0 - order_score) * 0.3;

    // Clamp to [0, 1]
    score.clamp(0.0, 1.0)
}

/// Score how well the functions follow canonical order (0.0 = chaos, 1.0 = perfect).
fn canonical_order_score(functions: &[Function]) -> f32 {
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

/// Calculate absurdity score for a tale (convenience function).
#[must_use]
pub fn absurdity_score(tale: &Tale, syntax: &Syntax) -> f32 {
    syntax.validate_tale(tale).absurdity
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tale::{InitialSituation, Moment};
    use crate::function::FunctionInstance;

    fn make_move(functions: &[Function]) -> Move {
        let mut m = Move::new();
        for f in functions {
            m.moments.push(Moment::new(FunctionInstance::new(*f)));
        }
        m
    }

    #[test]
    fn test_canonical_syntax() {
        let syntax = Syntax::canonical();
        assert!(!syntax.rules.is_empty());
    }

    #[test]
    fn test_valid_sequence() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Villainy,
            Function::Mediation,
            Function::Counteraction,
            Function::Departure,
        ]);

        let result = syntax.validate_move(&m);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_order_violation() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Departure,  // Should come after Counteraction
            Function::Counteraction,
        ]);

        let result = syntax.validate_move(&m);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_missing_complication() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Departure,
            Function::Return,
        ]);

        let result = syntax.validate_move(&m);
        // Should have warning about missing complication
        assert!(result.warnings.iter().any(|w| matches!(w, SyntaxWarning::MissingComplication)));
    }

    #[test]
    fn test_broken_pair() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Villainy,
            Function::Interdiction,  // Has Interdiction without Violation
        ]);

        let result = syntax.validate_move(&m);
        assert!(result.warnings.iter().any(|w| matches!(w, SyntaxWarning::BrokenPair { .. })));
    }

    #[test]
    fn test_mutual_exclusion() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Villainy,
            Function::Lack,  // Both Villainy and Lack
        ]);

        let result = syntax.validate_move(&m);
        assert!(result.warnings.iter().any(|w| matches!(w, SyntaxWarning::MutualExclusion { .. })));
    }

    #[test]
    fn test_absurdity_canonical() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Absentation,
            Function::Interdiction,
            Function::Violation,
            Function::Villainy,
            Function::Mediation,
            Function::Counteraction,
            Function::Departure,
            Function::DonorTest,
            Function::HeroReaction,
            Function::Acquisition,
            Function::Liquidation,
            Function::Return,
            Function::Wedding,
        ]);

        let result = syntax.validate_move(&m);
        // Canonical sequence should have low absurdity
        assert!(result.absurdity < 0.3);
    }

    #[test]
    fn test_absurdity_chaotic() {
        let syntax = Syntax::canonical();
        let m = make_move(&[
            Function::Wedding,      // End at the beginning
            Function::Return,
            Function::Departure,    // Middle
            Function::Absentation,  // Beginning at the end
        ]);

        let result = syntax.validate_move(&m);
        // Chaotic sequence should have high absurdity
        assert!(result.absurdity > 0.3);
    }

    #[test]
    fn test_validate_tale() {
        let syntax = Syntax::canonical();

        let mut tale = Tale::new(InitialSituation::default());
        tale.add_move(make_move(&[Function::Villainy, Function::Departure]));
        tale.add_move(make_move(&[Function::Return, Function::Wedding]));

        let result = syntax.validate_tale(&tale);
        assert!(result.valid);
    }

    #[test]
    fn test_empty_tale() {
        let syntax = Syntax::canonical();
        let tale = Tale::new(InitialSituation::default());

        let result = syntax.validate_tale(&tale);
        assert!(result.warnings.iter().any(|w| matches!(w, SyntaxWarning::EmptyTale)));
    }

    #[test]
    fn test_canonical_order_score() {
        // Perfect order
        let functions = vec![
            Function::Absentation,
            Function::Interdiction,
            Function::Villainy,
        ];
        assert_eq!(canonical_order_score(&functions), 1.0);

        // Reversed order
        let reversed = vec![
            Function::Villainy,
            Function::Interdiction,
            Function::Absentation,
        ];
        assert!(canonical_order_score(&reversed) < 0.5);
    }
}
