//!
//! The Ethereal IR block visited element.
//!

///
/// The Ethereal IR block visited element.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisitedElement {
    /// The block tag.
    pub tag: usize,
    /// The initial stack pattern.
    pub stack_pattern: String,
}

impl VisitedElement {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(tag: usize, stack_pattern: String) -> Self {
        Self { tag, stack_pattern }
    }
}
