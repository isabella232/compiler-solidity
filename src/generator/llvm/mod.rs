//!
//! The LLVM generator context.
//!

pub mod argument;
pub mod function;
pub mod intrinsic;
pub mod r#loop;

use std::collections::HashMap;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::parser::identifier::Identifier;
use crate::project::Project;

use self::function::r#return::Return as FunctionReturn;
use self::function::Function;
use self::intrinsic::Intrinsic;
use self::r#loop::Loop;

///
/// The LLVM generator context.
///
pub struct Context<'ctx, 'src> {
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

    /// The personality function, used for exception handling.
    personality: inkwell::values::FunctionValue<'ctx>,
    /// The exception throwing function.
    cxa_throw: inkwell::values::FunctionValue<'ctx>,

    /// The project source data.
    project: &'src mut Project,
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
}

impl<'ctx, 'src> Context<'ctx, 'src> {
    /// The functions hashmap default capacity.
    const FUNCTION_HASHMAP_INITIAL_CAPACITY: usize = 64;
    /// The loop stack default capacity.
    const LOOP_STACK_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        machine: &inkwell::targets::TargetMachine,
        identifier: &str,
        project: &'src mut Project,
    ) -> Self {
        Self::new_with_optimizer(
            llvm,
            machine,
            inkwell::OptimizationLevel::None,
            identifier,
            project,
        )
    }

    ///
    /// Initializes a new LLVM context, setting the optimization level.
    ///
    pub fn new_with_optimizer(
        llvm: &'ctx inkwell::context::Context,
        machine: &inkwell::targets::TargetMachine,
        optimization_level: inkwell::OptimizationLevel,
        identifier: &str,
        project: &'src mut Project,
    ) -> Self {
        let module = llvm.create_module(identifier);
        module.set_triple(&machine.get_triple());
        module.set_data_layout(&machine.get_target_data().get_data_layout());

        let internalize = matches!(optimization_level, inkwell::OptimizationLevel::Aggressive);
        let run_inliner = matches!(optimization_level, inkwell::OptimizationLevel::Aggressive);

        let pass_manager_builder = inkwell::passes::PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(optimization_level);
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
            builder: llvm.create_builder(),
            functions: HashMap::with_capacity(Self::FUNCTION_HASHMAP_INITIAL_CAPACITY),

            llvm,
            module,
            object: None,
            function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),

            personality,
            cxa_throw,

            project,
            optimization_level,
            pass_manager_module,
            pass_manager_function,

            is_native_bitwise_supported: true,
            is_unaligned_memory_access_supported: false,
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
        for index in 0..value.count_params() {
            if value
                .get_nth_param(index)
                .map(|argument| argument.get_type().is_pointer_type())
                .unwrap_or_default()
            {
                value.set_param_alignment(index, compiler_common::size::FIELD as u32);
            }
        }

        value.set_personality_function(self.personality);

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
    /// Gets a deployed library address.
    ///
    pub fn get_library_address(&self, path: &str) -> Option<inkwell::values::IntValue<'ctx>> {
        for (file_path, contracts) in self.project.libraries.iter() {
            for (contract_name, address) in contracts.iter() {
                let key = format!("{}:{}", file_path, contract_name);
                if key.as_str() == path {
                    return Some(
                        self.llvm
                            .custom_width_int_type(compiler_common::bitlength::FIELD as u32)
                            .const_int_from_string(
                                &address["0x".len()..],
                                inkwell::types::StringRadix::Hexadecimal,
                            )
                            .unwrap_or_else(|| {
                                panic!("Library `{}` address `{}` is invalid", key, address)
                            }),
                    );
                }
            }
        }

        None
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
    pub fn compile_dependency(&mut self, identifier: &str) -> Vec<u8> {
        let contract_path = self
            .project
            .contracts
            .iter()
            .find_map(|(path, contract)| {
                if contract.object.identifier.as_str() == identifier {
                    Some(path.to_owned())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("Dependency `{}` not found", identifier));

        let (_assembly_text, bytecode) = self
            .project
            .compile(
                Some(contract_path.as_str()),
                self.optimization_level,
                self.optimization_level,
                false,
            )
            .unwrap_or_else(|error| {
                panic!("Dependency `{}` compiling error: {:?}", identifier, error)
            });

        bytecode
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
        self.basic_block()
            .get_last_instruction()
            .expect("Always exists")
            .set_alignment(compiler_common::size::FIELD as u32)
            .expect("Alignment is valid");
        pointer
    }

    ///
    /// Builds a stack store instruction.
    ///
    /// Sets the alignment to 256 bits for stack and 1 bit for heap, parent, and child.
    ///
    pub fn build_store<V: BasicValue<'ctx>>(
        &self,
        pointer: inkwell::values::PointerValue<'ctx>,
        value: V,
    ) {
        let instruction = self.builder.build_store(pointer, value);

        let alignment = if inkwell::AddressSpace::from(compiler_common::AddressSpace::Stack)
            == pointer.get_type().get_address_space()
        {
            compiler_common::size::FIELD
        } else {
            1
        };

        instruction
            .set_alignment(alignment as u32)
            .expect("Alignment is valid");
    }

    ///
    /// Builds a stack load instruction.
    ///
    /// Sets the alignment to 256 bits for stack and 1 bit for heap, parent, and child.
    ///
    pub fn build_load(
        &self,
        pointer: inkwell::values::PointerValue<'ctx>,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let value = self.builder.build_load(pointer, name);

        let alignment = if inkwell::AddressSpace::from(compiler_common::AddressSpace::Stack)
            == pointer.get_type().get_address_space()
        {
            compiler_common::size::FIELD
        } else {
            1
        };

        self.basic_block()
            .get_last_instruction()
            .expect("Always exists")
            .set_alignment(alignment as u32)
            .expect("Alignment is valid");
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
    /// Builds a memory copy call.
    ///
    /// Sets the alignment to 1 bit for heap, parent, and child.
    ///
    pub fn build_memcpy(
        &self,
        intrinsic: Intrinsic,
        destination: inkwell::values::PointerValue<'ctx>,
        source: inkwell::values::PointerValue<'ctx>,
        size: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) {
        let intrinsic = self.get_intrinsic_function(intrinsic);

        let call_site_value = self.builder.build_call(
            intrinsic,
            &[
                destination.as_basic_value_enum(),
                source.as_basic_value_enum(),
                size.as_basic_value_enum(),
                self.integer_type(compiler_common::bitlength::BOOLEAN)
                    .const_zero()
                    .as_basic_value_enum(),
            ],
            name,
        );

        call_site_value.set_alignment_attribute(inkwell::attributes::AttributeLoc::Param(0), 1);
        call_site_value.set_alignment_attribute(inkwell::attributes::AttributeLoc::Param(1), 1);
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
    /// Reads the data size from the specified memory.
    ///
    pub fn read_header(
        &self,
        address_space: compiler_common::AddressSpace,
    ) -> inkwell::values::IntValue<'ctx> {
        let header_pointer = self.access_memory(
            self.field_const(
                (compiler_common::abi::OFFSET_HEADER * compiler_common::size::FIELD) as u64,
            ),
            address_space,
            "header_pointer",
        );
        self.build_load(header_pointer, "header_value")
            .into_int_value()
    }

    ///
    /// Writes the data size to the specified memory.
    ///
    pub fn write_header(
        &self,
        header: inkwell::values::IntValue<'ctx>,
        address_space: compiler_common::AddressSpace,
    ) {
        let header_pointer = self.access_memory(
            self.field_const(
                (compiler_common::abi::OFFSET_HEADER * compiler_common::size::FIELD) as u64,
            ),
            address_space,
            "header_pointer",
        );
        self.build_store(header_pointer, header);
    }

    ///
    /// Writes the error data to the parent memory.
    ///
    pub fn write_error(&self, message: &'static str) {
        self.write_header(self.field_const(1), compiler_common::AddressSpace::Parent);

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
        let parent_error_code_pointer = self.access_memory(
            self.field_const(
                (compiler_common::abi::OFFSET_DATA * compiler_common::size::FIELD) as u64,
            ),
            compiler_common::AddressSpace::Parent,
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
    /// Returns the memory pointer to `address_space at the `offset` bytes.
    ///
    pub fn access_memory(
        &self,
        offset: inkwell::values::IntValue<'ctx>,
        address_space: compiler_common::AddressSpace,
        name: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        self.builder.build_int_to_ptr(
            offset,
            self.field_type().ptr_type(address_space.into()),
            name,
        )
    }
}
