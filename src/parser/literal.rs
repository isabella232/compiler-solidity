//!
//! The YUL source code literal.
//!

///
/// Represents a literal in YUL without differentiating its type.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    /// The stringified value.
    pub value: String,
}
