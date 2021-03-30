//!
//! The expression statement.
//!

pub mod function_call;
pub mod literal;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;

use self::function_call::FunctionCall;
use self::literal::Literal;

///
/// The expression statement.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// The function call subexpression.
    FunctionCall(FunctionCall),
    /// The identifier operand.
    Identifier(String),
    /// The literal operand.
    Literal(Literal),
}

impl Expression {
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let lexeme = match initial {
            Some(lexeme) => lexeme,
            None => lexer.next(),
        };
        if let Lexeme::Literal(_) = lexeme {
            return Self::Literal(Literal::parse(lexer, Some(lexeme)));
        }

        match lexer.peek() {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {
                lexer.next();
                Self::FunctionCall(FunctionCall::parse(lexer, Some(lexeme)))
            }
            _ => Self::Identifier(lexeme.to_string()),
        }
    }

    pub fn into_llvm<'ctx>(self, context: &Context<'ctx>) -> inkwell::values::BasicValueEnum<'ctx> {
        match self {
            Self::Literal(inner) => inner.into_llvm(context),
            Self::Identifier(inner) => context
                .builder
                .build_load(context.variables[inner.as_str()], inner.as_str()),
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
