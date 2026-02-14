//! Narrative functions of dramatis personae (функции действующих лиц).
//!
//! Based on Vladimir Propp's "Morphology of the Folktale" (1928).
//! The 32 narrative functions represent atomic plot elements in the structure of fairy tales.

use serde::{Deserialize, Serialize};

use crate::Lang;

/// Narrative function of a dramatis persona (функция действующего лица).
/// 32 functions based on Propp's morphology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NarrativeFunction {
    // Preparatory section (подготовительная часть)
    /// α — absentation (отлучка)
    Absentation = 0,
    /// β — interdiction (запрет)
    Interdiction = 1,
    /// γ — violation (нарушение)
    Violation = 2,
    /// δ — reconnaissance (выведывание)
    Reconnaissance = 3,
    /// ε — delivery of information (выдача сведений)
    Delivery = 4,
    /// η — trickery (подвох)
    Trickery = 5,
    /// θ — complicity (пособничество)
    Complicity = 6,

    // Complication (завязка)
    /// A — villainy (вредительство)
    Villainy = 7,
    /// a — lack (недостача)
    Lack = 8,
    /// B — mediation (посредничество)
    Mediation = 9,
    /// C — beginning counteraction (начинающееся противодействие)
    Counteraction = 10,
    /// ↑ — departure (отправка)
    Departure = 11,

    // Donor sequence (даритель)
    /// D — first function of donor (первая функция дарителя)
    DonorTest = 12,
    /// E — hero's reaction (реакция героя)
    HeroReaction = 13,
    /// F — receipt of magical means (получение волшебного средства)
    Acquisition = 14,

    // Struggle sequence (борьба)
    /// G — spatial transference (пространственное перемещение)
    Guidance = 15,
    /// H — struggle (борьба)
    Struggle = 16,
    /// J — branding (клеймение)
    Branding = 17,
    /// I — victory (победа)
    Victory = 18,
    /// K — liquidation of lack (ликвидация недостачи)
    Liquidation = 19,

    // Return (возвращение)
    /// ↓ — return (возвращение)
    Return = 20,
    /// Pr — pursuit (преследование)
    Pursuit = 21,
    /// Rs — rescue (спасение)
    Rescue = 22,

    // Secondary sequence (новые испытания)
    /// O — unrecognized arrival (неузнанное прибытие)
    UnrecognizedArrival = 23,
    /// L — unfounded claims (необоснованные притязания)
    UnfoundedClaims = 24,
    /// M — difficult task (трудная задача)
    DifficultTask = 25,
    /// N — solution (решение)
    Solution = 26,

    // Resolution (развязка)
    /// Q — recognition (узнавание)
    Recognition = 27,
    /// Ex — exposure (обличение)
    Exposure = 28,
    /// T — transfiguration (трансфигурация)
    Transfiguration = 29,
    /// U — punishment (наказание)
    Punishment = 30,
    /// W — wedding (свадьба)
    Wedding = 31,
}

impl NarrativeFunction {
    /// All functions in canonical order.
    pub const ALL: [NarrativeFunction; 32] = [
        NarrativeFunction::Absentation,
        NarrativeFunction::Interdiction,
        NarrativeFunction::Violation,
        NarrativeFunction::Reconnaissance,
        NarrativeFunction::Delivery,
        NarrativeFunction::Trickery,
        NarrativeFunction::Complicity,
        NarrativeFunction::Villainy,
        NarrativeFunction::Lack,
        NarrativeFunction::Mediation,
        NarrativeFunction::Counteraction,
        NarrativeFunction::Departure,
        NarrativeFunction::DonorTest,
        NarrativeFunction::HeroReaction,
        NarrativeFunction::Acquisition,
        NarrativeFunction::Guidance,
        NarrativeFunction::Struggle,
        NarrativeFunction::Branding,
        NarrativeFunction::Victory,
        NarrativeFunction::Liquidation,
        NarrativeFunction::Return,
        NarrativeFunction::Pursuit,
        NarrativeFunction::Rescue,
        NarrativeFunction::UnrecognizedArrival,
        NarrativeFunction::UnfoundedClaims,
        NarrativeFunction::DifficultTask,
        NarrativeFunction::Solution,
        NarrativeFunction::Recognition,
        NarrativeFunction::Exposure,
        NarrativeFunction::Transfiguration,
        NarrativeFunction::Punishment,
        NarrativeFunction::Wedding,
    ];

