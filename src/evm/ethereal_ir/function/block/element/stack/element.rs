//!
//! The Ethereal IR block element stack element.
//!

///
/// The Ethereal IR block element stack element.
///
#[derive(Debug, Clone)]
pub enum Element {
    /// The unknown runtime value.
    Value,
    /// The known compile-time value.
    Constant(String),
    /// The known compile-time destination tag.
    Tag(usize),
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value => write!(f, "VALUE"),
            Self::Constant(value) => write!(f, "{}", value),
            Self::Tag(tag) => write!(f, "TAG_{}", tag),
        }
    }
}
