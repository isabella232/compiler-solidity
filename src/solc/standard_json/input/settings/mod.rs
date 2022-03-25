//!
//! The `solc --standard-json` input settings representation.
//!

pub mod optimizer;
pub mod selection;

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

use self::optimizer::Optimizer;
use self::selection::Selection;

///
/// The `solc --standard-json` input settings representation.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// The linker library addresses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libraries: Option<HashMap<String, HashMap<String, String>>>,
    /// The output selection filters.
    pub output_selection: serde_json::Value,
    /// The optimizer settings.
    pub optimizer: Optimizer,
}

impl Settings {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        libraries: HashMap<String, HashMap<String, String>>,
        output_selection: serde_json::Value,
        optimize: bool,
    ) -> Self {
        Self {
            libraries: Some(libraries),
            output_selection,
            optimizer: Optimizer::new(optimize),
        }
    }

    ///
    /// Generates the output selection pattern.
    ///
    pub fn get_output_selection(selections: Vec<Selection>) -> serde_json::Value {
        serde_json::json!({
            "*": {
                "*": selections.iter().map(Selection::to_string).collect::<Vec<String>>()
            }
        })
    }

    ///
    /// Generates the AST output selection pattern.
    ///
    pub fn get_ast_selection(mut files: Vec<String>) -> serde_json::Value {
        if files.is_empty() {
            files.push("*".to_owned());
        }
        let map = files
            .into_iter()
            .map(|file| {
                (
                    file,
                    serde_json::json!({ "": [Selection::AST.to_string()] }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();
        serde_json::Value::Object(map)
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
