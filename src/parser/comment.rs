//!
//! The YUL source code comment.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;

///
/// The YUL source code comment.
///
pub struct Comment;

impl Comment {
    ///
    /// Skips all lexemes until `*/` is found.
    ///
    pub fn parse<I>(iter: &mut I)
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        loop {
            let lexeme = iter.next().expect("unexpected eof");
            if let Lexeme::Symbol(Symbol::CommentEnd) = lexeme {
                break;
            }
        }
    }
}
