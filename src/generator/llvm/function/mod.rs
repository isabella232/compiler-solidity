//!
//! The LLVM generator function.
//!

pub mod r#return;

use std::collections::HashMap;

use self::r#return::Return;

///
/// The LLVM generator function.
///
#[derive(Debug, Clone)]
pub struct Function<'ctx> {
    /// The name.
    pub name: String,
    /// The LLVM value.
    pub value: inkwell::values::FunctionValue<'ctx>,
    /// The entry block.
    pub entry_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The throw/revert block.
    pub throw_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The catch block.
    pub catch_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return/leave block.
    pub return_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return value entity.
    pub r#return: Option<Return<'ctx>>,
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
        throw_block: inkwell::basic_block::BasicBlock<'ctx>,
        catch_block: inkwell::basic_block::BasicBlock<'ctx>,
        return_block: inkwell::basic_block::BasicBlock<'ctx>,
        r#return: Option<Return<'ctx>>,
    ) -> Self {
        Self {
            name,
            value,
            entry_block,
            throw_block,
            catch_block,
            return_block,
            r#return,
            stack: HashMap::with_capacity(Self::STACK_HASHMAP_INITIAL_CAPACITY),
        }
    }

    ///
    /// Sets the function return data.
    ///
    pub fn set_return(&mut self, r#return: Return<'ctx>) {
        self.r#return = Some(r#return);
    }

    ///
    /// Returns the pointer to the function return value.
    ///
    /// # Panics
    /// If the pointer has not been set yet.
    ///
    pub fn return_pointer(&self) -> Option<inkwell::values::PointerValue<'ctx>> {
        self.r#return
            .as_ref()
            .expect("Always exists")
            .return_pointer()
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
