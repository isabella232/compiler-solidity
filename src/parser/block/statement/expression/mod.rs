//!
//! The expression statement.
//!

pub mod function_call;
pub mod literal;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

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
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        if let Lexeme::Literal(_) = lexeme {
            return Ok(Self::Literal(Literal::parse(lexer, Some(lexeme))?));
        }

        match lexer.peek()? {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {
                lexer.next()?;
                Ok(Self::FunctionCall(FunctionCall::parse(
                    lexer,
                    Some(lexeme),
                )?))
            }
            _ => Ok(Self::Identifier(lexeme.to_string())),
        }
    }

    ///
    /// Converts the expression into an LLVM value.
    ///
    pub fn into_llvm<'ctx>(
        self,
        context: &LLVMContext<'ctx>,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        match self {
            Self::Literal(inner) => Some(inner.into_llvm(context)),
            Self::Identifier(inner) => Some(
                context
                    .builder
                    .build_load(context.variables[inner.as_str()], inner.as_str()),
            ),
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

        assert!(crate::parse(input).is_ok());
    }
}
