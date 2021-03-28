//!
//! The block statement.
//!

pub mod assignment;
pub mod expression;
pub mod for_loop;
pub mod function_definition;
pub mod if_conditional;
pub mod switch;
pub mod variable_declaration;

use crate::parser::block::Block;

use self::assignment::Assignment;
use self::expression::Expression;
use self::for_loop::ForLoop;
use self::function_definition::FunctionDefinition;
use self::if_conditional::IfConditional;
use self::switch::Switch;
use self::variable_declaration::VariableDeclaration;

///
/// The block statement.
///
#[derive(Debug, PartialEq)]
pub enum Statement {
    Block(Block),
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    IfConditional(IfConditional),
    Expression(Expression),
    Switch(Switch),
    ForLoop(ForLoop),
    Break,
    Continue,
    Leave,
}
