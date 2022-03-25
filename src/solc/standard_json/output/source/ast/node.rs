//!
//! The `solc --standard-json` AST node.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The `solc --standard-json` AST node.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    /// The node type.
    pub node_type: String,
    /// The node name.
    pub name: Option<String>,
}
