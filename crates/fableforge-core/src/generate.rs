//! Generation (генерация).
//!
//! Generate fairy tale structures either randomly or from templates.

use std::ops::Range;

use rand::prelude::*;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dramatis::{Attributes, Nature, Persona};
use crate::function::{Function, FunctionInstance, Phase};
use crate::syntax::Syntax;
use crate::tale::{InitialSituation, MagicalMeans, MagicalMeansKind, Moment, Move, Prosperity, Setting, Tale};

/// Generator of morphological structures (генератор морфологических структур).
pub trait Generator {
    /// Generate a tale.
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
        }
    }

    /// Set the syntax.
    #[must_use]
    pub fn with_syntax(mut self, syntax: Syntax) -> Self {
        self.syntax = syntax;
        self
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
    /// Maximum attempts exceeded.
    #[error("maximum generation attempts exceeded")]
    MaxAttemptsExceeded,
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

    /// Generate a random initial situation.
    fn generate_initial(&mut self) -> InitialSituation {
        let settings = [
            "В некотором царстве, в некотором государстве",
            "В тридевятом царстве, в тридесятом государстве",
            "В одной деревне",
            "В стольном граде",
        ];
        let times = [
            "Жили-были",
            "Давным-давно",
            "В стародавние времена",
        ];
        let prosperities = [
            Prosperity::Poor,
            Prosperity::Normal,
            Prosperity::Prosperous,
            Prosperity::Royal,
        ];

        InitialSituation {
            family: vec![],
            setting: Setting {
                place: Some(settings[self.rng.gen_range(0..settings.len())].to_string()),
                time: Some(times[self.rng.gen_range(0..times.len())].to_string()),
            },
            prosperity: prosperities[self.rng.gen_range(0..prosperities.len())],
        }
    }

    /// Generate core personae.
    fn generate_personae(&mut self) -> Vec<Persona> {
        let mut personae = Vec::new();
        let mut next_id = 1u32;

        // Always have a hero
        let hero_names = ["Иван", "Василиса", "Алёша", "Марья"];
        let hero = Persona::hero(next_id).with_attributes(
            Attributes::with_name(hero_names[self.rng.gen_range(0..hero_names.len())])
                .nature(Nature::Human),
        );
        personae.push(hero);
        next_id += 1;

        // Usually have a villain
        if self.rng.gen_bool(0.9) {
            let villain_names = ["Кощей Бессмертный", "Баба-Яга", "Змей Горыныч", "Чудо-Юдо"];
            let villain = Persona::villain(next_id).with_attributes(
                Attributes::with_name(villain_names[self.rng.gen_range(0..villain_names.len())])
                    .nature(if self.rng.gen_bool(0.5) {
                        Nature::Magical
                    } else {
                        Nature::Human
                    }),
            );
            personae.push(villain);
            next_id += 1;
        }

        // Sometimes have a donor
        if self.rng.gen_bool(0.7) {
            let donor_names = ["Старушка", "Старичок", "Леший", "Щука"];
            let donor = Persona::donor(next_id).with_attributes(
                Attributes::with_name(donor_names[self.rng.gen_range(0..donor_names.len())])
                    .nature(Nature::Magical),
            );
            personae.push(donor);
            next_id += 1;
        }

        // Sometimes have a princess
        if self.rng.gen_bool(0.6) {
            let princess_names = ["Елена Прекрасная", "Василиса Премудрая", "Царевна"];
            let princess = Persona::princess(next_id).with_attributes(
                Attributes::with_name(princess_names[self.rng.gen_range(0..princess_names.len())])
                    .epithet("прекрасная")
                    .nature(Nature::Human),
            );
            personae.push(princess);
        }

        personae
    }

    /// Generate a random move.
    fn generate_move(&mut self, is_first: bool, personae: &[Persona]) -> Move {
        let mut m = if is_first {
            Move::new()
        } else {
            Move::continuation()
        };

        let hero_id = personae
            .iter()
            .find(|p| p.is_hero())
            .map(|p| p.id);
        let villain_id = personae
            .iter()
            .find(|p| p.is_villain())
            .map(|p| p.id);

        // Select functions for this move
        let mut functions = Vec::new();

        // Optionally add preparatory functions
        if self.rng.gen_bool(0.6) {
            functions.push(Function::Absentation);
        }
        if self.rng.gen_bool(0.5) {
            functions.push(Function::Interdiction);
            functions.push(Function::Violation);
        }

        // Add complication (always needed)
        if self.rng.gen_bool(0.7) {
            functions.push(Function::Villainy);
        } else {
            functions.push(Function::Lack);
        }

        // Add mediation and departure
        if self.rng.gen_bool(0.8) {
            functions.push(Function::Mediation);
        }
        functions.push(Function::Counteraction);
        functions.push(Function::Departure);

        // Add donor sequence
        if self.rng.gen_bool(0.7) {
            functions.push(Function::DonorTest);
            functions.push(Function::HeroReaction);
            functions.push(Function::Acquisition);
        }

        // Add struggle sequence
        if self.rng.gen_bool(0.6) {
            functions.push(Function::Guidance);
            functions.push(Function::Struggle);
            functions.push(Function::Victory);
        }

        // Add liquidation
        functions.push(Function::Liquidation);

        // Add return
        functions.push(Function::Return);
        if self.rng.gen_bool(0.4) {
            functions.push(Function::Pursuit);
            functions.push(Function::Rescue);
        }

        // Add resolution
        if self.rng.gen_bool(0.5) {
            functions.push(Function::Recognition);
        }
        if self.rng.gen_bool(0.3) {
            functions.push(Function::Punishment);
        }
        if self.rng.gen_bool(0.8) {
            functions.push(Function::Wedding);
        }

        // Create moments
        for func in functions {
            let mut moment = Moment::new(FunctionInstance::new(func));

            // Assign agents based on function
            match func.phase() {
                Phase::Preparation | Phase::Complication => {
                    if func == Function::Villainy {
                        moment.agent = villain_id;
                        moment.patient = hero_id;
                    }
                }
                Phase::Donor | Phase::Struggle | Phase::Return => {
                    moment.agent = hero_id;
                }
                Phase::Recognition | Phase::Resolution => {
                    moment.agent = hero_id;
                }
            }

            m.add_moment(moment);
        }

        m
    }
}

