//!
//! The comment lexeme.
//!

///
/// The comment lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Comment {
    /// The `// ... \n` comment.
    SingleLine,
    /// The `/* ... */` comment.
    MultiLine,
}
