//!
//! The address space aliases.
//!

///
/// The address space aliases.
///
#[derive(Debug, Clone)]
pub enum AddressSpace {
    /// The stack.
    Stack,
    /// The heap.
    #[allow(dead_code)] // TODO: use
    Heap,
    /// The parent.
    Parent,
    /// The child.
    Child,
}

impl From<AddressSpace> for inkwell::AddressSpace {
    fn from(value: AddressSpace) -> Self {
        match value {
            AddressSpace::Stack => Self::Zero,
            AddressSpace::Heap => Self::One,
            AddressSpace::Parent => Self::Two,
            AddressSpace::Child => Self::Three,
        }
    }
}
