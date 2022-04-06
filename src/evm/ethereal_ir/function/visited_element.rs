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
    /// The initial stack state hash.
    pub stack_hash: md5::Digest,
}

impl VisitedElement {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(tag: usize, stack_hash: md5::Digest) -> Self {
        Self { tag, stack_hash }
    }
}
