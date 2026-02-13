//! Generation (генерация).
//!
//! Generate fairy tale structures. The actual narrative text will be
//! produced by an external LLM API based on these structures.

use std::ops::Range;

use rand::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dramatis::{Persona, Sphere};
use crate::function::{NarrativeFunction, NarrativeFunctionInstance, Phase};
use crate::syntax::Syntax;
use crate::tale::{InitialSituation, Moment, Move, Setting, Tale};

/// Generator of morphological structures (генератор морфологических структур).
pub trait Generator {
    /// Generate a tale structure.
    ///
    /// # Errors
    ///
    /// Returns a `GenError` if generation fails.
    fn generate(&mut self, config: &GenConfig) -> Result<Tale, GenError>;
}

/// Generation configuration (конфигурация генерации).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenConfig {
    /// Syntax rules to follow.
    #[serde(skip)]
    pub syntax: Syntax,
    /// Maximum allowed absurdity (0.0 = strict, 1.0 = anything goes).
    pub max_absurdity: f32,
    /// Range for number of moves.
    pub move_count: Range<usize>,
    /// Random seed (for reproducibility).
    pub seed: Option<u64>,
    /// Include initial situation (начальная ситуация).
    pub include_initial: bool,
}

impl GenConfig {
    /// Create a new configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            syntax: Syntax::canonical(),
            max_absurdity: 0.3,
            move_count: 1..3,
            seed: None,
            include_initial: true,
        }
    }

    /// Set maximum absurdity.
    #[must_use]
    pub fn with_max_absurdity(mut self, max: f32) -> Self {
        self.max_absurdity = max;
        self
    }

    /// Set move count range.
    #[must_use]
    pub fn with_move_count(mut self, range: Range<usize>) -> Self {
        self.move_count = range;
        self
    }

    /// Set random seed.
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Include or exclude initial situation.
    #[must_use]
    pub fn with_initial(mut self, include: bool) -> Self {
        self.include_initial = include;
        self
    }
}

impl Default for GenConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Generation error (ошибка генерации).
#[derive(Debug, Clone, Error)]
pub enum GenError {
    /// Cannot satisfy constraints.
    #[error("cannot satisfy constraints: {0}")]
    ConstraintViolation(String),
    /// Template error.
    #[error("template error: {0}")]
    TemplateError(String),
}

/// Random generation (случайная генерация).
pub struct RandomGen {
    rng: StdRng,
}

impl RandomGen {
    /// Create a new random generator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// Create with a specific seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Generate a random move following canonical structure.
    fn generate_move(&mut self, is_first: bool) -> Move {
        let mut m = if is_first {
            Move::new()
        } else {
            Move::continuation()
        };

        // Build function sequence following Propp's phases
        let mut functions = Vec::new();

        // Optionally add preparatory functions
        if self.rng.gen_bool(0.6) {
            functions.push(NarrativeFunction::Absentation);
        }
        if self.rng.gen_bool(0.5) {
            functions.push(NarrativeFunction::Interdiction);
            functions.push(NarrativeFunction::Violation);
        }

        // Complication (required)
        if self.rng.gen_bool(0.7) {
            functions.push(NarrativeFunction::Villainy);
        } else {
            functions.push(NarrativeFunction::Lack);
        }

        // Mediation and departure
        if self.rng.gen_bool(0.8) {
            functions.push(NarrativeFunction::Mediation);
        }
        functions.push(NarrativeFunction::Counteraction);
        functions.push(NarrativeFunction::Departure);

        // Donor sequence
        if self.rng.gen_bool(0.7) {
            functions.push(NarrativeFunction::DonorTest);
            functions.push(NarrativeFunction::HeroReaction);
            functions.push(NarrativeFunction::Acquisition);
        }

        // Struggle sequence
        if self.rng.gen_bool(0.6) {
            functions.push(NarrativeFunction::Guidance);
            functions.push(NarrativeFunction::Struggle);
            functions.push(NarrativeFunction::Victory);
        }

        // Liquidation
        functions.push(NarrativeFunction::Liquidation);

        // Return
        functions.push(NarrativeFunction::Return);
        if self.rng.gen_bool(0.4) {
            functions.push(NarrativeFunction::Pursuit);
            functions.push(NarrativeFunction::Rescue);
        }

        // Resolution
        if self.rng.gen_bool(0.5) {
            functions.push(NarrativeFunction::Recognition);
        }
        if self.rng.gen_bool(0.3) {
            functions.push(NarrativeFunction::Punishment);
        }
        if self.rng.gen_bool(0.8) {
            functions.push(NarrativeFunction::Wedding);
        }

        // Add moments
        for func in functions {
            m.moments.push(Moment::new(NarrativeFunctionInstance::new(func)));
        }

        m
    }

