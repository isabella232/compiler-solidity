//!
//! The `solc --standard-json` output error source location.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` output error source location.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// The start location.
    pub start: isize,
    /// The source file path.
    pub file: String,
    /// The end location.
    pub end: isize,
}
