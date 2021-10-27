//!
//! The compiler parser error.
//!

use crate::lexer::lexeme::Lexeme;

///
/// The compiler parser error.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// One of the common `expected-*` class errors.
    ExpectedOneOf {
        /// The list of the expected lexemes.
        expected: String,
        /// The invalid lexeme.
        found: Lexeme,
        /// The optional error hint text.
        help: Option<&'static str>,
    },
}

impl Error {
    ///
    /// A shortcut constructor.
    ///
    pub fn expected_one_of(
        expected: Vec<&'static str>,
        found: Lexeme,
        help: Option<&'static str>,
    ) -> Self {
        Self::ExpectedOneOf {
            expected: Self::format_one_of(expected.as_slice()),
            found,
            help,
        }
    }

    ///
    /// Converts a group of lexemes into a comma-separated list.
    ///
    /// E.g. ["function", "let", "if"] turns into `function`, `let`, `if`.
    ///
    pub fn format_one_of(lexemes: &[&'static str]) -> String {
        lexemes
            .iter()
            .map(|lexeme| format!("`{}`", lexeme))
            .collect::<Vec<String>>()
            .join(", ")
    }
}
