//!
//! The switch statement case.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::block::statement::expression::literal::Literal;
use crate::parser::block::Block;

///
/// The switch statement case.
///
#[derive(Debug, PartialEq)]
pub struct Case {
    /// The matched constant.
    pub literal: Literal,
    /// The case block.
    pub block: Block,
}

impl Case {
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let literal = match initial.unwrap_or_else(|| lexer.next()) {
            lexeme @ Lexeme::Literal(_) => Literal::parse(lexer, Some(lexeme)),
            lexeme => panic!("expected literal, got {}", lexeme),
        };

        match lexer.next() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, got {}", lexeme),
        }

        let block = Block::parse(lexer, None);

        Self { literal, block }
    }
}
