//!
//! The YUL syntax tree.
//!

pub mod block;
pub mod comment;
pub mod identifier;
pub mod r#type;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

use self::block::statement::Statement;
use self::block::Block;
use self::comment::Comment;

///
/// The upper module.
///
pub struct Module {
    /// The statement list.
    pub statements: Vec<Statement>,
}

impl Module {
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) -> Self {
        let mut statements = Vec::new();

        loop {
            let lexeme = lexer.next();
            match lexeme {
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    statements.push(Statement::Block(Block::parse(lexer, None)));
                }
                Lexeme::Symbol(Symbol::CommentStart) => {
                    Comment::parse(lexer, None);
                }
                Lexeme::EndOfFile => break,
                lexeme => panic!("expected one of `/*`, `{{`, got {}", lexeme),
            }
        }

        Self { statements }
    }
}
