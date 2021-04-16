//!
//! The LLVM context.
//!

use std::collections::HashMap;

use crate::parser::identifier::Identifier;

///
/// The LLVM context.
///
pub struct Context<'ctx> {
    /// The inner LLVM context.
    pub llvm: &'ctx inkwell::context::Context,
    /// The LLVM builder.
    pub builder: inkwell::builder::Builder<'ctx>,
    /// The current module.
    pub module: Option<inkwell::module::Module<'ctx>>,
    /// The current function.
    pub function: Option<inkwell::values::FunctionValue<'ctx>>,

    /// The block where the `continue` statement jumps to.
    pub continue_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// The block where the `break` statement jumps to.
    pub break_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// The block where the `leave` statement jumps to.
    pub leave_block: Option<inkwell::basic_block::BasicBlock<'ctx>>,

    /// The declared variables.
    pub variables: HashMap<String, inkwell::values::PointerValue<'ctx>>,
    /// The declared functions.
    pub functions: HashMap<String, inkwell::values::FunctionValue<'ctx>>,

    /// The optimization level.
    optimization_level: inkwell::OptimizationLevel,
    /// The optimization pass manager builder.
    pass_manager_builder: inkwell::passes::PassManagerBuilder,
    /// The module optimization pass manager.
    pass_manager_module: Option<inkwell::passes::PassManager<inkwell::module::Module<'ctx>>>,
    /// The function optimization pass manager.
    pass_manager_function:
        Option<inkwell::passes::PassManager<inkwell::values::FunctionValue<'ctx>>>,
}

impl<'ctx> Context<'ctx> {
    /// The variables hashmap default capacity.
    const VARIABLE_HASHMAP_INITIAL_CAPACITY: usize = 64;
    /// The functions hashmap default capacity.
    const FUNCTION_HASHMAP_INITIAL_CAPACITY: usize = 64;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(llvm: &'ctx inkwell::context::Context) -> Self {
        Self::new_with_optimizer(llvm, inkwell::OptimizationLevel::None)
    }

    ///
    /// Initializes a new LLVM context, setting the optimization level.
    ///
    pub fn new_with_optimizer(
        llvm: &'ctx inkwell::context::Context,
        optimization_level: inkwell::OptimizationLevel,
    ) -> Self {
        let pass_manager_builder = inkwell::passes::PassManagerBuilder::create();
        pass_manager_builder.set_optimization_level(optimization_level);

        Self {
            llvm,
            builder: llvm.create_builder(),
            module: None,
            function: None,

            break_block: None,
            continue_block: None,
            leave_block: None,

            variables: HashMap::with_capacity(Self::VARIABLE_HASHMAP_INITIAL_CAPACITY),
            functions: HashMap::with_capacity(Self::FUNCTION_HASHMAP_INITIAL_CAPACITY),

            optimization_level,
            pass_manager_builder,
            pass_manager_module: None,
            pass_manager_function: None,
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
            pass_manager_function.run_on(function);
        }

        let pass_manager_module = self
            .pass_manager_module
            .as_ref()
            .expect("Pass managers are created with the module");
        pass_manager_module.run_on(self.module());
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
    /// Writes the newly function to the hashmap.
    ///
    pub fn create_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
    ) -> inkwell::values::FunctionValue<'ctx> {
        let function = self.module().add_function(name, r#type, None);
        self.functions.insert(name.to_string(), function);
        function
    }

    ///
    /// Returns the current function.
    ///
    pub fn function(&self) -> inkwell::values::FunctionValue<'ctx> {
        self.function.expect("Always exists")
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
}
