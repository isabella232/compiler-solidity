//!
//! The Solidity compiler pipeline type.
//!

///
/// The Solidity compiler pipeline type.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Pipeline {
    /// The Yul intermediate representation.
    Yul,
    /// The EVM bytecode JSON representation.
    EVM,
}
