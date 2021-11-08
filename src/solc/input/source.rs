//!
//! The `solc --standard-json` input source representation.
//!

use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;

///
/// The `solc --standard-json` input source representation.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The source code file content.
    pub content: String,
}

impl From<String> for Source {
    fn from(content: String) -> Self {
        Self { content }
    }
}

impl TryFrom<&Path> for Source {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let content = if path.to_string_lossy() == "-" {
            let mut solidity_code = String::with_capacity(16384);
            std::io::stdin().read_to_string(&mut solidity_code)?;
            solidity_code
        } else {
            std::fs::read_to_string(path)?
        };

        Ok(Self { content })
    }
}
