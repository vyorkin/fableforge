//! Functions of dramatis personae (функции действующих лиц).
//!
//! Based on Vladimir Propp's "Morphology of the Folktale" (1928).
//! The 32 functions represent atomic plot elements in the structure of fairy tales.

use serde::{Deserialize, Serialize};

/// Function of a dramatis persona (функция действующего лица).
/// 32 functions based on Propp's morphology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Function {
    // Preparatory section (подготовительная часть)
    /// α — absentation (отлучка)
    Absentation,
    /// β — interdiction (запрет)
    Interdiction,
    /// γ — violation (нарушение)
    Violation,
    /// δ — reconnaissance (выведывание)
    Reconnaissance,
    /// ε — delivery of information (выдача сведений)
    Delivery,
    /// η — trickery (подвох)
    Trickery,
    /// θ — complicity (пособничество)
    Complicity,

    // Complication (завязка)
    /// A — villainy (вредительство)
    Villainy,
    /// a — lack (недостача)
    Lack,
    /// B — mediation (посредничество)
    Mediation,
    /// C — beginning counteraction (начинающееся противодействие)
    Counteraction,
    /// ↑ — departure (отправка)
    Departure,

    // Donor sequence (даритель)
    /// D — first function of donor (первая функция дарителя)
    DonorTest,
    /// E — hero's reaction (реакция героя)
    HeroReaction,
    /// F — receipt of magical means (получение волшебного средства)
    Acquisition,

    // Struggle sequence (борьба)
    /// G — spatial transference (пространственное перемещение)
    Guidance,
    /// H — struggle (борьба)
    Struggle,
    /// J — branding (клеймение)
    Branding,
    /// I — victory (победа)
    Victory,
    /// K — liquidation of lack (ликвидация недостачи)
    Liquidation,

    // Return (возвращение)
    /// ↓ — return (возвращение)
    Return,
    /// Pr — pursuit (преследование)
    Pursuit,
    /// Rs — rescue (спасение)
    Rescue,

    // Secondary sequence (новые испытания)
    /// O — unrecognized arrival (неузнанное прибытие)
    UnrecognizedArrival,
    /// L — unfounded claims (необоснованные притязания)
    UnfoundedClaims,
    /// M — difficult task (трудная задача)
    DifficultTask,
    /// N — solution (решение)
    Solution,

    // Resolution (развязка)
    /// Q — recognition (узнавание)
    Recognition,
    /// Ex — exposure (обличение)
    Exposure,
    /// T — transfiguration (трансфигурация)
    Transfiguration,
    /// U — punishment (наказание)
    Punishment,
    /// W — wedding (свадьба)
    Wedding,
}

impl Function {
    /// All functions in canonical order.
    pub const ALL: [Function; 32] = [
        Function::Absentation,
        Function::Interdiction,
        Function::Violation,
        Function::Reconnaissance,
        Function::Delivery,
        Function::Trickery,
        Function::Complicity,
        Function::Villainy,
        Function::Lack,
        Function::Mediation,
        Function::Counteraction,
        Function::Departure,
        Function::DonorTest,
        Function::HeroReaction,
        Function::Acquisition,
        Function::Guidance,
        Function::Struggle,
        Function::Branding,
        Function::Victory,
        Function::Liquidation,
        Function::Return,
        Function::Pursuit,
        Function::Rescue,
        Function::UnrecognizedArrival,
        Function::UnfoundedClaims,
        Function::DifficultTask,
        Function::Solution,
        Function::Recognition,
        Function::Exposure,
        Function::Transfiguration,
        Function::Punishment,
        Function::Wedding,
    ];

