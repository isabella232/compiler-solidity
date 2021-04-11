//!
//! The Yul compiler error.
//!

use crate::lexer::error::Error as LexerError;
use crate::parser::error::Error as ParserError;

///
/// The Yul compiler error.
///
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The lexer error.
    Lexer(LexerError),
    /// The parser error.
    Parser(ParserError),
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
