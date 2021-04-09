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
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) {
        loop {
            let lexeme = initial.take().unwrap_or_else(|| lexer.next());

            match lexeme {
                Lexeme::Symbol(Symbol::CommentEnd) => break,
                Lexeme::EndOfFile => panic!("Expected `*/`, found EOF"),
                _ => continue,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::Block;
    use crate::parser::Module;

    #[test]
    #[ignore]
    fn ok_parse() {
        let input = "/*123 comment ***/{}";

        assert_eq!(
            crate::parse(input),
            Module {
                block: Block { statements: vec![] }
            }
        );
    }

    #[test]
    #[should_panic]
    fn error_parse_expected_comment_end() {
        let input = "/* xxx yyy";

        crate::parse(input);
    }
}
