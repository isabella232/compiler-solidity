//!
//! The compiler lexer tests.
//!

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;

#[test]
fn whitespaces_should_be_ignored() {
    assert_eq!(
        crate::tests::tokenize("   a    b c\td"),
        [
            Lexeme::Identifier("a".to_owned()),
            Lexeme::Identifier("b".to_owned()),
            Lexeme::Identifier("c".to_owned()),
            Lexeme::Identifier("d".to_owned()),
        ]
    );
}

#[test]
fn single_line_comments_should_be_ignored() {
    assert_eq!(
        crate::tests::tokenize("   a////comment\nb c\td//comment"),
        [
            Lexeme::Identifier("a".to_owned()),
            Lexeme::Identifier("b".to_owned()),
            Lexeme::Identifier("c".to_owned()),
            Lexeme::Identifier("d".to_owned()),
        ]
    );
}

#[test]
fn multi_line_comments_should_be_tokenized() {
    assert_eq!(
        crate::tests::tokenize("/*123 comment function ***/{}"),
        [
            Lexeme::Symbol(Symbol::CommentStart),
            Lexeme::Identifier("123".to_owned()),
            Lexeme::Identifier("comment".to_owned()),
            Lexeme::Keyword(Keyword::Function),
            Lexeme::Identifier("**".to_owned()),
            Lexeme::Symbol(Symbol::CommentEnd),
            Lexeme::Symbol(Symbol::BracketCurlyLeft),
            Lexeme::Symbol(Symbol::BracketCurlyRight),
        ]
    );
}
