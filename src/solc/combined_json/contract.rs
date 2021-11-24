//!
//! The `solc <input>.sol --combined-json` contract representation.
//!

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

///
/// The contract representation.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Contract {
    /// The `solc` hashes output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<HashMap<String, String>>,
    /// The `solc` ABI output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<serde_json::Value>,
    /// The `solc` hexadecimal binary output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<String>,
    /// The `solc` hexadecimal binary runtime part output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_runtime: Option<String>,
    /// The factory dependencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory_deps: Option<HashMap<String, String>>,
}

impl Contract {
    ///
    /// Returns the signature hash of the specified contract entry.
    ///
    /// # Panics
    /// If the hashes have not been requested in the `solc` call.
    ///
    pub fn entry(&self, entry: &str) -> u32 {
        self.hashes
            .as_ref()
            .expect("Always exists")
            .iter()
            .find_map(|(contract_entry, hash)| {
                if contract_entry.starts_with(&(entry.to_owned() + "(")) {
                    Some(
                        u32::from_str_radix(hash.as_str(), compiler_common::BASE_HEXADECIMAL)
                            .expect("Test hash is always valid"),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("Entry `{}` not found", entry))
    }
}