    /// Propp's symbol for the function (символ Проппа).
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Function::Absentation => "α",
            Function::Interdiction => "β",
            Function::Violation => "γ",
            Function::Reconnaissance => "δ",
            Function::Delivery => "ε",
            Function::Trickery => "η",
            Function::Complicity => "θ",
            Function::Villainy => "A",
            Function::Lack => "a",
            Function::Mediation => "B",
            Function::Counteraction => "C",
            Function::Departure => "↑",
            Function::DonorTest => "D",
            Function::HeroReaction => "E",
            Function::Acquisition => "F",
            Function::Guidance => "G",
            Function::Struggle => "H",
            Function::Branding => "J",
            Function::Victory => "I",
            Function::Liquidation => "K",
            Function::Return => "↓",
            Function::Pursuit => "Pr",
            Function::Rescue => "Rs",
            Function::UnrecognizedArrival => "O",
            Function::UnfoundedClaims => "L",
            Function::DifficultTask => "M",
            Function::Solution => "N",
            Function::Recognition => "Q",
            Function::Exposure => "Ex",
            Function::Transfiguration => "T",
            Function::Punishment => "U",
            Function::Wedding => "W",
        }
    }

    /// Phase of the tale (фаза сказки).
    #[must_use]
    pub const fn phase(&self) -> Phase {
        match self {
            Function::Absentation
            | Function::Interdiction
            | Function::Violation
            | Function::Reconnaissance
            | Function::Delivery
            | Function::Trickery
            | Function::Complicity => Phase::Preparation,

            Function::Villainy
            | Function::Lack
            | Function::Mediation
            | Function::Counteraction
            | Function::Departure => Phase::Complication,

            Function::DonorTest | Function::HeroReaction | Function::Acquisition => Phase::Donor,

            Function::Guidance
            | Function::Struggle
            | Function::Branding
            | Function::Victory
            | Function::Liquidation => Phase::Struggle,

            Function::Return | Function::Pursuit | Function::Rescue => Phase::Return,

            Function::UnrecognizedArrival
            | Function::UnfoundedClaims
            | Function::DifficultTask
            | Function::Solution => Phase::Recognition,

            Function::Recognition
            | Function::Exposure
            | Function::Transfiguration
            | Function::Punishment
            | Function::Wedding => Phase::Resolution,
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
            Function::Villainy | Function::Lack | Function::Liquidation
        )
    }

    /// Canonical index (0-31) in Propp's sequence.
    #[must_use]
    pub const fn index(&self) -> usize {
        match self {
            Function::Absentation => 0,
            Function::Interdiction => 1,
            Function::Violation => 2,
            Function::Reconnaissance => 3,
            Function::Delivery => 4,
            Function::Trickery => 5,
            Function::Complicity => 6,
            Function::Villainy => 7,
            Function::Lack => 8,
            Function::Mediation => 9,
            Function::Counteraction => 10,
            Function::Departure => 11,
            Function::DonorTest => 12,
            Function::HeroReaction => 13,
            Function::Acquisition => 14,
            Function::Guidance => 15,
            Function::Struggle => 16,
            Function::Branding => 17,
            Function::Victory => 18,
            Function::Liquidation => 19,
            Function::Return => 20,
            Function::Pursuit => 21,
            Function::Rescue => 22,
            Function::UnrecognizedArrival => 23,
            Function::UnfoundedClaims => 24,
            Function::DifficultTask => 25,
            Function::Solution => 26,
            Function::Recognition => 27,
            Function::Exposure => 28,
            Function::Transfiguration => 29,
            Function::Punishment => 30,
            Function::Wedding => 31,
        }
    }

    /// Parse a function from its symbol.
    #[must_use]
    pub fn from_symbol(s: &str) -> Option<Self> {
        match s {
            "α" | "alpha" => Some(Function::Absentation),
            "β" | "beta" => Some(Function::Interdiction),
            "γ" | "gamma" => Some(Function::Violation),
            "δ" | "delta" => Some(Function::Reconnaissance),
            "ε" | "epsilon" => Some(Function::Delivery),
            "η" | "eta" => Some(Function::Trickery),
            "θ" | "theta" => Some(Function::Complicity),
            "A" => Some(Function::Villainy),
            "a" => Some(Function::Lack),
            "B" => Some(Function::Mediation),
            "C" => Some(Function::Counteraction),
            "↑" | "^" => Some(Function::Departure),
            "D" => Some(Function::DonorTest),
            "E" => Some(Function::HeroReaction),
            "F" => Some(Function::Acquisition),
            "G" => Some(Function::Guidance),
            "H" => Some(Function::Struggle),
            "J" => Some(Function::Branding),
            "I" => Some(Function::Victory),
            "K" => Some(Function::Liquidation),
            "↓" | "v" => Some(Function::Return),
            "Pr" => Some(Function::Pursuit),
            "Rs" => Some(Function::Rescue),
            "O" => Some(Function::UnrecognizedArrival),
            "L" => Some(Function::UnfoundedClaims),
            "M" => Some(Function::DifficultTask),
            "N" => Some(Function::Solution),
            "Q" => Some(Function::Recognition),
            "Ex" => Some(Function::Exposure),
            "T" => Some(Function::Transfiguration),
            "U" => Some(Function::Punishment),
            "W" => Some(Function::Wedding),
            _ => None,
        }
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
    pub const fn name(&self) -> &'static str {
        match self {
            Phase::Preparation => "Preparation",
            Phase::Complication => "Complication",
            Phase::Donor => "Donor",
            Phase::Struggle => "Struggle",
            Phase::Return => "Return",
            Phase::Recognition => "Recognition",
            Phase::Resolution => "Resolution",
        }
    }
}

