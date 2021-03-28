//!
//! The switch statement.
//!

pub mod case;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;
use crate::parser::literal::Literal;

use self::case::Case;

///
/// The switch statement.
///
#[derive(Debug, PartialEq)]
pub struct Switch {
    pub expression: Expression,
    pub cases: Vec<Case>,
    pub default: Option<Block>,
}

impl Switch {
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let expression = Expression::parse(iter, None);
        let mut cases = Vec::new();

        while let Lexeme::Keyword(Keyword::Case) =
            iter.peek().expect("unexpected eof in switch statement")
        {
            iter.next();

            // TODO: Check literal
            let literal = iter.next().expect("unexpected eof in switch statement");
            if iter.next().expect("unexpected eof in switch statement")
                != Lexeme::Symbol(Symbol::BracketCurlyLeft)
            {
                panic!("expected block in switch case");
            }
            cases.push(Case {
                label: Literal {
                    value: literal.to_string(),
                },
                body: Block::parse(iter),
            });
        }

        if let Lexeme::Keyword(Keyword::Default) =
            iter.peek().expect("unexpected eof in switch statement")
        {
            iter.next();

            if iter.next().expect("unexpected eof in switch statement")
                != Lexeme::Symbol(Symbol::BracketCurlyLeft)
            {
                panic!("expected block in switch case");
            }
            return Self {
                expression,
                cases,
                default: Some(Block::parse(iter)),
            };
        }

        if cases.is_empty() {
            panic!("expected either 'default' or at least one 'case' in switch statemet");
        }

        Self {
            expression,
            cases,
            default: None,
        }
    }
}
