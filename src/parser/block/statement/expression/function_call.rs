//!
//! The function call subexpression.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::statement::expression::Expression;

///
/// The function call subexpression.
///
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    /// The function name.
    pub name: String,
    /// The function arguments expression list.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn parse<I>(iter: &mut I, initial: Option<Lexeme>) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let name = match initial.expect("Always exists") {
            Lexeme::Identifier(identifier) => identifier,
            lexeme => panic!("expected an identifier, found {}", lexeme),
        };

        // function call
        let mut arguments = Vec::new();
        while let Some(lexeme) = iter.next() {
            if lexeme == Lexeme::Symbol(Symbol::ParenthesisRight) {
                break;
            }

            arguments.push(Expression::parse(iter, Some(lexeme)));

            match iter.peek().unwrap() {
                Lexeme::Symbol(Symbol::Comma) => {
                    iter.next();
                    continue;
                }
                Lexeme::Symbol(Symbol::ParenthesisRight) => {
                    iter.next();
                    break;
                }
                _ => break,
            }
        }

        Self { name, arguments }
    }
}
