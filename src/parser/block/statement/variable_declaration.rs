//!
//! The variable declaration statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::Expression;
use crate::parser::identifier::Identifier;

///
/// The variable declaration statement.
///
#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    pub names: Vec<Identifier>,
    pub initializer: Option<Expression>,
}

impl VariableDeclaration {
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let decl = Identifier::parse_typed_list(iter, Lexeme::Symbol(Symbol::Assignment));
        let init = Expression::parse(iter, None);

        Self {
            names: decl,
            initializer: Some(init),
        }
    }
}
