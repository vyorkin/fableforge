//! Tale structure (структура сказки).
//!
//! A fairy tale consists of an initial situation followed by one or more moves.
//! Each move is a sequence of moments (functions with participants).

use serde::{Deserialize, Serialize};

use crate::connective::Connective;
use crate::dramatis::{Persona, PersonaId};
use crate::function::FunctionInstance;

/// Unique identifier for magical means within a tale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MagicalMeansId(pub u32);

impl MagicalMeansId {
    /// Create a new magical means ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<u32> for MagicalMeansId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// A complete tale (сказка).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tale {
    /// Initial situation (начальная ситуация).
    pub initial: InitialSituation,
    /// One or more moves (ходы).
    pub moves: Vec<Move>,
    /// All personae in the tale.
    pub personae: Vec<Persona>,
}

impl Tale {
    /// Create a new tale.
    #[must_use]
    pub fn new(initial: InitialSituation) -> Self {
        Self {
            initial,
            moves: Vec::new(),
            personae: Vec::new(),
        }
    }

    /// Add a move to the tale.
    pub fn add_move(&mut self, m: Move) {
        self.moves.push(m);
    }

    /// Add a persona to the tale.
    pub fn add_persona(&mut self, persona: Persona) {
        self.personae.push(persona);
    }

    /// Get a persona by ID.
    #[must_use]
    pub fn get_persona(&self, id: PersonaId) -> Option<&Persona> {
        self.personae.iter().find(|p| p.id == id)
    }

    /// Get all moments across all moves.
    pub fn all_moments(&self) -> impl Iterator<Item = &Moment> {
        self.moves.iter().flat_map(|m| m.moments.iter())
    }

    /// Count total functions in the tale.
    #[must_use]
    pub fn function_count(&self) -> usize {
        self.moves.iter().map(|m| m.moments.len()).sum()
    }
}

/// Initial situation (начальная ситуация, "i" in Propp's notation).
///
/// The exposition establishes the family composition, setting, and state
/// of prosperity before any functions occur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitialSituation {
    /// Family composition (состав семьи).
    pub family: Vec<PersonaId>,
    /// Place and time (место и время).
    pub setting: Setting,
    /// State of prosperity (благополучие/бедность).
    pub prosperity: Prosperity,
}

impl InitialSituation {
    /// Create a new initial situation.
    #[must_use]
    pub fn new(setting: Setting, prosperity: Prosperity) -> Self {
        Self {
            family: Vec::new(),
            setting,
            prosperity,
        }
    }

    /// Add a family member.
    pub fn add_family_member(&mut self, id: PersonaId) {
        self.family.push(id);
    }
}

impl Default for InitialSituation {
    fn default() -> Self {
        Self {
            family: Vec::new(),
            setting: Setting::default(),
            prosperity: Prosperity::Normal,
        }
    }
}

/// Setting (место и время действия).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setting {
    /// Place description.
    pub place: Option<String>,
    /// Time description (often vague: "once upon a time").
    pub time: Option<String>,
}

impl Setting {
    /// Create a new setting.
    #[must_use]
    pub fn new(place: impl Into<String>) -> Self {
        Self {
            place: Some(place.into()),
            time: None,
        }
    }

    /// Set the time.
    #[must_use]
    pub fn with_time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            place: None,
            time: Some("Жили-были".to_string()),
        }
    }
}

/// State of prosperity (состояние благополучия).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Prosperity {
    /// Poverty (бедность).
    Poor,
    /// Normal state (обычное состояние).
    Normal,
    /// Prosperity (благополучие).
    Prosperous,
    /// Royal abundance (царское изобилие).
    Royal,
}

impl Prosperity {
    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Prosperity::Poor => "Poor",
            Prosperity::Normal => "Normal",
            Prosperity::Prosperous => "Prosperous",
            Prosperity::Royal => "Royal",
        }
    }
}

/// Move (ход) — a sequence from complication to resolution.
///
/// A tale can have one or more moves. Multiple moves can be:
/// - Sequential (one after another)
/// - Embedded (nested within another move)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Move {
    /// Sequence of moments.
    pub moments: Vec<Moment>,
    /// Relation to previous move (связь с предыдущим ходом).
    pub relation: MoveRelation,
}

impl Move {
    /// Create a new initial move.
    #[must_use]
    pub fn new() -> Self {
        Self {
            moments: Vec::new(),
            relation: MoveRelation::Initial,
        }
    }

    /// Create a continuation move.
    #[must_use]
    pub fn continuation() -> Self {
        Self {
            moments: Vec::new(),
            relation: MoveRelation::Continuation,
        }
    }

    /// Create an embedded move.
    #[must_use]
    pub fn embedded() -> Self {
        Self {
            moments: Vec::new(),
            relation: MoveRelation::Embedded,
        }
    }

    /// Add a moment to the move.
    pub fn add_moment(&mut self, moment: Moment) {
        self.moments.push(moment);
    }

    /// Add a function as a simple moment.
    pub fn add_function(&mut self, function: impl Into<FunctionInstance>) {
        self.moments.push(Moment::new(function.into()));
    }
}