impl Default for RandomGen {
    fn default() -> Self {
        Self::new()
    }
}

impl Generator for RandomGen {
    fn generate(&mut self, config: &GenConfig) -> Result<Tale, GenError> {
        // Apply seed if provided
        if let Some(seed) = config.seed {
            self.rng = StdRng::seed_from_u64(seed);
        }

        let personae = self.generate_personae();
        let initial = self.generate_initial();

        let mut tale = Tale::new(initial);
        tale.personae = personae.clone();

        // Generate moves
        let move_count = self.rng.gen_range(config.move_count.clone());
        for i in 0..move_count {
            let m = self.generate_move(i == 0, &personae);
            tale.add_move(m);
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

    /// Expand the template into a sequence of functions.
    fn expand_template(&mut self) -> Result<Vec<Function>, GenError> {
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
                TemplateElement::Gap { min, max } => {
                    // Fill gap with random functions from the appropriate phase
                    let count = self.rng.gen_range(*min..=*max);
                    for _ in 0..count {
                        let f = Function::ALL[self.rng.gen_range(0..Function::ALL.len())];
                        functions.push(f);
                    }
                }
                TemplateElement::Optional(inner) => {
                    if self.rng.gen_bool(0.5) {
                        match inner.as_ref() {
                            TemplateElement::Fixed(f) => functions.push(*f),
                            TemplateElement::OneOf(choices) if !choices.is_empty() => {
                                let choice = choices[self.rng.gen_range(0..choices.len())];
                                functions.push(choice);
                            }
                            _ => {}
                        }
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

        let initial = InitialSituation::default();
        let mut tale = Tale::new(initial);

        // Create basic personae
        tale.add_persona(Persona::hero(1u32).with_attributes(Attributes::with_name("Герой")));
        tale.add_persona(Persona::villain(2u32).with_attributes(Attributes::with_name("Злодей")));

        // Create a single move with all functions
        let mut m = Move::new();
        for f in functions {
            m.add_function(f);
        }
        tale.add_move(m);

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

/// Template — formula with gaps (шаблон — формула с пропусками).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Elements of the template.
    pub elements: Vec<TemplateElement>,
}

impl Template {
    /// Create an empty template.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Create a template from elements.
    #[must_use]
    pub fn from_elements(elements: Vec<TemplateElement>) -> Self {
        Self { elements }
    }

    /// Classic fairy tale (классическая волшебная сказка).
    ///
    /// A full sequence with all major phases.
    #[must_use]
    pub fn classic() -> Self {
        use Function::*;
        use TemplateElement::*;

        Self::from_elements(vec![
            // Preparation (optional)
            Optional(Box::new(Fixed(Absentation))),
            Optional(Box::new(Fixed(Interdiction))),
            Optional(Box::new(Fixed(Violation))),
            // Complication
            OneOf(vec![Villainy, Lack]),
            Fixed(Mediation),
            Fixed(Counteraction),
            Fixed(Departure),
            // Donor
            Fixed(DonorTest),
            Fixed(HeroReaction),
            Fixed(Acquisition),
            // Struggle
            Fixed(Guidance),
            Fixed(Struggle),
            Fixed(Victory),
            Fixed(Liquidation),
            // Return
            Fixed(Return),
            Optional(Box::new(Fixed(Pursuit))),
            Optional(Box::new(Fixed(Rescue))),
            // Resolution
            Optional(Box::new(Fixed(Recognition))),
            Optional(Box::new(Fixed(Punishment))),
            Fixed(Wedding),
        ])
    }

    /// Minimal move (минимальный ход).
    ///
    /// The smallest possible complete tale structure.
    #[must_use]
    pub fn minimal() -> Self {
        use Function::*;
        use TemplateElement::*;

        Self::from_elements(vec![
            OneOf(vec![Villainy, Lack]),
            Fixed(Departure),
            Fixed(Liquidation),
            Fixed(Return),
        ])
    }

    /// Quest template (поиск).
    ///
    /// Hero goes on a quest to find something.
    #[must_use]
    pub fn quest() -> Self {
        use Function::*;
        use TemplateElement::*;

        Self::from_elements(vec![
            Fixed(Lack),
            Fixed(Mediation),
            Fixed(Counteraction),
            Fixed(Departure),
            Fixed(DonorTest),
            Fixed(HeroReaction),
            Fixed(Acquisition),
            Fixed(Guidance),
            Fixed(Liquidation),
            Fixed(Return),
            Fixed(Wedding),
        ])
    }

    /// Combat template (борьба).
    ///
    /// Hero defeats a villain.
    #[must_use]
    pub fn combat() -> Self {
        use Function::*;
        use TemplateElement::*;

        Self::from_elements(vec![
            Fixed(Villainy),
            Fixed(Mediation),
            Fixed(Counteraction),
            Fixed(Departure),
            Optional(Box::new(Fixed(DonorTest))),
            Optional(Box::new(Fixed(HeroReaction))),
            Optional(Box::new(Fixed(Acquisition))),
            Fixed(Struggle),
            Fixed(Victory),
            Fixed(Liquidation),
            Fixed(Return),
            Fixed(Punishment),
            Fixed(Wedding),
        ])
    }

    /// Add an element.
    pub fn push(&mut self, element: TemplateElement) {
        self.elements.push(element);
    }
}

impl Default for Template {
    fn default() -> Self {
        Self::new()
    }
}

/// Element of a template (элемент шаблона).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateElement {
    /// Fixed function (фиксированная функция).
    Fixed(Function),
    /// One of the list (одна из списка).
    OneOf(Vec<Function>),
    /// Gap to fill (пропуск для заполнения).
    Gap { min: usize, max: usize },
    /// May be absent (может отсутствовать).
    Optional(Box<TemplateElement>),
}

/// Generate magical means.
#[must_use]
pub fn generate_magical_means(rng: &mut impl Rng) -> MagicalMeans {
    let kinds = [
        (MagicalMeansKind::Animal, &["Сивка-Бурка", "Серый волк", "Жар-птица"][..]),
        (MagicalMeansKind::Object, &["Меч-кладенец", "Волшебное кольцо", "Клубок ниток", "Ковёр-самолёт"][..]),
        (MagicalMeansKind::Substance, &["Живая вода", "Мёртвая вода", "Молодильные яблоки"][..]),
        (MagicalMeansKind::Quality, &["Шапка-невидимка", "Оборотничество", "Богатырская сила"][..]),
    ];

    let (kind, names) = &kinds[rng.gen_range(0..kinds.len())];
    let name = names[rng.gen_range(0..names.len())];

    MagicalMeans::new(1u32, *kind).with_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_gen() {
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
    fn test_template_classic() {
        let template = Template::classic();
        assert!(!template.elements.is_empty());
    }

    #[test]
    fn test_template_minimal() {
        let template = Template::minimal();
        assert!(!template.elements.is_empty());

        let mut generator = TemplateGen::with_seed(template, 42);
        let config = GenConfig::new().with_max_absurdity(0.8);

        let tale = generator.generate(&config).unwrap();
        assert!(!tale.moves.is_empty());
    }

    #[test]
    fn test_template_gen() {
        let template = Template::classic();
        let mut generator = TemplateGen::with_seed(template, 42);
        let config = GenConfig::new().with_max_absurdity(0.5);

        let tale = generator.generate(&config).unwrap();
        assert!(!tale.moves.is_empty());

        // Should have at least some core functions
        let has_complication = tale.all_moments().any(|m| {
            m.function.function == Function::Villainy || m.function.function == Function::Lack
        });
        assert!(has_complication);
    }

    #[test]
    fn test_generate_magical_means() {
        let mut rng = StdRng::seed_from_u64(42);
        let means = generate_magical_means(&mut rng);
        assert!(means.name.is_some());
    }

    #[test]
    fn test_gen_config_builder() {
        let config = GenConfig::new()
            .with_max_absurdity(0.2)
            .with_move_count(2..4)
            .with_seed(123);

        assert_eq!(config.max_absurdity, 0.2);
        assert_eq!(config.move_count, 2..4);
        assert_eq!(config.seed, Some(123));
    }
}
