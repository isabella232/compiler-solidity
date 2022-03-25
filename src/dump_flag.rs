//!
//! The Solidity IR dump flag.
//!

///
/// The intermediate representation dump flags.
///
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DumpFlag {
    /// Whether to dump the Yul code.
    Yul,
    /// Whether to dump the Ethereal IR code.
    EthIR,
    /// Whether to dump the EVM code.
    EVM,
    /// Whether to dump the LLVM IR code.
    LLVM,
    /// Whether to dump the assembly code.
    Assembly,
}

impl DumpFlag {
    ///
    /// A shortcut constructor for vector.
    ///
    pub fn initialize(yul: bool, ethir: bool, evm: bool, llvm: bool, assembly: bool) -> Vec<Self> {
        let mut vector = Vec::with_capacity(5);
        if yul {
            vector.push(Self::Yul);
        }
        if ethir {
            vector.push(Self::EthIR);
        }
        if evm {
            vector.push(Self::EVM);
        }
        if llvm {
            vector.push(Self::LLVM);
        }
        if assembly {
            vector.push(Self::Assembly);
        }
        vector
    }
}
