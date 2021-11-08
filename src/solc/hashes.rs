//!
//! The `solc <input>.sol --combined-json hashes` output representation.
//!

use std::collections::HashMap;

use serde::Deserialize;

///
/// The `solc <input>.sol --combined-json hashes` output representation.
///
#[derive(Debug, Deserialize)]
pub struct Hashes {
    /// The contract entries.
    pub contracts: HashMap<String, Contract>,
    /// The compiler version.
    pub version: String,
}

///
/// The contract entry.
///
#[derive(Debug, Deserialize)]
pub struct Contract {
    /// The signature-hash mapping.
    pub hashes: HashMap<String, String>,
}

impl Hashes {
    ///
    /// Returns the signature hash of the specified contract and entry.
    ///
    pub fn entry(&self, path: &str, entry: &str) -> u32 {
        self.contracts
            .iter()
            .find_map(|(name, contract)| {
                if name.starts_with(path) {
                    Some(contract)
                } else {
                    None
                }
            })
            .expect("Always exists")
            .hashes
            .iter()
            .find_map(|(contract_entry, hash)| {
                if contract_entry.starts_with(&(entry.to_owned() + "(")) {
                    Some(
                        u32::from_str_radix(hash.as_str(), compiler_common::base::HEXADECIMAL)
                            .expect("Test hash is always valid"),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("Entry `{}` not found", entry))
    }

    ///
    /// Returns the full contract path which can be found is `combined-json` output.
    ///
    pub fn get_contract_path(&self, path: &str) -> Option<String> {
        self.contracts.iter().find_map(|(key, _value)| {
            if let Some(last_slash_position) = key.rfind('/') {
                if let Some(colon_position) = key.rfind(':') {
                    if &key[last_slash_position + 1..colon_position] == path {
                        return Some(key.to_owned());
                    }
                }
            }

            None
        })
    }
}
