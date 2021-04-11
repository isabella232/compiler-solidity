//!
//! The YUL source code comment.
//!

use crate::error::Error;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;

///
/// The YUL source code comment.
///
pub struct Comment;

impl Comment {
    ///
    /// Skips all lexemes until `*/` is found.
    ///
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) -> Result<(), Error> {
        loop {
            let lexeme = crate::parser::take_or_next(initial.take(), lexer)?;

            match lexeme {
                Lexeme::Symbol(Symbol::CommentEnd) => break,
                lexeme @ Lexeme::EndOfFile => {
                    return Err(ParserError::expected_one_of(vec!["*/"], lexeme, None).into())
                }
                _ => continue,
            }
        }

        Ok(())
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
            Ok(Module {
                block: Block { statements: vec![] }
            })
        );
    }

    #[test]
    fn error_expected_comment_end() {
        let input = "/* xxx yyy";

        assert!(crate::parse(input).is_err());
    }
}