    /// Generate basic personae structure.
    fn generate_personae(&mut self) -> Vec<Persona> {
        let mut personae = Vec::new();
        let mut next_id = 1u32;

        // Always have a hero
        personae.push(Persona::new(next_id, vec![Sphere::Hero]));
        next_id += 1;

        // Usually have a villain
        if self.rng.gen_bool(0.9) {
            personae.push(Persona::new(next_id, vec![Sphere::Villain]));
            next_id += 1;
        }

        // Sometimes have a donor
        if self.rng.gen_bool(0.7) {
            personae.push(Persona::new(next_id, vec![Sphere::Donor]));
            next_id += 1;
        }

        // Sometimes have a princess
        if self.rng.gen_bool(0.6) {
            personae.push(Persona::new(next_id, vec![Sphere::Princess]));
        }

        personae
    }

    /// Generate initial situation (начальная ситуация).
    fn generate_initial_situation(&mut self) -> InitialSituation {
        // Propp's typical initial situation elements
        const TIMES: &[&str] = &[
            "давным-давно",
            "в стародавние времена",
            "в некотором царстве",
            "однажды",
            "в прежние времена",
        ];

        const FAMILY_CONTEXTS: &[&str] = &[
            "жили-были старик со старухой",
            "жил-был царь с царицей",
            "в одной деревне жила бедная вдова",
            "у одного купца было три сына",
            "жил-был мужик с женой",
            "в тридевятом царстве жил король",
        ];

        const PROSPERITY_STATES: &[&str] = &[
            "и жили они в достатке",
            "но были они бедны",
            "и было у них всего вдоволь",
            "и не знали они горя",
            "но счастья им не хватало",
        ];

        let time = TIMES[self.rng.gen_range(0..TIMES.len())].to_string();
        let family = FAMILY_CONTEXTS[self.rng.gen_range(0..FAMILY_CONTEXTS.len())];
        let prosperity = PROSPERITY_STATES[self.rng.gen_range(0..PROSPERITY_STATES.len())];

        let context = format!("{}, {}", family, prosperity);

        InitialSituation::new()
            .with_setting(Setting::new().time(time))
            .with_context(context)
    }
}

impl Default for RandomGen {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for RandomGen {
    fn generate(&mut self, config: &GenConfig) -> Result<Tale, GenError> {
        if let Some(seed) = config.seed {
            self.rng = StdRng::seed_from_u64(seed);
        }

        let personae = self.generate_personae();
        let initial = if config.include_initial {
            Some(self.generate_initial_situation())
        } else {
            None
        };

        let mut tale = Tale {
            initial,
            personae,
            ..Default::default()
        };

        // Assign persona IDs to moments where appropriate
        let hero_id = tale.personae.iter()
            .find(|p| p.spheres.contains(&Sphere::Hero))
            .map(|p| p.id);
        let villain_id = tale.personae.iter()
            .find(|p| p.spheres.contains(&Sphere::Villain))
            .map(|p| p.id);

        // Generate moves
        let move_count = self.rng.gen_range(config.move_count.clone());
        for i in 0..move_count {
            let mut m = self.generate_move(i == 0);

            // Assign agents based on function phase
            for moment in &mut m.moments {
                match moment.function.function.phase() {
                    Phase::Complication if moment.function.function == NarrativeFunction::Villainy => {
                        moment.agent = villain_id;
                        moment.patient = hero_id;
                    }
                    Phase::Donor | Phase::Struggle | Phase::Return | Phase::Resolution => {
                        moment.agent = hero_id;
                    }
                    _ => {}
                }
            }

            tale.moves.push(m);
        }

        // Validate
        let result = config.syntax.validate_tale(&tale);
        if result.absurdity > config.max_absurdity {
            return Err(GenError::ConstraintViolation(format!(
                "absurdity {} exceeds maximum {}",
                result.absurdity, config.max_absurdity
            )));
        }

        Ok(tale)
    }
}

/// Template-based generation (генерация по шаблону).
pub struct TemplateGen {
    rng: StdRng,
    template: Template,
}

impl TemplateGen {
    /// Create a new template generator.
    #[must_use]
    pub fn new(template: Template) -> Self {
        Self {
            rng: StdRng::from_entropy(),
            template,
        }
    }

    /// Create with a specific seed.
    #[must_use]
    pub fn with_seed(template: Template, seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            template,
        }
    }

    fn expand_template(&mut self) -> Result<Vec<NarrativeFunction>, GenError> {
        let mut functions = Vec::new();

        for element in &self.template.elements {
            match element {
                TemplateElement::Fixed(f) => {
                    functions.push(*f);
                }
                TemplateElement::OneOf(choices) => {
                    if choices.is_empty() {
                        return Err(GenError::TemplateError("empty OneOf".to_string()));
                    }
                    let choice = choices[self.rng.gen_range(0..choices.len())];
                    functions.push(choice);
                }
                TemplateElement::Optional(inner) => {
                    if self.rng.gen_bool(0.5)
                        && let TemplateElement::Fixed(f) = inner.as_ref()
                    {
                        functions.push(*f);
                    }
                }
            }
        }

        Ok(functions)
    }
}

