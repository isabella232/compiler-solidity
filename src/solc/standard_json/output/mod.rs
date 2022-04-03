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
        self,
        libraries: HashMap<String, HashMap<String, String>>,
        pipeline: SolcPipeline,
        version: semver::Version,
        dump_flags: &[DumpFlag],
    ) -> Result<Project, String> {
        let input_contracts = self
            .contracts
            .ok_or_else(|| "Solidity compiler error".to_owned())?;
        let mut project_contracts = HashMap::with_capacity(input_contracts.len());

        for (path, contracts) in input_contracts.into_iter() {
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
                        let source_identifier = self
                            .sources
                            .as_ref()
                            .ok_or_else(|| "Contract source IDs not found".to_owned())?
                            .get(path.as_str())
                            .ok_or_else(|| format!("Contract `{}` source ID not found", full_path))?
                            .id
                            .to_string();

                        let evm = match contract.evm {
                            Some(evm) => evm,
                            None => continue,
                        };
                        let assembly = match evm.assembly {
                            Some(assembly) => assembly,
                            None => continue,
                        };

                        ProjectContractSource::new_evm(source_identifier, assembly)
                    }
                };

                let project_contract = ProjectContract::new(full_path.clone(), name, source);
                project_contracts.insert(full_path, project_contract);
            }
        }

        Ok(Project::new(version, project_contracts, libraries))
    }
}
