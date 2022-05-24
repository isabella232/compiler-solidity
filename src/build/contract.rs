//!
//! The Solidity contract build.
//!

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::solc::combined_json::contract::Contract as CombinedJsonContract;

///
/// The Solidity contract build.
///
#[derive(Debug)]
pub struct Contract {
    /// The contract path.
    pub path: String,
    /// The auxiliary identifier. Used to identify Yul objects.
    pub identifier: String,
    /// The LLVM module build.
    pub build: compiler_llvm_context::Build,
}

impl Contract {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(path: String, identifier: String, build: compiler_llvm_context::Build) -> Self {
        Self {
            path,
            identifier,
            build,
        }
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
    ) -> anyhow::Result<()> {
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
                    "Refusing to overwrite an existing file {:?} (use --overwrite to force).",
                    file_path
                );
            } else {
                File::create(&file_path)
                    .map_err(|error| {
                        anyhow::anyhow!("File {:?} creating error: {}", file_path, error)
                    })?
                    .write_all(self.build.assembly_text.as_bytes())
                    .map_err(|error| {
                        anyhow::anyhow!("File {:?} writing error: {}", file_path, error)
                    })?;
            }
        }

        if output_binary {
            let file_name = format!("{}.{}", file_name, compiler_common::EXTENSION_ZKEVM_BINARY);
            let mut file_path = path.to_owned();
            file_path.push(file_name);

            if file_path.exists() && !overwrite {
                eprintln!(
                    "Refusing to overwrite an existing file {:?} (use --overwrite to force).",
                    file_path
                );
            } else {
                File::create(&file_path)
                    .map_err(|error| {
                        anyhow::anyhow!("File {:?} creating error: {}", file_path, error)
                    })?
                    .write_all(self.build.bytecode.as_slice())
                    .map_err(|error| {
                        anyhow::anyhow!("File {:?} writing error: {}", file_path, error)
                    })?;
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
    ) -> anyhow::Result<()> {
        let hexadecimal_bytecode = hex::encode(self.build.bytecode);
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

        combined_json_contract.factory_deps = Some(self.build.factory_dependencies);

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
