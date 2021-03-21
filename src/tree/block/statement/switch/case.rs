use crate::tree::block::Block;
use crate::tree::literal::Literal;

#[derive(Debug, PartialEq)]
pub struct Case {
    pub label: Literal,
    pub body: Block,
}
