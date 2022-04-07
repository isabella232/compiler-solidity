//!
//! The `solc --standard-json` AST output.
//!

use serde::Deserialize;
use serde::Serialize;

use crate::solc::standard_json::output::error::Error as SolcStandardJsonOutputError;

///
/// The `solc --standard-json` AST output.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(clippy::upper_case_acronyms)]
#[serde(rename_all = "camelCase")]
pub struct AST {
    /// The node type.
    pub node_type: Option<String>,
    /// The node name.
    pub name: Option<String>,
    /// The node location in the source code.
    pub src: Option<String>,

    /// The next level.
    pub nodes: Option<Vec<Self>>,
    /// The function body.
    pub body: Option<Box<Self>>,
    /// The function statements.
    pub statements: Option<Vec<Self>>,
    /// The function expressions.
    pub expression: Option<Box<Self>>,
    /// The function arguments.
    pub arguments: Option<Vec<Self>>,
}

impl AST {
    ///
    /// Returns the list of warnings for some specific parts of the AST.
    ///
    pub fn get_warnings(&self) -> anyhow::Result<Vec<SolcStandardJsonOutputError>> {
        let mut warnings = Vec::new();
        if let Some(warning) = self.check_ecrecover() {
            warnings.push(warning);
        }

        if let Some(nodes) = self.nodes.as_ref() {
            for node in nodes.iter() {
                warnings.extend(node.get_warnings()?);
            }
        }
        if let Some(body) = self.body.as_ref() {
            warnings.extend(body.get_warnings()?);
        }
        if let Some(nodes) = self.statements.as_ref() {
            for node in nodes.iter() {
                warnings.extend(node.get_warnings()?);
            }
        }
        if let Some(expression) = self.expression.as_ref() {
            warnings.extend(expression.get_warnings()?);
        }
        if let Some(nodes) = self.arguments.as_ref() {
            for node in nodes.iter() {
                warnings.extend(node.get_warnings()?);
            }
        }

        Ok(warnings)
    }

    ///
    /// Checks the AST node for `ecrecover`.
    ///
    pub fn check_ecrecover(&self) -> Option<SolcStandardJsonOutputError> {
        if let Some(node_type) = self.node_type.as_ref() {
            if node_type.as_str() != "FunctionCall" {
                return None;
            }
        }

        let expression = self.expression.as_ref()?;
        if let Some(node_type) = expression.node_type.as_ref() {
            if node_type.as_str() != "Identifier" {
                return None;
            }
        }
        if let Some(name) = expression.name.as_ref() {
            if name.as_str() != "ecrecover" {
                return None;
            }
        }

        Some(SolcStandardJsonOutputError::warning_ecrecover(
            self.src.as_deref(),
        ))
    }

    ///
    /// Returns the name of the last contract.
    ///
    pub fn last_contract_name(&self) -> anyhow::Result<String> {
        self.nodes
            .as_ref()
            .ok_or_else(|| {
                anyhow::anyhow!("The last contract cannot be found in the empty list of nodes")
            })?
            .iter()
            .filter(|node| node.node_type.as_deref() == Some("ContractDefinition"))
            .last()
            .and_then(|node| node.name.as_ref())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("The last contract not found in the AST"))
    }
}
