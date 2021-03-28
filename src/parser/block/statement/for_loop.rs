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
    /// The index variables initialization block.
    pub initializer: Block,
    /// The continue condition block.
    pub condition: Expression,
    /// The index variables mutating block.
    pub finalizer: Block,
    /// The loop body.
    pub body: Block,
}

impl ForLoop {
    pub fn parse<I>(iter: &mut I, _initial: Option<Lexeme>) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        match iter.next().unwrap() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let initializer = Block::parse(iter, None);

        let condition = Expression::parse(iter, None);

        match iter.next().unwrap() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let finalizer = Block::parse(iter, None);

        match iter.next().unwrap() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let body = Block::parse(iter, None);

        Self {
            initializer,
            condition,
            finalizer,
            body,
        }
    }
}
