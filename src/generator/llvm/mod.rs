//!
//! The LLVM context.
//!

pub mod function;
pub mod r#loop;

use std::collections::HashMap;

use crate::parser::identifier::Identifier;
use crate::target::Target;

use self::function::Function;
use self::r#loop::Loop;

///
/// The LLVM context.
///
pub struct Context<'ctx> {
    /// The target to build for.
    pub target: Target,
    /// The LLVM builder.
    pub builder: inkwell::builder::Builder<'ctx>,
    /// The declared functions.
    pub functions: HashMap<String, Function<'ctx>>,
    /// The heap representation.
    pub heap: Option<inkwell::values::GlobalValue<'ctx>>,

    /// The inner LLVM context.
    llvm: &'ctx inkwell::context::Context,
    /// The current module.
    module: Option<inkwell::module::Module<'ctx>>,
    /// The current object name.
    object: Option<String>,
    /// The current function.
    function: Option<Function<'ctx>>,
    /// The loop context stack.
    loop_stack: Vec<Loop<'ctx>>,

    /// The optimization level.
    optimization_level: inkwell::OptimizationLevel,
    /// The optimization pass manager builder.
    pass_manager_builder: inkwell::passes::PassManagerBuilder,
    /// The link-time optimization pass manager.
    pass_manager_link_time: Option<inkwell::passes::PassManager<inkwell::module::Module<'ctx>>>,
    /// The module optimization pass manager.
    pass_manager_module: Option<inkwell::passes::PassManager<inkwell::module::Module<'ctx>>>,
    /// The function optimization pass manager.
    pass_manager_function:
        Option<inkwell::passes::PassManager<inkwell::values::FunctionValue<'ctx>>>,

    /// The test entry hash.
    pub test_entry_hash: Option<String>,
}

impl<'ctx> Context<'ctx> {
    /// The functions hashmap default capacity.
    const FUNCTION_HASHMAP_INITIAL_CAPACITY: usize = 64;
    /// The loop stack default capacity.
    const LOOP_STACK_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(llvm: &'ctx inkwell::context::Context, target: Target) -> Self {
        Self::new_with_optimizer(llvm, target, inkwell::OptimizationLevel::None)
    }

