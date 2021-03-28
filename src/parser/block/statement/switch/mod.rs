//!
//! The switch statement.
//!

pub mod case;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;

use self::case::Case;

///
/// The switch statement.
///
#[derive(Debug, PartialEq)]
pub struct Switch {
    /// The expression being matched.
    pub expression: Expression,
    /// The non-default cases.
    pub cases: Vec<Case>,
    /// The optional default case, if `cases` do not cover all possible values.
    pub default: Option<Block>,
}

///
/// The parsing state.
///
pub enum State {
    /// After match expression.
    CaseOrDefaultKeyword,
    /// After `case`.
    CaseBlock,
    /// After `default`.
    DefaultBlock,
}

impl Switch {
    pub fn parse<I>(iter: &mut I, _initial: Option<Lexeme>) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let mut state = State::CaseOrDefaultKeyword;

        let expression = Expression::parse(iter, None);
        let mut cases = Vec::new();
        let mut default = None;

        loop {
            match state {
                State::CaseOrDefaultKeyword => match iter.peek().unwrap() {
                    Lexeme::Keyword(Keyword::Case) => state = State::CaseBlock,
                    Lexeme::Keyword(Keyword::Default) => state = State::DefaultBlock,
                    _ => break,
                },
                State::CaseBlock => {
                    iter.next();
                    cases.push(Case::parse(iter));
                    state = State::CaseOrDefaultKeyword;
                }
                State::DefaultBlock => {
                    iter.next();
                    match iter.next().expect("unexpected eof in switch statement") {
                        Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
                        lexeme => panic!("expected `{{`, got {}", lexeme),
                    }
                    default = Some(Block::parse(iter, None));
                    break;
                }
            }
        }

        if cases.is_empty() && default.is_none() {
            panic!(
                "expected either the 'default' block or at least one 'case' in switch statement"
            );
        }

        Self {
            expression,
            cases,
            default,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_single_case() {
        let input = r#"{
            switch expr case \"a\" {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_single_case_default() {
        let input = r#"{
            switch expr case \"a\" {} default {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_multiple_cases() {
        let input = r#"{
            switch expr case \"a\" {} case \"b\" {} case \"c\" {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_multiple_cases_default() {
        let input = r#"{
            switch expr case \"a\" {} case \"b\" {} case \"c\" {} default {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_default() {
        let input = r#"{
            switch expr default {}
        }"#;

        crate::tests::parse(input);
    }
}
