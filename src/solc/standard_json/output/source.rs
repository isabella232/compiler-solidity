//!
//! The `solc --standard-json` output source.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The source code ID.
    pub id: usize,
}
