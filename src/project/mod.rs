//!
//! The processed input data representation.
//!

pub mod contract;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::Lexer;
use crate::parser::statement::object::Object;

use self::contract::Contract;

///
/// The processes input data representation.
///
#[derive(Debug, Clone)]
pub struct Project {
    /// The contract data,
    pub contracts: HashMap<String, Contract>,
    /// The library addresses.
    pub libraries: HashMap<String, HashMap<String, String>>,
}

impl Project {
    ///
    /// The shortcut constructor.
    ///
    pub fn new(
        contracts: HashMap<String, Contract>,
        libraries: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        Self {
            contracts,
            libraries,
        }
    }

    ///
    /// Compiles the specified contract, setting its text assembly and binary bytecode.
    ///
    pub fn compile(
        &mut self,
        contract_path: Option<&str>,
        opt_level_llvm_middle: inkwell::OptimizationLevel,
        opt_level_llvm_back: inkwell::OptimizationLevel,
        dump_llvm: bool,
    ) -> Result<(String, Vec<u8>), Error> {
        if let Some(contract_path) = contract_path {
            if let Some(contract) = self.contracts.get(contract_path) {
                if let (Some(assembly_text), Some(bytecode)) =
                    (contract.assembly.as_deref(), contract.bytecode.as_deref())
                {
                    return Ok((assembly_text.to_owned(), bytecode.to_owned()));
                }
            }
        }

        let (contract_path, main_object) = match contract_path {
            Some(contract_path) => {
                let object = self
                    .contracts
                    .get(contract_path)
                    .cloned()
                    .ok_or(Error::ContractNotFound)?
                    .object;
                (contract_path.to_owned(), object)
            }
            None if self.contracts.len() == 1 => self
                .contracts
                .iter()
                .last()
                .ok_or(Error::ContractNotFound)
                .map(|(path, contract)| (path.to_owned(), contract.object.to_owned()))?,
            None if self.contracts.len() > 1 => return Err(Error::ContractNotSpecified),
            _ => return Err(Error::ContractNotFound),
        };

        let (assembly_text, bytecode) = {
            let llvm = inkwell::context::Context::create();
            let target_machine = compiler_common::vm::target_machine(opt_level_llvm_back)
                .ok_or_else(|| {
                    Error::LLVM(format!(
                        "Target machine `{}` creation error",
                        compiler_common::vm::TARGET_NAME
                    ))
                })?;
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
                println!("Contract `{}` LLVM IR:\n", contract_path);
                println!("{}", llvm_code);
            }

            let buffer = target_machine
                .write_to_memory_buffer(context.module(), inkwell::targets::FileType::Assembly)
                .map_err(|error| Error::LLVM(format!("Code compiling error: {}", error)))?;
            let assembly_text = String::from_utf8_lossy(buffer.as_slice()).to_string();

            // Prevents calling the destructor. Is suspected in segmentation faults.
            std::mem::forget(buffer);

            let assembly = zkevm_assembly::Assembly::try_from(assembly_text.clone())
                .unwrap_or_else(|error| {
                    panic!(
                        "Dependency `{}` assembly parsing error: {}",
                        contract_path, error
                    )
                });
            let bytecode = Vec::<u8>::from(&assembly);

            (assembly_text, bytecode)
        };

        let contract = self
            .contracts
            .get_mut(contract_path.as_str())
            .expect("Always exists");
        contract.assembly = Some(assembly_text.clone());
        contract.bytecode = Some(bytecode.clone());

        Ok((assembly_text, bytecode))
    }

    ///
    /// Compiles all contracts, setting their text assembly and binary bytecode.
    ///
    #[allow(clippy::needless_collect)]
    #[allow(clippy::too_many_arguments)]
    pub fn compile_all(
        &mut self,
        output_directory: PathBuf,
        opt_level_llvm_middle: inkwell::OptimizationLevel,
        opt_level_llvm_back: inkwell::OptimizationLevel,
        dump_llvm: bool,
        output_assembly: bool,
        output_binary: bool,
        overwrite: bool,
    ) -> Result<(), Error> {
        let contract_paths: Vec<String> = self.contracts.keys().cloned().collect();

        for contract_path in contract_paths.iter() {
            self.compile(
                Some(contract_path.as_str()),
                opt_level_llvm_middle,
                opt_level_llvm_back,
                dump_llvm,
            )?;
        }
        for contract_path in contract_paths.into_iter() {
            self.contracts
                .get(contract_path.as_str())
                .as_ref()
                .expect("Always exists")
                .write_to_directory(&output_directory, output_assembly, output_binary, overwrite)?;
        }

        if output_assembly || output_binary {
            println!(
                "Compiler run successful. Artifact(s) can be found in directory {:?}.",
                output_directory
            );
        } else {
            println!("Compiler run successful. No output requested. Use --asm and --bin flags.");
        }

        Ok(())
    }

    ///
    /// Parses the test Yul source code and returns the source data.
    ///
    /// Only for integration testing purposes.
    ///
    pub fn try_from_test_yul(yul: &str) -> Result<Self, Error> {
        let mut lexer = Lexer::new(yul.to_owned());
        let path = "./test.sol".to_owned();
        let name = "Test".to_owned();
        let full_path = format!("{}:{}", path, name);
        let object = Object::parse(&mut lexer, None)?;

        let mut project_contracts = HashMap::with_capacity(1);
        project_contracts.insert(full_path, Contract::new(path, name, yul.to_owned(), object));
        Ok(Self {
            contracts: project_contracts,
            libraries: HashMap::new(),
        })
    }
}
