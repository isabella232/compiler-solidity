//!
//! The expression statement.
//!

pub mod function_call;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::identifier::Identifier;
use crate::parser::literal::Literal;

use self::function_call::FunctionCall;

///
/// The expression statement.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// The function call subexpression.
    FunctionCall(FunctionCall),
    /// The identifier operand.
    Identifier(Identifier),
    /// The literal operand.
    Literal(Literal),
}

impl Expression {
    pub fn parse<I>(iter: &mut I, initial: Option<Lexeme>) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let lexeme =
            initial.unwrap_or_else(|| iter.next().expect("expected an expression, eof found"));
        match lexeme {
            Lexeme::Keyword(Keyword::True) | Lexeme::Keyword(Keyword::False) => {
                return Expression::Literal(Literal {
                    value: lexeme.to_string(),
                });
            }
            Lexeme::Identifier(identifier) if !Identifier::is_valid(identifier.as_str()) => {
                return Expression::Literal(Literal { value: identifier });
            }
            Lexeme::Identifier(identifier) if identifier.as_str() == "hex" => {
                // TODO: Check the hex
                return Expression::Literal(Literal { value: identifier });
            }
            _ => {}
        }

        match iter.peek().unwrap() {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {
                iter.next();
            }
            _ => {
                return Expression::Identifier(Identifier {
                    name: lexeme.to_string(),
                    yul_type: None,
                })
            }
        }

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

        Self::FunctionCall(FunctionCall {
            name: lexeme.to_string(),
            arguments,
        })
    }
}
