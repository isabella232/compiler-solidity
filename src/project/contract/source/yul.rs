//!
//! The `solc --standard-json` contract Yul source.
//!

use crate::yul::parser::statement::object::Object;

///
/// The `solc --standard-json` contract Yul source.
///
#[derive(Debug, Clone)]
pub struct Yul {
    /// The Yul source code.
    pub source: String,
    /// The Yul AST object.
    pub object: Object,
}

impl Yul {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(source: String, object: Object) -> Self {
        Self { source, object }
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Yul
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.object.declare(context)
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.object.into_llvm(context)
    }
}
