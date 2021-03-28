//!
//! The for-loop statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;

///
/// The for-loop statement.
///
#[derive(Debug, PartialEq)]
pub struct ForLoop {
    pub initializer: Block,
    pub condition: Expression,
    pub finalizer: Block,
    pub body: Block,
}

impl ForLoop {
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        match iter.next() {
            Some(Lexeme::Symbol(Symbol::BracketCurlyLeft)) => {}
            _ => panic!("expected a block in for loop initializer"),
        }

        let pre = Block::parse(iter);

        let cond = Expression::parse(iter, None);

        match iter.next() {
            Some(Lexeme::Symbol(Symbol::BracketCurlyLeft)) => {}
            _ => panic!("expected a block in for loop initializer"),
        }

        let post = Block::parse(iter);

        match iter.next() {
            Some(Lexeme::Symbol(Symbol::BracketCurlyLeft)) => {}
            _ => panic!("expected a block in for loop initializer"),
        }

        let body = Block::parse(iter);

        Self {
            initializer: pre,
            condition: cond,
            finalizer: post,
            body,
        }
    }
}
