//!
//! The switch statement case.
//!

use crate::error::Error;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::block::Block;
use crate::parser::statement::expression::literal::Literal;

///
/// The switch statement case.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Case {
    /// The matched constant.
    pub literal: Literal,
    /// The case block.
    pub block: Block,
}

impl Case {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let literal = match lexeme {
            lexeme @ Lexeme::Literal(_) => Literal::parse(lexer, Some(lexeme))?,
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["{literal}"], lexeme, None).into())
            }
        };

        let block = Block::parse(lexer, None)?;

        Ok(Self { literal, block })
    }
}
