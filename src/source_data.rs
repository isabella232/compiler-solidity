//!
//! The processed input data representation.
//!

use std::collections::HashMap;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::Lexer;
use crate::parser::statement::object::Object;

///
/// The processes input data representation.
///
pub struct SourceData {
    /// The main contract.
    pub main: Object,
    /// The dependency contracts,
    pub dependencies: HashMap<String, Object>,
    /// The library addresses.
    pub libraries: HashMap<String, String>,
}

impl SourceData {
    ///
    /// The simple constructor.
    ///
    pub fn new(object: Object) -> Self {
        Self {
            main: object,
            dependencies: HashMap::new(),
            libraries: HashMap::new(),
        }
    }

    ///
    /// The complex constructor.
    ///
    pub fn new_with_relations(
        main: Object,
        dependencies: HashMap<String, Object>,
        libraries: HashMap<String, String>,
    ) -> Self {
        Self {
            main,
            dependencies,
            libraries,
        }
    }

    ///
    /// Parses the source code and returns the source data.
    ///
    pub fn try_from_yul(input: &str) -> Result<Self, Error> {
        let mut lexer = Lexer::new(input.to_owned());
        let object = Object::parse(&mut lexer, None)?;
        Ok(Self::new(object))
    }

    ///
    /// Compiles the source code data.
    ///
    pub fn compile(
        self,
        opt_level_llvm_middle: inkwell::OptimizationLevel,
        opt_level_llvm_back: inkwell::OptimizationLevel,
        dump_llvm: bool,
    ) -> Result<String, Error> {
        let llvm = inkwell::context::Context::create();
        let target_machine =
            compiler_common::vm::target_machine(opt_level_llvm_back).ok_or_else(|| {
                Error::LLVM(format!(
                    "Target machine `{}` creation error",
                    compiler_common::vm::TARGET_NAME
                ))
            })?;
        let mut context = LLVMContext::new_with_optimizer(
            &llvm,
            &target_machine,
            opt_level_llvm_middle,
            self.main.identifier.as_str(),
            self.dependencies,
            self.libraries,
        );

        self.main.into_llvm(&mut context);
        context
            .verify()
            .map_err(|error| Error::LLVM(error.to_string()))?;
        context.optimize();
        context
            .verify()
            .map_err(|error| Error::LLVM(error.to_string()))?;
        if dump_llvm {
            let llvm_code = context.module().print_to_string().to_string();
            eprintln!("The LLVM IR code:\n");
            println!("{}", llvm_code);
        }

        let buffer = target_machine
            .write_to_memory_buffer(context.module(), inkwell::targets::FileType::Assembly)
            .map_err(|error| Error::LLVM(format!("Code compiling error: {}", error)))?;
        let llvm_ir = String::from_utf8_lossy(buffer.as_slice()).to_string();

        Ok(llvm_ir)
    }
}
