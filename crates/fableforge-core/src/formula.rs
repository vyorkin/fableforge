//! Formula representation (формульное представление).
//!
//! A formula is a symbolic representation of a tale's structure
//! using Propp's notation. Examples from Appendix II:
//! - "β³γ³A¹B⁴C↑" (Tale 50)
//! - "αβγ A B ↑ D E F H I K ↓ W" (classic structure)

use std::fmt::{self, Display, Formatter, Write as FmtWrite};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use winnow::ascii::multispace0;
use winnow::combinator::{alt, delimited, opt, repeat};
use winnow::prelude::*;
use winnow::token::{any, one_of};
use winnow::ModalResult;

use crate::function::{NarrativeFunction, NarrativeFunctionInstance};
use crate::tale::{Move, Tale};

/// Tale formula in Propp's notation (формула сказки).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Formula {
    elements: Vec<FormulaElement>,
}

impl Formula {
    /// Create an empty formula.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a formula from elements.
    #[must_use]
    pub fn from_elements(elements: Vec<FormulaElement>) -> Self {
        Self { elements }
    }

    /// Parse formula from string using Propp's notation.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if the string cannot be parsed.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        parse_formula
            .parse(input)
            .map_err(|e| ParseError::InvalidSyntax(e.to_string()))
    }

    /// Format as plain text using Propp's symbols.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            write_element_text(&mut out, elem);
        }
        out
    }

    /// Format as LaTeX (for typesetting like in Propp's book).
    #[must_use]
    pub fn to_latex(&self) -> String {
        let mut out = String::from("$");
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                out.push_str(" \\; ");
            }
            write_element_latex(&mut out, elem);
        }
        out.push('$');
        out
    }

    /// Get all elements.
    #[must_use]
    pub fn elements(&self) -> &[FormulaElement] {
        &self.elements
    }

    /// Add an element.
    pub fn push(&mut self, element: FormulaElement) {
        self.elements.push(element);
    }

    /// Add a function.
    pub fn push_function(&mut self, function: impl Into<NarrativeFunctionInstance>) {
        self.elements
            .push(FormulaElement::Function(function.into()));
    }

    /// Add a move break.
    pub fn push_move_break(&mut self) {
        self.elements.push(FormulaElement::MoveBreak);
    }

    /// Convert from a tale.
    #[must_use]
    pub fn from_tale(tale: &Tale) -> Self {
        let mut formula = Self::new();
        for (i, m) in tale.moves.iter().enumerate() {
            if i > 0 {
                formula.push_move_break();
            }
            formula.extend_from_move(m);
        }
        formula
    }

    /// Extend formula with moments from a move.
    pub fn extend_from_move(&mut self, m: &Move) {
        for moment in &m.moments {
            self.elements
                .push(FormulaElement::Function(moment.function.clone()));
        }
    }

    /// Get all functions in the formula.
    pub fn functions(&self) -> impl Iterator<Item = &NarrativeFunctionInstance> {
        self.elements.iter().filter_map(|e| match e {
            FormulaElement::Function(f) => Some(f),
            FormulaElement::Optional(inner) => match inner.as_ref() {
                FormulaElement::Function(f) => Some(f),
                _ => None,
            },
            _ => None,
        })
    }
}

