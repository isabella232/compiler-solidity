//!
//! The contract data representation.
//!

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::Error;
use crate::parser::statement::object::Object;

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
        }
    }

    ///
    /// Writes the contract text assembly and bytecode to files.
    ///
    pub fn write_to_directory(
        &self,
        path: &Path,
        output_assembly: bool,
        output_binary: bool,
        overwrite: bool,
    ) -> Result<(), Error> {
        std::fs::create_dir_all(path)?;

        let file_name = format!("{}.{}", self.path.replace('/', "."), self.name);

        if output_assembly {
            let file_name = format!(
                "{}.{}",
                file_name,
                compiler_common::extension::ZKEVM_ASSEMBLY
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
            let file_name = format!("{}.{}", file_name, compiler_common::extension::ZKEVM_BINARY);
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
}
