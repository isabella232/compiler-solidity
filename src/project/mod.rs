//!
//! The processed input data representation.
//!

pub mod contract;

use std::collections::HashMap;

use crate::build::contract::Contract as ContractBuild;
use crate::build::Build;
use crate::dump_flag::DumpFlag;
use crate::project::contract::source::Source;
use crate::project::contract::state::State;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::object::Object;

use self::contract::state::State as ContractState;
use self::contract::Contract;

///
/// The processes input data representation.
///
#[derive(Debug)]
pub struct Project {
    /// The Solidity project version.
    pub version: semver::Version,
    /// The contract data,
    pub contract_states: HashMap<String, ContractState>,
    /// The mapping of auxiliary identifiers, e.g. Yul object names, to full contract paths.
    pub identifier_paths: HashMap<String, String>,
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
        let capacity = contracts.len();

        let mut identifier_paths = HashMap::with_capacity(capacity);
        for (path, contract) in contracts.iter() {
            identifier_paths.insert(contract.identifier().to_owned(), path.to_owned());
        }

        Self {
            version,
            contract_states: contracts
                .into_iter()
                .map(|(path, contract)| (path, ContractState::Source(contract)))
                .collect(),
            identifier_paths,
            libraries,
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
        let contract = match self.contract_states.remove(contract_path) {
            Some(ContractState::Source(contract)) => contract,
            Some(ContractState::Build(build)) => return Ok(build),
            None => anyhow::bail!("Contract `{}` not found in the project", contract_path),
        };

        let identifier = contract.identifier().to_owned();
        let build = contract.compile(self, optimizer_settings, dump_flags)?;
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
        let contract_paths: Vec<String> = self.contract_states.keys().cloned().collect();
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
            self.contract_states
                .insert(contract_path, ContractState::Build(contract_build));
        }

        let mut build = Build::with_capacity(self.contract_states.len());
        for (path, state) in self.contract_states.into_iter() {
            match state {
                State::Build(contract_build) => {
                    build.contracts.insert(path, contract_build);
                }
                State::Source(_) => panic!("Contract `{}` must be built at this point", path),
            }
        }
        Ok(build)
    }

    ///
    /// Returns a clone without the build artifacts.
    ///
    pub fn clone_source(&self) -> Self {
        Self::new(
            self.version.to_owned(),
            self.contract_states
                .iter()
                .filter_map(|(path, state)| {
                    if let ContractState::Source(source) = state {
                        Some((path.to_owned(), source.to_owned()))
                    } else {
                        None
                    }
                })
                .collect(),
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
        let path = "Test".to_owned();
        let object = Object::parse(&mut lexer, None)
            .map_err(|error| anyhow::anyhow!("Yul object `{}` parsing error: {}", path, error,))?;

        let mut project_contracts = HashMap::with_capacity(1);
        project_contracts.insert(
            path.clone(),
            Contract::new(path, Source::new_yul(yul.to_owned(), object)),
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
        if let Some(ContractState::Build(build)) = self.contract_states.get(contract_path.as_str())
        {
            return Ok(build.build.hash.to_owned());
        }

        let build = self
            .compile(
                contract_path.as_str(),
                optimizer_settings,
                DumpFlag::from_context(dump_flags.as_slice()),
            )
            .map_err(|error| {
                anyhow::anyhow!(
                    "Dependency contract `{}` compiling error: {}",
                    identifier,
                    error
                )
            })?;
        let hash = build.build.hash.clone();
        self.contract_states
            .insert(contract_path, ContractState::Build(build));

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
