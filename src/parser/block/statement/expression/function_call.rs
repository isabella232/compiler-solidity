//!
//! The function call subexpression.
//!

use crate::parser::block::statement::expression::Expression;

///
/// The function call subexpression.
///
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    /// The function name.
    pub name: String,
    /// The function arguments expression list.
    pub arguments: Vec<Expression>,
}
