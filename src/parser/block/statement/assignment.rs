//!
//! The assignment expression statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::identifier::Identifier;
use crate::parser::block::statement::expression::Expression;

///
/// The assignment expression statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub names: Vec<Identifier>,
    pub initializer: Expression,
}

impl Assignment {
    pub fn parse<I>(iter: &mut I, initial: Lexeme) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        match iter.peek() {
            Some(Lexeme::Symbol(Symbol::Assignment)) => {
                iter.next();
                Self {
                    names: vec![Identifier {
                        name: initial.to_string(),
                        yul_type: None,
                    }],
                    initializer: Expression::parse(iter, None),
                }
            }
            Some(Lexeme::Symbol(Symbol::Comma)) => {
                let identifiers = Identifier::parse_list(iter, initial);
                Self {
                    names: identifiers,
                    initializer: Expression::parse(iter, None),
                }
            }
            _ => unreachable!(),
        }
    }
}
