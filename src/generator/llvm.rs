//!
//! The LLVM context.
//!

use std::collections::HashMap;

use crate::parser::identifier::Identifier;
use crate::parser::Module;

///
/// The LLVM context.
///
pub struct Context<'ctx> {
    /// The inner LLVM context.
    pub llvm: &'ctx inkwell::context::Context,
    /// The LLVM builder.
    pub builder: inkwell::builder::Builder<'ctx>,
    /// The current module.
    pub module: inkwell::module::Module<'ctx>,
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
}

impl<'ctx> Context<'ctx> {
    /// The variables hashmap default capacity.
    const VARIABLE_HASHMAP_INITIAL_CAPACITY: usize = 64;
    /// The functions hashmap default capacity.
    const FUNCTION_HASHMAP_INITIAL_CAPACITY: usize = 64;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(llvm: &'ctx inkwell::context::Context) -> Self {
        Self {
            llvm,
            builder: llvm.create_builder(),
            module: llvm.create_module("module"),
            function: None,

            break_block: None,
            continue_block: None,
            leave_block: None,

            variables: HashMap::with_capacity(Self::VARIABLE_HASHMAP_INITIAL_CAPACITY),
            functions: HashMap::with_capacity(Self::FUNCTION_HASHMAP_INITIAL_CAPACITY),
        }
    }

    ///
    /// Compiles and runs the module, returning the `entry` function result.
    ///
    pub fn compile(mut self, module: Module, entry: Option<String>) -> Option<u64> {
        module.into_llvm(&mut self);
        println!("{}", self.module.print_to_string().to_string());

        let execution_engine = self
            .module
            .create_interpreter_execution_engine()
            .expect("Execution engine creation");
        let entry = self
            .module
            .get_function(entry?.as_str())
            .expect("Always exists");
        let result = unsafe { execution_engine.run_function(entry, &[]) }.as_int(false);
        Some(result)
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
    /// Writes the newly function to the hashmap.
    ///
    pub fn create_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
    ) -> inkwell::values::FunctionValue<'ctx> {
        let function = self.module.add_function(name, r#type, None);
        self.functions.insert(name.to_string(), function);
        function
    }

    ///
    /// Returns the current function.
    ///
    pub fn function(&self) -> inkwell::values::FunctionValue<'ctx> {
        self.function.expect("Always exists")
    }
}
