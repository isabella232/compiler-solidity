//!
//! The Ethereal IR block queue element.
//!

///
/// The Ethereal IR block queue element.
///
#[derive(Debug, Clone)]
pub struct QueueElement {
    /// The block tag.
    pub tag: usize,
    /// The block predecessor. Unset for the function entry.
    pub predecessor: Option<usize>,
    /// The tags collected so far.
    pub vertical_tags_buffer: Vec<usize>,
    /// The predecessor final stack offset.
    pub stack_offset: isize,
}

impl QueueElement {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        tag: usize,
        predecessor: Option<usize>,
        vertical_tags_buffer: Vec<usize>,
        stack_offset: isize,
    ) -> Self {
        Self {
            tag,
            predecessor,
            vertical_tags_buffer,
            stack_offset,
        }
    }
}
