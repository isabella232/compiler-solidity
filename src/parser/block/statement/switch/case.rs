//!
//! The switch statement case.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::literal::Literal;
use crate::parser::block::Block;

///
/// The switch statement case.
///
#[derive(Debug, PartialEq)]
pub struct Case {
    /// The matched constant.
    pub label: Literal,
    /// The case block.
    pub body: Block,
}

impl Case {
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let literal = iter.next().expect("unexpected eof in switch statement");

        match iter.next().expect("unexpected eof in switch statement") {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, got {}", lexeme),
        }

        Self {
            label: Literal {
                value: literal.to_string(),
            },
            body: Block::parse(iter, None),
        }
    }
}
