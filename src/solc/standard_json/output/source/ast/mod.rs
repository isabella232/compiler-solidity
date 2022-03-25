//!
//! The `solc --standard-json` AST output.
//!

pub mod node;

use serde::Deserialize;
use serde::Serialize;

use self::node::Node;

///
/// The `solc --standard-json` AST output.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Ast {
    /// The nodes.
    pub nodes: Vec<Node>,
}

impl Ast {
    pub fn last_contract(&self) -> Option<String> {
        self.nodes
            .iter()
            .filter(|node| node.node_type == "ContractDefinition")
            .last()
            .map(|node| {
                node.name
                    .as_ref()
                    .expect("The contract definition node should have a name")
                    .clone()
            })
    }
}