    /// Propp's symbol for the function (символ Проппа).
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            NarrativeFunction::Absentation => "α",
            NarrativeFunction::Interdiction => "β",
            NarrativeFunction::Violation => "γ",
            NarrativeFunction::Reconnaissance => "δ",
            NarrativeFunction::Delivery => "ε",
            NarrativeFunction::Trickery => "η",
            NarrativeFunction::Complicity => "θ",
            NarrativeFunction::Villainy => "A",
            NarrativeFunction::Lack => "a",
            NarrativeFunction::Mediation => "B",
            NarrativeFunction::Counteraction => "C",
            NarrativeFunction::Departure => "↑",
            NarrativeFunction::DonorTest => "D",
            NarrativeFunction::HeroReaction => "E",
            NarrativeFunction::Acquisition => "F",
            NarrativeFunction::Guidance => "G",
            NarrativeFunction::Struggle => "H",
            NarrativeFunction::Branding => "J",
            NarrativeFunction::Victory => "I",
            NarrativeFunction::Liquidation => "K",
            NarrativeFunction::Return => "↓",
            NarrativeFunction::Pursuit => "Pr",
            NarrativeFunction::Rescue => "Rs",
            NarrativeFunction::UnrecognizedArrival => "O",
            NarrativeFunction::UnfoundedClaims => "L",
            NarrativeFunction::DifficultTask => "M",
            NarrativeFunction::Solution => "N",
            NarrativeFunction::Recognition => "Q",
            NarrativeFunction::Exposure => "Ex",
            NarrativeFunction::Transfiguration => "T",
            NarrativeFunction::Punishment => "U",
            NarrativeFunction::Wedding => "W",
        }
    }

    /// Display name of the function.
    #[must_use]
    pub const fn name(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => match self {
                NarrativeFunction::Absentation => "Absentation",
                NarrativeFunction::Interdiction => "Interdiction",
                NarrativeFunction::Violation => "Violation",
                NarrativeFunction::Reconnaissance => "Reconnaissance",
                NarrativeFunction::Delivery => "Delivery",
                NarrativeFunction::Trickery => "Trickery",
                NarrativeFunction::Complicity => "Complicity",
                NarrativeFunction::Villainy => "Villainy",
                NarrativeFunction::Lack => "Lack",
                NarrativeFunction::Mediation => "Mediation",
                NarrativeFunction::Counteraction => "Counteraction",
                NarrativeFunction::Departure => "Departure",
                NarrativeFunction::DonorTest => "Donor Test",
                NarrativeFunction::HeroReaction => "Hero's Reaction",
                NarrativeFunction::Acquisition => "Acquisition",
                NarrativeFunction::Guidance => "Guidance",
                NarrativeFunction::Struggle => "Struggle",
                NarrativeFunction::Branding => "Branding",
                NarrativeFunction::Victory => "Victory",
                NarrativeFunction::Liquidation => "Liquidation",
                NarrativeFunction::Return => "Return",
                NarrativeFunction::Pursuit => "Pursuit",
                NarrativeFunction::Rescue => "Rescue",
                NarrativeFunction::UnrecognizedArrival => "Unrecognized Arrival",
                NarrativeFunction::UnfoundedClaims => "Unfounded Claims",
                NarrativeFunction::DifficultTask => "Difficult Task",
                NarrativeFunction::Solution => "Solution",
                NarrativeFunction::Recognition => "Recognition",
                NarrativeFunction::Exposure => "Exposure",
                NarrativeFunction::Transfiguration => "Transfiguration",
                NarrativeFunction::Punishment => "Punishment",
                NarrativeFunction::Wedding => "Wedding",
            },
            Lang::Ru => match self {
                NarrativeFunction::Absentation => "Отлучка",
                NarrativeFunction::Interdiction => "Запрет",
                NarrativeFunction::Violation => "Нарушение",
                NarrativeFunction::Reconnaissance => "Выведывание",
                NarrativeFunction::Delivery => "Выдача сведений",
                NarrativeFunction::Trickery => "Подвох",
                NarrativeFunction::Complicity => "Пособничество",
                NarrativeFunction::Villainy => "Вредительство",
                NarrativeFunction::Lack => "Недостача",
                NarrativeFunction::Mediation => "Посредничество",
                NarrativeFunction::Counteraction => "Противодействие",
                NarrativeFunction::Departure => "Отправка",
                NarrativeFunction::DonorTest => "Испытание дарителя",
                NarrativeFunction::HeroReaction => "Реакция героя",
                NarrativeFunction::Acquisition => "Получение средства",
                NarrativeFunction::Guidance => "Перемещение",
                NarrativeFunction::Struggle => "Борьба",
                NarrativeFunction::Branding => "Клеймение",
                NarrativeFunction::Victory => "Победа",
                NarrativeFunction::Liquidation => "Ликвидация недостачи",
                NarrativeFunction::Return => "Возвращение",
                NarrativeFunction::Pursuit => "Преследование",
                NarrativeFunction::Rescue => "Спасение",
                NarrativeFunction::UnrecognizedArrival => "Неузнанное прибытие",
                NarrativeFunction::UnfoundedClaims => "Притязания",
                NarrativeFunction::DifficultTask => "Трудная задача",
                NarrativeFunction::Solution => "Решение",
                NarrativeFunction::Recognition => "Узнавание",
                NarrativeFunction::Exposure => "Обличение",
                NarrativeFunction::Transfiguration => "Трансфигурация",
                NarrativeFunction::Punishment => "Наказание",
                NarrativeFunction::Wedding => "Свадьба",
            },
        }
    }

    /// Phase of the tale (фаза сказки).
    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self {
            NarrativeFunction::Absentation
            | NarrativeFunction::Interdiction
            | NarrativeFunction::Violation
            | NarrativeFunction::Reconnaissance
            | NarrativeFunction::Delivery
            | NarrativeFunction::Trickery
            | NarrativeFunction::Complicity => Phase::Preparation,

            NarrativeFunction::Villainy
            | NarrativeFunction::Lack
            | NarrativeFunction::Mediation
            | NarrativeFunction::Counteraction
            | NarrativeFunction::Departure => Phase::Complication,

            NarrativeFunction::DonorTest
            | NarrativeFunction::HeroReaction
            | NarrativeFunction::Acquisition => Phase::Donor,

            NarrativeFunction::Guidance
            | NarrativeFunction::Struggle
            | NarrativeFunction::Branding
            | NarrativeFunction::Victory
            | NarrativeFunction::Liquidation => Phase::Struggle,

            NarrativeFunction::Return
            | NarrativeFunction::Pursuit
            | NarrativeFunction::Rescue => Phase::Return,

            NarrativeFunction::UnrecognizedArrival
            | NarrativeFunction::UnfoundedClaims
            | NarrativeFunction::DifficultTask
            | NarrativeFunction::Solution => Phase::Recognition,

            NarrativeFunction::Recognition
            | NarrativeFunction::Exposure
            | NarrativeFunction::Transfiguration
            | NarrativeFunction::Punishment
            | NarrativeFunction::Wedding => Phase::Resolution,
        }
    }

    /// Whether function is mandatory for a minimal move (обязательная функция).
    ///
    /// According to Propp, a minimal move requires at least villainy (A) or lack (a),
    /// and liquidation (K) to form a complete narrative arc.
    #[must_use]
    pub const fn is_core(&self) -> bool {
        matches!(
            self,
            NarrativeFunction::Villainy | NarrativeFunction::Lack | NarrativeFunction::Liquidation
        )
    }

    /// Canonical index (0-31) in Propp's sequence.
    #[must_use]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    /// Parse a function from its symbol.
    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        match s {
            "α" | "alpha" => Some(NarrativeFunction::Absentation),
            "β" | "beta" => Some(NarrativeFunction::Interdiction),
            "γ" | "gamma" => Some(NarrativeFunction::Violation),
            "δ" | "delta" => Some(NarrativeFunction::Reconnaissance),
            "ε" | "epsilon" => Some(NarrativeFunction::Delivery),
            "η" | "eta" => Some(NarrativeFunction::Trickery),
            "θ" | "theta" => Some(NarrativeFunction::Complicity),
            "A" => Some(NarrativeFunction::Villainy),
            "a" => Some(NarrativeFunction::Lack),
            "B" => Some(NarrativeFunction::Mediation),
            "C" => Some(NarrativeFunction::Counteraction),
            "↑" | "^" => Some(NarrativeFunction::Departure),
            "D" => Some(NarrativeFunction::DonorTest),
            "E" => Some(NarrativeFunction::HeroReaction),
            "F" => Some(NarrativeFunction::Acquisition),
            "G" => Some(NarrativeFunction::Guidance),
            "H" => Some(NarrativeFunction::Struggle),
            "J" => Some(NarrativeFunction::Branding),
            "I" => Some(NarrativeFunction::Victory),
            "K" => Some(NarrativeFunction::Liquidation),
            "↓" | "v" => Some(NarrativeFunction::Return),
            "Pr" => Some(NarrativeFunction::Pursuit),
            "Rs" => Some(NarrativeFunction::Rescue),
            "O" => Some(NarrativeFunction::UnrecognizedArrival),
            "L" => Some(NarrativeFunction::UnfoundedClaims),
            "M" => Some(NarrativeFunction::DifficultTask),
            "N" => Some(NarrativeFunction::Solution),
            "Q" => Some(NarrativeFunction::Recognition),
            "Ex" => Some(NarrativeFunction::Exposure),
            "T" => Some(NarrativeFunction::Transfiguration),
            "U" => Some(NarrativeFunction::Punishment),
            "W" => Some(NarrativeFunction::Wedding),
            _ => None,
        }
    }
}