/// Instance of a function in a specific tale (экземпляр функции).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionInstance {
    /// The function type.
    pub function: Function,
    /// Subtype index (подвид: A1, A2... D1, D2...).
    pub subtype: Option<u8>,
    /// Negative form (негативная форма).
    pub negated: bool,
}

impl FunctionInstance {
    /// Create a new function instance.
    #[must_use]
    pub const fn new(function: Function) -> Self {
        Self {
            function,
            subtype: None,
            negated: false,
        }
    }

    /// Create a function instance with a subtype.
    #[must_use]
    pub const fn with_subtype(function: Function, subtype: u8) -> Self {
        Self {
            function,
            subtype: Some(subtype),
            negated: false,
        }
    }

    /// Create a negated function instance.
    #[must_use]
    pub const fn negated(function: Function) -> Self {
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
}

impl From<Function> for FunctionInstance {
    fn from(function: Function) -> Self {
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
        assert_eq!(Function::ALL.len(), 32);
    }

    #[test]
    fn test_function_symbols() {
        assert_eq!(Function::Absentation.symbol(), "α");
        assert_eq!(Function::Villainy.symbol(), "A");
        assert_eq!(Function::Departure.symbol(), "↑");
        assert_eq!(Function::Return.symbol(), "↓");
        assert_eq!(Function::Wedding.symbol(), "W");
    }

    #[test]
    fn test_function_phases() {
        assert_eq!(Function::Absentation.phase(), Phase::Preparation);
        assert_eq!(Function::Villainy.phase(), Phase::Complication);
        assert_eq!(Function::DonorTest.phase(), Phase::Donor);
        assert_eq!(Function::Struggle.phase(), Phase::Struggle);
        assert_eq!(Function::Return.phase(), Phase::Return);
        assert_eq!(Function::DifficultTask.phase(), Phase::Recognition);
        assert_eq!(Function::Wedding.phase(), Phase::Resolution);
    }

    #[test]
    fn test_core_functions() {
        assert!(Function::Villainy.is_core());
        assert!(Function::Lack.is_core());
        assert!(Function::Liquidation.is_core());
        assert!(!Function::Absentation.is_core());
        assert!(!Function::Wedding.is_core());
    }

    #[test]
    fn test_function_from_symbol() {
        assert_eq!(Function::from_symbol("α"), Some(Function::Absentation));
        assert_eq!(Function::from_symbol("A"), Some(Function::Villainy));
        assert_eq!(Function::from_symbol("↑"), Some(Function::Departure));
        assert_eq!(Function::from_symbol("Pr"), Some(Function::Pursuit));
        assert_eq!(Function::from_symbol("invalid"), None);
    }

    #[test]
    fn test_function_instance_notation() {
        let f1 = FunctionInstance::new(Function::Villainy);
        assert_eq!(f1.to_notation(), "A");

        let f2 = FunctionInstance::with_subtype(Function::DonorTest, 2);
        assert_eq!(f2.to_notation(), "D²");

        let f3 = FunctionInstance::negated(Function::Mediation);
        assert_eq!(f3.to_notation(), "neg-B");
    }

    #[test]
    fn test_superscript() {
        assert_eq!(superscript(0), "⁰");
        assert_eq!(superscript(1), "¹");
        assert_eq!(superscript(9), "⁹");
        assert_eq!(superscript(12), "¹²");
    }
}
