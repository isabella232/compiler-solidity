//!
//! File type for input and output files.
//!

use std::path::PathBuf;

///
/// File type for input and output files.
///
#[derive(Debug)]
pub enum FileType {
    Solidity,
    Yul,
    Zinc,
    Unknown,
}

impl FileType {
    ///
    /// Provide FileType for a given file based on its extension.
    ///
    pub fn new(file: &PathBuf) -> Self {
        let extension = file.extension().and_then(std::ffi::OsStr::to_str);
        match extension {
            Some("sol") => FileType::Solidity,
            Some("yul") => FileType::Yul,
            Some("zinc") => FileType::Zinc,
            Some(_) | None => FileType::Unknown,
        }
    }
}
