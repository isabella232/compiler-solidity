//!
//! The `solc --standard-json` input settings representation.
//!

pub mod optimizer;

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

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
    pub fn new(library_map: Vec<String>) -> Self {
        let mut libraries = HashMap::new();
        for (index, library) in library_map.into_iter().enumerate() {
            let mut path_and_address = library.split('=');
            let path = path_and_address
                .next()
                .unwrap_or_else(|| panic!("The library #{} path is missing", index));
            let mut file_and_contract = path.split(':');
            let file = file_and_contract
                .next()
                .unwrap_or_else(|| panic!("The library `{}` file name is missing", path));
            let contract = file_and_contract
                .next()
                .unwrap_or_else(|| panic!("The library `{}` contract name is missing", path));
            let address = path_and_address
                .next()
                .unwrap_or_else(|| panic!("The library `{}` address is missing", path));
            libraries
                .entry(file.to_owned())
                .or_insert_with(HashMap::new)
                .insert(contract.to_owned(), address.to_owned());
        }

        Self {
            libraries,
            output_selection: serde_json::json!({
                "*": {
                    "*": [
                        "irOptimized"
                    ]
                }
            }),
            optimizer: Optimizer::default(),
        }
    }
}
