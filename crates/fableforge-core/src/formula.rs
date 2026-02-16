//! Formula representation (формульное представление).
//!
//! A formula is a symbolic representation of a tale's structure
//! using Propp's notation. Examples from Appendix II:
//! - "β³γ³A¹B⁴C↑" (Tale 50)
//! - "αβγ A B ↑ D E F H I K ↓ W" (classic structure)

use std::fmt::{self, Display, Formatter, Write as FmtWrite};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use winnow::{
    ModalResult,
    ascii::multispace0,
    combinator::{alt, delimited, opt, repeat},
    prelude::*,
    token::{any, one_of},
};

use crate::{
    dramatis::{Persona, Sphere},
    function::{NarrativeFunction, NarrativeFunctionInstance, Phase},
    tale::{Moment, Move, Tale},
};

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
    pub fn push_function(
        &mut self,
        function: impl Into<NarrativeFunctionInstance>,
    ) {
        self.elements.push(FormulaElement::Function(
            function.into(),
        ));
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
            self.elements.push(FormulaElement::Function(
                moment.function.clone(),
            ));
        }
        // Add embedded moves as bracketed elements
        for em in &m.embedded_moves {
            let mut inner = Vec::new();
            for moment in &em.moments {
                inner.push(FormulaElement::Function(
                    moment.function.clone(),
                ));
            }
            self.elements.push(FormulaElement::Embedded(inner));
        }
    }

    /// Get all functions in the formula (not including embedded).
    pub fn functions(
        &self,
    ) -> impl Iterator<Item = &NarrativeFunctionInstance> {
        self.elements.iter().filter_map(|e| match e {
            FormulaElement::Function(f) => Some(f),
            FormulaElement::Optional(inner) => match inner.as_ref() {
                FormulaElement::Function(f) => Some(f),
                _ => None,
            },
            _ => None,
        })
    }

    /// Convert a parsed formula into a `Tale`.
    ///
    /// Splits elements by `MoveBreak` into moves, creates `Moment`s from
    /// function elements, unwraps `Optional` (treats as present), skips
    /// `Triplication`, and infers personae from the functions present.
    #[must_use]
    pub fn to_tale(&self) -> Tale {
        // Split elements into groups by MoveBreak, collecting embedded elements
        // per group
        let mut move_groups: Vec<Vec<NarrativeFunctionInstance>> =
            vec![Vec::new()];
        let mut embedded_groups: Vec<Vec<Vec<NarrativeFunctionInstance>>> =
            vec![Vec::new()];

        for elem in &self.elements {
            match elem {
                FormulaElement::MoveBreak => {
                    move_groups.push(Vec::new());
                    embedded_groups.push(Vec::new());
                }
                FormulaElement::Function(inst) => {
                    move_groups.last_mut().unwrap().push(inst.clone());
                }
                FormulaElement::Optional(inner) => {
                    // Unwrap optional — treat as present
                    if let FormulaElement::Function(inst) = inner.as_ref() {
                        move_groups.last_mut().unwrap().push(inst.clone());
                    }
                }
                FormulaElement::Triplication => {
                    // Skip triplication markers
                }
                FormulaElement::Embedded(elements) => {
                    let funcs: Vec<NarrativeFunctionInstance> = elements
                        .iter()
                        .filter_map(|e| {
                            if let FormulaElement::Function(inst) = e {
                                Some(inst.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    embedded_groups.last_mut().unwrap().push(funcs);
                }
            }
        }

        // Remove empty trailing groups (keep indices aligned)
        let mut keep_indices = Vec::new();
        for (i, g) in move_groups.iter().enumerate() {
            if !g.is_empty() {
                keep_indices.push(i);
            }
        }
        let move_groups: Vec<_> = keep_indices
            .iter()
            .map(|&i| move_groups[i].clone())
            .collect();
        let embedded_groups: Vec<_> = keep_indices
            .iter()
            .map(|&i| embedded_groups[i].clone())
            .collect();

        // Collect all functions to infer personae
        let all_funcs: Vec<NarrativeFunction> = move_groups
            .iter()
            .flat_map(|g| g.iter().map(|inst| inst.function))
            .collect();

        let personae = Self::infer_personae(&all_funcs);

        let hero_id = personae
            .iter()
            .find(|p| p.spheres.contains(&Sphere::Hero))
            .map(|p| p.id);
        let villain_id = personae
            .iter()
            .find(|p| p.spheres.contains(&Sphere::Villain))
            .map(|p| p.id);

        // Build moves
        let moves: Vec<Move> = move_groups
            .into_iter()
            .zip(embedded_groups)
            .enumerate()
            .map(|(i, (funcs, embedded_funcs))| {
                let mut m = if i == 0 {
                    Move::new()
                } else {
                    Move::continuation()
                };
                for inst in funcs {
                    let mut moment = Moment::new(inst.clone());
                    // Assign agents/patients based on phase
                    match inst.function.phase() {
                        Phase::Complication
                            if inst.function == NarrativeFunction::Villainy =>
                        {
                            moment.agent = villain_id;
                            moment.patient = hero_id;
                        }
                        Phase::Donor
                        | Phase::Struggle
                        | Phase::Return
                        | Phase::Resolution => {
                            moment.agent = hero_id;
                        }
                        _ => {}
                    }
                    m.moments.push(moment);
                }
                // Add embedded moves
                for em_funcs in embedded_funcs {
                    let mut em = Move::embedded();
                    for inst in em_funcs {
                        let mut moment = Moment::new(inst.clone());
                        match inst.function.phase() {
                            Phase::Donor
                            | Phase::Struggle
                            | Phase::Return
                            | Phase::Recognition
                            | Phase::Resolution => {
                                moment.agent = hero_id;
                            }
                            _ => {}
                        }
                        em.moments.push(moment);
                    }
                    m.embedded_moves.push(em);
                }
                m
            })
            .collect();

        Tale {
            initial: None,
            moves,
            personae,
        }
    }

    /// Infer personae from a list of narrative functions.
    fn infer_personae(funcs: &[NarrativeFunction]) -> Vec<Persona> {
        let mut personae = Vec::new();
        let mut next_id = 1u32;

        // Always add Hero
        personae.push(Persona::new(
            next_id,
            vec![Sphere::Hero],
        ));
        next_id += 1;

        // Villainy present → add Villain
        if funcs.contains(&NarrativeFunction::Villainy) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Villain],
            ));
            next_id += 1;
        }

        // Donor sequence present → add Donor
        if funcs.iter().any(|f| {
            matches!(
                f,
                NarrativeFunction::DonorTest
                    | NarrativeFunction::HeroReaction
                    | NarrativeFunction::Acquisition
            )
        }) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Donor],
            ));
            next_id += 1;
        }

        // Recognition/Wedding present → add Princess
        if funcs.iter().any(|f| {
            matches!(
                f,
                NarrativeFunction::Recognition | NarrativeFunction::HappyEnding
            )
        }) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Princess],
            ));
            next_id += 1;
        }

        // Mediation present → add Dispatcher
        if funcs.contains(&NarrativeFunction::Mediation) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Dispatcher],
            ));
            next_id += 1;
        }

        // Guidance present → add Helper
        if funcs.contains(&NarrativeFunction::Guidance) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::Helper],
            ));
            next_id += 1;
        }

        // UnfoundedClaims or Exposure present → add FalseHero
        if funcs.iter().any(|f| {
            matches!(
                f,
                NarrativeFunction::UnfoundedClaims
                    | NarrativeFunction::Exposure
            )
        }) {
            personae.push(Persona::new(
                next_id,
                vec![Sphere::FalseHero],
            ));
            let _ = next_id;
        }

        personae
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
        FormulaElement::Embedded(elements) => {
            out.push('[');
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_element_text(out, elem);
            }
            out.push(']');
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
        FormulaElement::Embedded(elements) => {
            out.push_str("\\left[");
            for (i, elem) in elements.iter().enumerate() {
                if i > 0 {
                    out.push_str(" \\; ");
                }
                write_element_latex(out, elem);
            }
            out.push_str("\\right]");
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
    const SUPERSCRIPTS: [char; 10] =
        ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
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
    /// Embedded move in square brackets (вложенный ход).
    Embedded(Vec<FormulaElement>),
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
    let elements: Vec<FormulaElement> =
        repeat(0.., parse_element).parse_next(input)?;
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
        parse_embedded,
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

fn parse_embedded(input: &mut &str) -> ModalResult<FormulaElement> {
    let elements: Vec<FormulaElement> =
        delimited('[', repeat(0.., parse_element), ']').parse_next(input)?;
    Ok(FormulaElement::Embedded(elements))
}

fn parse_optional(input: &mut &str) -> ModalResult<FormulaElement> {
    let inner = delimited('(', parse_element, ')').parse_next(input)?;
    Ok(FormulaElement::Optional(Box::new(
        inner,
    )))
}

fn parse_function(input: &mut &str) -> ModalResult<FormulaElement> {
    // Check for negation prefix
    let negated = opt(alt(("neg-", "neg", "¬"))).parse_next(input)?.is_some();

    // Parse the function symbol
    let func = parse_function_symbol(input)?;

    // Parse optional superscript subtype
    let subtype = opt(parse_superscript).parse_next(input)?;

    Ok(FormulaElement::Function(
        NarrativeFunctionInstance {
            function: func,
            subtype,
            negated,
        },
    ))
}

fn parse_function_symbol(input: &mut &str) -> ModalResult<NarrativeFunction> {
    // Try two-letter symbols first
    if let Some(func) = try_parse_two_letter(input) {
        return Ok(func);
    }

    // Try single character/symbol
    let c = any.parse_next(input)?;
    let symbol = match c {
        'α' | 'a'
            if input.starts_with(|c: char| !c.is_ascii_alphabetic())
                || input.is_empty() =>
        {
            // Check if 'a' is Lack (lowercase a) vs part of "alpha"
            if c == 'a' {
                // 'a' alone is Lack
                return NarrativeFunction::from_symbol("a").ok_or_else(|| {
                    winnow::error::ErrMode::Backtrack(
                        winnow::error::ContextError::new(),
                    )
                });
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
        '↓' | 'v' if !input.starts_with(|c: char| c.is_ascii_alphabetic()) => {
            "↓"
        }
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
            ));
        }
    };

    NarrativeFunction::from_symbol(symbol).ok_or_else(|| {
        winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new())
    })
}

