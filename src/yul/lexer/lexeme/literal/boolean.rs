//!
//! The boolean literal lexeme.
//!

use std::fmt;

use crate::yul::lexer::lexeme::keyword::Keyword;

///
/// The boolean literal lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Boolean {
    /// Created from the `false` keyword.
    False,
    /// Created from the `true` keyword.
    True,
}

impl Boolean {
    ///
    /// Creates a `false` value.
    ///
    pub fn r#false() -> Self {
        Self::False
    }

    ///
    /// Creates a `true` value.
    ///
    pub fn r#true() -> Self {
        Self::True
    }
}

impl TryFrom<Keyword> for Boolean {
    type Error = Keyword;

    fn try_from(keyword: Keyword) -> Result<Self, Self::Error> {
        Ok(match keyword {
            Keyword::False => Self::False,
            Keyword::True => Self::True,
            unknown => return Err(unknown),
        })
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::False => write!(f, "false"),
            Self::True => write!(f, "true"),
        }
    }
}
