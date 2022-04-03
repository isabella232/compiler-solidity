//!
//! The processed input data representation.
//!

pub mod contract;

use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use compiler_llvm_context::WriteLLVM;

use crate::dump_flag::DumpFlag;
use crate::error::Error;
use crate::project::contract::source::Source;
use crate::solc::combined_json::CombinedJson;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::object::Object;

use self::contract::Contract;

///
/// The processes input data representation.
///
#[derive(Debug, Clone)]
pub struct Project {
    /// The Solidity project version.
    pub version: semver::Version,
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
        version: semver::Version,
        contracts: HashMap<String, Contract>,
        libraries: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        Self {
            version,
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
        optimization_level_middle: inkwell::OptimizationLevel,
        optimization_level_back: inkwell::OptimizationLevel,
        dump_flags: Vec<DumpFlag>,
    ) -> Result<String, Error> {
        if let Some(contract) = self.contracts.get(contract_path) {
            if let Some(ref hash) = contract.hash {
                return Ok(hash.to_owned());
            }
        }

        let llvm = inkwell::context::Context::create();
        let target_machine = crate::target_machine(optimization_level_back).ok_or_else(|| {
            Error::LLVM(format!(
                "Target machine `{}` creation error",
                compiler_common::VM_TARGET_NAME
            ))
        })?;
        let dump_flags = compiler_llvm_context::DumpFlag::initialize(
            dump_flags.contains(&DumpFlag::Yul),
            dump_flags.contains(&DumpFlag::EthIR),
            dump_flags.contains(&DumpFlag::EVM),
            false,
            dump_flags.contains(&DumpFlag::LLVM),
            dump_flags.contains(&DumpFlag::Assembly),
        );
        let mut source = self
            .contracts
            .get(contract_path)
            .ok_or_else(|| Error::ContractNotFound(contract_path.to_owned()))?
            .source
            .to_owned();
        let module_name = match source {
            Source::Yul(ref yul) => yul.object.identifier.to_owned(),
            Source::EVM(ref evm) => evm.source_id.to_string(),
        };
        let mut context = match source {
            Source::Yul(_) => compiler_llvm_context::Context::new(
                &llvm,
                &target_machine,
                optimization_level_middle,
                optimization_level_back,
                module_name.as_str(),
                Some(self),
                dump_flags.clone(),
            ),
            Source::EVM(_) => {
                let version = self.version.to_owned();
                compiler_llvm_context::Context::new_evm(
                    &llvm,
                    &target_machine,
                    optimization_level_middle,
                    optimization_level_back,
                    module_name.as_str(),
                    Some(self),
                    dump_flags.clone(),
                    compiler_llvm_context::ContextEVMData::new(version),
                )
            }
        };

        source
            .declare(&mut context)
            .map_err(|error| Error::LLVM(error.to_string()))?;
        source
            .into_llvm(&mut context)
            .map_err(|error| Error::LLVM(error.to_string()))?;
        if dump_flags.contains(&compiler_llvm_context::DumpFlag::LLVM) {
            let llvm_code = context.module().print_to_string().to_string();
            eprintln!("Contract `{}` LLVM IR unoptimized:\n", contract_path);
            println!("{}", llvm_code);
        }
        context
            .verify()
            .map_err(|error| Error::LLVM(error.to_string()))?;
        let is_optimized = context.optimize();
        if dump_flags.contains(&compiler_llvm_context::DumpFlag::LLVM) && is_optimized {
            let llvm_code = context.module().print_to_string().to_string();
            eprintln!("Contract `{}` LLVM IR optimized:\n", contract_path);
            println!("{}", llvm_code);
        }
        context
            .verify()
            .map_err(|error| Error::LLVM(error.to_string()))?;

        let buffer = target_machine
            .write_to_memory_buffer(context.module(), inkwell::targets::FileType::Assembly)
            .map_err(|error| Error::LLVM(error.to_string()))?;
        let assembly_text = String::from_utf8_lossy(buffer.as_slice()).to_string();
        if dump_flags.contains(&compiler_llvm_context::DumpFlag::Assembly) {
            eprintln!("Contract `{}` assembly:\n", contract_path);
            println!("{}", assembly_text);
        }

        let assembly = zkevm_assembly::Assembly::try_from(assembly_text.clone())?;

        let bytecode: Vec<u8> = assembly
            .clone()
            .compile_to_bytecode()?
            .into_iter()
            .flatten()
            .collect();

        let hash = compiler_common::keccak256(bytecode.as_slice());

        let contract = self
            .contracts
            .get_mut(contract_path)
            .expect("Always exists");
        contract.assembly_text = Some(assembly_text);
        contract.assembly = Some(assembly);
        contract.bytecode = Some(bytecode);
        contract.hash = Some(hash.clone());

        Ok(hash)
    }

