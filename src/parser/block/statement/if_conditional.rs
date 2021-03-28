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
    pub body: Block,
}

impl IfConditional {
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let expression = Expression::parse(iter, None);
        let block_start = iter.next().expect("unexpected eof in if statement");
        if block_start != Lexeme::Symbol(Symbol::BracketCurlyLeft) {
            panic!(
                "unexpected token {} followed after the condition in if statement",
                block_start
            );
        }
        let block = Block::parse(iter);

        Self {
            condition: expression,
            body: block,
        }
    }
}