fn try_parse_two_letter(input: &mut &str) -> Option<NarrativeFunction> {
    let prefixes = [
        ("Pr", NarrativeFunction::Pursuit),
        ("Rs", NarrativeFunction::Rescue),
        ("Ex", NarrativeFunction::Exposure),
    ];

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
        (
            "delta",
            NarrativeFunction::Reconnaissance,
        ),
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
    alt((
        parse_unicode_superscript,
        parse_caret_number,
    ))
    .parse_next(input)
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
    let c = one_of(['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'])
        .parse_next(input)?;
    let digit = superscript_char_to_digit(c);

    // Try to parse second digit for numbers >= 10
    let second = opt(one_of([
        '⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹',
    ]))
    .parse_next(input)?;

    if let Some(c2) = second {
        Ok(digit * 10 + superscript_char_to_digit(c2))
    } else {
        Ok(digit)
    }
}

fn parse_caret_number(input: &mut &str) -> ModalResult<u8> {
    '^'.parse_next(input)?;
    let digits: String = repeat(1..=2, one_of('0'..='9')).parse_next(input)?;
    digits.parse().map_err(|_| {
        winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new())
    })
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
        assert_eq!(
            funcs[1],
            NarrativeFunction::Interdiction
        );
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
            assert_eq!(
                f.function,
                NarrativeFunction::Interdiction
            );
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
        let has_break = formula
            .elements()
            .iter()
            .any(|e| matches!(e, FormulaElement::MoveBreak));
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
        assert_eq!(
            formula.functions().count(),
            reparsed.functions().count()
        );
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
        m2.add_function(NarrativeFunction::HappyEnding);
        tale.moves.push(m2);

        let formula = Formula::from_tale(&tale);
        assert_eq!(formula.to_text(), "A ↑ || ↓ W");
    }

    /// Test optional elements in parentheses
    #[test]
    fn test_optional_elements() {
        let formula = Formula::parse("A (B) ↑").unwrap();

        let has_optional = formula
            .elements()
            .iter()
            .any(|e| matches!(e, FormulaElement::Optional(_)));
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

    // ========================================================================
    // to_tale() tests
    // ========================================================================

    /// Roundtrip: from_tale → to_tale preserves function sequence
    #[test]
    fn test_roundtrip_from_tale_to_tale() {
        let mut tale = Tale::default();
        tale.personae.push(crate::dramatis::Persona::new(
            1u32,
            vec![crate::dramatis::Sphere::Hero],
        ));
        tale.personae.push(crate::dramatis::Persona::new(
            2u32,
            vec![crate::dramatis::Sphere::Villain],
        ));

        let mut m = Move::new();
        m.add_function(NarrativeFunction::Villainy);
        m.add_function(NarrativeFunction::Mediation);
        m.add_function(NarrativeFunction::Departure);
        m.add_function(NarrativeFunction::Liquidation);
        m.add_function(NarrativeFunction::Return);
        m.add_function(NarrativeFunction::HappyEnding);
        tale.moves.push(m);

        let formula = Formula::from_tale(&tale);
        let reconstructed = formula.to_tale();

        // Function sequence must match
        let original_funcs: Vec<_> =
            tale.all_moments().map(|m| m.function.function).collect();
        let reconstructed_funcs: Vec<_> = reconstructed
            .all_moments()
            .map(|m| m.function.function)
            .collect();
        assert_eq!(original_funcs, reconstructed_funcs);
    }

    /// Parse a formula string and convert to tale
    #[test]
    fn test_to_tale_from_parsed() {
        let formula = Formula::parse("A B ↑ K ↓ W").unwrap();
        let tale = formula.to_tale();

        // Single move (no MoveBreak)
        assert_eq!(tale.moves.len(), 1);

        // Correct function sequence
        let funcs: Vec<_> =
            tale.all_moments().map(|m| m.function.function).collect();
        assert_eq!(
            funcs,
            vec![
                NarrativeFunction::Villainy,
                NarrativeFunction::Mediation,
                NarrativeFunction::Departure,
                NarrativeFunction::Liquidation,
                NarrativeFunction::Return,
                NarrativeFunction::HappyEnding,
            ]
        );

        // Personae inferred: Hero, Villain (Villainy), Princess (Wedding)
        assert!(
            tale.personae
                .iter()
                .any(|p| p.spheres.contains(&crate::dramatis::Sphere::Hero))
        );
        assert!(
            tale.personae
                .iter()
                .any(|p| p.spheres.contains(&crate::dramatis::Sphere::Villain))
        );
        assert!(
            tale.personae.iter().any(|p| p
                .spheres
                .contains(&crate::dramatis::Sphere::Princess))
        );
    }

    /// to_tale with multiple moves (MoveBreak)
    #[test]
    fn test_to_tale_multiple_moves() {
        let formula = Formula::parse("A B ↑ K || ↓ W").unwrap();
        let tale = formula.to_tale();

        assert_eq!(tale.moves.len(), 2);
        assert_eq!(tale.moves[0].moments.len(), 4); // A B ↑ K
        assert_eq!(tale.moves[1].moments.len(), 2); // ↓ W
    }

    /// to_tale unwraps Optional elements
    #[test]
    fn test_to_tale_optional_unwrapped() {
        let formula = Formula::parse("A (B) ↑").unwrap();
        let tale = formula.to_tale();

        let funcs: Vec<_> =
            tale.all_moments().map(|m| m.function.function).collect();
        assert_eq!(
            funcs,
            vec![
                NarrativeFunction::Villainy,
                NarrativeFunction::Mediation,
                NarrativeFunction::Departure,
            ]
        );
    }

    /// to_tale preserves subtypes
    #[test]
    fn test_to_tale_preserves_subtypes() {
        let formula = Formula::parse("A¹ B⁴ ↑").unwrap();
        let tale = formula.to_tale();

        assert_eq!(
            tale.moves[0].moments[0].function.subtype,
            Some(1)
        );
        assert_eq!(
            tale.moves[0].moments[1].function.subtype,
            Some(4)
        );
    }

    /// to_tale infers Donor sphere when donor functions present
    #[test]
    fn test_to_tale_infers_donor() {
        let formula = Formula::parse("A ↑ D E F K ↓").unwrap();
        let tale = formula.to_tale();

        assert!(
            tale.personae
                .iter()
                .any(|p| p.spheres.contains(&crate::dramatis::Sphere::Donor))
        );
    }

    // ========================================================================
    // Embedded move tests
    // ========================================================================

    /// Test parsing embedded moves in square brackets
    #[test]
    fn test_parse_embedded() {
        let formula = Formula::parse("A ↑ [a C ↑ K ↓] K ↓ W").unwrap();

        let has_embedded = formula
            .elements()
            .iter()
            .any(|e| matches!(e, FormulaElement::Embedded(_)));
        assert!(has_embedded);
    }

    /// Test text output of embedded moves
    #[test]
    fn test_embedded_text_output() {
        let formula = Formula::parse("A ↑ [a C ↑ K ↓] K ↓ W").unwrap();
        let text = formula.to_text();
        assert!(
            text.contains("[a C ↑ K ↓]"),
            "Expected embedded in text output, got: {}",
            text
        );
    }

    /// Test LaTeX output of embedded moves
    #[test]
    fn test_embedded_latex_output() {
        let formula = Formula::parse("A ↑ [a C ↑ K ↓] K ↓ W").unwrap();
        let latex = formula.to_latex();
        assert!(
            latex.contains("\\left["),
            "Expected \\left[ in LaTeX output, got: {}",
            latex
        );
        assert!(
            latex.contains("\\right]"),
            "Expected \\right] in LaTeX output, got: {}",
            latex
        );
    }

    /// Test from_tale with embedded moves
    #[test]
    fn test_from_tale_with_embedded() {
        let mut tale = Tale::default();
        let mut m = Move::new();
        m.add_function(NarrativeFunction::Villainy);
        m.add_function(NarrativeFunction::Departure);

        let mut em = Move::embedded();
        em.add_function(NarrativeFunction::Lack);
        em.add_function(NarrativeFunction::Liquidation);
        em.add_function(NarrativeFunction::Return);
        m.embedded_moves.push(em);

        tale.moves.push(m);

        let formula = Formula::from_tale(&tale);
        let text = formula.to_text();
        assert!(
            text.contains("[a K ↓]"),
            "Expected embedded in formula text, got: {}",
            text
        );
    }

    /// Test to_tale with embedded moves
    #[test]
    fn test_to_tale_with_embedded() {
        let formula = Formula::parse("A ↑ [a C ↑ K ↓] K ↓ W").unwrap();
        let tale = formula.to_tale();

        assert_eq!(tale.moves.len(), 1);
        assert_eq!(tale.moves[0].embedded_moves.len(), 1);

        let em = &tale.moves[0].embedded_moves[0];
        assert_eq!(
            em.relation,
            crate::tale::MoveRelation::Embedded
        );
        let em_funcs: Vec<_> =
            em.moments.iter().map(|m| m.function.function).collect();
        assert_eq!(
            em_funcs,
            vec![
                NarrativeFunction::Lack,
                NarrativeFunction::Counteraction,
                NarrativeFunction::Departure,
                NarrativeFunction::Liquidation,
                NarrativeFunction::Return,
            ]
        );
    }

    /// Test infer_personae includes Dispatcher and Helper
    #[test]
    fn test_infer_dispatcher_and_helper() {
        let formula = Formula::parse("A B ↑ G H I K ↓ W").unwrap();
        let tale = formula.to_tale();

        assert!(
            tale.personae.iter().any(|p| p
                .spheres
                .contains(&crate::dramatis::Sphere::Dispatcher)),
            "Expected Dispatcher inferred from Mediation (B)"
        );
        assert!(
            tale.personae
                .iter()
                .any(|p| p.spheres.contains(&crate::dramatis::Sphere::Helper)),
            "Expected Helper inferred from Guidance (G)"
        );
    }
}
