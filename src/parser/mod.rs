//!
//! The YUL syntax tree.
//!

pub mod block;
pub mod comment;
pub mod r#type;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;

use self::block::statement::Statement;
use self::block::Block;
use self::comment::Comment;

///
/// The compiler parser.
///
pub struct Parser {}

impl Parser {
    ///
    /// The parser entry point.
    ///
    pub fn parse<I>(iter: I) -> Vec<Statement>
    where
        I: Iterator<Item = Lexeme>,
    {
        let mut result = Vec::new();
        let peekable = &mut iter.peekable();
        while let Some(lexeme) = peekable.next() {
            match lexeme {
                Lexeme::Symbol(Symbol::CommentStart) => {
                    Comment::parse(peekable);
                }
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    result.push(Statement::Block(Block::parse(peekable, None)));
                }
                lexeme => panic!("expected /* or {{, got {}", lexeme),
            }
        }
        result
    }
}
