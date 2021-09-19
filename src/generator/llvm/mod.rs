//!
//! The LLVM generator context.
//!

pub mod argument;
pub mod dependency;
pub mod function;
pub mod intrinsic;
pub mod r#loop;

use std::collections::HashMap;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::parser::identifier::Identifier;
use crate::parser::statement::object::Object;
use crate::target::Target;

use self::dependency::Dependency;
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
    module: inkwell::module::Module<'ctx>,
    /// The current object.
    object: Option<String>,
    /// The current function.
    function: Option<Function<'ctx>>,
    /// The loop context stack.
    loop_stack: Vec<Loop<'ctx>>,
    /// The main contract dependencies.
    dependencies: HashMap<String, Dependency>,

    /// The personality function, used for exception handling.
    personality: inkwell::values::FunctionValue<'ctx>,
    /// The exception throwing function.
    cxa_throw: inkwell::values::FunctionValue<'ctx>,

    /// The optimization level.
    optimization_level: inkwell::OptimizationLevel,
    /// The module optimization pass manager.
    pass_manager_module: inkwell::passes::PassManager<inkwell::module::Module<'ctx>>,
    /// The function optimization pass manager.
    pass_manager_function: inkwell::passes::PassManager<inkwell::values::FunctionValue<'ctx>>,

    /// Whether the native bitwise operations are supported by the back-end.
    pub is_native_bitwise_supported: bool,
    /// Whether the unaligned memory access is supported by the back-end.
    pub is_unaligned_memory_access_supported: bool,

    /// The test heap representation.
    heap: Option<inkwell::values::GlobalValue<'ctx>>,
    /// The test contract storage representation.
    storage: Option<inkwell::values::GlobalValue<'ctx>>,
    /// The test calldata representation.
    calldata: Option<inkwell::values::GlobalValue<'ctx>>,
}

