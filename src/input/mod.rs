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
use crate::parser::error::Error as ParserError;
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
    pub fn try_into_source_data(self, contract_path: Option<&str>) -> Result<SourceData, Error> {
        if let Some(errors) = self.errors {
            for error in errors.into_iter() {
                println!("{}", error);
            }
        }

        let contracts = self.contracts.ok_or(Error::Solidity("Compilation"))?;

        let mut main = None;
        let mut dependencies = HashMap::new();
        let mut libraries = HashMap::new();
        let mut contract_index = 0;

        for (file, contracts) in contracts.into_iter() {
            for (identifier, contract) in contracts.into_iter() {
                let current_path = format!("{}:{}", file, identifier);
                let mut lexer = Lexer::new(contract.ir_optimized);
                let object = Object::parse(&mut lexer, None)?;

                libraries.insert(current_path.clone(), format!("{:064x}", contract_index));
                contract_index += 1;

                if let Some(contract_path) = contract_path {
                    if current_path.as_str() == contract_path {
                        main = Some(object);
                        continue;
                    }
                }

                dependencies.insert(object.identifier.clone(), object);
            }
        }

        if contract_path.is_none() && dependencies.len() == 1 {
            main = dependencies.remove(
                dependencies
                    .keys()
                    .next()
                    .cloned()
                    .expect("Always exists")
                    .as_str(),
            );
        }

        match (main, dependencies.is_empty(), contract_path) {
            (None, _, _) => Err(ParserError::ContractNotFound.into()),
            (Some(_), false, None) => Err(ParserError::ContractNotSpecified.into()),
            (Some(main), _, _) => Ok(SourceData::new_with_relations(
                main,
                dependencies,
                libraries,
            )),
        }
    }
}
