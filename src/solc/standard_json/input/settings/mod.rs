//!
//! The `solc --standard-json` input settings representation.
//!

pub mod optimizer;
pub mod selection;

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::solc::pipeline::Pipeline as SolcPipeline;

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
    pub fn get_output_selection(
        mut files: Vec<String>,
        pipeline: SolcPipeline,
    ) -> serde_json::Value {
        if files.is_empty() {
            files.push("*".to_owned());
        }

        let general_selections = vec![Selection::AST];
        let per_contract_selections = vec![
            Selection::ABI,
            match pipeline {
                SolcPipeline::Yul => Selection::Yul,
                SolcPipeline::EVM => Selection::EVM,
            },
        ];

        let map = files
            .into_iter()
            .map(|file| {
                (
                    file,
                    serde_json::json!({
                        "": general_selections,
                        "*": per_contract_selections,
                    }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();
        serde_json::Value::Object(map)
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
    ) -> anyhow::Result<HashMap<String, HashMap<String, String>>> {
        let mut libraries = HashMap::with_capacity(input.len());
        for (index, library) in input.into_iter().enumerate() {
            let mut path_and_address = library.split('=');
            let path = path_and_address
                .next()
                .ok_or_else(|| anyhow::anyhow!("The library #{} path is missing", index))?;
            let mut file_and_contract = path.split(':');
            let file = file_and_contract
                .next()
                .ok_or_else(|| anyhow::anyhow!("The library `{}` file name is missing", path))?;
            let contract = file_and_contract.next().ok_or_else(|| {
                anyhow::anyhow!("The library `{}` contract name is missing", path)
            })?;
            let address = path_and_address
                .next()
                .ok_or_else(|| anyhow::anyhow!("The library `{}` address is missing", path))?;
            libraries
                .entry(file.to_owned())
                .or_insert_with(HashMap::new)
                .insert(contract.to_owned(), address.to_owned());
        }
        Ok(libraries)
    }
}
