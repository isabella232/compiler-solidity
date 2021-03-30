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
    pub label: Literal,
    /// The case block.
    pub body: Block,
}

impl Case {
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) -> Self {
        let literal = lexer.next();

        match lexer.next() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, got {}", lexeme),
        }

        Self {
            label: Literal {
                value: literal.to_string(),
            },
            body: Block::parse(lexer, None),
        }
    }
}
