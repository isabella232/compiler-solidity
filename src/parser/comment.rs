//!
//! The YUL source code comment.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

///
/// The YUL source code comment.
///
pub struct Comment;

impl Comment {
    ///
    /// Skips all lexemes until `*/` is found.
    ///
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) {
        loop {
            match lexer.next() {
                Lexeme::Symbol(Symbol::CommentEnd) => break,
                Lexeme::EndOfFile => panic!("expected `*/`, found EOF"),
                _ => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::statement::Statement;
    use crate::parser::block::Block;

    #[test]
    fn ok_parse() {
        let input = "/*123 comment ***/{}";

        assert_eq!(
            crate::tests::parse(input),
            [Statement::Block(Block { statements: vec![] })]
        );
    }

    #[test]
    #[should_panic]
    fn error_parse_expected_comment_end() {
        let input = "/* xxx yyy";

        crate::tests::parse(input);
    }
}