impl Generator for TemplateGen {
    fn generate(&mut self, config: &GenConfig) -> Result<Tale, GenError> {
        if let Some(seed) = config.seed {
            self.rng = StdRng::seed_from_u64(seed);
        }

        let functions = self.expand_template()?;

        let mut tale = Tale::default();

        // Create basic personae
        tale.personae.push(Persona::new(1u32, vec![Sphere::Hero]));
        tale.personae.push(Persona::new(2u32, vec![Sphere::Villain]));

        // Create a single move with all functions
        let mut m = Move::new();
        for f in functions {
            m.add_function(f);
        }
        tale.moves.push(m);

        // Validate
        let result = config.syntax.validate_tale(&tale);
        if result.absurdity > config.max_absurdity {
            return Err(GenError::ConstraintViolation(format!(
                "absurdity {} exceeds maximum {}",
                result.absurdity, config.max_absurdity
            )));
        }

        Ok(tale)
    }
}

/// Template — formula with variations (шаблон — формула с вариациями).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Template {
    /// Elements of the template.
    pub elements: Vec<TemplateElement>,
}

impl Template {
    /// Create an empty template.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Classic fairy tale (классическая волшебная сказка).
    #[must_use]
    pub fn classic() -> Self {
        use NarrativeFunction::*;
        use TemplateElement::*;

        Self {
            elements: vec![
                Optional(Box::new(Fixed(Absentation))),
                Optional(Box::new(Fixed(Interdiction))),
                Optional(Box::new(Fixed(Violation))),
                OneOf(vec![Villainy, Lack]),
                Fixed(Mediation),
                Fixed(Counteraction),
                Fixed(Departure),
                Fixed(DonorTest),
                Fixed(HeroReaction),
                Fixed(Acquisition),
                Fixed(Guidance),
                Fixed(Struggle),
                Fixed(Victory),
                Fixed(Liquidation),
                Fixed(Return),
                Optional(Box::new(Fixed(Pursuit))),
                Optional(Box::new(Fixed(Rescue))),
                Optional(Box::new(Fixed(Recognition))),
                Optional(Box::new(Fixed(Punishment))),
                Fixed(Wedding),
            ],
        }
    }

    /// Minimal move (минимальный ход).
    #[must_use]
    pub fn minimal() -> Self {
        use NarrativeFunction::*;
        use TemplateElement::*;

        Self {
            elements: vec![
                OneOf(vec![Villainy, Lack]),
                Fixed(Departure),
                Fixed(Liquidation),
                Fixed(Return),
            ],
        }
    }
}

/// Element of a template (элемент шаблона).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateElement {
    /// Fixed function (фиксированная функция).
    Fixed(NarrativeFunction),
    /// One of the list (одна из списка).
    OneOf(Vec<NarrativeFunction>),
    /// May be absent (может отсутствовать).
    Optional(Box<TemplateElement>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_gen_produces_valid_tale() {
        let mut generator = RandomGen::with_seed(42);
        let config = GenConfig::new()
            .with_max_absurdity(0.5)
            .with_move_count(1..2);

        let tale = generator.generate(&config).unwrap();
        assert!(!tale.moves.is_empty());
        assert!(!tale.personae.is_empty());
    }

    #[test]
    fn test_random_gen_reproducible() {
        let config = GenConfig::new().with_seed(42);

        let mut gen1 = RandomGen::new();
        let tale1 = gen1.generate(&config).unwrap();

        let mut gen2 = RandomGen::new();
        let tale2 = gen2.generate(&config).unwrap();

        assert_eq!(tale1.moves.len(), tale2.moves.len());
    }

    #[test]
    fn test_template_minimal() {
        let template = Template::minimal();
        let mut generator = TemplateGen::with_seed(template, 42);
        let config = GenConfig::new().with_max_absurdity(0.8);

        let tale = generator.generate(&config).unwrap();
        assert!(!tale.moves.is_empty());

        // Should have at least complication function
        let has_complication = tale.all_moments().any(|m| {
            m.function.function == NarrativeFunction::Villainy
                || m.function.function == NarrativeFunction::Lack
        });
        assert!(has_complication);
    }

    #[test]
    fn test_template_classic() {
        let template = Template::classic();
        let mut generator = TemplateGen::with_seed(template, 42);
        let config = GenConfig::new().with_max_absurdity(0.5);

        let tale = generator.generate(&config).unwrap();
        assert!(!tale.moves.is_empty());
    }

    #[test]
    fn test_initial_situation_generated() {
        let mut generator = RandomGen::with_seed(42);
        let config = GenConfig::new().with_move_count(1..2);

        let tale = generator.generate(&config).unwrap();
        assert!(tale.initial.is_some());

        let initial = tale.initial.unwrap();
        assert!(initial.setting.is_some());
        assert!(initial.context.is_some());
        assert!(initial.setting.unwrap().time.is_some());
    }

    #[test]
    fn test_initial_situation_can_be_disabled() {
        let mut generator = RandomGen::with_seed(42);
        let config = GenConfig::new()
            .with_move_count(1..2)
            .with_initial(false);

        let tale = generator.generate(&config).unwrap();
        assert!(tale.initial.is_none());
    }
}