    ///
    /// Initializes a new LLVM context, setting the optimization level.
    ///
    pub fn new_with_optimizer(
        llvm: &'ctx inkwell::context::Context,
        target: Target,
        optimization_level: inkwell::OptimizationLevel,
    ) -> Self {
        let pass_manager_builder = inkwell::passes::PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(optimization_level);

        Self {
            target,
            builder: llvm.create_builder(),
            functions: HashMap::with_capacity(Self::FUNCTION_HASHMAP_INITIAL_CAPACITY),
            heap: None,

            llvm,
            module: None,
            object: None,
            function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),

            optimization_level,
            pass_manager_builder,
            pass_manager_link_time: None,
            pass_manager_module: None,
            pass_manager_function: None,

            test_entry_hash: None,
        }
    }

    ///
    /// Returns the optimization level.
    ///
    pub fn optimization_level(&self) -> inkwell::OptimizationLevel {
        self.optimization_level
    }

    ///
    /// Optimizes the current module.
    ///
    /// Should be only run when the entire module has been translated.
    ///
    pub fn optimize(&self) {
        let pass_manager_function = self
            .pass_manager_function
            .as_ref()
            .expect("Pass managers are created with the module");
        for (_, function) in self.functions.iter() {
            function.optimize(&pass_manager_function);
        }

        let pass_manager_module = self
            .pass_manager_module
            .as_ref()
            .expect("Pass managers are created with the module");
        pass_manager_module.run_on(self.module());

        let pass_manager_link_time = self
            .pass_manager_link_time
            .as_ref()
            .expect("Pass managers are created with the module");
        pass_manager_link_time.run_on(self.module());
    }

    ///
    /// Verifies the current module.
    ///
    /// # Panics
    /// If verification fails.
    ///
    pub fn verify(&self) -> Result<(), inkwell::support::LLVMString> {
        self.module().verify()
    }

    ///
    /// Creates a new module in the context.
    ///
    pub fn create_module(&mut self, name: &str) {
        let module = self.llvm.create_module(name);

        let pass_manager_link_time = inkwell::passes::PassManager::create(());
        self.pass_manager_builder
            .populate_lto_pass_manager(&pass_manager_link_time, true, true);
        self.pass_manager_link_time = Some(pass_manager_link_time);

        let pass_manager_module = inkwell::passes::PassManager::create(());
        self.pass_manager_builder
            .populate_module_pass_manager(&pass_manager_module);
        self.pass_manager_module = Some(pass_manager_module);

        let pass_manager_function = inkwell::passes::PassManager::create(&module);
        self.pass_manager_builder
            .populate_function_pass_manager(&pass_manager_function);
        self.pass_manager_function = Some(pass_manager_function);

        self.module = Some(module);
    }

    ///
    /// Returns the current module reference.
    ///
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        self.module
            .as_ref()
            .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION)
    }

    ///
    /// Sets the current YUL object name.
    ///
    pub fn set_object(&mut self, name: String) {
        self.object = Some(name);
    }

    ///
    /// Returns the current YUL object name.
    ///
    pub fn object(&self) -> &str {
        self.object.as_ref().expect("Always exists")
    }

    ///
    /// Allocates the heap, if it has not been allocated yet.
    ///
    pub fn allocate_heap(&mut self, size: usize) {
        if self.heap.is_some() {
            return;
        }

        let global = match self.target {
            Target::LLVM => {
                let r#type = self
                    .integer_type(compiler_const::bitlength::BYTE)
                    .array_type(size as u32);
                let global = self.module().add_global(r#type, None, "heap");
                global.set_initializer(&r#type.const_zero());
                global
            }
            Target::zkEVM => {
                let r#type = self
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Local);
                self.module().add_global(r#type, None, "heap")
            }
        };
        self.heap = Some(global);
    }

    ///
    /// Returns the heap pointer with the `offset` bytes offset, optionally casted to `r#type`.
    ///
    pub fn access_heap(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
        r#type: Option<inkwell::types::IntType<'ctx>>,
    ) -> inkwell::values::PointerValue<'ctx> {
        let pointer = self.heap.expect("Always exists").as_pointer_value();
        let mut indexes = Vec::with_capacity(2);
        if let Target::LLVM = self.target {
            indexes.push(
                self.integer_type(compiler_const::bitlength::BYTE * 4)
                    .const_zero(),
            );
        }
        indexes.push(offset);
        let pointer = unsafe { self.builder.build_gep(pointer, indexes.as_slice(), "") };
        let r#type = r#type.unwrap_or_else(|| self.integer_type(compiler_const::bitlength::FIELD));
        let pointer = self.builder.build_pointer_cast(
            pointer,
            r#type.ptr_type(inkwell::AddressSpace::Local),
            "",
        );
        pointer
    }

    ///
    /// Appends a function to the current module.
    ///
    pub fn add_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
        linkage: Option<inkwell::module::Linkage>,
    ) {
        let value = self.module().add_function(name, r#type, linkage);

        let entry_block = self.llvm.append_basic_block(value, "entry");
        let return_block = self.llvm.append_basic_block(value, "return");

        let function = Function::new(name.to_owned(), value, entry_block, return_block, None);
        self.functions.insert(name.to_string(), function.clone());
        self.function = Some(function);
    }

    ///
    /// Returns the current function.
    ///
    pub fn function(&self) -> &Function<'ctx> {
        self.function
            .as_ref()
            .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION)
    }

    ///
    /// Returns the current function as a mutable reference.
    ///
    pub fn function_mut(&mut self) -> &mut Function<'ctx> {
        self.function
            .as_mut()
            .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION)
    }

    ///
    /// Sets the current function.
    ///
    /// # Panics
    /// If the function with `name` does not exist.
    ///
    pub fn set_function(&mut self, name: &str) {
        self.function = Some(
            self.functions
                .get(name)
                .cloned()
                .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION),
        );
    }

    ///
    /// Updates the current function, setting the return and heap pointers.
    ///
    /// # Panics
    /// If the function with `name` does not exist.
    ///
    pub fn update_function(
        &mut self,
        return_pointer: Option<inkwell::values::PointerValue<'ctx>>,
    ) -> Function<'ctx> {
        let name = self.function().name.clone();

        if let Some(return_pointer) = return_pointer {
            self.functions
                .get_mut(name.as_str())
                .expect("Always exists")
                .return_pointer = Some(return_pointer);
            self.function_mut().return_pointer = Some(return_pointer);
        }

        self.function().to_owned()
    }

    ///
    /// Appends a new basic block to the current function.
    ///
    pub fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.llvm.append_basic_block(self.function().value, name)
    }

    ///
    /// Sets the current basic block.
    ///
    pub fn set_basic_block(&mut self, block: inkwell::basic_block::BasicBlock<'ctx>) {
        self.builder.position_at_end(block);
    }

    ///
    /// Pushes a new loop context to the stack.
    ///
    pub fn push_loop(
        &mut self,
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        join_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        self.loop_stack
            .push(Loop::new(body_block, continue_block, join_block));
    }

    ///
    /// Pops the current loop context from the stack.
    ///
    pub fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    ///
    /// Returns the current loop context.
    ///
    pub fn r#loop(&self) -> &Loop<'ctx> {
        self.loop_stack
            .last()
            .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION)
    }

    ///
    /// Builds an unconditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_unconditional_branch(
        &self,
        destination_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        if self
            .builder
            .get_insert_block()
            .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION)
            .get_terminator()
            .is_some()
        {
            return;
        }

        self.builder.build_unconditional_branch(destination_block);
    }

    ///
    /// Returns the integer type of the specified bitlength.
    ///
    pub fn integer_type(&self, bitlength: usize) -> inkwell::types::IntType<'ctx> {
        self.llvm.custom_width_int_type(bitlength as u32)
    }

    ///
    /// Returns the structure type with specified fields.
    ///
    pub fn structure_type(
        &self,
        field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>>,
    ) -> inkwell::types::StructType<'ctx> {
        self.llvm.struct_type(field_types.as_slice(), false)
    }

    ///
    /// Returns the function type for the specified parameters.
    ///
    pub fn function_type(
        &self,
        return_values: &[Identifier],
        argument_types: &[inkwell::types::BasicTypeEnum<'ctx>],
    ) -> inkwell::types::FunctionType<'ctx> {
        if return_values.is_empty() {
            return self.llvm.void_type().fn_type(argument_types, false);
        }

        if return_values.len() == 1 {
            let yul_type = return_values[0].yul_type.to_owned().unwrap_or_default();
            return yul_type.into_llvm(self).fn_type(argument_types, false);
        }

        let return_types: Vec<_> = return_values
            .iter()
            .map(|identifier| {
                let yul_type = identifier.yul_type.to_owned().unwrap_or_default();
                inkwell::types::BasicTypeEnum::IntType(yul_type.into_llvm(self))
            })
            .collect();
        let return_type = self.llvm.struct_type(return_types.as_slice(), false);
        return_type.fn_type(argument_types, false)
    }

    ///
    /// Sets the test entry hash, extracted using the `solc` compiler.
    ///
    pub fn set_test_entry_hash(&mut self, hash: String) {
        self.test_entry_hash = Some(hash);
    }
}
