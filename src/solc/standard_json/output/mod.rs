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
use crate::evm::assembly::data::Data as AssemblyData;
use crate::evm::assembly::instruction::Instruction;
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
            self.preprocess_factory_dependencies()?;
        }

        let files = self
            .contracts
            .ok_or_else(|| "Files not found in the output".to_owned())?;
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
    /// The pass, which replaces with factory dependency indexes with actual contract paths.
    ///
    fn preprocess_factory_dependencies(&mut self) -> Result<(), String> {
        let files = match self.contracts.as_mut() {
            Some(files) => files,
            None => return Ok(()),
        };
        let files_length = files.len();
        let mut auxdata_path_mapping = HashMap::with_capacity(files_length);

        for (path, contracts) in files.iter() {
            for (name, contract) in contracts.iter() {
                if let Some(auxdata) = contract
                    .evm
                    .as_ref()
                    .and_then(|evm| evm.assembly.as_ref())
                    .and_then(|assembly| assembly.data.as_ref())
                    .and_then(|data_map| data_map.get("0"))
                    .and_then(|data| data.get_auxdata())
                {
                    let full_name = format!("{}:{}", path, name);
                    auxdata_path_mapping.insert(auxdata.to_owned(), full_name);
                }
            }
        }

        for (path, contracts) in files.iter_mut() {
            for (name, contract) in contracts.iter_mut() {
                let full_name = format!("{}:{}", path, name);
                let mut index_path_mapping = HashMap::with_capacity(files_length);
                index_path_mapping.insert("0".repeat(compiler_common::SIZE_FIELD * 2), full_name);

                if let Some(data_map) = contract
                    .evm
                    .as_mut()
                    .and_then(|evm| evm.assembly.as_mut())
                    .and_then(|assembly| assembly.data.as_mut())
                {
                    for (index, data) in data_map.iter_mut() {
                        if index.as_str() != "0" {
                            let auxdata = match data {
                                AssemblyData::Assembly(assembly) => assembly
                                    .data
                                    .as_mut()
                                    .and_then(|data_map| data_map.get("0"))
                                    .and_then(|data| data.get_auxdata()),
                                AssemblyData::Hash(_) => None,
                            };
                            if let Some(auxdata) = auxdata {
                                let full_name = auxdata_path_mapping
                                    .get(auxdata)
                                    .cloned()
                                    .ok_or_else(|| {
                                        format!("Contract path not found for auxdata `{}`", auxdata)
                                    })?;
                                *data = AssemblyData::Hash(full_name.clone());

                                let mut index_extended =
                                    "0".repeat(compiler_common::SIZE_FIELD * 2 - index.len());
                                index_extended.push_str(index.as_str());
                                index_path_mapping.insert(index_extended, full_name);
                            }
                        }
                    }
                }

                let mut assembly = contract.evm.as_mut().and_then(|evm| evm.assembly.as_mut());
                if let Some(constructor_instructions) = assembly
                    .as_mut()
                    .and_then(|assembly| assembly.code.as_deref_mut())
                {
                    Instruction::replace_contract_identifiers(
                        constructor_instructions,
                        &index_path_mapping,
                    )?;
                };
                if let Some(runtime_instructions) = assembly
                    .as_mut()
                    .and_then(|assembly| assembly.data.as_mut())
                    .and_then(|data_map| data_map.get_mut("0"))
                    .and_then(|data| match data {
                        AssemblyData::Assembly(assembly) => assembly.code.as_deref_mut(),
                        AssemblyData::Hash(_) => None,
                    })
                {
                    Instruction::replace_contract_identifiers(
                        runtime_instructions,
                        &index_path_mapping,
                    )?;
                }
            }
        }

        Ok(())
    }
}
