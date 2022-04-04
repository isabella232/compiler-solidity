//!
//! The JSON assembly runtime code representation.
//!

use serde::Deserialize;
use serde::Serialize;

use crate::evm::assembly::Assembly;

///
/// The JSON assembly runtime code representation.
///
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Data {
    /// The assembly code wrapper.
    Assembly(Assembly),
    /// The hash representation.
    Hash(String),
}

impl Data {
    ///
    /// Gets the `auxdata` string.
    ///
    pub fn get_auxdata(&self) -> Option<&str> {
        match self {
            Self::Assembly(assembly) => assembly.auxdata.as_deref(),
            Self::Hash(_) => None,
        }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assembly(assembly) => writeln!(f, "{}", assembly),
            Self::Hash(value) => writeln!(f, "{}", value),
        }
    }
}
