//!
//! File type for input and output files.
//!

use std::ffi::OsStr;
use std::path::Path;

///
/// File type for input and output files.
///
#[derive(Debug)]
pub enum FileType {
    /// A `*.sol` file.
    Solidity,
    /// A `*.yul` file.
    Yul,
    /// A file with unknown extension.
    Unknown(String),
}

impl FileType {
    ///
    /// Extracts the file type from the file `path` based on its extension.
    ///
    pub fn new(path: &Path) -> Self {
        let extension = path.extension().and_then(OsStr::to_str);
        match extension.unwrap_or_default() {
            "sol" => FileType::Solidity,
            "yul" => FileType::Yul,
            extension => FileType::Unknown(extension.to_owned()),
        }
    }
}
