//!
//! The `solc --standard-json` output contract.
//!

use serde::Deserialize;

///
/// The `solc --standard-json` output contract.
///
#[derive(Debug, Deserialize)]
pub struct Contract {
    /// The contract optimized IR code.
    #[serde(rename = "irOptimized")]
    pub ir_optimized: String,
}
