//!
//! File type for input and output files.
//!

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
    /// A `*.zn` file.
    Zinc,
    /// A file with unknown extension.
    Unknown,
}

impl FileType {
    ///
    /// Extracts the file type from the file `path` based on its extension.
    ///
    pub fn new(path: &Path) -> Self {
        let extension = path.extension().and_then(std::ffi::OsStr::to_str);
        match extension {
            Some("sol") => FileType::Solidity,
            Some("yul") => FileType::Yul,
            Some("zinc") => FileType::Zinc,
            Some(_) | None => FileType::Unknown,
        }
    }
}
