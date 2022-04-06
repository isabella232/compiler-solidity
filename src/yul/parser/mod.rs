//!
//! The YUL code block.
//!

pub mod identifier;
pub mod statement;
pub mod r#type;

use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;

///
/// Returns the `token` value if it is `Some(_)`, otherwise takes the next token from the `stream`.
///
pub fn take_or_next(mut lexeme: Option<Lexeme>, lexer: &mut Lexer) -> anyhow::Result<Lexeme> {
    match lexeme.take() {
        Some(lexeme) => Ok(lexeme),
        None => Ok(lexer.next()?),
    }
}