impl Default for Move {
    fn default() -> Self {
        Self::new()
    }
}

/// Relation between moves (связь между ходами).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveRelation {
    /// First move (первый ход).
    Initial,
    /// Continuation after Rs (продолжение).
    Continuation,
    /// Embedded/nested move (вложенный ход).
    Embedded,
}

/// Moment (момент) — a function with participants and connective.
///
/// A moment represents one function being performed, along with
/// information about who performs it, who is affected, and any
/// magical means or connectives involved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Moment {
    /// The function instance.
    pub function: FunctionInstance,
    /// Who performs the function (кто выполняет).
    pub agent: Option<PersonaId>,
    /// Whom it affects (на кого направлено).
    pub patient: Option<PersonaId>,
    /// Magical means involved (волшебное средство).
    pub magical_means: Option<MagicalMeans>,
    /// Connective to previous moment (связка).
    pub connective: Option<Connective>,
}

impl Moment {
    /// Create a new moment.
    #[must_use]
    pub fn new(function: FunctionInstance) -> Self {
        Self {
            function,
            agent: None,
            patient: None,
            magical_means: None,
            connective: None,
        }
    }

    /// Set the agent.
    #[must_use]
    pub fn with_agent(mut self, agent: PersonaId) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Set the patient.
    #[must_use]
    pub fn with_patient(mut self, patient: PersonaId) -> Self {
        self.patient = Some(patient);
        self
    }

    /// Set the magical means.
    #[must_use]
    pub fn with_magical_means(mut self, means: MagicalMeans) -> Self {
        self.magical_means = Some(means);
        self
    }

    /// Set the connective.
    #[must_use]
    pub fn with_connective(mut self, connective: Connective) -> Self {
        self.connective = Some(connective);
        self
    }
}

/// Magical means (волшебное средство) — what the hero receives from the donor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagicalMeans {
    /// Unique identifier.
    pub id: MagicalMeansId,
    /// Kind of magical means.
    pub kind: MagicalMeansKind,
    /// Optional name.
    pub name: Option<String>,
}

impl MagicalMeans {
    /// Create new magical means.
    #[must_use]
    pub fn new(id: impl Into<MagicalMeansId>, kind: MagicalMeansKind) -> Self {
        Self {
            id: id.into(),
            kind,
            name: None,
        }
    }

    /// Set the name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Kind of magical means (вид волшебного средства).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MagicalMeansKind {
    /// Horse, wolf, eagle (конь, волк, орёл).
    Animal,
    /// Sword, ring, ball of thread (меч, кольцо, клубок).
    Object,
    /// Living water, rejuvenating apples (живая вода, молодильные яблоки).
    Substance,
    /// Ability to transform (способность превращаться).
    Quality,
}

impl MagicalMeansKind {
    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            MagicalMeansKind::Animal => "Animal",
            MagicalMeansKind::Object => "Object",
            MagicalMeansKind::Substance => "Substance",
            MagicalMeansKind::Quality => "Quality",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::Function;

    #[test]
    fn test_tale_creation() {
        let initial = InitialSituation::default();
        let tale = Tale::new(initial);
        assert!(tale.moves.is_empty());
        assert!(tale.personae.is_empty());
    }

    #[test]
    fn test_move_creation() {
        let mut m = Move::new();
        m.add_function(Function::Villainy);
        m.add_function(Function::Departure);
        assert_eq!(m.moments.len(), 2);
    }

    #[test]
    fn test_moment_builder() {
        let moment = Moment::new(Function::Villainy.into())
            .with_agent(PersonaId::new(1))
            .with_patient(PersonaId::new(2));

        assert_eq!(moment.agent, Some(PersonaId::new(1)));
        assert_eq!(moment.patient, Some(PersonaId::new(2)));
    }

    #[test]
    fn test_magical_means() {
        let means = MagicalMeans::new(1u32, MagicalMeansKind::Object).with_name("Magic Sword");
        assert_eq!(means.kind, MagicalMeansKind::Object);
        assert_eq!(means.name.as_deref(), Some("Magic Sword"));
    }

    #[test]
    fn test_initial_situation() {
        let mut initial = InitialSituation::new(
            Setting::new("In a certain kingdom").with_time("Long, long ago"),
            Prosperity::Royal,
        );
        initial.add_family_member(PersonaId::new(1));
        initial.add_family_member(PersonaId::new(2));

        assert_eq!(initial.family.len(), 2);
        assert_eq!(initial.prosperity, Prosperity::Royal);
    }

    #[test]
    fn test_tale_all_moments() {
        let mut tale = Tale::new(InitialSituation::default());

        let mut m1 = Move::new();
        m1.add_function(Function::Villainy);
        m1.add_function(Function::Departure);
        tale.add_move(m1);

        let mut m2 = Move::continuation();
        m2.add_function(Function::Return);
        tale.add_move(m2);

        assert_eq!(tale.function_count(), 3);
        assert_eq!(tale.all_moments().count(), 3);
    }
}
