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

    ///
    /// Parses the value from the source code slice.
    ///
    pub fn parse(input: &str) -> Option<Self> {
        let decimal = regex::Regex::new("^[0-9]+$").expect("Regexp is valid");
        let hexadecimal = regex::Regex::new(r#"^0x[0-9a-fA-F]+$"#).expect("Regexp is valid");

        if decimal.is_match(input) {
            Some(Self::new_decimal(input.to_owned()))
        } else if hexadecimal.is_match(input) {
            Some(Self::new_hexadecimal(input.to_owned()))
        } else {
            None
        }
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
