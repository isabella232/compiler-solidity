use crate::tree::block::statement::expression::Expression;
use crate::tree::identifier::Identifier;

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub names: Vec<Identifier>,
    pub initializer: Expression,
}
