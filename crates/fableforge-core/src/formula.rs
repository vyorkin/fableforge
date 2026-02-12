//! Formula representation (формульное представление).
//!
//! A formula is a symbolic representation of a tale's structure
//! using Propp's notation. Example: "αβγ A¹ B ↑ D E F G H I K ↓ W"

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::function::{Function, FunctionInstance};
use crate::tale::{Move, Tale};

/// Tale formula in Propp's notation (формула сказки).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formula {
    elements: Vec<FormulaElement>,
}

impl Formula {
    /// Create an empty formula.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Create a formula from elements.
    #[must_use]
    pub fn from_elements(elements: Vec<FormulaElement>) -> Self {
        Self { elements }
    }

    /// Parse formula from string (парсинг формулы).
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if the string cannot be parsed.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let mut elements = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            // Skip whitespace
            if c.is_whitespace() {
                continue;
            }

            // Move break
            if c == '|' {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    elements.push(FormulaElement::MoveBreak);
                    continue;
                }
                // Single pipe also treated as move break
                elements.push(FormulaElement::MoveBreak);
                continue;
            }

            // Triplication marker
            if c == '³' || (c == '^' && chars.peek() == Some(&'3')) {
                if c == '^' {
                    chars.next();
                }
                elements.push(FormulaElement::Triplication);
                continue;
            }

            // Optional marker (parentheses)
            if c == '(' {
                // Simplified: just parse content until closing paren
                let mut inner = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == ')' {
                        chars.next();
                        break;
                    }
                    inner.push(chars.next().unwrap());
                }
                if let Ok(inner_formula) = Formula::parse(&inner) {
                    for elem in inner_formula.elements {
                        elements.push(FormulaElement::Optional(Box::new(elem)));
                    }
                }
                continue;
            }

            // Try to parse as function
            let mut symbol = c.to_string();

            // Check for two-letter symbols (Pr, Rs, Ex)
            if let Some(&next) = chars.peek() {
                let two_char = format!("{}{}", c, next);
                if matches!(two_char.as_str(), "Pr" | "Rs" | "Ex") {
                    chars.next();
                    symbol = two_char;
                }
            }

            // Check for negation prefix
            let negated = symbol == "neg" || symbol == "¬";
            if negated {
                if chars.peek() == Some(&'-') {
                    chars.next();
                }
                symbol.clear();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || ch == '|' || ch == '(' || ch == ')' {
                        break;
                    }
                    symbol.push(chars.next().unwrap());
                    // Check for two-letter symbols
                    if matches!(symbol.as_str(), "Pr" | "Rs" | "Ex") {
                        break;
                    }
                    if Function::from_symbol(&symbol).is_some() {
                        break;
                    }
                }
            }

            // Parse function
            if let Some(func) = Function::from_symbol(&symbol) {
                // Check for superscript subtype
                let mut subtype = None;
                if let Some(&next) = chars.peek()
                    && let Some(digit) = parse_superscript(next)
                {
                    chars.next();
                    subtype = Some(digit);
                    // Check for second digit
                    if let Some(&next2) = chars.peek()
                        && let Some(digit2) = parse_superscript(next2)
                    {
                        chars.next();
                        subtype = Some(digit * 10 + digit2);
                    }
                }

                let instance = FunctionInstance {
                    function: func,
                    subtype,
                    negated,
                };
                elements.push(FormulaElement::Function(instance));
            } else if !symbol.is_empty() && !negated {
                return Err(ParseError::UnknownSymbol(symbol));
            }
        }

        Ok(Self { elements })
    }

    /// Serialize to Propp's notation (символы Проппа).
    #[must_use]
    pub fn to_propp_notation(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        for elem in &self.elements {
            match elem {
                FormulaElement::Function(inst) => {
                    parts.push(inst.to_notation());
                }
                FormulaElement::MoveBreak => {
                    parts.push("||".to_string());
                }
                FormulaElement::Triplication => {
                    parts.push("³".to_string());
                }
                FormulaElement::Optional(inner) => {
                    let inner_str = match inner.as_ref() {
                        FormulaElement::Function(inst) => inst.to_notation(),
                        FormulaElement::MoveBreak => "||".to_string(),
                        FormulaElement::Triplication => "³".to_string(),
                        FormulaElement::Optional(_) => String::new(),
                    };
                    parts.push(format!("({})", inner_str));
                }
            }
        }

        parts.join(" ")
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
    pub fn push_function(&mut self, function: impl Into<FunctionInstance>) {
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
    pub fn functions(&self) -> impl Iterator<Item = &FunctionInstance> {
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

impl Default for Formula {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Formula {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_propp_notation())
    }
}

/// Parse superscript digit.
fn parse_superscript(c: char) -> Option<u8> {
    match c {
        '⁰' => Some(0),
        '¹' => Some(1),
        '²' => Some(2),
        '³' => Some(3),
        '⁴' => Some(4),
        '⁵' => Some(5),
        '⁶' => Some(6),
        '⁷' => Some(7),
        '⁸' => Some(8),
        '⁹' => Some(9),
        _ => None,
    }
}

/// Element of a formula (элемент формулы).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormulaElement {
    /// A function instance.
    Function(FunctionInstance),
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
    /// Unmatched parenthesis.
    #[error("unmatched parenthesis")]
    UnmatchedParen,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let formula = Formula::parse("A B C").unwrap();
        assert_eq!(formula.elements.len(), 3);
    }

    #[test]
    fn test_parse_greek() {
        let formula = Formula::parse("α β γ").unwrap();
        assert_eq!(formula.elements.len(), 3);
        assert!(matches!(
            &formula.elements[0],
            FormulaElement::Function(f) if f.function == Function::Absentation
        ));
    }

    #[test]
    fn test_parse_with_subtypes() {
        let formula = Formula::parse("A¹ D²").unwrap();
        assert_eq!(formula.elements.len(), 2);

        if let FormulaElement::Function(f) = &formula.elements[0] {
            assert_eq!(f.function, Function::Villainy);
            assert_eq!(f.subtype, Some(1));
        } else {
            panic!("Expected function");
        }

        if let FormulaElement::Function(f) = &formula.elements[1] {
            assert_eq!(f.function, Function::DonorTest);
            assert_eq!(f.subtype, Some(2));
        } else {
            panic!("Expected function");
        }
    }

    #[test]
    fn test_parse_two_letter() {
        let formula = Formula::parse("Pr Rs Ex").unwrap();
        assert_eq!(formula.elements.len(), 3);
    }

    #[test]
    fn test_parse_move_break() {
        let formula = Formula::parse("A B || C D").unwrap();
        assert_eq!(formula.elements.len(), 5);
        assert!(matches!(formula.elements[2], FormulaElement::MoveBreak));
    }

    #[test]
    fn test_parse_arrows() {
        let formula = Formula::parse("↑ ↓").unwrap();
        assert_eq!(formula.elements.len(), 2);
    }

    #[test]
    fn test_to_propp_notation() {
        let mut formula = Formula::new();
        formula.push_function(Function::Absentation);
        formula.push_function(Function::Interdiction);
        formula.push_function(FunctionInstance::with_subtype(Function::Villainy, 2));
        formula.push_move_break();
        formula.push_function(Function::Wedding);

        let notation = formula.to_propp_notation();
        assert_eq!(notation, "α β A² || W");
    }

    #[test]
    fn test_display() {
        let formula = Formula::parse("A B C").unwrap();
        assert_eq!(format!("{}", formula), "A B C");
    }

    #[test]
    fn test_roundtrip() {
        let original = "α β γ A B ↑ D E F || ↓ W";
        let formula = Formula::parse(original).unwrap();
        let output = formula.to_propp_notation();
        // Roundtrip should produce equivalent formula
        let reparsed = Formula::parse(&output).unwrap();
        assert_eq!(formula.elements.len(), reparsed.elements.len());
    }

    #[test]
    fn test_from_tale() {
        let mut tale = Tale::new(crate::tale::InitialSituation::default());

        let mut m1 = Move::new();
        m1.add_function(Function::Villainy);
        m1.add_function(Function::Departure);
        tale.add_move(m1);

        let mut m2 = Move::continuation();
        m2.add_function(Function::Return);
        m2.add_function(Function::Wedding);
        tale.add_move(m2);

        let formula = Formula::from_tale(&tale);
        assert_eq!(formula.to_propp_notation(), "A ↑ || ↓ W");
    }

    #[test]
    fn test_unknown_symbol() {
        let result = Formula::parse("X Y Z");
        assert!(result.is_err());
    }
}
