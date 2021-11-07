//!
//! The `solc --standard-json` output source.
//!

use serde::Deserialize;

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The source code ID.
    pub id: usize,
}
