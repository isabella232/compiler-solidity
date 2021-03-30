//!
//! The lexeme.
//!

pub mod keyword;
pub mod literal;
pub mod symbol;

use std::fmt;

use self::keyword::Keyword;
use self::literal::Literal;
use self::symbol::Symbol;

///
/// The lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Lexeme {
    /// The keyword lexeme.
    Keyword(Keyword),
    /// The symbol lexeme.
    Symbol(Symbol),
    /// The identifier lexeme.
    Identifier(String),
    /// The literal lexeme.
    Literal(Literal),
    /// The end-of-file lexeme.
    EndOfFile,
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword(inner) => write!(f, "{}", inner),
            Self::Symbol(inner) => write!(f, "{}", inner),
            Self::Identifier(inner) => write!(f, "{}", inner),
            Self::Literal(inner) => write!(f, "{}", inner),
            Self::EndOfFile => write!(f, "EOF"),
        }
    }
}
