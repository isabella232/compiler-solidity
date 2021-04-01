//!
//! The compiler lexer tests.
//!

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::literal::integer::Integer as IntegerLiteral;
use crate::lexer::lexeme::literal::Literal;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

///
/// Consumes the source code and returns the vector of lexems.
///
fn tokenize(input: &str) -> Vec<Lexeme> {
    Lexer::new(input.to_owned()).tokenize()
}

#[test]
fn ok_identifiers_with_whitespaces() {
    assert_eq!(
        tokenize("   a    b c\td"),
        [
            Lexeme::Identifier("a".to_owned()),
            Lexeme::Identifier("b".to_owned()),
            Lexeme::Identifier("c".to_owned()),
            Lexeme::Identifier("d".to_owned()),
        ]
    );
}

#[test]
fn ok_identifiers_with_comments() {
    assert_eq!(
        tokenize("   a////comment\nb c\td//comment"),
        [
            Lexeme::Identifier("a".to_owned()),
            Lexeme::Identifier("b".to_owned()),
            Lexeme::Identifier("c".to_owned()),
            Lexeme::Identifier("d".to_owned()),
        ]
    );
}

#[test]
#[ignore]
fn ok_multiline_comments_tokenization() {
    assert_eq!(
        tokenize("/*123 comment function ***/{}"),
        [
            Lexeme::Symbol(Symbol::CommentStart),
            Lexeme::Literal(Literal::Integer(IntegerLiteral::new_decimal(
                "123".to_owned()
            ))),
            Lexeme::Identifier("comment".to_owned()),
            Lexeme::Keyword(Keyword::Function),
            Lexeme::Identifier("**".to_owned()),
            Lexeme::Symbol(Symbol::CommentEnd),
            Lexeme::Symbol(Symbol::BracketCurlyLeft),
            Lexeme::Symbol(Symbol::BracketCurlyRight),
        ]
    );
}
