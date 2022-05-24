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
#[derive(Debug)]
pub struct Project {
    /// The Solidity project version.
    pub version: semver::Version,
    /// The contract data,
    pub contracts: HashMap<String, Contract>,
    /// The library addresses.
    pub libraries: HashMap<String, HashMap<String, String>>,

    /// The contract builds.
    pub builds: HashMap<String, ContractBuild>,
    /// The mapping of auxiliary identifiers to paths.
    pub identifier_paths: HashMap<String, String>,
    /// The factory dependencies.
    pub factory_dependencies: HashMap<String, HashMap<String, String>>,
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
        let capacity = contracts.len();

        let mut identifier_paths = HashMap::with_capacity(capacity);
        for (path, contract) in contracts.iter() {
            identifier_paths.insert(contract.identifier().to_owned(), path.to_owned());
        }

        Self {
            version,
            contracts,
            libraries,

            builds: HashMap::with_capacity(capacity),
            identifier_paths,
            factory_dependencies: HashMap::with_capacity(capacity),
        }
    }

    ///
    /// Compiles the specified contract, setting its build artifacts.
    ///
    pub fn compile(
        &mut self,
        contract_path: &str,
        optimizer_settings: compiler_llvm_context::OptimizerSettings,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<ContractBuild> {
        if let Some(build) = self.builds.remove(contract_path) {
            return Ok(build);
        }

        let contract = self.contracts.remove(contract_path).ok_or_else(|| {
            anyhow::anyhow!("Contract `{}` not found in the project", contract_path)
        })?;
        let identifier = contract.identifier().to_owned();
        let mut build = contract.compile(self, contract_path, optimizer_settings, dump_flags)?;

        if let Some(factory_dependencies) = self.factory_dependencies.remove(contract_path) {
            build.factory_dependencies = factory_dependencies;
        }
        let build = ContractBuild::new(contract_path.to_owned(), identifier, build);
        Ok(build)
    }

    ///
    /// Compiles all contracts, returning their build artifacts.
    ///
    #[allow(clippy::needless_collect)]
    pub fn compile_all(
        mut self,
        optimizer_settings: compiler_llvm_context::OptimizerSettings,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<Build> {
        let mut build = Build::with_capacity(self.contracts.len());
        let contract_paths: Vec<String> = self.contracts.keys().cloned().collect();
        for contract_path in contract_paths.into_iter() {
            let contract_build = self
                .compile(
                    contract_path.as_str(),
                    optimizer_settings.clone(),
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
    /// Returns a clone without the build artifacts.
    ///
    pub fn clone_source(&self) -> Self {
        Self::new(
            self.version.to_owned(),
            self.contracts.to_owned(),
            self.libraries.to_owned(),
        )
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
        Ok(Self::new(
            version.to_owned(),
            project_contracts,
            HashMap::new(),
        ))
    }
}

impl compiler_llvm_context::Dependency for Project {
    fn compile(
        &mut self,
        identifier: &str,
        parent_identifier: &str,
        optimizer_settings: compiler_llvm_context::OptimizerSettings,
        dump_flags: Vec<compiler_llvm_context::DumpFlag>,
    ) -> anyhow::Result<String> {
        let contract_path = self
            .identifier_paths
            .get(identifier)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Dependency contract `{}` not found in the project",
                    identifier
                )
            })?;
        if let Some(build) = self.builds.get(contract_path.as_str()) {
            return Ok(build.build.hash.to_owned());
        }

        let dump_flags = DumpFlag::initialize(
            dump_flags.contains(&compiler_llvm_context::DumpFlag::Yul),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::EthIR),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::EVM),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::LLVM),
            dump_flags.contains(&compiler_llvm_context::DumpFlag::Assembly),
        );

        let build = self
            .compile(contract_path.as_str(), optimizer_settings, dump_flags)
            .map_err(|error| {
                anyhow::anyhow!(
                    "Dependency contract `{}` compiling error: {}",
                    identifier,
                    error
                )
            })?;
        let hash = build.build.hash.clone();
        self.builds.insert(contract_path.clone(), build);

        let parent_path = self
            .identifier_paths
            .get(parent_identifier)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Parent contract `{}` of the dependency `{}` not found in the project",
                    parent_identifier,
                    identifier
                )
            })?;
        self.factory_dependencies
            .entry(parent_path)
            .or_insert_with(HashMap::new)
            .insert(hash.clone(), contract_path);

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

        anyhow::bail!("Library `{}` not found in the project", path);
    }
}
