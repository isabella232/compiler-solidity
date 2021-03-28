//!
//! The switch statement case.
//!

use crate::parser::block::Block;
use crate::parser::literal::Literal;

///
/// The switch statement case.
///
#[derive(Debug, PartialEq)]
pub struct Case {
    pub label: Literal,
    pub body: Block,
}
