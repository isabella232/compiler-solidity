//!
//! The `solc --standard-json` output error source location.
//!

use serde::Deserialize;

///
/// The `solc --standard-json` output error source location.
///
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    /// The start location.
    pub start: isize,
    /// The source file path.
    pub file: String,
    /// The end location.
    pub end: isize,
}
