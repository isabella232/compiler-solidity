//!
//! The Ethereal IR block queue element.
//!

use crate::evm::ethereal_ir::function::block::element::stack::Stack;

///
/// The Ethereal IR block queue element.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueueElement {
    /// The block tag.
    pub tag: usize,
    /// The block predecessor. Unset for the function entry.
    pub predecessor: Option<usize>,
    /// The predecessor last stack state.
    pub stack: Stack,
}

impl QueueElement {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(tag: usize, predecessor: Option<usize>, stack: Stack) -> Self {
        Self {
            tag,
            predecessor,
            stack,
        }
    }
}
