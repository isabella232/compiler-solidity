//!
//! The `solc --standard-json` output contract EVM data.
//!

pub mod bytecode;

use serde::Deserialize;
use serde::Serialize;

use crate::evm::assembly::Assembly;

use self::bytecode::Bytecode;

///
/// The `solc --standard-json` output contract EVM data.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EVM {
    /// The contract assembly code.
    #[serde(rename = "legacyAssembly")]
    pub assembly: Option<Assembly>,
    /// The contract bytecode.
    /// Is reset by that of zkEVM before yielding the compiled project artifacts.
    pub bytecode: Option<Bytecode>,
}

impl EVM {
    ///
    /// A shortcut constructor for the zkEVM bytecode.
    ///
    pub fn new_zkevm_bytecode(bytecode: String) -> Self {
        Self {
            assembly: None,
            bytecode: Some(Bytecode::new(bytecode)),
        }
    }
}
