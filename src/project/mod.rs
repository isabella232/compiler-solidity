//!
//! The processed input data representation.
//!

pub mod contract;

use std::collections::HashMap;

use crate::build::contract::Contract as ContractBuild;
use crate::build::Build;
use crate::dump_flag::DumpFlag;
use crate::project::contract::source::Source;
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
    /// A shortcut constructor.
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
    /// Compiles the specified contract, setting its build artifacts.
    ///
    pub fn compile(
        &mut self,
        contract_path: &str,
        optimization_level_middle: inkwell::OptimizationLevel,
        optimization_level_back: inkwell::OptimizationLevel,
        run_inliner: bool,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<ContractBuild> {
        let contract = self.contracts.get(contract_path).cloned().ok_or_else(|| {
            anyhow::anyhow!("Contract `{}` not found in the project", contract_path)
        })?;
        if let Some(build) = contract.build.as_ref() {
            return Ok(ContractBuild::new(
                contract_path.to_owned(),
                build.to_owned(),
            ));
        }

        let mut build = contract.compile(
            self,
            contract_path,
            optimization_level_middle,
            optimization_level_back,
            run_inliner,
            dump_flags,
        )?;

        let contract = self
            .contracts
            .get_mut(contract_path)
            .expect("Always exists");
        build.factory_dependencies = contract.factory_dependencies.clone();
        contract.build = Some(build.clone());

        let build = ContractBuild::new(contract_path.to_owned(), build);

        Ok(build)
    }

    ///
    /// Compiles all contracts, returning their build artifacts.
    ///
    #[allow(clippy::needless_collect)]
    pub fn compile_all(
        mut self,
        optimize: bool,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<Build> {
        let (optimization_level, run_inliner) = if optimize {
            (inkwell::OptimizationLevel::Aggressive, true)
        } else {
            (inkwell::OptimizationLevel::None, false)
        };

        let mut build = Build::with_capacity(self.contracts.len());
        let contract_paths: Vec<String> = self.contracts.keys().cloned().collect();
        for contract_path in contract_paths.into_iter() {
            let contract_build = self
                .compile(
                    contract_path.as_str(),
                    optimization_level,
                    optimization_level,
                    run_inliner,
                    dump_flags.clone(),
                )
                .map_err(|error| {
                    anyhow::anyhow!("Contract `{}` compiling error: {:?}", contract_path, error)
                })?;
            build.contracts.insert(contract_path, contract_build);
        }

        Ok(build)
    }

    ///
    /// Parses the test Yul source code and returns the source data.
    ///
    /// Only for integration testing purposes.
    ///
    pub fn try_from_test_yul(yul: &str, version: &semver::Version) -> anyhow::Result<Self> {
        let mut lexer = Lexer::new(yul.to_owned());
        let name = "Test".to_owned();
        let object = Object::parse(&mut lexer, None)
            .map_err(|error| anyhow::anyhow!("Yul object `{}` parsing error: {}", name, error,))?;

        let mut project_contracts = HashMap::with_capacity(1);
        project_contracts.insert(
            name.clone(),
            Contract::new(name.clone(), name, Source::new_yul(yul.to_owned(), object)),
        );
        Ok(Self {
            version: version.to_owned(),
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
        run_inliner: bool,
        dump_flags: Vec<compiler_llvm_context::DumpFlag>,
    ) -> anyhow::Result<compiler_llvm_context::Build> {
        let contract_path = self
            .contracts
            .iter()
            .find_map(|(path, contract)| {
                if match contract.source {
                    Source::Yul(ref yul) => yul.object.identifier.as_str() == identifier,
                    Source::EVM(ref evm) => evm.full_path.as_str() == identifier,
                } {
                    Some(path.to_owned())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Dependency contract `{}` not found in the project",
                    identifier
                )
            })?;

        let dump_flags = DumpFlag::initialize(
            dump_flags.contains(&compiler_llvm_context::DumpFlag::Yul),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::EthIR),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::EVM),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::LLVM),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::Assembly),
        );
        let build = self
            .compile(
                contract_path.as_str(),
                optimization_level_middle,
                optimization_level_back,
                run_inliner,
                dump_flags,
            )
            .map_err(|error| {
                anyhow::anyhow!(
                    "Dependency contract `{}` compiling error: {}",
                    identifier,
                    error
                )
            })?;

        self.contracts
            .iter_mut()
            .find_map(|(_path, contract)| {
                if match contract.source {
                    Source::Yul(ref yul) => yul.object.identifier.as_str() == parent_identifier,
                    Source::EVM(ref evm) => evm.full_path.as_str() == parent_identifier,
                } {
                    Some(contract)
                } else {
                    None
                }
            })
            .as_mut()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Parent `{}` of the dependency contract `{}` not found in the project",
                    parent_identifier,
                    identifier
                )
            })?
            .insert_factory_dependency(build.build.hash.clone(), contract_path);

        Ok(build.build)
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

        anyhow::bail!("Library `{}` not found in the project", path);
    }
}