impl Display for Formula {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

fn write_element_text(out: &mut String, elem: &FormulaElement) {
    match elem {
        FormulaElement::Function(inst) => {
            if inst.negated {
                out.push_str("neg-");
            }
            out.push_str(inst.function.symbol());
            if let Some(sub) = inst.subtype {
                out.push_str(&superscript_text(sub));
            }
        }
        FormulaElement::MoveBreak => out.push_str("||"),
        FormulaElement::Triplication => out.push('³'),
        FormulaElement::Optional(inner) => {
            out.push('(');
            write_element_text(out, inner);
            out.push(')');
        }
    }
}

fn write_element_latex(out: &mut String, elem: &FormulaElement) {
    match elem {
        FormulaElement::Function(inst) => {
            if inst.negated {
                out.push_str("\\neg ");
            }
            out.push_str(symbol_to_latex(inst.function.symbol()));
            if let Some(sub) = inst.subtype {
                let _ = write!(out, "^{{{}}}", sub);
            }
        }
        FormulaElement::MoveBreak => out.push_str("\\|"),
        FormulaElement::Triplication => out.push_str("^{3}"),
        FormulaElement::Optional(inner) => {
            out.push('(');
            write_element_latex(out, inner);
            out.push(')');
        }
    }
}

fn symbol_to_latex(s: &str) -> &str {
    match s {
        "α" => "\\alpha",
        "β" => "\\beta",
        "γ" => "\\gamma",
        "δ" => "\\delta",
        "ε" => "\\varepsilon",
        "η" => "\\eta",
        "θ" => "\\theta",
        "↑" => "\\uparrow",
        "↓" => "\\downarrow",
        other => other,
    }
}

fn superscript_text(n: u8) -> String {
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

/// Element of a formula (элемент формулы).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormulaElement {
    /// A function instance.
    Function(NarrativeFunctionInstance),
    /// Move separator "||" (разделитель ходов).
    MoveBreak,
    /// Triplication marker "³" (утроение).
    Triplication,
    /// Optional element in parentheses (опциональный элемент).
    Optional(Box<FormulaElement>),
}

/// Error parsing a formula.
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    /// Unknown symbol encountered.
    #[error("unknown symbol: {0}")]
    UnknownSymbol(String),
    /// Invalid syntax.
    #[error("invalid syntax: {0}")]
    InvalidSyntax(String),
}

// ============================================================================
// Winnow parser implementation
// ============================================================================

fn parse_formula(input: &mut &str) -> ModalResult<Formula> {
    let elements: Vec<FormulaElement> = repeat(0.., parse_element).parse_next(input)?;
    Ok(Formula::from_elements(elements))
}

fn parse_element(input: &mut &str) -> ModalResult<FormulaElement> {
    // Skip leading whitespace
    multispace0.parse_next(input)?;

    if input.is_empty() {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    alt((
        parse_move_break,
        parse_triplication,
        parse_optional,
        parse_function,
    ))
    .parse_next(input)
}

fn parse_move_break(input: &mut &str) -> ModalResult<FormulaElement> {
    "||".parse_next(input)?;
    Ok(FormulaElement::MoveBreak)
}

fn parse_triplication(input: &mut &str) -> ModalResult<FormulaElement> {
    alt(("³", "^3")).parse_next(input)?;
    Ok(FormulaElement::Triplication)
}

fn parse_optional(input: &mut &str) -> ModalResult<FormulaElement> {
    let inner = delimited('(', parse_element, ')').parse_next(input)?;
    Ok(FormulaElement::Optional(Box::new(inner)))
}

fn parse_function(input: &mut &str) -> ModalResult<FormulaElement> {
    // Check for negation prefix
    let negated = opt(alt(("neg-", "neg", "¬"))).parse_next(input)?.is_some();

    // Parse the function symbol
    let func = parse_function_symbol(input)?;

    // Parse optional superscript subtype
    let subtype = opt(parse_superscript).parse_next(input)?;

    Ok(FormulaElement::Function(NarrativeFunctionInstance {
        function: func,
        subtype,
        negated,
    }))
}

fn parse_function_symbol(input: &mut &str) -> ModalResult<NarrativeFunction> {
    // Try two-letter symbols first
    if let Some(func) = try_parse_two_letter(input) {
        return Ok(func);
    }

    // Try single character/symbol
    let c = any.parse_next(input)?;
    let symbol = match c {
        'α' | 'a' if input.starts_with(|c: char| !c.is_ascii_alphabetic()) || input.is_empty() => {
            // Check if 'a' is Lack (lowercase a) vs part of "alpha"
            if c == 'a' {
                // 'a' alone is Lack
                return NarrativeFunction::from_symbol("a")
                    .ok_or_else(|| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()));
            }
            "α"
        }
        'α' => "α",
        'β' => "β",
        'γ' => "γ",
        'δ' => "δ",
        'ε' => "ε",
        'η' => "η",
        'θ' => "θ",
        '↑' | '^' => "↑",
        '↓' | 'v' if !input.starts_with(|c: char| c.is_ascii_alphabetic()) => "↓",
        'A' => "A",
        'B' => "B",
        'C' => "C",
        'D' => "D",
        'E' => "E",
        'F' => "F",
        'G' => "G",
        'H' => "H",
        'I' => "I",
        'J' => "J",
        'K' => "K",
        'L' => "L",
        'M' => "M",
        'N' => "N",
        'O' => "O",
        'Q' => "Q",
        'T' => "T",
        'U' => "U",
        'W' => "W",
        _ => {
            return Err(winnow::error::ErrMode::Backtrack(
                winnow::error::ContextError::new(),
            ))
        }
    };

    NarrativeFunction::from_symbol(symbol)
        .ok_or_else(|| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))
}

