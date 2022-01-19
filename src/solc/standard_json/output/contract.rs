//!
//! The `solc --standard-json` output contract.
//!

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

///
/// The `solc --standard-json` output contract.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The contract optimized IR code.
    #[serde(skip_serializing)]
    pub ir_optimized: String,
    /// The contract ABI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<Value>,
    /// Contract's bytecode and related objects
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<Value>,
    /// Contract's factory dependencies
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factory_dependencies: Option<HashMap<String, String>>,
    /// Contract's zkEVM bytecode hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
