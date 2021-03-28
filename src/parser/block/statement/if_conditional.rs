//!
//! The if-conditional statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;

///
/// The if-conditional statement.
///
#[derive(Debug, PartialEq)]
pub struct IfConditional {
    pub condition: Expression,
    pub block: Block,
}

impl IfConditional {
    pub fn parse<I>(iter: &mut I, _initial: Option<Lexeme>) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let condition = Expression::parse(iter, None);

        match iter.next().unwrap() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let block = Block::parse(iter, None);

        Self { condition, block }
    }
}
