//!
//! The address space aliases.
//!

///
/// The address space aliases.
///
#[derive(Debug, Clone)]
pub enum AddressSpace {
    /// The stack memory.
    Stack,
    /// The heap memory.
    Heap,
    /// The shared parent contract memory.
    Parent,
    /// The shared child contract memory.
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
