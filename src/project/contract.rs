//!
//! The contract data representation.
//!

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::Error;
use crate::parser::statement::object::Object;
use crate::solc::combined_json::contract::Contract as CombinedJsonContract;

///
/// The contract data representation.
///
#[derive(Debug, Clone)]
pub struct Contract {
    /// The absolute file path.
    pub path: String,
    /// The contract type name.
    pub name: String,
    /// The Yul source code.
    pub source: String,
    /// The Yul AST object.
    pub object: Object,
    /// The zkEVM text assembly.
    pub assembly: Option<String>,
    /// The zkEVM binary bytecode.
    pub bytecode: Option<Vec<u8>>,
    /// The zkEVM binary bytecode hash.
    pub hash: Option<String>,
    /// The factory dependencies.
    pub factory_dependencies: HashMap<String, String>,
}

impl Contract {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(path: String, name: String, source: String, object: Object) -> Self {
        Self {
            path,
            name,
            source,
            object,
            assembly: None,
            bytecode: None,
            hash: None,
            factory_dependencies: HashMap::new(),
        }
    }

    ///
    /// Inserts a factory dependency.
    ///
    pub fn insert_factory_dependency(&mut self, hash: String, path: String) {
        self.factory_dependencies
            .insert(hash, Self::short_path(path.as_str()).to_owned());
    }

    ///
    /// Writes the contract text assembly and bytecode to files.
    ///
    pub fn write_to_directory(
        self,
        path: &Path,
        output_assembly: bool,
        output_binary: bool,
        overwrite: bool,
    ) -> Result<(), Error> {
        let file_name = Self::short_path(self.path.as_str());

        if output_assembly {
            let file_name = format!(
                "{}.{}",
                file_name,
                compiler_common::EXTENSION_ZKEVM_ASSEMBLY
            );
            let mut file_path = path.to_owned();
            file_path.push(file_name);

            if file_path.exists() && !overwrite {
                eprintln!(
                    "Refusing to overwrite existing file {:?} (use --overwrite to force).",
                    file_path
                );
            } else {
                File::create(&file_path)
                    .map_err(Error::FileSystem)?
                    .write_all(self.assembly.as_ref().expect("Always exists").as_bytes())
                    .map_err(Error::FileSystem)?;
            }
        }

        if output_binary {
            let file_name = format!("{}.{}", file_name, compiler_common::EXTENSION_ZKEVM_BINARY);
            let mut file_path = path.to_owned();
            file_path.push(file_name);

            if file_path.exists() && !overwrite {
                eprintln!(
                    "Refusing to overwrite existing file {:?} (use --overwrite to force).",
                    file_path
                );
            } else {
                File::create(&file_path)
                    .map_err(Error::FileSystem)?
                    .write_all(self.bytecode.as_ref().expect("Always exists").as_slice())
                    .map_err(Error::FileSystem)?;
            }
        }

        Ok(())
    }

    ///
    /// Writes the contract text assembly and bytecode to the combined JSON.
    ///
    pub fn write_to_combined_json(
        self,
        combined_json_contract: &mut CombinedJsonContract,
    ) -> Result<(), Error> {
        let hexadecimal_bytecode = self.bytecode.map(hex::encode).expect("Always exists");
        match (
            combined_json_contract.bin.as_mut(),
            combined_json_contract.bin_runtime.as_mut(),
        ) {
            (Some(bin), Some(bin_runtime)) => {
                *bin = hexadecimal_bytecode;
                *bin_runtime = bin.clone();
            }
            (Some(bin), None) => {
                *bin = hexadecimal_bytecode;
            }
            (None, Some(bin_runtime)) => {
                *bin_runtime = hexadecimal_bytecode;
            }
            (None, None) => {}
        }

        combined_json_contract.factory_deps = Some(self.factory_dependencies);

        Ok(())
    }

    ///
    /// Converts the full path to a short one.
    ///
    pub fn short_path(path: &str) -> &str {
        path.rfind('/')
            .map(|last_slash| &path[last_slash + 1..])
            .unwrap_or_else(|| path)
    }
}
