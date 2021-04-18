//!
//! The LLVM generator function context.
//!

use std::collections::HashMap;

///
/// The LLVM generator function context.
///
#[derive(Debug, Clone)]
pub struct Function<'ctx> {
    /// The name.
    pub name: String,
    /// The LLVM value.
    pub value: inkwell::values::FunctionValue<'ctx>,
    /// The entry block.
    pub entry_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return or leave block.
    pub return_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return value pointer.
    pub return_pointer: Option<inkwell::values::PointerValue<'ctx>>,
    /// The stack representation.
    pub stack: HashMap<String, inkwell::values::PointerValue<'ctx>>,
}

impl<'ctx> Function<'ctx> {
    /// The stack hashmap default capacity.
    const STACK_HASHMAP_INITIAL_CAPACITY: usize = 64;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        name: String,
        value: inkwell::values::FunctionValue<'ctx>,
        entry_block: inkwell::basic_block::BasicBlock<'ctx>,
        return_block: inkwell::basic_block::BasicBlock<'ctx>,
        return_pointer: Option<inkwell::values::PointerValue<'ctx>>,
    ) -> Self {
        Self {
            name,
            value,
            entry_block,
            return_block,
            return_pointer,
            stack: HashMap::with_capacity(Self::STACK_HASHMAP_INITIAL_CAPACITY),
        }
    }

    ///
    /// Optimizes the function using the pass manager.
    ///
    pub fn optimize(
        &self,
        pass_manager: &inkwell::passes::PassManager<inkwell::values::FunctionValue<'ctx>>,
    ) {
        pass_manager.run_on(&self.value);
    }
}
