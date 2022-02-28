//!
//! The `solc --standard-json` output contract.
//!

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` output contract.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The contract optimized IR code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir_optimized: Option<String>,
    /// The contract ABI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<serde_json::Value>,
    /// Contract's bytecode and related objects
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<serde_json::Value>,
    /// Contract's factory dependencies
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factory_dependencies: Option<HashMap<String, String>>,
    /// Contract's zkEVM bytecode hash
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
