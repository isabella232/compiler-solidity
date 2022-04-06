//!
//! The contract data representation.
//!

pub mod source;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::solc::combined_json::contract::Contract as CombinedJsonContract;

use self::source::Source;

///
/// The contract data representation.
///
#[derive(Debug, Clone)]
pub struct Contract {
    /// The absolute file path.
    pub path: String,
    /// The contract type name.
    pub name: String,
    /// The source code data.
    pub source: Source,
    /// The zkEVM text assembly.
    pub assembly_text: Option<String>,
    /// The zkEVM binary assembly.
    pub assembly: Option<zkevm_assembly::Assembly>,
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
    pub fn new(path: String, name: String, source: Source) -> Self {
        Self {
            path,
            name,
            source,
            assembly_text: None,
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
        self.factory_dependencies.insert(hash, path);
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
                    .write_all(
                        self.assembly_text
                            .as_ref()
                            .expect("Always exists")
                            .as_bytes(),
                    )
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
                    .write_all(self.bytecode.as_ref().expect("Always exists").as_slice())
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

impl<D> compiler_llvm_context::WriteLLVM<D> for Contract
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.source.declare(context)
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.source.into_llvm(context)
    }
}
