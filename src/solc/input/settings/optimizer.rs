//!
//! The `solc --standard-json` input settings optimizer representation.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` input settings optimizer representation.
///
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Optimizer {
    /// Whether the optimizer is enabled.
    pub enabled: bool,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self { enabled: false }
    }
}