    ///
    /// Compiles all contracts, setting their text assembly and binary bytecode.
    ///
    #[allow(clippy::needless_collect)]
    pub fn compile_all(&mut self, optimize: bool, dump_flags: Vec<DumpFlag>) -> Result<(), String> {
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
                dump_flags.clone(),
            )
            .map_err(|error| {
                format!("Contract `{}` compiling error: {:?}", contract_path, error)
            })?;
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
        let version = semver::Version::from_str(
            std::env::var("CARGO_PKG_VERSION")
                .expect("Always exists")
                .as_str(),
        )
        .expect("Always valid");

        let mut lexer = Lexer::new(yul.to_owned());
        let name = "Test".to_owned();
        let object = Object::parse(&mut lexer, None)?;

        let mut project_contracts = HashMap::with_capacity(1);
        project_contracts.insert(
            name.clone(),
            Contract::new(name.clone(), name, Source::new_yul(yul.to_owned(), object)),
        );
        Ok(Self {
            version,
            contracts: project_contracts,
            libraries: HashMap::new(),
        })
    }
}

impl compiler_llvm_context::Dependency for Project {
    fn compile(
        &mut self,
        identifier: &str,
        parent_identifier: &str,
        optimization_level_middle: inkwell::OptimizationLevel,
        optimization_level_back: inkwell::OptimizationLevel,
        dump_flags: Vec<compiler_llvm_context::DumpFlag>,
    ) -> anyhow::Result<String> {
        let contract_path = self
            .contracts
            .iter()
            .find_map(|(path, contract)| {
                if match contract.source {
                    Source::Yul(ref yul) => yul.object.identifier.as_str() == identifier,
                    Source::EVM(ref evm) => evm.source_id.to_string().as_str() == identifier,
                } {
                    Some(path.to_owned())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("Dependency `{}` not found", identifier))?;

        let dump_flags = DumpFlag::initialize(
            dump_flags.contains(&compiler_llvm_context::DumpFlag::Yul),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::EthIR),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::EVM),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::LLVM),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::Assembly),
        );
        let hash = self
            .compile(
                contract_path.as_str(),
                optimization_level_middle,
                optimization_level_back,
                dump_flags,
            )
            .map_err(|error| {
                anyhow::anyhow!("Dependency `{}` compiling error: {:?}", identifier, error)
            })?;

        self.contracts
            .iter_mut()
            .find_map(|(_path, contract)| {
                if match contract.source {
                    Source::Yul(ref yul) => yul.object.identifier.as_str() == parent_identifier,
                    Source::EVM(ref evm) => evm.source_id.to_string().as_str() == parent_identifier,
                } {
                    Some(contract)
                } else {
                    None
                }
            })
            .as_mut()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Parent `{}` of dependency `{}` not found",
                    parent_identifier,
                    identifier
                )
            })?
            .insert_factory_dependency(hash.clone(), contract_path);

        Ok(hash)
    }

    fn resolve_library(&self, path: &str) -> anyhow::Result<String> {
        for (file_path, contracts) in self.libraries.iter() {
            for (contract_name, address) in contracts.iter() {
                let key = format!("{}:{}", file_path, contract_name);
                if key.as_str() == path {
                    return Ok(address["0x".len()..].to_owned());
                }
            }
        }

        anyhow::bail!("Library `{}` not found", path)
    }
}
