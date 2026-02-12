//! FableForge Core — Morphological fairy tale generator.
//!
//! This crate implements Vladimir Propp's morphological analysis from
//! "Morphology of the Folktale" (1928). A fairy tale is modeled as:
//!
//! **tale = initial situation + one or more moves**
//!
//! # Modules
//!
//! - [`function`] — The 32 functions of dramatis personae
//! - [`dramatis`] — Characters and their spheres of action
//! - [`tale`] — Tale structure: moves, moments, initial situation
//! - [`connective`] — Connective elements: motivations, transference, triplication
//! - [`formula`] — Symbolic notation parsing and serialization
//! - [`syntax`] — Validation rules and absurdity scoring
//! - [`gen`] — Random and template-based generation

pub mod connective;
pub mod dramatis;
pub mod formula;
pub mod function;
pub mod generate;
pub mod syntax;
pub mod tale;

// Re-exports for convenience
pub use connective::{Connective, Motivation, Temporal, Transference, Triplication};
pub use dramatis::{Attributes, Nature, Persona, PersonaId, Sphere};
pub use formula::{Formula, FormulaElement, ParseError};
pub use function::{Function, FunctionInstance, Phase};
pub use generate::{GenConfig, GenError, Generator, RandomGen, Template, TemplateElement, TemplateGen};
pub use syntax::{Rule, Syntax, SyntaxError, SyntaxWarning, ValidationResult};
pub use tale::{
    InitialSituation, MagicalMeans, MagicalMeansId, MagicalMeansKind, Moment, Move, MoveRelation,
    Prosperity, Setting, Tale,
};
