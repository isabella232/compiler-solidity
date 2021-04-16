//!
//! The Yul compiler error.
//!

use crate::lexer::error::Error as LexerError;
use crate::parser::error::Error as ParserError;

///
/// The Yul compiler error.
///
#[derive(Debug)]
pub enum Error {
    /// The target error.
    Target(String),
    /// The reader error.
    Reader(std::io::Error),
    /// The lexer error.
    Lexer(LexerError),
    /// The parser error.
    Parser(ParserError),
    /// The LLVM error.
    #[allow(clippy::upper_case_acronyms)]
    LLVM(String),
}

impl PartialEq<Self> for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Lexer(first), Self::Lexer(second)) => first == second,
            (Self::Parser(first), Self::Parser(second)) => first == second,
            _ => false,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Reader(error)
    }
}

impl From<LexerError> for Error {
    fn from(error: LexerError) -> Self {
        Self::Lexer(error)
    }
}

impl From<ParserError> for Error {
    fn from(error: ParserError) -> Self {
        Self::Parser(error)
    }
}
