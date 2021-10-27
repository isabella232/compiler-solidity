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
    /// The contract objects,
    pub objects: HashMap<String, Object>,
    /// The library addresses.
    pub libraries: HashMap<String, String>,
}

impl SourceData {
    ///
    /// The shortcut constructor.
    ///
    pub fn new(objects: HashMap<String, Object>, libraries: HashMap<String, String>) -> Self {
        Self { objects, libraries }
    }

    ///
    /// Compiles the source code data.
    ///
    pub fn compile(
        &self,
        contract_path: Option<&str>,
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

        let main_object = match contract_path {
            Some(contract_path) => self
                .objects
                .get(contract_path)
                .cloned()
                .ok_or(Error::ContractNotFound)?,
            None if self.objects.len() == 1 => self
                .objects
                .iter()
                .last()
                .ok_or(Error::ContractNotFound)?
                .1
                .to_owned(),
            None if self.objects.len() > 1 => return Err(Error::ContractNotSpecified),
            _ => return Err(Error::ContractNotFound),
        };

        let mut context = LLVMContext::new_with_optimizer(
            &llvm,
            &target_machine,
            opt_level_llvm_middle,
            main_object.identifier.as_str(),
            self,
        );
        main_object.into_llvm(&mut context);
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

    ///
    /// Parses the test Yul source code and returns the source data.
    ///
    /// Only for integration testing purposes.
    ///
    pub fn try_from_test_yul(yul: &str) -> Result<Self, Error> {
        let mut lexer = Lexer::new(yul.to_owned());
        let object = Object::parse(&mut lexer, None)?;

        let mut objects = HashMap::with_capacity(1);
        objects.insert("Test".to_owned(), object);
        Ok(Self {
            objects,
            libraries: HashMap::new(),
        })
    }
}
