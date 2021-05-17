//!
//! The LLVM generator context.
//!

pub mod address_space;
pub mod function;
pub mod intrinsic;
pub mod r#loop;

use std::collections::HashMap;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::parser::identifier::Identifier;
use crate::target::Target;

use self::address_space::AddressSpace;
use self::function::r#return::Return as FunctionReturn;
use self::function::Function;
use self::intrinsic::Intrinsic;
use self::r#loop::Loop;

///
/// The LLVM generator context.
///
pub struct Context<'ctx> {
    /// The target to build for.
    pub target: Target,
    /// The LLVM builder.
    pub builder: inkwell::builder::Builder<'ctx>,
    /// The declared functions.
    pub functions: HashMap<String, Function<'ctx>>,

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

    /// The test heap representation.
    pub heap: Option<inkwell::values::GlobalValue<'ctx>>,
    /// The test contract storage representation.
    pub storage: Option<inkwell::values::GlobalValue<'ctx>>,
    /// The test calldata representation.
    pub calldata: Option<inkwell::values::GlobalValue<'ctx>>,
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

            heap: None,
            storage: None,
            calldata: None,
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
        let revert_block = self.llvm.append_basic_block(value, "revert");
        let return_block = self.llvm.append_basic_block(value, "return");

        let function = Function::new(
            name.to_owned(),
            value,
            entry_block,
            revert_block,
            return_block,
            None,
        );
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
    /// Updates the current function, setting the return entity.
    ///
    /// # Panics
    /// If the function with `name` does not exist.
    ///
    pub fn update_function(&mut self, r#return: FunctionReturn<'ctx>) -> Function<'ctx> {
        let name = self.function().name.clone();

        self.functions
            .get_mut(name.as_str())
            .expect("Always exists")
            .r#return = Some(r#return.clone());
        self.function_mut().r#return = Some(r#return);

        self.function().to_owned()
    }

    ///
    /// Returns the specified intrinsic function.
    ///
    pub fn get_intrinsic_function(
        &self,
        intrinsic: Intrinsic,
    ) -> inkwell::values::FunctionValue<'ctx> {
        self.module()
            .get_intrinsic_function(intrinsic.name(), intrinsic.argument_types(self).as_slice())
            .expect(compiler_const::panic::VALIDATED_DURING_CODE_GENERATION)
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
    /// Returns the current basic block.
    ///
    pub fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.builder.get_insert_block().expect("Always exists")
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
    /// Builds a stack allocation instruction.
    ///
    /// Sets the alignment to 256 bits.
    ///
    pub fn build_alloca<T: BasicType<'ctx>>(
        &self,
        r#type: T,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        let pointer = self.builder.build_alloca(r#type, name);
        if let Target::zkEVM = self.target {
            self.basic_block()
                .get_last_instruction()
                .expect("Always exists")
                .set_alignment(compiler_const::size::FIELD as u32)
                .expect("Alignment is valid");
        }
        pointer
    }

    ///
    /// Builds a stack store instruction.
    ///
    /// Sets the alignment to 256 bits.
    ///
    pub fn build_store<V: BasicValue<'ctx>>(
        &self,
        pointer: inkwell::values::PointerValue<'ctx>,
        value: V,
    ) {
        let instruction = self.builder.build_store(pointer, value);
        if let Target::zkEVM = self.target {
            instruction
                .set_alignment(compiler_const::size::FIELD as u32)
                .expect("Alignment is valid");
        }
    }

    ///
    /// Builds a stack load instruction.
    ///
    /// Sets the alignment to 256 bits.
    ///
    pub fn build_load(
        &self,
        pointer: inkwell::values::PointerValue<'ctx>,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let value = self.builder.build_load(pointer, name);
        if let Target::zkEVM = self.target {
            self.basic_block()
                .get_last_instruction()
                .expect("Always exists")
                .set_alignment(compiler_const::size::FIELD as u32)
                .expect("Alignment is valid");
        }
        value
    }

    ///
    /// Builds a conditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_conditional_branch(
        &self,
        comparison: inkwell::values::IntValue<'ctx>,
        then_block: inkwell::basic_block::BasicBlock<'ctx>,
        else_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder
            .build_conditional_branch(comparison, then_block, else_block);
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
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder.build_unconditional_branch(destination_block);
    }

    ///
    /// Builds a return.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_return(&self, value: Option<&dyn BasicValue<'ctx>>) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder.build_return(value);
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
        mut argument_types: Vec<inkwell::types::BasicTypeEnum<'ctx>>,
    ) -> inkwell::types::FunctionType<'ctx> {
        if return_values.is_empty() {
            return self
                .llvm
                .void_type()
                .fn_type(argument_types.as_slice(), false);
        }

        if return_values.len() == 1 {
            let yul_type = return_values[0].yul_type.to_owned().unwrap_or_default();
            return yul_type
                .into_llvm(self)
                .fn_type(argument_types.as_slice(), false);
        }

        let return_types: Vec<_> = return_values
            .iter()
            .map(|identifier| {
                let yul_type = identifier.yul_type.to_owned().unwrap_or_default();
                inkwell::types::BasicTypeEnum::IntType(yul_type.into_llvm(self))
            })
            .collect();
        let return_type = self
            .llvm
            .struct_type(return_types.as_slice(), false)
            .ptr_type(AddressSpace::Stack.into());
        argument_types.insert(0, return_type.as_basic_type_enum());
        return_type.fn_type(argument_types.as_slice(), false)
    }

    ///
    /// Returns the heap pointer with the `offset` bytes offset, optionally casted to `r#type`.
    ///
    /// Mostly for testing.
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
            r#type.ptr_type(AddressSpace::Stack.into()),
            "",
        );
        pointer
    }

    ///
    /// Returns the storage pointer with the `offset` fields offset.
    ///
    /// Only for testing.
    ///
    pub fn access_storage(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        let pointer = self.storage.expect("Always exists").as_pointer_value();
        let indexes = vec![
            self.integer_type(compiler_const::bitlength::BYTE * 4)
                .const_zero(),
            offset,
        ];
        let pointer = unsafe { self.builder.build_gep(pointer, indexes.as_slice(), "") };
        pointer
    }

    ///
    /// Returns the calldata with the `offset` fields offset.
    ///
    pub fn access_calldata(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        match self.target {
            Target::LLVM => {
                let pointer = self.calldata.expect("Always exists").as_pointer_value();
                let indexes = vec![
                    self.integer_type(compiler_const::bitlength::BYTE * 4)
                        .const_zero(),
                    offset,
                ];
                let pointer = unsafe { self.builder.build_gep(pointer, indexes.as_slice(), "") };
                pointer
            }
            Target::zkEVM => {
                let pointer = self
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(AddressSpace::Child.into())
                    .const_zero();
                let pointer = unsafe { self.builder.build_gep(pointer, &[offset], "") };
                pointer
            }
        }
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
                    .ptr_type(AddressSpace::Stack.into());
                self.module().add_global(r#type, None, "heap")
            }
        };
        self.heap = Some(global);
    }

    ///
    /// Allocates the contract storage, if it has not been allocated yet.
    ///
    pub fn allocate_storage(&mut self, size: usize) {
        if !matches!(self.target, Target::LLVM) {
            return;
        }

        if self.storage.is_some() {
            return;
        }

        let r#type = self
            .integer_type(compiler_const::bitlength::FIELD)
            .array_type(size as u32);
        let global = self.module().add_global(r#type, None, "storage");
        global.set_initializer(&r#type.const_zero());
        self.storage = Some(global);
    }

    ///
    /// Allocates the calldata, if it has not been allocated yet.
    ///
    pub fn allocate_calldata(&mut self, size: usize) {
        if !matches!(self.target, Target::LLVM) {
            return;
        }

        if self.calldata.is_some() {
            return;
        }

        let r#type = self
            .integer_type(compiler_const::bitlength::FIELD)
            .array_type(size as u32);
        let global = self.module().add_global(r#type, None, "calldata");
        global.set_initializer(&r#type.const_zero());
        self.calldata = Some(global);
    }

    ///
    /// Sets the test entry hash, extracted using the `solc` compiler.
    ///
    /// Only for testing.
    ///
    pub fn set_test_entry_hash(&mut self, hash: String) {
        self.test_entry_hash = Some(hash);
    }

    ///
    /// Marks all functions except the specified `entry` with private linkage.
    ///
    /// Only for testing.
    ///
    pub fn set_test_linkage(&mut self, entry: &str) {
        for (_type_id, function) in self.functions.iter_mut() {
            function
                .value
                .set_linkage(if function.name.as_str() == entry {
                    inkwell::module::Linkage::External
                } else {
                    inkwell::module::Linkage::Private
                });
        }
    }
}
