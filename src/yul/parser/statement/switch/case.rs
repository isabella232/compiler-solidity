//!
//! The switch statement case.
//!

use crate::error::Error;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::expression::literal::Literal;

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
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

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
