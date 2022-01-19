//!
//! The `solc --standard-json` output source.
//!

use serde::{Deserialize, Serialize};

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The source code ID.
    pub id: usize,
}
