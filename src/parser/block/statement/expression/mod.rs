//!
//! The expression statement.
//!

pub mod function_call;
pub mod identifier;
pub mod literal;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Generator;

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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let lexeme = match initial {
            Some(lexeme) => lexeme,
            None => lexer.next(),
        };
        match lexeme {
            Lexeme::Keyword(Keyword::True) | Lexeme::Keyword(Keyword::False) => {
                return Self::Literal(Literal {
                    value: lexeme.to_string(),
                });
            }
            Lexeme::Identifier(identifier) if !Identifier::is_valid(identifier.as_str()) => {
                return Self::Literal(Literal { value: identifier });
            }
            Lexeme::Identifier(identifier) if identifier.as_str() == "hex" => {
                // TODO: Check the hex
                return Self::Literal(Literal { value: identifier });
            }
            _ => {}
        }

        match lexer.peek() {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {
                lexer.next();
                Self::FunctionCall(FunctionCall::parse(lexer, Some(lexeme)))
            }
            _ => Self::Identifier(Identifier {
                name: lexeme.to_string(),
                yul_type: None,
            }),
        }
    }

    pub fn into_llvm<'ctx, 'a>(
        self,
        context: &'ctx Generator<'ctx, 'a>,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        match self {
            Self::Literal(inner) => inner.into_llvm(context),
            Self::Identifier(inner) => context
                .builder
                .build_load(context.variables[inner.name.as_str()], ""),
            Self::FunctionCall(inner) => inner.into_llvm(context),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_list() {
        let input = r#"{
            id
            3
            foo(x, y)
        }"#;

        crate::tests::parse(input);
    }
}
