//!
//! The `solc --standard-json` expected output selection.
//!

///
/// The `solc --standard-json` expected output selection.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Selection {
    /// The ABI JSON representation.
    ABI,
    /// The Yul IR.
    Yul,
    /// The EVM legacy assembly JSON representation.
    EVM,
    /// The AST JSON representation.
    AST,
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ABI => write!(f, "abi"),
            Self::Yul => write!(f, "irOptimized"),
            Self::EVM => write!(f, "evm.legacyAssembly"),
            Self::AST => write!(f, "ast"),
        }
    }
}
