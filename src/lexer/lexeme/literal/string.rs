//!
//! The string literal lexeme.
//!

use std::fmt;

///
/// The string literal lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub struct String {
    /// The inner string contents.
    pub inner: std::string::String,
    /// Whether the string is hexadecimal.
    pub is_hexadecimal: bool,
}

impl String {
    ///
    /// Creates a string literal value.
    ///
    pub fn new(inner: ::std::string::String, is_hexadecimal: bool) -> Self {
        Self {
            inner,
            is_hexadecimal,
        }
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
