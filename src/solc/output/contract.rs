//!
//! The `solc --standard-json` output contract.
//!

use serde::Deserialize;

///
/// The `solc --standard-json` output contract.
///
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The contract optimized IR code.
    pub ir_optimized: String,
}
