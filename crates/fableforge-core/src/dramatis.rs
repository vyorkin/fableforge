//! Dramatis personae (действующие лица).
//!
//! Characters in fairy tales are defined by their spheres of action,
//! not by their individual traits. One character may fulfill multiple spheres,
//! or one sphere may be distributed among several characters.

use serde::{Deserialize, Serialize};

/// Unique identifier for a persona within a tale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonaId(pub u32);

impl PersonaId {
    /// Create a new persona ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<u32> for PersonaId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

/// Sphere of action (сфера действия / круг действий).
/// One character may combine multiple spheres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sphere {
    /// Hero-seeker or hero-victim (герой-искатель или герой-жертва).
    Hero,
    /// Villain/antagonist (вредитель/антагонист).
    Villain,
    /// Donor/provider (даритель/снабдитель).
    Donor,
    /// Magical helper (волшебный помощник).
    Helper,
    /// Princess and her father (царевна и её отец).
    Princess,
    /// Dispatcher (отправитель).
    Dispatcher,
    /// False hero (ложный герой).
    FalseHero,
}

impl Sphere {
    /// All spheres of action.
    pub const ALL: [Sphere; 7] = [
        Sphere::Hero,
        Sphere::Villain,
        Sphere::Donor,
        Sphere::Helper,
        Sphere::Princess,
        Sphere::Dispatcher,
        Sphere::FalseHero,
    ];

    /// Display name of the sphere.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Sphere::Hero => "Hero",
            Sphere::Villain => "Villain",
            Sphere::Donor => "Donor",
            Sphere::Helper => "Helper",
            Sphere::Princess => "Princess",
            Sphere::Dispatcher => "Dispatcher",
            Sphere::FalseHero => "False Hero",
        }
    }

    /// Russian name of the sphere.
    #[must_use]
    pub const fn name_ru(&self) -> &'static str {
        match self {
            Sphere::Hero => "Герой",
            Sphere::Villain => "Вредитель",
            Sphere::Donor => "Даритель",
            Sphere::Helper => "Помощник",
            Sphere::Princess => "Царевна",
            Sphere::Dispatcher => "Отправитель",
            Sphere::FalseHero => "Ложный герой",
        }
    }
}

/// Nature of a character (природа персонажа).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nature {
    /// Human being (человек).
    Human,
    /// Animal (животное).
    Animal,
    /// Magical creature (волшебное существо).
    Magical,
    /// Animated object (одушевлённый предмет).
    Object,
}

impl Nature {
    /// Display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Nature::Human => "Human",
            Nature::Animal => "Animal",
            Nature::Magical => "Magical",
            Nature::Object => "Object",
        }
    }
}

/// Character attributes (атрибуты персонажа).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    /// Character's name.
    pub name: Option<String>,
    /// Constant epithet (постоянный эпитет), e.g., "fair", "wise".
    pub epithet: Option<String>,
    /// Nature of the character.
    pub nature: Option<Nature>,
    /// External appearance (внешний облик).
    pub appearance: Option<String>,
}

impl Attributes {
    /// Create empty attributes.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            epithet: None,
            nature: None,
            appearance: None,
        }
    }

    /// Create attributes with a name.
    #[must_use]
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Set the epithet.
    #[must_use]
    pub fn epithet(mut self, epithet: impl Into<String>) -> Self {
        self.epithet = Some(epithet.into());
        self
    }

    /// Set the nature.
    #[must_use]
    pub fn nature(mut self, nature: Nature) -> Self {
        self.nature = Some(nature);
        self
    }

    /// Set the appearance.
    #[must_use]
    pub fn appearance(mut self, appearance: impl Into<String>) -> Self {
        self.appearance = Some(appearance.into());
        self
    }
}

/// A character in the tale (персонаж сказки).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Persona {
    /// Unique identifier within the tale.
    pub id: PersonaId,
    /// Spheres of action (сферы действия).
    pub spheres: Vec<Sphere>,
    /// Character attributes (атрибуты).
    pub attributes: Attributes,
}

impl Persona {
    /// Create a new persona with given ID and spheres.
    #[must_use]
    pub fn new(id: impl Into<PersonaId>, spheres: Vec<Sphere>) -> Self {
        Self {
            id: id.into(),
            spheres,
            attributes: Attributes::default(),
        }
    }

    /// Create a hero persona.
    #[must_use]
    pub fn hero(id: impl Into<PersonaId>) -> Self {
        Self::new(id, vec![Sphere::Hero])
    }

    /// Create a villain persona.
    #[must_use]
    pub fn villain(id: impl Into<PersonaId>) -> Self {
        Self::new(id, vec![Sphere::Villain])
    }

    /// Create a donor persona.
    #[must_use]
    pub fn donor(id: impl Into<PersonaId>) -> Self {
        Self::new(id, vec![Sphere::Donor])
    }

    /// Create a helper persona.
    #[must_use]
    pub fn helper(id: impl Into<PersonaId>) -> Self {
        Self::new(id, vec![Sphere::Helper])
    }

    /// Create a princess persona.
    #[must_use]
    pub fn princess(id: impl Into<PersonaId>) -> Self {
        Self::new(id, vec![Sphere::Princess])
    }

    /// Set attributes.
    #[must_use]
    pub fn with_attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }

    /// Check if this persona fulfills a given sphere.
    #[must_use]
    pub fn has_sphere(&self, sphere: Sphere) -> bool {
        self.spheres.contains(&sphere)
    }

    /// Check if this is the hero.
    #[must_use]
    pub fn is_hero(&self) -> bool {
        self.has_sphere(Sphere::Hero)
    }

    /// Check if this is a villain.
    #[must_use]
    pub fn is_villain(&self) -> bool {
        self.has_sphere(Sphere::Villain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_count() {
        assert_eq!(Sphere::ALL.len(), 7);
    }

    #[test]
    fn test_persona_creation() {
        let hero = Persona::hero(1u32);
        assert!(hero.is_hero());
        assert!(!hero.is_villain());
        assert_eq!(hero.spheres.len(), 1);
    }

    #[test]
    fn test_persona_with_multiple_spheres() {
        let persona = Persona::new(1u32, vec![Sphere::Hero, Sphere::Dispatcher]);
        assert!(persona.has_sphere(Sphere::Hero));
        assert!(persona.has_sphere(Sphere::Dispatcher));
        assert!(!persona.has_sphere(Sphere::Villain));
    }

    #[test]
    fn test_attributes_builder() {
        let attrs = Attributes::with_name("Ivan")
            .epithet("brave")
            .nature(Nature::Human)
            .appearance("young man");

        assert_eq!(attrs.name.as_deref(), Some("Ivan"));
        assert_eq!(attrs.epithet.as_deref(), Some("brave"));
        assert_eq!(attrs.nature, Some(Nature::Human));
        assert_eq!(attrs.appearance.as_deref(), Some("young man"));
    }

    #[test]
    fn test_persona_with_attributes() {
        let hero = Persona::hero(1u32).with_attributes(Attributes::with_name("Ivan the Fool"));
        assert_eq!(hero.attributes.name.as_deref(), Some("Ivan the Fool"));
    }
}
