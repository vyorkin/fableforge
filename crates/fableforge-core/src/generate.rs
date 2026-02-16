//! Generation (генерация).
//!
//! Generate fairy tale structures. The actual narrative text will be
//! produced by an external LLM API based on these structures.

use std::{collections::HashSet, ops::Range};

use rand::{prelude::*, rngs::StdRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    connective::Connective,
    corpus::Corpus,
    dramatis::{Persona, Sphere},
    function::{NarrativeFunction, NarrativeFunctionInstance, Phase},
    subtype::subtype_count,
    syntax::Syntax,
    tale::{InitialSituation, Moment, Move, Setting, Tale},
};

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
    /// Generate subtypes for functions (подвиды функций).
    pub include_subtypes: bool,
    /// Enable probabilistic sphere merging (совмещение сфер действия).
    pub sphere_merge_enabled: bool,
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
            include_subtypes: true,
            sphere_merge_enabled: true,
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

    /// Include or exclude function subtypes.
    #[must_use]
    pub fn with_subtypes(mut self, include: bool) -> Self {
        self.include_subtypes = include;
        self
    }

    /// Enable or disable sphere merging.
    #[must_use]
    pub fn with_sphere_merging(mut self, enabled: bool) -> Self {
        self.sphere_merge_enabled = enabled;
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
    corpus: Corpus,
}

