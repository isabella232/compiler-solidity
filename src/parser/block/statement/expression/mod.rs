//!
//! The expression statement.
//!

pub mod function_call;
pub mod identifier;
pub mod literal;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;

use self::function_call::FunctionCall;
use self::identifier::Identifier;
use self::literal::Literal;

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
        let lexeme = match initial {
            Some(lexeme) => lexeme,
            None => iter.next().unwrap(),
        };
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
                Expression::FunctionCall(FunctionCall::parse(iter, Some(lexeme)))
            }
            _ => Expression::Identifier(Identifier {
                name: lexeme.to_string(),
                yul_type: None,
            }),
        }
    }
}
