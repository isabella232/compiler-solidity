//!
//! The `solc --standard-json` output representation.
//!

pub mod contract;
pub mod error;
pub mod source;

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::dump_flag::DumpFlag;
use crate::evm::assembly::instruction::Instruction;
use crate::evm::assembly::Assembly;
use crate::project::contract::source::Source as ProjectContractSource;
use crate::project::contract::Contract as ProjectContract;
use crate::project::Project;
use crate::solc::pipeline::Pipeline as SolcPipeline;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::object::Object;

use self::contract::Contract;
use self::error::Error as SolidityError;
use self::source::Source;

///
/// The `solc --standard-json` output representation.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Output {
    /// The file-contract hashmap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracts: Option<HashMap<String, HashMap<String, Contract>>>,
    /// The source code mapping data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sources: Option<HashMap<String, Source>>,
    /// The compilation errors and warnings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<SolidityError>>,
}

impl Output {
    ///
    /// Converts the `solc` JSON output into a convenient project representation.
    ///
    pub fn try_into_project(
        mut self,
        libraries: HashMap<String, HashMap<String, String>>,
        pipeline: SolcPipeline,
        version: semver::Version,
        dump_flags: &[DumpFlag],
    ) -> Result<Project, String> {
        if let SolcPipeline::EVM = pipeline {
            self.preprocess_dependencies()?;
        }

        let files = self
            .contracts
            .ok_or_else(|| "There are no files in the output".to_owned())?;
        let mut project_contracts = HashMap::with_capacity(files.len());

        for (path, contracts) in files.into_iter() {
            for (name, contract) in contracts.into_iter() {
                let full_path = format!("{}:{}", path, name);

                let source = match pipeline {
                    SolcPipeline::Yul => {
                        let ir_optimized = match contract.ir_optimized {
                            Some(ir_optimized) => ir_optimized,
                            None => continue,
                        };
                        if ir_optimized.is_empty() {
                            continue;
                        }

                        if dump_flags.contains(&DumpFlag::Yul) {
                            eprintln!("Contract `{}` Yul:\n", full_path);
                            println!("{}", ir_optimized);
                        }

                        let mut lexer = Lexer::new(ir_optimized.clone());
                        let object = Object::parse(&mut lexer, None).map_err(|error| {
                            format!("Contract `{}` parsing error: {:?}", full_path, error)
                        })?;

                        ProjectContractSource::new_yul(ir_optimized, object)
                    }
                    SolcPipeline::EVM => {
                        let evm = match contract.evm {
                            Some(evm) => evm,
                            None => continue,
                        };
                        let assembly = match evm.assembly {
                            Some(assembly) => assembly,
                            None => continue,
                        };

                        ProjectContractSource::new_evm(full_path.clone(), assembly)
                    }
                };

                let project_contract = ProjectContract::new(full_path.clone(), name, source);
                project_contracts.insert(full_path, project_contract);
            }
        }

        Ok(Project::new(version, project_contracts, libraries))
    }

    ///
    /// The pass, which replaces with dependency indexes with actual data.
    ///
    fn preprocess_dependencies(&mut self) -> Result<(), String> {
        let files = match self.contracts.as_mut() {
            Some(files) => files,
            None => return Ok(()),
        };
        let files_length = files.len();
        let mut hash_path_mapping = HashMap::with_capacity(files_length);

        for (path, contracts) in files.iter() {
            for (name, contract) in contracts.iter() {
                let full_path = format!("{}:{}", path, name);
                let hash = match contract
                    .evm
                    .as_ref()
                    .and_then(|evm| evm.assembly.as_ref())
                    .map(|assembly| assembly.keccak256())
                {
                    Some(hash) => hash,
                    None => continue,
                };

                hash_path_mapping.insert(hash, full_path);
            }
        }

        for (path, contracts) in files.iter_mut() {
            for (name, contract) in contracts.iter_mut() {
                let assembly = match contract.evm.as_mut().and_then(|evm| evm.assembly.as_mut()) {
                    Some(assembly) => assembly,
                    None => continue,
                };

                let full_path = format!("{}:{}", path, name);
                Self::preprocess_dependency_level(
                    full_path.as_str(),
                    assembly,
                    &hash_path_mapping,
                )?;
            }
        }

        Ok(())
    }

    ///
    /// Preprocesses an assembly JSON structure dependency data map.
    ///
    fn preprocess_dependency_level(
        full_path: &str,
        assembly: &mut Assembly,
        hash_path_mapping: &HashMap<String, String>,
    ) -> Result<(), String> {
        assembly.set_full_path(full_path.to_owned());

        let constructor_index_path_mapping =
            assembly.constructor_dependencies_pass(full_path, hash_path_mapping)?;
        if let Some(constructor_instructions) = assembly.code.as_deref_mut() {
            Instruction::replace_data_aliases(
                constructor_instructions,
                &constructor_index_path_mapping,
            )?;
        };

        let selector_index_path_mapping =
            assembly.selector_dependencies_pass(full_path, hash_path_mapping)?;
        if let Some(selector_instructions) = assembly
            .data
            .as_mut()
            .and_then(|data_map| data_map.get_mut("0"))
            .and_then(|data| data.get_assembly_mut())
            .and_then(|assembly| assembly.code.as_deref_mut())
        {
            Instruction::replace_data_aliases(selector_instructions, &selector_index_path_mapping)?;
        }

        Ok(())
    }
}
