//!
//! The LLVM context.
//!

use std::collections::HashMap;

use crate::parser::block::statement::Statement;
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
    pub module: inkwell::module::Module<'ctx>,
    /// The current function.
    pub function: Option<inkwell::values::FunctionValue<'ctx>>,

    /// The block where the `continue` statement jumps to.
    pub continue_bb: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// The block where the `break` statement jumps to.
    pub break_bb: Option<inkwell::basic_block::BasicBlock<'ctx>>,
    /// The block where the `leave` statement jumps to.
    pub leave_bb: Option<inkwell::basic_block::BasicBlock<'ctx>>,

    /// The declared variables.
    pub variables: HashMap<String, inkwell::values::PointerValue<'ctx>>,
    /// The declared functions.
    pub functions: HashMap<String, inkwell::values::FunctionValue<'ctx>>,
}

impl<'ctx> Context<'ctx> {
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

    pub fn create_function(
        &mut self,
        name: &str,
        fn_t: inkwell::types::FunctionType<'ctx>,
    ) -> inkwell::values::FunctionValue<'ctx> {
        let function = self.module.add_function(name, fn_t, None);
        self.functions.insert(name.to_string(), function);
        function
    }

    pub fn compile(statement: Statement, entry: Option<String>) -> u64 {
        let llvm = inkwell::context::Context::create();
        let module = llvm.create_module("module");
        let builder = llvm.create_builder();

        let mut context = Context {
            llvm: &llvm,
            builder,
            module,
            function: None,

            break_bb: None,
            continue_bb: None,
            leave_bb: None,

            variables: HashMap::new(),
            functions: HashMap::new(),
        };

        match statement {
            Statement::Block(block) => {
                block.into_llvm_module(&mut context);
            }
            _ => unreachable!(),
        }
        println!("{}", context.module.print_to_string().to_string());
        match entry {
            Some(name) => {
                let execution_engine = context
                    .module
                    .create_interpreter_execution_engine()
                    .unwrap();
                let entry = context.module.get_function(name.as_str()).unwrap();
                let result = unsafe { execution_engine.run_function(entry, &[]) }.as_int(false);
                println!("Result: {:?}", result);
                result
            }
            None => 0,
        }
    }
}