impl From<NarrativeFunction> for u8 {
    fn from(f: NarrativeFunction) -> Self {
        f as u8
    }
}

impl From<NarrativeFunction> for usize {
    fn from(f: NarrativeFunction) -> Self {
        f as usize
    }
}

/// Phase of the tale (фаза сказки).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    /// α-θ (подготовительная часть)
    Preparation,
    /// A-↑ (завязка)
    Complication,
    /// D-F (даритель)
    Donor,
    /// G-K (борьба)
    Struggle,
    /// ↓-Rs (возвращение)
    Return,
    /// O-N (узнавание)
    Recognition,
    /// Q-W (развязка)
    Resolution,
}

impl Phase {
    /// All phases in order.
    pub const ALL: [Phase; 7] = [
        Phase::Preparation,
        Phase::Complication,
        Phase::Donor,
        Phase::Struggle,
        Phase::Return,
        Phase::Recognition,
        Phase::Resolution,
    ];

    /// Display name of the phase.
    #[must_use]
    pub const fn name(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => match self {
                Phase::Preparation => "Preparation",
                Phase::Complication => "Complication",
                Phase::Donor => "Donor",
                Phase::Struggle => "Struggle",
                Phase::Return => "Return",
                Phase::Recognition => "Recognition",
                Phase::Resolution => "Resolution",
            },
            Lang::Ru => match self {
                Phase::Preparation => "Подготовительная часть",
                Phase::Complication => "Завязка",
                Phase::Donor => "Даритель",
                Phase::Struggle => "Борьба",
                Phase::Return => "Возвращение",
                Phase::Recognition => "Узнавание",
                Phase::Resolution => "Развязка",
            },
        }
    }
}

