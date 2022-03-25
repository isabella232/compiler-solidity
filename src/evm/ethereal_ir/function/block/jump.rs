//!
//! The Ethereal IR block jump.
//!

///
/// The Ethereal IR block jump.
///
#[derive(Debug, Clone)]
pub struct Jump {
    /// The destination tag.
    pub destination: usize,
    /// The tags collected by the position of the jump
    pub tags: Vec<usize>,
    /// The position of the jump in the block.
    pub position: usize,
}

impl Jump {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(destination: usize, tags: Vec<usize>, position: usize) -> Self {
        Self {
            destination,
            tags,
            position,
        }
    }
}
