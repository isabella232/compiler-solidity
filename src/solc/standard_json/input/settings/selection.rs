//!
//! The `solc --standard-json` expected output selection.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` expected output selection.
///
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Selection {
    /// The ABI JSON representation.
    #[serde(rename = "abi")]
    ABI,
    /// The AST JSON representation.
    #[serde(rename = "ast")]
    AST,
    /// The Yul IR.
    #[serde(rename = "irOptimized")]
    Yul,
    /// The EVM legacy assembly JSON representation.
    #[serde(rename = "evm.legacyAssembly")]
    EVM,
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ABI => write!(f, "abi"),
            Self::AST => write!(f, "ast"),
            Self::Yul => write!(f, "irOptimized"),
            Self::EVM => write!(f, "evm.legacyAssembly"),
        }
    }
}
