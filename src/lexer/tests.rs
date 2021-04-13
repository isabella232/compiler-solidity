//!
//! The compiler lexer tests.
//!

use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

///
/// Consumes the source code and returns the vector of lexems.
///
fn tokenize(input: &str) -> Vec<Lexeme> {
    Lexer::new(input.to_owned())
        .tokenize()
        .expect("Test data is valid")
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
