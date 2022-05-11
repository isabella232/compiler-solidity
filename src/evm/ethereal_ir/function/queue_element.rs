//!
//! The Ethereal IR block queue element.
//!

use crate::evm::ethereal_ir::function::block::element::stack::Stack;

///
/// The Ethereal IR block queue element.
///
#[derive(Debug, Clone)]
pub struct QueueElement {
    /// The block key.
    pub block_key: compiler_llvm_context::FunctionBlockKey,
    /// The block predecessor.
    pub predecessor: Option<compiler_llvm_context::FunctionBlockKey>,
    /// The predecessor's last stack state.
    pub stack: Stack,
}

impl QueueElement {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        block_key: compiler_llvm_context::FunctionBlockKey,
        predecessor: Option<compiler_llvm_context::FunctionBlockKey>,
        stack: Stack,
    ) -> Self {
        Self {
            block_key,
            predecessor,
            stack,
        }
    }
}
