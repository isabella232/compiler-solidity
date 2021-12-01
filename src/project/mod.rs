//!
//! The processed input data representation.
//!

pub mod contract;

use std::collections::HashMap;
use std::path::Path;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::Lexer;
use crate::parser::statement::object::Object;
use crate::solc::combined_json::CombinedJson;

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
        contract_path: &str,
        opt_level_llvm_middle: inkwell::OptimizationLevel,
        opt_level_llvm_back: inkwell::OptimizationLevel,
        dump_llvm: bool,
    ) -> Result<String, Error> {
        if let Some(contract) = self.contracts.get(contract_path) {
            if let Some(ref bytecode) = contract.hash {
                return Ok(bytecode.to_owned());
            }
        }

        let object = self
            .contracts
            .get(contract_path)
            .cloned()
            .ok_or_else(|| Error::ContractNotFound(contract_path.to_owned()))?
            .object;

        let (assembly_text, bytecode) = {
            let llvm = inkwell::context::Context::create();
            let target_machine = crate::target_machine(opt_level_llvm_back).ok_or_else(|| {
                Error::LLVM(format!(
                    "Target machine `{}` creation error",
                    compiler_common::VM_TARGET_NAME
                ))
            })?;
            let mut context = LLVMContext::new_with_optimizer(
                &llvm,
                &target_machine,
                opt_level_llvm_middle,
                object.identifier.as_str(),
                self,
            );
            object
                .into_llvm(&mut context)
                .map_err(|error| Error::LLVM(error.to_string()))?;
            context
                .verify()
                .map_err(|error| Error::LLVM(error.to_string()))?;
            context.optimize();
            context
                .verify()
                .map_err(|error| Error::LLVM(error.to_string()))?;
            if dump_llvm {
                let llvm_code = context.module().print_to_string().to_string();
                eprintln!("Contract `{}` LLVM IR:\n", contract_path);
                println!("{}", llvm_code);
            }

            let buffer = target_machine
                .write_to_memory_buffer(context.module(), inkwell::targets::FileType::Assembly)
                .map_err(|error| Error::LLVM(format!("Code compiling error: {}", error)))?;
            let assembly_text = String::from_utf8_lossy(buffer.as_slice()).to_string();

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

        let hash = compiler_common::keccak256(bytecode.as_slice());

        let contract = self
            .contracts
            .get_mut(contract_path)
            .expect("Always exists");
        contract.assembly = Some(assembly_text);
        contract.bytecode = Some(bytecode);
        contract.hash = Some(hash.clone());

        Ok(hash)
    }

    ///
    /// Compiles all contracts, setting their text assembly and binary bytecode.
    ///
    #[allow(clippy::needless_collect)]
    pub fn compile_all(&mut self, optimize: bool, dump_llvm: bool) -> Result<(), Error> {
        let optimization_level = if optimize {
            inkwell::OptimizationLevel::Aggressive
        } else {
            inkwell::OptimizationLevel::None
        };

        let contract_paths: Vec<String> = self.contracts.keys().cloned().collect();
        for contract_path in contract_paths.iter() {
            self.compile(
                contract_path.as_str(),
                optimization_level,
                optimization_level,
                dump_llvm,
            )?;
        }

        Ok(())
    }

    ///
    /// Writes all contracts to the specified directory.
    ///
    pub fn write_to_directory(
        self,
        output_directory: &Path,
        output_assembly: bool,
        output_binary: bool,
        overwrite: bool,
    ) -> Result<(), Error> {
        for (_path, contract) in self.contracts.into_iter() {
            contract.write_to_directory(
                output_directory,
                output_assembly,
                output_binary,
                overwrite,
            )?;
        }

        Ok(())
    }

    ///
    /// Writes all contracts assembly and bytecode to the combined JSON.
    ///
    pub fn write_to_combined_json(self, combined_json: &mut CombinedJson) -> Result<(), Error> {
        for (path, contract) in self.contracts.into_iter() {
            let combined_json_contract = combined_json
                .contracts
                .iter_mut()
                .find_map(|(json_path, contract)| {
                    if path.ends_with(json_path) {
                        Some(contract)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| Error::ContractNotFound(path.to_owned()))?;

            contract.write_to_combined_json(combined_json_contract)?;
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
        let name = "Test".to_owned();
        let object = Object::parse(&mut lexer, None)?;

        let mut project_contracts = HashMap::with_capacity(1);
        project_contracts.insert(
            name.clone(),
            Contract::new(name.clone(), name, yul.to_owned(), object),
        );
        Ok(Self {
            contracts: project_contracts,
            libraries: HashMap::new(),
        })
    }
}
