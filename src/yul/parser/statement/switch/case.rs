//!
//! The switch statement case.
//!

use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        let literal = match lexeme {
            lexeme @ Lexeme::Literal(_) => Literal::parse(lexer, Some(lexeme))?,
            lexeme => {
                anyhow::bail!("Expected one of {:?}, found `{}`", ["{literal}"], lexeme);
            }
        };

        let block = Block::parse(lexer, None)?;

        Ok(Self { literal, block })
    }
}
