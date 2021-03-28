//!
//! The lexeme.
//!

pub mod keyword;
pub mod symbol;

use std::fmt;

use self::keyword::Keyword;
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
}

impl fmt::Display for Lexeme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyword(inner) => write!(f, "{}", inner),
            Self::Symbol(inner) => write!(f, "{}", inner),
            Self::Identifier(inner) => write!(f, "{}", inner),
        }
    }
}
