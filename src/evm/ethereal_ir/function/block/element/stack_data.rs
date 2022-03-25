//!
//! The Ethereal IR block element stack data.
//!

///
/// The Ethereal IR block element stack data.
///
#[derive(Debug, Clone)]
pub struct StackData {
    /// The current offset.
    pub current: isize,
}

impl StackData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(current: isize) -> Self {
        Self { current }
    }
}

impl std::fmt::Display for StackData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.current)
    }
}