impl<'ctx> Context<'ctx> {
    /// The functions hashmap default capacity.
    const FUNCTION_HASHMAP_INITIAL_CAPACITY: usize = 64;
    /// The loop stack default capacity.
    const LOOP_STACK_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        machine: Option<&inkwell::targets::TargetMachine>,
        identifier: &str,
        dependencies: HashMap<String, Object>,
    ) -> Self {
        Self::new_with_optimizer(
            llvm,
            machine,
            inkwell::OptimizationLevel::None,
            identifier,
            dependencies,
        )
    }

    ///
    /// Initializes a new LLVM context, setting the optimization level.
    ///
    pub fn new_with_optimizer(
        llvm: &'ctx inkwell::context::Context,
        machine: Option<&inkwell::targets::TargetMachine>,
        optimization_level: inkwell::OptimizationLevel,
        identifier: &str,
        dependencies: HashMap<String, Object>,
    ) -> Self {
        let module = llvm.create_module(identifier);
        if let Some(machine) = machine {
            module.set_triple(&machine.get_triple());
            module.set_data_layout(&machine.get_target_data().get_data_layout());
        }

        let dependencies = dependencies
            .into_iter()
            .map(|(identifier, object)| (identifier, Dependency::new_object(object)))
            .collect();

        let internalize = matches!(optimization_level, inkwell::OptimizationLevel::Aggressive);
        let run_inliner = matches!(optimization_level, inkwell::OptimizationLevel::Aggressive);

        let pass_manager_builder = inkwell::passes::PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(optimization_level);
        pass_manager_builder.set_inliner_with_threshold(0);
        pass_manager_builder.set_disable_unroll_loops(matches!(
            optimization_level,
            inkwell::OptimizationLevel::Aggressive
        ));

        let pass_manager_module = inkwell::passes::PassManager::create(());
        pass_manager_builder.populate_lto_pass_manager(
            &pass_manager_module,
            internalize,
            run_inliner,
        );
        pass_manager_builder.populate_module_pass_manager(&pass_manager_module);

        let pass_manager_function = inkwell::passes::PassManager::create(&module);
        pass_manager_builder.populate_function_pass_manager(&pass_manager_function);

        let personality = module.add_function(
            compiler_common::identifier::FUNCTION_PERSONALITY,
            llvm.i32_type().fn_type(&[], false),
            None,
        );

        let cxa_throw = module.add_function(
            compiler_common::identifier::FUNCTION_CXA_THROW,
            llvm.void_type().fn_type(
                vec![
                    llvm.i8_type()
                        .ptr_type(compiler_common::AddressSpace::Stack.into())
                        .as_basic_type_enum();
                    3
                ]
                .as_slice(),
                false,
            ),
            Some(inkwell::module::Linkage::External),
        );
        cxa_throw.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(27, 0),
        );

        Self {
            target: machine.into(),
            builder: llvm.create_builder(),
            functions: HashMap::with_capacity(Self::FUNCTION_HASHMAP_INITIAL_CAPACITY),

            llvm,
            module,
            object: None,
            function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),
            dependencies,

            personality,
            cxa_throw,

            optimization_level,
            pass_manager_module,
            pass_manager_function,

            is_native_bitwise_supported: true,
            is_unaligned_memory_access_supported: false,

            heap: None,
            storage: None,
            calldata: None,
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
        for (_, function) in self.functions.iter() {
            function.optimize(&self.pass_manager_function);
        }
        self.pass_manager_module.run_on(self.module());
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
    /// Returns the current module reference.
    ///
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    ///
    /// Sets the current object name.
    ///
    pub fn set_object(&mut self, name: &str) {
        self.object = Some(name.to_owned());
    }

    ///
    /// Returns the current object name.
    ///
    pub fn object(&self) -> &str {
        self.object.as_deref().expect("Must exist at this point")
    }

    ///
    /// Appends a function to the current module.
    ///
    pub fn add_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
        linkage: Option<inkwell::module::Linkage>,
        set_current: bool,
    ) {
        let value = self.module().add_function(name, r#type, linkage);
        if let Target::zkEVM = self.target {
            for index in 0..value.count_params() {
                if value
                    .get_nth_param(index)
                    .map(|argument| argument.get_type().is_pointer_type())
                    .unwrap_or_default()
                {
                    value.set_param_alignment(index, compiler_common::size::FIELD as u32);
                }
            }
        }

        if let Target::zkEVM = self.target {
            value.set_personality_function(self.personality);
        }

        let entry_block = self.llvm.append_basic_block(value, "entry");
        let throw_block = self.llvm.append_basic_block(value, "throw");
        let catch_block = self.llvm.append_basic_block(value, "catch");
        let return_block = self.llvm.append_basic_block(value, "return");

        let function = Function::new(
            name.to_owned(),
            value,
            entry_block,
            throw_block,
            catch_block,
            return_block,
            None,
        );
        self.functions.insert(name.to_string(), function.clone());
        if set_current {
            self.function = Some(function);
        }
    }

    ///
    /// Returns the current function.
    ///
    pub fn function(&self) -> &Function<'ctx> {
        self.function.as_ref().expect("Must be declared before use")
    }

    ///
    /// Returns the current function as a mutable reference.
    ///
    pub fn function_mut(&mut self) -> &mut Function<'ctx> {
        self.function.as_mut().expect("Must be declared before use")
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
                .expect("Must be declared before use"),
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
            .unwrap_or_else(|| panic!("Intrinsic function `{}` does not exist", intrinsic.name()))
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
    pub fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>) {
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
            .expect("The current context is not in a loop")
    }

    ///
    /// Compiles the dependency object.
    ///
    pub fn compile_dependency(&mut self, identifier: &str) -> &[u8] {
        let dependency = self
            .dependencies
            .remove(identifier)
            .unwrap_or_else(|| panic!("Dependency `{}` not found", identifier));
        let bytecode = dependency.compile(self.target, self.optimization_level);
        self.dependencies
            .insert(identifier.to_owned(), Dependency::new_bytecode(bytecode));
        match self.dependencies.get(identifier).expect("Always exists") {
            Dependency::Parsed(_object) => panic!("Dependency is always compiled at this point"),
            Dependency::Compiled(bytecode) => bytecode.as_slice(),
        }
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
                .set_alignment(compiler_common::size::FIELD as u32)
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
                .set_alignment(compiler_common::size::FIELD as u32)
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
                .set_alignment(compiler_common::size::FIELD as u32)
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
    /// Builds a call.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_call(
        &self,
        function: inkwell::values::FunctionValue<'ctx>,
        args: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        let call_site_value = self.builder.build_call(function, args, name);

        if let Target::x86 = self.target {
            return call_site_value.try_as_basic_value().left();
        }

        if name == compiler_common::identifier::FUNCTION_CXA_THROW {
            return call_site_value.try_as_basic_value().left();
        }

        for index in 0..function.count_params() {
            if function
                .get_nth_param(index)
                .map(|argument| argument.get_type().is_pointer_type())
                .unwrap_or_default()
            {
                call_site_value.set_alignment_attribute(
                    inkwell::attributes::AttributeLoc::Param(index),
                    compiler_common::size::FIELD as u32,
                );
            }
        }

        if call_site_value
            .try_as_basic_value()
            .map_left(|value| value.is_pointer_value())
            .left_or_default()
        {
            call_site_value.set_alignment_attribute(
                inkwell::attributes::AttributeLoc::Return,
                compiler_common::size::FIELD as u32,
            );
        }

        call_site_value.try_as_basic_value().left()
    }

    ///
    /// Builds an invoke.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_invoke(
        &self,
        function: inkwell::values::FunctionValue<'ctx>,
        args: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        if let Target::x86 = self.target {
            return self.build_call(function, args, name);
        }

        let join_block = self.append_basic_block("join");

        let call_site_value = self.builder.build_invoke(
            function,
            args,
            join_block,
            self.function().catch_block,
            name,
        );

        for index in 0..function.count_params() {
            if function
                .get_nth_param(index)
                .map(|argument| argument.get_type().is_pointer_type())
                .unwrap_or_default()
            {
                call_site_value.set_alignment_attribute(
                    inkwell::attributes::AttributeLoc::Param(index),
                    compiler_common::size::FIELD as u32,
                );
            }
        }

        if call_site_value
            .try_as_basic_value()
            .map_left(|value| value.is_pointer_value())
            .left_or_default()
        {
            call_site_value.set_alignment_attribute(
                inkwell::attributes::AttributeLoc::Return,
                compiler_common::size::FIELD as u32,
            );
        }

        self.set_basic_block(join_block);

        call_site_value.try_as_basic_value().left()
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
    /// Builds an unreachable.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_unreachable(&self) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder.build_unreachable();
    }

    ///
    /// Builds an exception catching block sequence.
    ///
    pub fn build_catch_block(&self) -> Option<inkwell::values::AnyValueEnum<'ctx>> {
        if !matches!(self.target, Target::zkEVM) {
            return None;
        }

        let landing_pad_type = self.structure_type(vec![
            self.integer_type(compiler_common::bitlength::BYTE)
                .ptr_type(compiler_common::AddressSpace::Stack.into())
                .as_basic_type_enum(),
            self.integer_type(compiler_common::bitlength::BYTE * 4)
                .as_basic_type_enum(),
        ]);
        let landing_pad = self.builder.build_landing_pad(
            landing_pad_type,
            self.personality,
            vec![self
                .integer_type(compiler_common::bitlength::BYTE)
                .ptr_type(compiler_common::AddressSpace::Stack.into())
                .const_zero()
                .as_basic_value_enum()],
            "landing",
        );

        self.build_call(
            self.cxa_throw,
            vec![
                self.integer_type(compiler_common::bitlength::BYTE)
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .const_null()
                    .as_basic_value_enum();
                3
            ]
            .as_slice(),
            compiler_common::identifier::FUNCTION_CXA_THROW,
        );

        Some(landing_pad)
    }

    ///
    /// Builds an error throwing block sequence.
    ///
    pub fn build_throw_block(&self) {
        if !matches!(self.target, Target::zkEVM) {
            return;
        }

        self.build_call(
            self.cxa_throw,
            vec![
                self.integer_type(compiler_common::bitlength::BYTE)
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .const_null()
                    .as_basic_value_enum();
                3
            ]
            .as_slice(),
            compiler_common::identifier::FUNCTION_CXA_THROW,
        );
    }

    ///
    /// Writes the error data to the parent memory.
    ///
    pub fn write_error(&self, message: &'static str) {
        let parent_return_data_size_pointer = self.access_calldata(
            self.field_const(
                (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD)
                    as u64,
            ),
            "parent_return_data_size_pointer",
        );
        self.build_store(parent_return_data_size_pointer, self.field_const(1));

        let error_hash = compiler_common::hashes::keccak256(message.as_bytes());
        let error_code = self
            .field_type()
            .const_int_from_string(
                error_hash.as_str(),
                inkwell::types::StringRadix::Hexadecimal,
            )
            .expect("Always valid");
        let error_code_shifted = self.builder.build_left_shift(
            error_code,
            self.field_const(
                (compiler_common::bitlength::BYTE * (compiler_common::size::FIELD - 4)) as u64,
            ),
            "error_code_shifted",
        );
        let parent_error_code_pointer = self.access_calldata(
            self.field_const(
                (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD)
                    as u64,
            ),
            "parent_error_code_pointer",
        );
        self.build_store(parent_error_code_pointer, error_code_shifted);
    }

    ///
    /// Returns a field type constants.
    ///
    pub fn field_const(&self, value: u64) -> inkwell::values::IntValue<'ctx> {
        self.field_type().const_int(value, false)
    }

    ///
    /// Returns the void type.
    ///
    pub fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.llvm.void_type()
    }

    ///
    /// Returns the integer type of the specified bitlength.
    ///
    pub fn integer_type(&self, bitlength: usize) -> inkwell::types::IntType<'ctx> {
        self.llvm.custom_width_int_type(bitlength as u32)
    }

    ///
    /// Returns the default field type.
    ///
    pub fn field_type(&self) -> inkwell::types::IntType<'ctx> {
        self.llvm
            .custom_width_int_type(compiler_common::bitlength::FIELD as u32)
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
            .ptr_type(compiler_common::AddressSpace::Stack.into());
        argument_types.insert(0, return_type.as_basic_type_enum());
        return_type.fn_type(argument_types.as_slice(), false)
    }

    ///
    /// Adjusts the specified offset to the beginning of the next 32-byte cell.
    ///
    pub fn adjust_offset(
        &self,
        initial: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> inkwell::values::IntValue<'ctx> {
        let remainder = self.builder.build_int_unsigned_rem(
            initial,
            self.field_const(compiler_common::size::FIELD as u64),
            format!("{}_remainder", name).as_str(),
        );
        let adjustment = self.builder.build_int_sub(
            self.field_const(compiler_common::size::FIELD as u64),
            remainder,
            format!("{}_adjustment", name).as_str(),
        );
        let adjustment_remainder = self.builder.build_int_unsigned_rem(
            adjustment,
            self.field_const(compiler_common::size::FIELD as u64),
            format!("{}_adjustment_remainder", name).as_str(),
        );
        let adjusted = self.builder.build_int_add(
            initial,
            adjustment_remainder,
            format!("{}_adjusted", name).as_str(),
        );
        adjusted
    }

    ///
    /// Returns the heap pointer with the `offset` bytes offset, optionally casted to `r#type`.
    ///
    /// Mostly for testing.
    ///
    pub fn access_heap(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        match self.target {
            Target::x86 => {
                let pointer = self.heap.expect("Always exists").as_pointer_value();
                let pointer = self.builder.build_pointer_cast(
                    pointer,
                    self.field_type()
                        .ptr_type(compiler_common::AddressSpace::Heap.into()),
                    format!("{}_casted", name).as_str(),
                );
                let pointer = unsafe {
                    self.builder.build_gep(
                        pointer,
                        &[self.field_const(0), offset],
                        format!("{}_shifted", name).as_str(),
                    )
                };
                pointer
            }
            Target::zkEVM => self.builder.build_int_to_ptr(
                offset,
                self.field_type()
                    .ptr_type(compiler_common::AddressSpace::Heap.into()),
                name,
            ),
        }
    }

    ///
    /// Returns the storage pointer with the `offset` fields offset.
    ///
    /// Only for testing.
    ///
    pub fn access_storage(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        let pointer = self.storage.expect("Always exists").as_pointer_value();
        let indexes = vec![self.field_const(0), offset];
        let pointer = unsafe {
            self.builder.build_gep(
                pointer,
                indexes.as_slice(),
                format!("{}_shifted", name).as_str(),
            )
        };
        pointer
    }

    ///
    /// Returns the calldata with the `offset` fields offset.
    ///
    pub fn access_calldata(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        match self.target {
            Target::x86 => {
                let pointer = self.calldata.expect("Always exists").as_pointer_value();
                let pointer = unsafe {
                    self.builder.build_gep(
                        pointer,
                        &[self.field_const(0), offset],
                        format!("{}_shifted", name).as_str(),
                    )
                };
                pointer
            }
            Target::zkEVM => self.builder.build_int_to_ptr(
                offset,
                self.field_type()
                    .ptr_type(compiler_common::AddressSpace::Parent.into()),
                name,
            ),
        }
    }

    ///
    /// Allocates the heap, if it has not been allocated yet.
    ///
    pub fn allocate_heap(&mut self, size: usize) {
        if !matches!(self.target, Target::x86) {
            return;
        }

        if self.heap.is_some() {
            return;
        }

        let r#type = self
            .integer_type(compiler_common::bitlength::BYTE)
            .array_type(size as u32);
        let global = self.module().add_global(r#type, None, "heap");
        global.set_initializer(&r#type.const_zero());
        self.heap = Some(global);
    }

    ///
    /// Allocates the contract storage, if it has not been allocated yet.
    ///
    pub fn allocate_storage(&mut self, size: usize) {
        if !matches!(self.target, Target::x86) {
            return;
        }

        if self.storage.is_some() {
            return;
        }

        let r#type = self.field_type().array_type(size as u32);
        let global = self.module().add_global(r#type, None, "storage");
        global.set_initializer(&r#type.const_zero());
        self.storage = Some(global);
    }

    ///
    /// Allocates the calldata, if it has not been allocated yet.
    ///
    pub fn allocate_calldata(&mut self, size: usize) {
        if !matches!(self.target, Target::x86) {
            return;
        }

        if self.calldata.is_some() {
            return;
        }

        let r#type = self.field_type().array_type(size as u32);
        let global = self.module().add_global(r#type, None, "calldata");
        global.set_initializer(&r#type.const_zero());
        self.calldata = Some(global);
    }
}
