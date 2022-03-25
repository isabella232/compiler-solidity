//!
//! The Ethereal IR block exit.
//!

///
/// The Ethereal IR block exit.
///
#[derive(Debug, Clone)]
pub enum Exit {
    /// The function call representation.
    Call {
        /// The tag of the callee block.
        callee: usize,
    },
    /// The block fallthrough representation.
    Fallthrough {
        /// The tag of the destination block.
        destination: usize,
    },
    /// The unconditional jump representation.
    Unconditional,
    /// The function return representation.
    Return,
}

impl Exit {
    ///
    /// A shortcut constructor.
    ///
    pub fn call(callee: usize) -> Self {
        Self::Call { callee }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn fallthrough(destination: usize) -> Self {
        Self::Fallthrough { destination }
    }
}