fn try_parse_two_letter(input: &mut &str) -> Option<NarrativeFunction> {
    let prefixes = [("Pr", NarrativeFunction::Pursuit), ("Rs", NarrativeFunction::Rescue), ("Ex", NarrativeFunction::Exposure)];

    for (prefix, func) in prefixes {
        if input.starts_with(prefix) {
            *input = &input[prefix.len()..];
            return Some(func);
        }
    }

    // Also try ASCII names for Greek letters
    let greek = [
        ("alpha", NarrativeFunction::Absentation),
        ("beta", NarrativeFunction::Interdiction),
        ("gamma", NarrativeFunction::Violation),
        ("delta", NarrativeFunction::Reconnaissance),
        ("epsilon", NarrativeFunction::Delivery),
        ("eta", NarrativeFunction::Trickery),
        ("theta", NarrativeFunction::Complicity),
    ];

    for (prefix, func) in greek {
        if input.starts_with(prefix) {
            *input = &input[prefix.len()..];
            return Some(func);
        }
    }

    None
}

fn parse_superscript(input: &mut &str) -> ModalResult<u8> {
    alt((parse_unicode_superscript, parse_caret_number)).parse_next(input)
}

fn superscript_char_to_digit(c: char) -> u8 {
    match c {
        '⁰' => 0,
        '¹' => 1,
        '²' => 2,
        '³' => 3,
        '⁴' => 4,
        '⁵' => 5,
        '⁶' => 6,
        '⁷' => 7,
        '⁸' => 8,
        '⁹' => 9,
        _ => unreachable!(),
    }
}

fn parse_unicode_superscript(input: &mut &str) -> ModalResult<u8> {
    let c = one_of(['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹']).parse_next(input)?;
    let digit = superscript_char_to_digit(c);

    // Try to parse second digit for numbers >= 10
    let second = opt(one_of(['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'])).parse_next(input)?;

    if let Some(c2) = second {
        Ok(digit * 10 + superscript_char_to_digit(c2))
    } else {
        Ok(digit)
    }
}

