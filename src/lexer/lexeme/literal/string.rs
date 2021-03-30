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
}

impl String {
    ///
    /// Creates a string literal value.
    ///
    pub fn new(inner: ::std::string::String) -> Self {
        Self { inner }
    }
}

impl From<std::string::String> for String {
    fn from(value: std::string::String) -> Self {
        Self { inner: value }
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
