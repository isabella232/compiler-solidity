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
use crate::parser::error::Error as ParserError;

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

        let identifier = match lexeme {
            Lexeme::Literal(_) => return Ok(Self::Literal(Literal::parse(lexer, Some(lexeme))?)),
            Lexeme::Identifier(identifier) => identifier,
            lexeme => {
                return Err(ParserError::expected_one_of(
                    vec!["{literal}", "{identifier}"],
                    lexeme,
                    None,
                )
                .into())
            }
        };

        match lexer.peek()? {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {
                lexer.next()?;
                Ok(Self::FunctionCall(FunctionCall::parse(
                    lexer,
                    Some(Lexeme::Identifier(identifier)),
                )?))
            }
            _ => Ok(Self::Identifier(identifier)),
        }
    }

    ///
    /// Converts the expression into an LLVM value.
    ///
    pub fn into_llvm<'ctx>(
        self,
        context: &mut LLVMContext<'ctx>,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        match self {
            Self::Literal(inner) => Some(inner.into_llvm(context)),
            Self::Identifier(inner) => Some(
                context
                    .builder
                    .build_load(context.function().stack[inner.as_str()], inner.as_str()),
            ),
            Self::FunctionCall(inner) => inner.into_llvm(context),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_list() {
        let input = r#"object "Test" { code {
            id
            3
            foo(x, y)
        }}"#;

        assert!(crate::parse(input).is_ok());
    }
}