fn parse_caret_number(input: &mut &str) -> ModalResult<u8> {
    '^'.parse_next(input)?;
    let digits: String = repeat(1..=2, one_of('0'..='9')).parse_next(input)?;
    digits
        .parse()
        .map_err(|_| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Tests using real formulas from Propp's Appendix II
    // ========================================================================

    /// Formula from a simplified classic tale structure
    #[test]
    fn test_parse_classic_structure() {
        let formula = Formula::parse("α β γ A B ↑ D E F H I K ↓ W").unwrap();

        let funcs: Vec<_> = formula.functions().map(|f| f.function).collect();
        assert_eq!(funcs[0], NarrativeFunction::Absentation);
        assert_eq!(funcs[1], NarrativeFunction::Interdiction);
        assert_eq!(funcs[2], NarrativeFunction::Violation);
        assert_eq!(funcs[3], NarrativeFunction::Villainy);
        assert_eq!(funcs[4], NarrativeFunction::Mediation);
        assert_eq!(funcs[5], NarrativeFunction::Departure);
    }

    /// Formula with subtypes (like A¹, D², etc.)
    #[test]
    fn test_parse_with_subtypes() {
        // Simplified version inspired by Tale 50: β³γ³A¹B⁴C↑
        let formula = Formula::parse("β³ γ³ A¹ B⁴ C ↑").unwrap();

        let elems = formula.elements();

        // β³ - interdiction with subtype 3
        if let FormulaElement::Function(f) = &elems[0] {
            assert_eq!(f.function, NarrativeFunction::Interdiction);
            assert_eq!(f.subtype, Some(3));
        }

        // A¹ - villainy with subtype 1
        if let FormulaElement::Function(f) = &elems[2] {
            assert_eq!(f.function, NarrativeFunction::Villainy);
            assert_eq!(f.subtype, Some(1));
        }
    }

    /// Formula with move breaks (multiple moves)
    #[test]
    fn test_parse_multiple_moves() {
        let formula = Formula::parse("A B ↑ K ↓ || A B ↑ K ↓ W").unwrap();

        // Should have move break between two sequences
        let has_break = formula.elements().iter().any(|e| matches!(e, FormulaElement::MoveBreak));
        assert!(has_break);
    }

    /// Test negated functions
    #[test]
    fn test_parse_negated() {
        let formula = Formula::parse("neg-B A ¬K").unwrap();

        let funcs: Vec<_> = formula.functions().collect();
        assert!(funcs[0].negated); // neg-B
        assert!(!funcs[1].negated); // A
        assert!(funcs[2].negated); // ¬K
    }

    /// Test text output matches Propp's notation
    #[test]
    fn test_to_text_roundtrip() {
        let original = "α β γ A B ↑ D E F K ↓ W";
        let formula = Formula::parse(original).unwrap();
        let output = formula.to_text();

        // Parse the output again - should produce same structure
        let reparsed = Formula::parse(&output).unwrap();
        assert_eq!(formula.functions().count(), reparsed.functions().count());
    }

    /// Test LaTeX output
    #[test]
    fn test_to_latex() {
        let formula = Formula::parse("α β A¹ ↑ K ↓").unwrap();
        let latex = formula.to_latex();

        assert!(latex.starts_with('$'));
        assert!(latex.ends_with('$'));
        assert!(latex.contains("\\alpha"));
        assert!(latex.contains("\\beta"));
        assert!(latex.contains("\\uparrow"));
        assert!(latex.contains("\\downarrow"));
        assert!(latex.contains("A^{1}"));
    }

    /// Test from_tale conversion
    #[test]
    fn test_from_tale() {
        let mut tale = Tale::default();

        let mut m1 = Move::new();
        m1.add_function(NarrativeFunction::Villainy);
        m1.add_function(NarrativeFunction::Departure);
        tale.moves.push(m1);

        let mut m2 = Move::continuation();
        m2.add_function(NarrativeFunction::Return);
        m2.add_function(NarrativeFunction::Wedding);
        tale.moves.push(m2);

        let formula = Formula::from_tale(&tale);
        assert_eq!(formula.to_text(), "A ↑ || ↓ W");
    }

    /// Test optional elements in parentheses
    #[test]
    fn test_optional_elements() {
        let formula = Formula::parse("A (B) ↑").unwrap();

        let has_optional = formula.elements().iter().any(|e| matches!(e, FormulaElement::Optional(_)));
        assert!(has_optional);
    }

    /// Test two-letter function symbols
    #[test]
    fn test_two_letter_symbols() {
        let formula = Formula::parse("↓ Pr Rs").unwrap();

        let funcs: Vec<_> = formula.functions().map(|f| f.function).collect();
        assert_eq!(funcs[0], NarrativeFunction::Return);
        assert_eq!(funcs[1], NarrativeFunction::Pursuit);
        assert_eq!(funcs[2], NarrativeFunction::Rescue);
    }

    /// Test Lack function (lowercase 'a')
    #[test]
    fn test_lack_function() {
        let formula = Formula::parse("a B ↑").unwrap();

        let funcs: Vec<_> = formula.functions().map(|f| f.function).collect();
        assert_eq!(funcs[0], NarrativeFunction::Lack);
    }
}
