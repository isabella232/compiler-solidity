//!
//! The code generator.
//!

pub mod llvm;

use self::llvm::Context as LLVMContext;

///
/// Implemented by items which are translated into LLVM IR.
///
#[allow(clippy::upper_case_acronyms)]
pub trait ILLVMWritable {
    ///
    /// Translates the entity into LLVM IR.
    ///
    fn into_llvm(self, context: &mut LLVMContext) -> anyhow::Result<()>;
}