/// Instance of a narrative function in a specific tale (экземпляр функции).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NarrativeFunctionInstance {
    /// The function type.
    pub function: NarrativeFunction,
    /// Subtype index (подвид: A1, A2... D1, D2...).
    pub subtype: Option<u8>,
    /// Negative form (негативная форма).
    pub negated: bool,
}

impl NarrativeFunctionInstance {
    /// Create a new function instance.
    #[must_use]
    pub const fn new(function: NarrativeFunction) -> Self {
        Self {
            function,
            subtype: None,
            negated: false,
        }
    }

    /// Create a function instance with a subtype.
    #[must_use]
    pub const fn with_subtype(function: NarrativeFunction, subtype: u8) -> Self {
        Self {
            function,
            subtype: Some(subtype),
            negated: false,
        }
    }

    /// Create a negated function instance.
    #[must_use]
    pub const fn negated(function: NarrativeFunction) -> Self {
        Self {
            function,
            subtype: None,
            negated: true,
        }
    }

    /// Format as Propp notation (e.g., "A¹", "D²", "neg-B").
    #[must_use]
    pub fn to_notation(&self) -> String {
        let mut s = String::new();
        if self.negated {
            s.push_str("neg-");
        }
        s.push_str(self.function.symbol());
        if let Some(sub) = self.subtype {
            s.push_str(&superscript(sub));
        }
        s
    }

