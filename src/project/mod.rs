//!
//! The processed input data representation.
//!

pub mod contract;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

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
        project: Arc<RwLock<Self>>,
        contract_path: &str,
        optimizer_settings: compiler_llvm_context::OptimizerSettings,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<()> {
        let mut project_guard = project.write().expect("Sync");
        match project_guard.contract_states.remove(contract_path) {
            Some(ContractState::Source(contract)) => {
                let waiter = ContractState::waiter();
                project_guard.contract_states.insert(
                    contract_path.to_owned(),
                    ContractState::Waiter(waiter.clone()),
                );
                std::mem::drop(project_guard);

                let identifier = contract.identifier().to_owned();
                let build = contract.compile(project.clone(), optimizer_settings, dump_flags)?;
                let build = ContractBuild::new(contract_path.to_owned(), identifier, build);
                project
                    .write()
                    .expect("Sync")
                    .contract_states
                    .insert(contract_path.to_owned(), ContractState::Build(build));
                waiter.1.notify_all();
                Ok(())
            }
            Some(ContractState::Waiter(waiter)) => {
                project_guard.contract_states.insert(
                    contract_path.to_owned(),
                    ContractState::Waiter(waiter.clone()),
                );
                std::mem::drop(project_guard);

                let _guard = waiter.1.wait(waiter.0.lock().expect("Sync"));
                Ok(())
            }
            Some(ContractState::Build(build)) => {
                project_guard
                    .contract_states
                    .insert(contract_path.to_owned(), ContractState::Build(build));
                Ok(())
            }
            None => {
                anyhow::bail!("Contract `{}` not found in the project", contract_path);
            }
        }
    }

    ///
    /// Compiles all contracts, returning their build artifacts.
    ///
    #[allow(clippy::needless_collect)]
    pub fn compile_all(
        self,
        optimizer_settings: compiler_llvm_context::OptimizerSettings,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<Build> {
        let project = Arc::new(RwLock::new(self));

        let contract_paths: Vec<String> = project
            .read()
            .expect("Sync")
            .contract_states
            .keys()
            .cloned()
            .collect();
        let results: Vec<anyhow::Result<()>> = contract_paths
            .into_par_iter()
            .map(|contract_path| {
                Self::compile(
                    project.clone(),
                    contract_path.as_str(),
                    optimizer_settings.clone(),
                    dump_flags.clone(),
                )
                .map_err(|error| {
                    anyhow::anyhow!("Contract `{}` compiling error: {:?}", contract_path, error)
                })
            })
            .collect();
        for result in results.into_iter() {
            if let Err(error) = result {
                return Err(error);
            }
        }

        let project = Arc::try_unwrap(project)
            .expect("No other references must exist at this point")
            .into_inner()
            .expect("Sync");
        let mut build = Build::with_capacity(project.contract_states.len());
        for (path, state) in project.contract_states.into_iter() {
            match state {
                State::Build(contract_build) => {
                    build.contracts.insert(path, contract_build);
                }
                _ => panic!("Contract `{}` must be built at this point", path),
            }
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
        project: Arc<RwLock<Self>>,
        identifier: &str,
        optimizer_settings: compiler_llvm_context::OptimizerSettings,
        dump_flags: Vec<compiler_llvm_context::DumpFlag>,
    ) -> anyhow::Result<String> {
        let contract_path = project
            .read()
            .expect("Lock")
            .identifier_paths
            .get(identifier)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Dependency contract `{}` not found in the project",
                    identifier
                )
            })?;

        Self::compile(
            project.clone(),
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

        match project
            .read()
            .expect("Lock")
            .contract_states
            .get(contract_path.as_str())
        {
            Some(ContractState::Build(build)) => Ok(build.build.hash.to_owned()),
            Some(_) => panic!(
                "Dependency contract `{}` must be built at this point",
                contract_path
            ),
            None => anyhow::bail!(
                "Dependency contract `{}` not found in the project",
                contract_path
            ),
        }
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
