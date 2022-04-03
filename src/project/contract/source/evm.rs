//!
//! The `solc --standard-json` contract EVM legacy assembly source.
//!

use crate::evm::assembly::Assembly;

///
/// The `solc --standard-json` contract EVM legacy assembly source.
///
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub struct EVM {
    /// The source code identifier.
    pub source_identifier: String,
    /// The EVM legacy assembly source code.
    pub assembly: Assembly,
}

impl EVM {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(source_identifier: String, assembly: Assembly) -> Self {
        Self {
            source_identifier,
            assembly,
        }
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for EVM
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.assembly.declare(context)
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.assembly.into_llvm(context)
    }
}
