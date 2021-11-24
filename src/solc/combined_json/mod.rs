//!
//! The `solc <input>.sol --combined-json` output representation.
//!

pub mod contract;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

use self::contract::Contract;

///
/// The `solc <input>.sol --combined-json` output representation.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct CombinedJson {
    /// The contract entries.
    pub contracts: HashMap<String, Contract>,
    /// The compiler version.
    pub version: String,
}

impl CombinedJson {
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
            .entry(entry)
    }

    ///
    /// Returns the full contract path which can be found in `combined-json` output.
    ///
    pub fn get_full_path(&self, name: &str) -> Option<String> {
        self.contracts.iter().find_map(|(path, _value)| {
            if let Some(last_slash_position) = path.rfind('/') {
                if let Some(colon_position) = path.rfind(':') {
                    if &path[last_slash_position + 1..colon_position] == name {
                        return Some(path.to_owned());
                    }
                }
            }

            None
        })
    }

    ///
    /// Writes the JSON to the specified directory.
    ///
    pub fn write_to_directory(self, output_directory: &Path, overwrite: bool) -> Result<(), Error> {
        let mut file_path = output_directory.to_owned();
        file_path.push(format!("combined.{}", compiler_common::EXTENSION_JSON));

        if file_path.exists() && !overwrite {
            eprintln!(
                "Refusing to overwrite existing file {:?} (use --overwrite to force).",
                file_path
            );
            return Ok(());
        }

        File::create(&file_path)
            .map_err(Error::FileSystem)?
            .write_all(serde_json::to_vec(&self).expect("Always valid").as_slice())
            .map_err(Error::FileSystem)?;

        Ok(())
    }
}
