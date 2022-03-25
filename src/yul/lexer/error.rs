//!
//! The compiler lexer error.
//!

///
/// The compiler lexer error.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// An invalid lexeme has been found.
    InvalidLexeme {
        /// The stringified invalid lexeme.
        found: String,
    },
}

impl Error {
    ///
    /// A shortcut constructor.
    ///
    pub fn invalid_lexeme(found: String) -> Self {
        Self::InvalidLexeme { found }
    }
}
