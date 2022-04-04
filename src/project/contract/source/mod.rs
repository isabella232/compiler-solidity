//!
//! The `solc --standard-json` contract source.
//!

pub mod evm;
pub mod yul;

use crate::evm::assembly::Assembly;
use crate::yul::parser::statement::object::Object;

use self::evm::EVM;
use self::yul::Yul;

///
/// The `solc --standard-json` contract source.
///
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum Source {
    /// The Yul source representation.
    Yul(Yul),
    /// The EVM legacy assembly source representation.
    EVM(EVM),
}

impl Source {
    ///
    /// A shortcut constructor.
    ///
    pub fn new_yul(source: String, object: Object) -> Self {
        Self::Yul(Yul::new(source, object))
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_evm(full_path: String, assembly: Assembly) -> Self {
        Self::EVM(EVM::new(full_path, assembly))
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Source
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        match self {
            Self::Yul(inner) => inner.declare(context),
            Self::EVM(inner) => inner.declare(context),
        }
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        match self {
            Self::Yul(inner) => inner.into_llvm(context),
            Self::EVM(inner) => inner.into_llvm(context),
        }
    }
}
