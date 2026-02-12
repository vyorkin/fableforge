//! FableForge Core — Morphological fairy tale generator.
//!
//! This crate implements Vladimir Propp's morphological analysis from
//! "Morphology of the Folktale" (1928). A fairy tale is modeled as:
//!
//! **tale = initial situation + one or more moves**
//!
//! The generated tale structure serves as input for LLM-based text generation.
//!
//! # Modules
//!
//! - [`function`] — The 31 narrative functions of dramatis personae
//! - [`dramatis`] — Characters and their spheres of action
//! - [`tale`] — Tale structure: moves, moments, initial situation
//! - [`connective`] — Connective elements between functions
//! - [`formula`] — Symbolic notation parsing and serialization
//! - [`syntax`] — Validation rules and absurdity scoring
//! - [`generate`] — Random and template-based generation

use serde::{Deserialize, Serialize};

pub mod connective;
pub mod dramatis;
pub mod formula;
pub mod function;
pub mod generate;
pub mod syntax;
pub mod tale;

/// Language for localized output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Lang {
    /// English
    #[default]
    En,
    /// Russian
    Ru,
}

// Re-exports for convenience
pub use connective::Connective;
pub use dramatis::{Attributes, Persona, PersonaId, Sphere};
pub use formula::{Formula, FormulaElement, ParseError};
pub use function::{NarrativeFunction, NarrativeFunctionInstance, Phase};
pub use generate::{GenConfig, GenError, Generator, RandomGen, Template, TemplateElement, TemplateGen};
pub use syntax::{Rule, Syntax, SyntaxError, SyntaxWarning, ValidationResult};
pub use tale::{InitialSituation, Moment, Move, MoveRelation, Setting, Tale};