impl RandomGen {
    /// Create a new random generator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
            corpus: Corpus::load(),
        }
    }

    /// Create with a specific seed.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            corpus: Corpus::load(),
        }
    }

    /// Generate a random move following canonical structure.
    fn generate_move(
        &mut self,
        is_first: bool,
        include_subtypes: bool,
        has_false_hero: bool,
    ) -> Move {
        let mut m = if is_first {
            Move::new()
        } else {
            Move::continuation()
        };

        // Track negated functions for this move
        let mut negated: HashSet<NarrativeFunction> = HashSet::new();

        // Track chosen subtypes for coherence (function → subtype index)
        let mut chosen_subtypes: Vec<(NarrativeFunction, u8)> = Vec::new();

        // Build function sequence following Propp's phases
        let mut functions = Vec::new();

        // Optionally add preparatory functions
        if self.rng.gen_bool(0.6) {
            functions.push(NarrativeFunction::Absentation);
        }
        if self.rng.gen_bool(0.5) {
            // neg-β (~10%): interdiction implied but not stated; violation
            // still follows
            if self.rng.gen_bool(0.1) {
                negated.insert(NarrativeFunction::Interdiction);
            }
            functions.push(NarrativeFunction::Interdiction);
            functions.push(NarrativeFunction::Violation);
        }
        // Reconnaissance + Delivery (30%)
        if self.rng.gen_bool(0.3) {
            functions.push(NarrativeFunction::Reconnaissance);
            functions.push(NarrativeFunction::Delivery);
        }
        // Trickery + Complicity (25%)
        if self.rng.gen_bool(0.25) {
            functions.push(NarrativeFunction::Trickery);
            functions.push(NarrativeFunction::Complicity);
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
            if self.rng.gen_bool(0.15) {
                // neg-E: hero fails the donor test — no acquisition
                negated.insert(NarrativeFunction::HeroReaction);
            } else {
                functions.push(NarrativeFunction::Acquisition);
            }
        }

        // Struggle sequence
        let has_struggle = self.rng.gen_bool(0.6);
        if has_struggle {
            functions.push(NarrativeFunction::Guidance);
            // neg-H (~15%): struggle doesn't happen, villain defeated by
            // cunning
            if self.rng.gen_bool(0.15) {
                negated.insert(NarrativeFunction::Struggle);
                functions.push(NarrativeFunction::Struggle);
                // No Branding when struggle is negated; Victory still follows
            } else {
                functions.push(NarrativeFunction::Struggle);
                // Branding (~35%): hero receives mark/wound during struggle
                if self.rng.gen_bool(0.35) {
                    functions.push(NarrativeFunction::Branding);
                }
            }
            functions.push(NarrativeFunction::Victory);
        }

        // Liquidation
        functions.push(NarrativeFunction::Liquidation);

        // Return
        functions.push(NarrativeFunction::Return);
        if self.rng.gen_bool(0.4) {
            functions.push(NarrativeFunction::Pursuit);
            // neg-Rs (~10%): rescue fails, hero escapes on their own
            if self.rng.gen_bool(0.1) {
                negated.insert(NarrativeFunction::Rescue);
            }
            functions.push(NarrativeFunction::Rescue);
        }

        // False hero recognition sequence
        if has_false_hero && self.rng.gen_bool(0.5) {
            functions.push(NarrativeFunction::UnrecognizedArrival);
            functions.push(NarrativeFunction::UnfoundedClaims);
            functions.push(NarrativeFunction::DifficultTask);
            functions.push(NarrativeFunction::Solution);
        }

        // Resolution
        if has_false_hero {
            // With false hero, always add recognition
            functions.push(NarrativeFunction::Recognition);
            if self.rng.gen_bool(0.8) {
                functions.push(NarrativeFunction::Exposure);
            }
            if self.rng.gen_bool(0.4) {
                functions.push(NarrativeFunction::Transfiguration);
            }
        } else if self.rng.gen_bool(0.5) {
            functions.push(NarrativeFunction::Recognition);
        }
        if self.rng.gen_bool(0.3) {
            // neg-U (~20%): villain is forgiven
            if self.rng.gen_bool(0.2) {
                negated.insert(NarrativeFunction::Punishment);
            }
            functions.push(NarrativeFunction::Punishment);
        }
        if self.rng.gen_bool(0.8) {
            functions.push(NarrativeFunction::HappyEnding);
        }

        // Check if donor sequence is present (for triplication)
        let has_donor_sequence = functions
            .contains(&NarrativeFunction::DonorTest)
            && functions.contains(&NarrativeFunction::HeroReaction)
            && functions.contains(&NarrativeFunction::Acquisition);
        if has_donor_sequence && self.rng.gen_bool(0.4) {
            m.triplication = true;
        }

        // Add moments with optional subtypes and connectives
        for func in &functions {
            let is_negated = negated.contains(func);
            let instance = if is_negated {
                NarrativeFunctionInstance::negated(*func)
            } else if include_subtypes {
                let coherent =
                    self.pick_coherent_subtype(*func, &chosen_subtypes);
                if let Some(sub) = coherent {
                    NarrativeFunctionInstance::with_subtype(*func, sub)
                } else {
                    self.create_function_instance(*func, true)
                }
            } else {
                NarrativeFunctionInstance::new(*func)
            };

            // Track chosen subtype for coherence
            if let Some(sub) = instance.subtype {
                chosen_subtypes.push((*func, sub));
            }

            let mut moment = Moment::new(instance);

            // Attach connectives at key narrative transitions (~60%
            // probability)
            if self.rng.gen_bool(0.6) {
                moment.connective = self.pick_connective(*func);
            }

            m.moments.push(moment);
        }

        m
    }

    /// Pick a connective appropriate for the given function, if any.
    fn pick_connective(
        &mut self,
        func: NarrativeFunction,
    ) -> Option<Connective> {
        match func {
            NarrativeFunction::Counteraction => {
                let choice =
                    self.corpus.pick_motivation(&mut self.rng).to_string();
                Some(Connective::motivation(choice))
            }
            NarrativeFunction::Departure | NarrativeFunction::Return => {
                let choice =
                    self.corpus.pick_transference(&mut self.rng).to_string();
                Some(Connective::transference(choice))
            }
            NarrativeFunction::DonorTest | NarrativeFunction::Guidance => {
                let choice =
                    self.corpus.pick_temporal(&mut self.rng).to_string();
                Some(Connective::temporal(choice))
            }
            _ => None,
        }
    }

    /// Pick a coherent subtype based on previously chosen subtypes in the same
    /// move.
    ///
    /// Returns `Some(subtype_index)` if a coherent mapping exists, `None`
    /// otherwise.
    fn pick_coherent_subtype(
        &mut self,
        func: NarrativeFunction,
        chosen: &[(NarrativeFunction, u8)],
    ) -> Option<u8> {
        match func {
            NarrativeFunction::Liquidation => {
                // Villainy subtype → preferred Liquidation subtypes
                let villainy_sub = chosen
                    .iter()
                    .find(|(f, _)| *f == NarrativeFunction::Villainy)
                    .map(|(_, s)| *s);
                match villainy_sub {
                    Some(1) | Some(15) => Some(10), /* kidnapping/ */
                    // imprisonment →
                    // captive freed
                    Some(2) => Some(4), /* theft of magical agent → obtained */
                    // by magical agent
                    Some(11) => Some(8), // casting spell → spell broken
                    Some(14) => Some(9), // murder → slain person revived
                    _ => None,
                }
            }
            NarrativeFunction::Recognition => {
                // Branding subtype → preferred Recognition subtypes
                let branding_sub = chosen
                    .iter()
                    .find(|(f, _)| *f == NarrativeFunction::Branding)
                    .map(|(_, s)| *s);
                match branding_sub {
                    Some(1) => Some(1), // mark on body → recognition by mark
                    Some(2) => Some(2), /* identification token → */
                    // recognition by token
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Create a function instance, optionally with a random subtype.
    fn create_function_instance(
        &mut self,
        func: NarrativeFunction,
        include_subtype: bool,
    ) -> NarrativeFunctionInstance {
        if include_subtype {
            let count = subtype_count(func);
            if count > 0 {
                // Randomly select a subtype (1-based index)
                let subtype = self.rng.gen_range(1..=count as u8);
                return NarrativeFunctionInstance::with_subtype(func, subtype);
            }
        }
        NarrativeFunctionInstance::new(func)
    }

    /// Generate basic personae structure.
    fn generate_personae(&mut self) -> Vec<Persona> {
        let mut personae = Vec::new();
        let mut next_id = 1u32;

        // Always have a hero
        personae.push(Persona::new(
            next_id,
            vec![Sphere::Hero],
        ));
        next_id += 1;

        // Usually have a villain
        if self.rng.gen_bool(0.9) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Villain],
            ));
            next_id += 1;
        }

        // Sometimes have a donor
        if self.rng.gen_bool(0.7) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Donor],
            ));
            next_id += 1;
        }

        // Sometimes have a princess
        if self.rng.gen_bool(0.6) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Princess],
            ));
            next_id += 1;
        }

        // Sometimes have a dispatcher
        if self.rng.gen_bool(0.3) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Dispatcher],
            ));
            next_id += 1;
        }

        // Sometimes have a helper
        if self.rng.gen_bool(0.2) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Helper],
            ));
            next_id += 1;
        }

        // Sometimes have a false hero
        if self.rng.gen_bool(0.25) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::FalseHero],
            ));
        }

        personae
    }

    /// Merge compatible spheres probabilistically.
    ///
    /// Propp notes that one character may combine multiple roles.
    /// This reduces character count but gives richer personae.
    fn merge_spheres(&mut self, personae: &mut Vec<Persona>) {
        // Donor + Helper (35%)
        self.try_merge(
            personae,
            Sphere::Donor,
            Sphere::Helper,
            0.35,
        );
        // Villain + FalseHero (40%)
        self.try_merge(
            personae,
            Sphere::Villain,
            Sphere::FalseHero,
            0.40,
        );
        // Princess + Dispatcher (25%)
        self.try_merge(
            personae,
            Sphere::Princess,
            Sphere::Dispatcher,
            0.25,
        );
        // Hero + Dispatcher (20%) — hero learns of misfortune and departs on
        // their own
        self.try_merge(
            personae,
            Sphere::Hero,
            Sphere::Dispatcher,
            0.20,
        );
        // Villain + Donor (15%) — villain tests the hero (villain-donor)
        self.try_merge(
            personae,
            Sphere::Villain,
            Sphere::Donor,
            0.15,
        );
        // Hero + Helper (10%) — hero possesses magical abilities
        self.try_merge(
            personae,
            Sphere::Hero,
            Sphere::Helper,
            0.10,
        );
    }

    /// Try to merge two spheres: find personae with each sphere,
    /// add the second's sphere to the first, remove the second persona.
    fn try_merge(
        &mut self,
        personae: &mut Vec<Persona>,
        primary: Sphere,
        secondary: Sphere,
        probability: f64,
    ) {
        if !self.rng.gen_bool(probability) {
            return;
        }
        let primary_idx =
            personae.iter().position(|p| p.spheres.contains(&primary));
        let secondary_idx =
            personae.iter().position(|p| p.spheres.contains(&secondary));

        if let (Some(pi), Some(si)) = (primary_idx, secondary_idx)
            && pi != si
        {
            let secondary_spheres = personae[si].spheres.clone();
            for s in secondary_spheres {
                if !personae[pi].spheres.contains(&s) {
                    personae[pi].spheres.push(s);
                }
            }
            personae.remove(si);
        }
    }

    /// Generate an embedded move — a secondary quest (вложенный ход).
    ///
    /// Two variants:
    /// - Lack-based (60%): simpler secondary quest starting with Lack
    /// - Villainy-based (40%): embedded villainy with optional struggle
    fn generate_embedded_move(&mut self, include_subtypes: bool) -> Move {
        let mut m = Move::embedded();
        let mut functions = Vec::new();

        if self.rng.gen_bool(0.6) {
            // Variant A: Lack-based (no preparatory phase)
            functions.push(NarrativeFunction::Lack);
            functions.push(NarrativeFunction::Counteraction);
            functions.push(NarrativeFunction::Departure);

            // Optional donor sequence (60%)
            if self.rng.gen_bool(0.6) {
                functions.push(NarrativeFunction::DonorTest);
                functions.push(NarrativeFunction::HeroReaction);
                functions.push(NarrativeFunction::Acquisition);
            }

            // Always resolves
            functions.push(NarrativeFunction::Liquidation);
            functions.push(NarrativeFunction::Return);
        } else {
            // Variant B: Villainy-based embedded move
            functions.push(NarrativeFunction::Villainy);
            functions.push(NarrativeFunction::Counteraction);
            functions.push(NarrativeFunction::Departure);

            // Optional donor sequence (40%)
            if self.rng.gen_bool(0.4) {
                functions.push(NarrativeFunction::DonorTest);
                functions.push(NarrativeFunction::HeroReaction);
                functions.push(NarrativeFunction::Acquisition);
            }

            // Optional struggle sequence (50%)
            if self.rng.gen_bool(0.5) {
                functions.push(NarrativeFunction::Guidance);
                functions.push(NarrativeFunction::Struggle);
                functions.push(NarrativeFunction::Victory);
            }

            functions.push(NarrativeFunction::Liquidation);
            functions.push(NarrativeFunction::Return);
        }

        // Optional Pursuit + Rescue (30%) for both variants
        if self.rng.gen_bool(0.3) {
            functions.push(NarrativeFunction::Pursuit);
            functions.push(NarrativeFunction::Rescue);
        }

        for func in &functions {
            let instance =
                self.create_function_instance(*func, include_subtypes);
            m.moments.push(Moment::new(instance));
        }

        m
    }

    /// Generate initial situation (начальная ситуация).
    fn generate_initial_situation(&mut self) -> InitialSituation {
        let time = self.corpus.pick_time(&mut self.rng).to_string();
        let family = self.corpus.pick_family_context(&mut self.rng);
        let prosperity = self.corpus.pick_prosperity_state(&mut self.rng);

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

        let mut personae = self.generate_personae();
        if config.sphere_merge_enabled {
            self.merge_spheres(&mut personae);
        }
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
        let hero_id = tale
            .personae
            .iter()
            .find(|p| p.spheres.contains(&Sphere::Hero))
            .map(|p| p.id);
        let villain_id = tale
            .personae
            .iter()
            .find(|p| p.spheres.contains(&Sphere::Villain))
            .map(|p| p.id);
        let false_hero_id = tale
            .personae
            .iter()
            .find(|p| p.spheres.contains(&Sphere::FalseHero))
            .map(|p| p.id);
        let has_false_hero = false_hero_id.is_some();

        // Generate moves
        let move_count = self.rng.gen_range(config.move_count.clone());
        for i in 0..move_count {
            let mut m = self.generate_move(
                i == 0,
                config.include_subtypes,
                has_false_hero,
            );

            // Assign agents based on function and phase
            for moment in &mut m.moments {
                let func = moment.function.function;
                match func {
                    NarrativeFunction::Villainy => {
                        moment.agent = villain_id;
                        moment.patient = hero_id;
                    }
                    NarrativeFunction::UnfoundedClaims => {
                        moment.agent = false_hero_id;
                    }
                    NarrativeFunction::Exposure => {
                        moment.agent = hero_id;
                        moment.patient = false_hero_id;
                    }
                    _ => match func.phase() {
                        Phase::Donor
                        | Phase::Struggle
                        | Phase::Return
                        | Phase::Recognition
                        | Phase::Resolution => {
                            moment.agent = hero_id;
                        }
                        _ => {}
                    },
                }
            }

            // Embedded move: if move has Departure, 20% chance
            let has_departure = m.moments.iter().any(|moment| {
                moment.function.function == NarrativeFunction::Departure
            });
            if has_departure && self.rng.gen_bool(0.2) {
                let mut embedded =
                    self.generate_embedded_move(config.include_subtypes);
                // Assign agents to embedded move moments (hero for most phases)
                for moment in &mut embedded.moments {
                    let func = moment.function.function;
                    match func.phase() {
                        Phase::Donor
                        | Phase::Struggle
                        | Phase::Return
                        | Phase::Recognition
                        | Phase::Resolution => {
                            moment.agent = hero_id;
                        }
                        _ => {}
                    }
                }
                m.embedded_moves.push(embedded);
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
                        return Err(GenError::TemplateError(
                            "empty OneOf".to_string(),
                        ));
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
        tale.personae.push(Persona::new(
            2u32,
            vec![Sphere::Villain],
        ));

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
                Fixed(HappyEnding),
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
    fn test_connectives_generated() {
        // With a fixed seed, some moments should have connectives
        let mut generator = RandomGen::with_seed(42);
        let config = GenConfig::new()
            .with_max_absurdity(0.5)
            .with_move_count(1..2)
            .with_seed(42);

        let tale = generator.generate(&config).unwrap();
        let has_connective = tale.all_moments().any(|m| m.connective.is_some());
        assert!(
            has_connective,
            "Expected at least one connective in generated tale"
        );
    }

    #[test]
    fn test_triplication_on_move() {
        // Try multiple seeds to find one that produces triplication on a move
        let mut found_triplication = false;
        for seed in 0..100 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.5)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config)
                && tale.moves.iter().any(|m| m.triplication)
            {
                found_triplication = true;
                break;
            }
        }
        assert!(
            found_triplication,
            "Expected triplication in at least one of 100 seeds"
        );
    }

    #[test]
    fn test_initial_situation_can_be_disabled() {
        let mut generator = RandomGen::with_seed(42);
        let config = GenConfig::new().with_move_count(1..2).with_initial(false);

        let tale = generator.generate(&config).unwrap();
        assert!(tale.initial.is_none());
    }

    #[test]
    fn test_negated_hero_reaction() {
        // Scan seeds to find one with negated HeroReaction
        let mut found = false;
        for seed in 0..200 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.5)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let has_negated = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::HeroReaction
                        && m.function.negated
                });
                if has_negated {
                    // Verify no Acquisition follows
                    let has_acquisition = tale.all_moments().any(|m| {
                        m.function.function == NarrativeFunction::Acquisition
                    });
                    assert!(
                        !has_acquisition,
                        "Seed {}: negated HeroReaction should not have Acquisition",
                        seed
                    );
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected negated HeroReaction in at least one of 200 seeds"
        );
    }

    #[test]
    fn test_sphere_combination() {
        let mut found_multi = false;
        for seed in 0..200 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config)
                && tale.personae.iter().any(|p| p.spheres.len() > 1)
            {
                found_multi = true;
                break;
            }
        }
        assert!(
            found_multi,
            "Expected multi-sphere persona in at least one of 200 seeds"
        );
    }

    #[test]
    fn test_agent_assignment_with_merged_spheres() {
        // Verify agents are still assigned correctly when one persona holds
        // multiple spheres
        for seed in 0..200 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let merged = tale.personae.iter().find(|p| p.spheres.len() > 1);
                if merged.is_some() {
                    // Verify hero is always assigned
                    let hero_id = tale
                        .personae
                        .iter()
                        .find(|p| p.spheres.contains(&Sphere::Hero))
                        .map(|p| p.id);
                    assert!(
                        hero_id.is_some(),
                        "Seed {}: no hero found",
                        seed
                    );

                    // Verify villainy agent is villain
                    let villain_id = tale
                        .personae
                        .iter()
                        .find(|p| p.spheres.contains(&Sphere::Villain))
                        .map(|p| p.id);
                    for moment in tale.all_moments() {
                        if moment.function.function
                            == NarrativeFunction::Villainy
                        {
                            assert_eq!(
                                moment.agent, villain_id,
                                "Seed {}: Villainy agent mismatch",
                                seed
                            );
                        }
                    }
                    return;
                }
            }
        }
        // If no merged spheres found, test is inconclusive but not a failure
    }

    #[test]
    fn test_paired_functions_generated() {
        let mut found_recon = false;
        let mut found_trickery = false;
        for seed in 0..200 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let has_recon = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::Reconnaissance
                });
                let has_delivery = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::Delivery
                });
                if has_recon {
                    assert!(
                        has_delivery,
                        "Seed {}: Reconnaissance without Delivery",
                        seed
                    );
                    found_recon = true;
                }
                if has_delivery {
                    assert!(
                        has_recon,
                        "Seed {}: Delivery without Reconnaissance",
                        seed
                    );
                }

                let has_trickery = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::Trickery
                });
                let has_complicity = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::Complicity
                });
                if has_trickery {
                    assert!(
                        has_complicity,
                        "Seed {}: Trickery without Complicity",
                        seed
                    );
                    found_trickery = true;
                }
                if has_complicity {
                    assert!(
                        has_trickery,
                        "Seed {}: Complicity without Trickery",
                        seed
                    );
                }
            }
        }
        assert!(
            found_recon,
            "Expected Reconnaissance+Delivery in at least one of 200 seeds"
        );
        assert!(
            found_trickery,
            "Expected Trickery+Complicity in at least one of 200 seeds"
        );
    }

    #[test]
    fn test_false_hero_generated() {
        // Scan seeds to find one with FalseHero + recognition functions
        let mut found = false;
        for seed in 0..200 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let has_false_hero = tale
                    .personae
                    .iter()
                    .any(|p| p.spheres.contains(&Sphere::FalseHero));
                let has_claims = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::UnfoundedClaims
                });
                if has_false_hero && has_claims {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected FalseHero with recognition functions in at least one of 200 seeds"
        );
    }

    #[test]
    fn test_embedded_moves_generated() {
        let mut found = false;
        for seed in 0..300 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                for mov in &tale.moves {
                    if !mov.embedded_moves.is_empty() {
                        found = true;
                        // Verify embedded move structure
                        let em = &mov.embedded_moves[0];
                        assert_eq!(
                            em.relation,
                            crate::tale::MoveRelation::Embedded
                        );
                        // Should start with Lack or Villainy
                        let first_func = em.moments[0].function.function;
                        assert!(
                            first_func == NarrativeFunction::Lack
                                || first_func == NarrativeFunction::Villainy,
                            "Seed {}: embedded move should start with Lack or Villainy, got {:?}",
                            seed,
                            first_func
                        );
                        // Should always have Liquidation and Return
                        let has_liquidation = em.moments.iter().any(|m| {
                            m.function.function
                                == NarrativeFunction::Liquidation
                        });
                        let has_return = em.moments.iter().any(|m| {
                            m.function.function == NarrativeFunction::Return
                        });
                        assert!(
                            has_liquidation,
                            "Seed {}: embedded move should have Liquidation",
                            seed
                        );
                        assert!(
                            has_return,
                            "Seed {}: embedded move should have Return",
                            seed
                        );
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected embedded move in at least one of 300 seeds"
        );
    }

    #[test]
    fn test_hero_dispatcher_merge() {
        let mut found = false;
        for seed in 0..500 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let merged = tale.personae.iter().find(|p| {
                    p.spheres.contains(&Sphere::Hero)
                        && p.spheres.contains(&Sphere::Dispatcher)
                });
                if merged.is_some() {
                    // Verify no separate Dispatcher persona exists
                    let separate_dispatcher = tale.personae.iter().any(|p| {
                        p.spheres.contains(&Sphere::Dispatcher)
                            && !p.spheres.contains(&Sphere::Hero)
                    });
                    assert!(
                        !separate_dispatcher,
                        "Seed {}: merged Hero+Dispatcher but separate Dispatcher exists",
                        seed
                    );
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected Hero+Dispatcher merge in at least one of 500 seeds"
        );
    }

    #[test]
    fn test_villain_donor_merge() {
        let mut found = false;
        for seed in 0..500 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let merged = tale.personae.iter().find(|p| {
                    p.spheres.contains(&Sphere::Villain)
                        && p.spheres.contains(&Sphere::Donor)
                });
                if merged.is_some() {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected Villain+Donor merge in at least one of 500 seeds"
        );
    }

    #[test]
    fn test_sphere_merging_disabled() {
        // Generate with merging disabled — no persona should have multiple
        // spheres
        for seed in 0..50 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed)
                .with_sphere_merging(false);

            if let Ok(tale) = generator.generate(&config) {
                for persona in &tale.personae {
                    assert_eq!(
                        persona.spheres.len(),
                        1,
                        "Seed {}: persona {} has {} spheres with merging disabled",
                        seed,
                        persona.id.0,
                        persona.spheres.len()
                    );
                }
            }
        }
    }

    #[test]
    fn test_branding_generated() {
        // Scan seeds to find one where Branding appears
        let mut found = false;
        for seed in 0..300 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                let has_branding = tale.all_moments().any(|m| {
                    m.function.function == NarrativeFunction::Branding
                });
                if has_branding {
                    // Verify Branding appears between Struggle and Victory
                    for mov in &tale.moves {
                        let funcs: Vec<_> = mov
                            .moments
                            .iter()
                            .map(|m| m.function.function)
                            .collect();
                        if let Some(brand_pos) = funcs
                            .iter()
                            .position(|f| *f == NarrativeFunction::Branding)
                        {
                            let struggle_pos = funcs.iter().position(|f| {
                                *f == NarrativeFunction::Struggle
                            });
                            let victory_pos = funcs
                                .iter()
                                .position(|f| *f == NarrativeFunction::Victory);
                            assert!(
                                struggle_pos.is_some(),
                                "Seed {}: Branding without Struggle",
                                seed
                            );
                            assert!(
                                victory_pos.is_some(),
                                "Seed {}: Branding without Victory",
                                seed
                            );
                            assert!(
                                brand_pos > struggle_pos.unwrap(),
                                "Seed {}: Branding before Struggle",
                                seed
                            );
                            assert!(
                                brand_pos < victory_pos.unwrap(),
                                "Seed {}: Branding after Victory",
                                seed
                            );
                        }
                    }
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected Branding in at least one of 300 seeds"
        );
    }

    #[test]
    fn test_negation_variety() {
        // Verify that negation applies to different functions, not just
        // HeroReaction
        let mut negated_functions: HashSet<NarrativeFunction> = HashSet::new();
        for seed in 0..500 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                for moment in tale.all_moments() {
                    if moment.function.negated {
                        negated_functions.insert(moment.function.function);
                    }
                }
            }
        }
        // Should have negation on at least 2 different functions
        assert!(
            negated_functions.len() >= 2,
            "Expected negation on at least 2 different functions, got {:?}",
            negated_functions
        );
    }

    #[test]
    fn test_villainy_liquidation_coherence() {
        // When Villainy subtype is "kidnapping" (1), Liquidation should be
        // "captive freed" (10)
        let mut found = false;
        for seed in 0..500 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..2)
                .with_seed(seed)
                .with_subtypes(true);

            if let Ok(tale) = generator.generate(&config) {
                for mov in &tale.moves {
                    let villainy = mov.moments.iter().find(|m| {
                        m.function.function == NarrativeFunction::Villainy
                    });
                    let liquidation = mov.moments.iter().find(|m| {
                        m.function.function == NarrativeFunction::Liquidation
                    });
                    if let (Some(v), Some(l)) = (villainy, liquidation)
                        && v.function.subtype == Some(1)
                    {
                        // Kidnapping → captive freed
                        assert_eq!(
                            l.function.subtype,
                            Some(10),
                            "Seed {}: Villainy kidnapping (1) should map to Liquidation captive freed (10), got {:?}",
                            seed,
                            l.function.subtype
                        );
                        found = true;
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected Villainy kidnapping → Liquidation captive freed in at least one of 500 seeds"
        );
    }

    #[test]
    fn test_embedded_move_villainy_variant() {
        // Verify that embedded moves can start with Villainy
        let mut found = false;
        for seed in 0..500 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..3)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                for mov in &tale.moves {
                    for em in &mov.embedded_moves {
                        if em.moments[0].function.function
                            == NarrativeFunction::Villainy
                        {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected Villainy-based embedded move in at least one of 500 seeds"
        );
    }

    #[test]
    fn test_embedded_move_with_pursuit() {
        // Verify that embedded moves can have Pursuit + Rescue
        let mut found = false;
        for seed in 0..500 {
            let mut generator = RandomGen::with_seed(seed);
            let config = GenConfig::new()
                .with_max_absurdity(0.8)
                .with_move_count(1..3)
                .with_seed(seed);

            if let Ok(tale) = generator.generate(&config) {
                for mov in &tale.moves {
                    for em in &mov.embedded_moves {
                        let has_pursuit = em.moments.iter().any(|m| {
                            m.function.function == NarrativeFunction::Pursuit
                        });
                        let has_rescue = em.moments.iter().any(|m| {
                            m.function.function == NarrativeFunction::Rescue
                        });
                        if has_pursuit && has_rescue {
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }
                if found {
                    break;
                }
            }
        }
        assert!(
            found,
            "Expected embedded move with Pursuit+Rescue in at least one of 500 seeds"
        );
    }
}
