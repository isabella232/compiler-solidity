//!
//! The integer literal lexeme.
//!

use std::fmt;

///
/// The integer literal lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Integer {
    /// An integer literal, like `42`.
    Decimal {
        /// The inner literal contents.
        inner: String,
    },
    /// A hexadecimal literal, like `0xffff`.
    Hexadecimal {
        /// The inner literal contents.
        inner: String,
    },
}

impl Integer {
    ///
    /// Creates a decimal value.
    ///
    pub fn new_decimal(inner: String) -> Self {
        Self::Decimal { inner }
    }

    ///
    /// Creates a hexadecimal value.
    ///
    pub fn new_hexadecimal(inner: String) -> Self {
        Self::Hexadecimal { inner }
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decimal { inner } => write!(f, "{}", inner),
            Self::Hexadecimal { inner } => write!(f, "{}", inner),
        }
    }
}
