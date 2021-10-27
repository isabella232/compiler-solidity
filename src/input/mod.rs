//!
//! The `solc --standard-json` output representation.
//!

pub mod contract;
pub mod error;
pub mod source;

use std::collections::HashMap;

use serde::Deserialize;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::parser::statement::object::Object;
use crate::project::contract::Contract as ProjectContract;
use crate::project::Project;

use self::contract::Contract;
use self::error::Error as SolidityError;
use self::source::Source;

///
/// The `solc --standard-json` output representation.
///
#[derive(Debug, Deserialize)]
pub struct Input {
    /// The file-contract hashmap.
    pub contracts: Option<HashMap<String, HashMap<String, Contract>>>,
    /// The source code mapping data.
    pub sources: HashMap<String, Source>,
    /// The compilation errors and warnings.
    pub errors: Option<Vec<SolidityError>>,
}

impl Input {
    ///
    /// If there is only contract, it is returned regardless of `contract_path`.
    /// If there is more than one contract, the `contract_path` must be specified, otherwise, an
    /// error is returned.
    /// If there are no contracts, an error is returned.
    ///
    /// Returns the main contract object and its dependencies.
    ///
    pub fn try_into_project(
        self,
        libraries: HashMap<String, String>,
        print_warnings: bool,
    ) -> Result<Project, Error> {
        if let Some(errors) = self.errors {
            for error in errors.into_iter() {
                if error.severity.as_str() == "warning" && !print_warnings {
                    continue;
                }

                println!("{}", error);
            }
        }

        let input_contracts = self
            .contracts
            .ok_or(Error::Solidity("Solidity compiler error"))?;
        let mut project_contracts = HashMap::with_capacity(input_contracts.len());
        for (path, contracts) in input_contracts.into_iter() {
            for (name, contract) in contracts.into_iter() {
                if contract.ir_optimized.is_empty() {
                    continue;
                }

                let full_path = format!("{}:{}", path, name);
                let mut lexer = Lexer::new(contract.ir_optimized);
                let object = Object::parse(&mut lexer, None)?;
                let project_contract = ProjectContract::new(path.clone(), name, object);
                project_contracts.insert(full_path, project_contract);
            }
        }

        Ok(Project::new(project_contracts, libraries))
    }
}
