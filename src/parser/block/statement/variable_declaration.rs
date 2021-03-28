//!
//! The variable declaration statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::identifier::Identifier;
use crate::parser::block::statement::expression::Expression;

///
/// The variable declaration statement.
///
#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    /// The variable bindings list.
    pub bindings: Vec<Identifier>,
    /// The variable initializing expression.
    pub expression: Option<Expression>,
}

impl VariableDeclaration {
    pub fn parse<I>(iter: &mut I, _initial: Option<Lexeme>) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let bindings = Identifier::parse_typed_list(iter, Lexeme::Symbol(Symbol::Assignment));

        let expressions = Expression::parse(iter, None);

        Self {
            bindings,
            expression: Some(expressions),
        }
    }
}
