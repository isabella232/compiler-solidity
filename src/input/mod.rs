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
use crate::source_data::SourceData;

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
    pub fn try_into_source_data(
        self,
        libraries: HashMap<String, String>,
        print_warnings: bool,
    ) -> Result<SourceData, Error> {
        if let Some(errors) = self.errors {
            for error in errors.into_iter() {
                if error.severity.as_str() == "warning" && !print_warnings {
                    continue;
                }

                println!("{}", error);
            }
        }

        let contracts = self
            .contracts
            .ok_or(Error::Solidity("Solidity compiler error"))?;
        let mut objects = HashMap::with_capacity(contracts.len());
        for (file, contracts) in contracts.into_iter() {
            for (identifier, contract) in contracts.into_iter() {
                if contract.ir_optimized.is_empty() {
                    continue;
                }

                let current_path = format!("{}:{}", file, identifier);
                let mut lexer = Lexer::new(contract.ir_optimized);
                let object = Object::parse(&mut lexer, None)?;
                objects.insert(current_path, object);
            }
        }

        Ok(SourceData::new(objects, libraries))
    }
}
