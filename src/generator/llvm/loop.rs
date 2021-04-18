//!
//! The LLVM generator loop context.
//!

///
/// The LLVM generator loop context.
///
#[derive(Debug, Clone)]
pub struct Loop<'ctx> {
    /// The loop current block.
    pub body_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The increment block before the body.
    pub continue_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The join block after the body.
    pub break_block: inkwell::basic_block::BasicBlock<'ctx>,
}

impl<'ctx> Loop<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        break_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Self {
        Self {
            body_block,
            continue_block,
            break_block,
        }
    }
}
