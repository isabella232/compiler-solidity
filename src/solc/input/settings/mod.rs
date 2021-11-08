//!
//! The `solc --standard-json` input settings representation.
//!

pub mod optimizer;

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

use self::optimizer::Optimizer;

///
/// The `solc --standard-json` input settings representation.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// The linker library addresses.
    pub libraries: HashMap<String, HashMap<String, String>>,
    /// The output selection filters.
    pub output_selection: serde_json::Value,
    /// The optimizer settings.
    pub optimizer: Optimizer,
}

impl Settings {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(libraries: HashMap<String, HashMap<String, String>>, optimize: bool) -> Self {
        Self {
            libraries,
            output_selection: serde_json::json!({
                "*": {
                    "*": [
                        "irOptimized"
                    ]
                }
            }),
            optimizer: Optimizer::new(optimize),
        }
    }

    ///
    /// Parses the library list and returns their double hashmap with path and name as keys.
    ///
    pub fn parse_libraries(
        input: Vec<String>,
    ) -> Result<HashMap<String, HashMap<String, String>>, Error> {
        let mut libraries = HashMap::with_capacity(input.len());
        for (index, library) in input.into_iter().enumerate() {
            let mut path_and_address = library.split('=');
            let path = path_and_address
                .next()
                .ok_or_else(|| format!("The library #{} path is missing", index))
                .map_err(Error::LibraryInput)?;
            let mut file_and_contract = path.split(':');
            let file = file_and_contract
                .next()
                .ok_or_else(|| format!("The library `{}` file name is missing", path))
                .map_err(Error::LibraryInput)?;
            let contract = file_and_contract
                .next()
                .ok_or_else(|| format!("The library `{}` contract name is missing", path))
                .map_err(Error::LibraryInput)?;
            let address = path_and_address
                .next()
                .ok_or_else(|| format!("The library `{}` address is missing", path))
                .map_err(Error::LibraryInput)?;
            libraries
                .entry(file.to_owned())
                .or_insert_with(HashMap::new)
                .insert(contract.to_owned(), address.to_owned());
        }
        Ok(libraries)
    }
}