    /// Get detailed description including subtype.
    ///
    /// Returns a tuple of (function name, optional subtype description).
    #[must_use]
    pub fn description(&self, lang: Lang) -> (String, Option<String>) {
        let name = self.function.name(lang).to_string();
        let subtype_desc = self.subtype.and_then(|idx| {
            crate::subtype::subtype(self.function, idx).map(|info| info.name(lang).to_string())
        });
        (name, subtype_desc)
    }

    /// Get full description as a single string.
    ///
    /// Format: "Function name — subtype description" or just "Function name".
    /// Negated functions are prefixed with "Невыполнение:" (Ru) or "Failure:" (En).
    #[must_use]
    pub fn full_description(&self, lang: Lang) -> String {
        let (name, subtype_desc) = self.description(lang);
        let base = match subtype_desc {
            Some(desc) => format!("{} — {}", name, desc),
            None => name,
        };
        if self.negated {
            let prefix = match lang {
                Lang::Ru => "Невыполнение",
                Lang::En => "Failure",
            };
            format!("{}: {}", prefix, base)
        } else {
            base
        }
    }
}

impl From<NarrativeFunction> for NarrativeFunctionInstance {
    fn from(function: NarrativeFunction) -> Self {
        Self::new(function)
    }
}

/// Convert a digit to superscript.
fn superscript(n: u8) -> String {
    const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    if n < 10 {
        SUPERSCRIPTS[n as usize].to_string()
    } else {
        n.to_string()
            .chars()
            .map(|c| {
                c.to_digit(10)
                    .map(|d| SUPERSCRIPTS[d as usize])
                    .unwrap_or(c)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_count() {
        assert_eq!(NarrativeFunction::ALL.len(), 32);
    }

    #[test]
    fn test_function_symbols() {
        assert_eq!(NarrativeFunction::Absentation.symbol(), "α");
        assert_eq!(NarrativeFunction::Villainy.symbol(), "A");
        assert_eq!(NarrativeFunction::Departure.symbol(), "↑");
        assert_eq!(NarrativeFunction::Return.symbol(), "↓");
        assert_eq!(NarrativeFunction::Wedding.symbol(), "W");
    }

    #[test]
    fn test_function_names() {
        assert_eq!(NarrativeFunction::Villainy.name(Lang::En), "Villainy");
        assert_eq!(NarrativeFunction::Villainy.name(Lang::Ru), "Вредительство");
    }

    #[test]
    fn test_function_phases() {
        assert_eq!(NarrativeFunction::Absentation.phase(), Phase::Preparation);
        assert_eq!(NarrativeFunction::Villainy.phase(), Phase::Complication);
        assert_eq!(NarrativeFunction::DonorTest.phase(), Phase::Donor);
        assert_eq!(NarrativeFunction::Struggle.phase(), Phase::Struggle);
        assert_eq!(NarrativeFunction::Return.phase(), Phase::Return);
        assert_eq!(NarrativeFunction::DifficultTask.phase(), Phase::Recognition);
        assert_eq!(NarrativeFunction::Wedding.phase(), Phase::Resolution);
    }

    #[test]
    fn test_core_functions() {
        assert!(NarrativeFunction::Villainy.is_core());
        assert!(NarrativeFunction::Lack.is_core());
        assert!(NarrativeFunction::Liquidation.is_core());
        assert!(!NarrativeFunction::Absentation.is_core());
        assert!(!NarrativeFunction::Wedding.is_core());
    }

    #[test]
    fn test_function_index() {
        assert_eq!(NarrativeFunction::Absentation.index(), 0);
        assert_eq!(NarrativeFunction::Wedding.index(), 31);
        // Verify repr(u8) works correctly
        assert_eq!(u8::from(NarrativeFunction::Villainy), 7);
        assert_eq!(usize::from(NarrativeFunction::Wedding), 31);
    }

    #[test]
    fn test_function_from_symbol() {
        assert_eq!(
            NarrativeFunction::from_symbol("α"),
            Some(NarrativeFunction::Absentation)
        );
        assert_eq!(
            NarrativeFunction::from_symbol("A"),
            Some(NarrativeFunction::Villainy)
        );
        assert_eq!(
            NarrativeFunction::from_symbol("↑"),
            Some(NarrativeFunction::Departure)
        );
        assert_eq!(
            NarrativeFunction::from_symbol("Pr"),
            Some(NarrativeFunction::Pursuit)
        );
        assert_eq!(NarrativeFunction::from_symbol("invalid"), None);
    }

    #[test]
    fn test_function_instance_notation() {
        let f1 = NarrativeFunctionInstance::new(NarrativeFunction::Villainy);
        assert_eq!(f1.to_notation(), "A");

        let f2 = NarrativeFunctionInstance::with_subtype(NarrativeFunction::DonorTest, 2);
        assert_eq!(f2.to_notation(), "D²");

        let f3 = NarrativeFunctionInstance::negated(NarrativeFunction::Mediation);
        assert_eq!(f3.to_notation(), "neg-B");
    }

    #[test]
    fn test_superscript() {
        assert_eq!(superscript(0), "⁰");
        assert_eq!(superscript(1), "¹");
        assert_eq!(superscript(9), "⁹");
        assert_eq!(superscript(12), "¹²");
    }

    #[test]
    fn test_phase_names() {
        assert_eq!(Phase::Preparation.name(Lang::En), "Preparation");
        assert_eq!(Phase::Preparation.name(Lang::Ru), "Подготовительная часть");
    }

    #[test]
    fn test_function_instance_description() {
        let f = NarrativeFunctionInstance::new(NarrativeFunction::Villainy);
        let (name, subtype) = f.description(Lang::Ru);
        assert_eq!(name, "Вредительство");
        assert!(subtype.is_none());
    }

    #[test]
    fn test_function_instance_description_with_subtype() {
        let f = NarrativeFunctionInstance::with_subtype(NarrativeFunction::Villainy, 1);
        let (name, subtype) = f.description(Lang::Ru);
        assert_eq!(name, "Вредительство");
        assert_eq!(subtype, Some("Похищение человека".to_string()));
    }

    #[test]
    fn test_function_instance_full_description() {
        let f1 = NarrativeFunctionInstance::new(NarrativeFunction::DonorTest);
        assert_eq!(f1.full_description(Lang::Ru), "Испытание дарителя");

        let f2 = NarrativeFunctionInstance::with_subtype(NarrativeFunction::DonorTest, 8);
        assert_eq!(f2.full_description(Lang::Ru), "Испытание дарителя — Загадка");
        assert_eq!(f2.full_description(Lang::En), "Donor Test — Riddle");
    }

    #[test]
    fn test_negated_full_description() {
        let f = NarrativeFunctionInstance::negated(NarrativeFunction::HeroReaction);
        assert!(f.full_description(Lang::Ru).starts_with("Невыполнение:"));
        assert!(f.full_description(Lang::En).starts_with("Failure:"));
        assert!(f.full_description(Lang::Ru).contains("Реакция героя"));
        assert!(f.full_description(Lang::En).contains("Hero's Reaction"));
    }
}
