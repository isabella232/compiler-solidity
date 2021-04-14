//!
//! The YUL code block.
//!

pub mod error;
pub mod identifier;
pub mod statement;
pub mod r#type;

use crate::error::Error;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

///
/// Returns the `token` value if it is `Some(_)`, otherwise takes the next token from the `stream`.
///
pub fn take_or_next(mut lexeme: Option<Lexeme>, lexer: &mut Lexer) -> Result<Lexeme, Error> {
    match lexeme.take() {
        Some(lexeme) => Ok(lexeme),
        None => Ok(lexer.next()?),
    }
}
