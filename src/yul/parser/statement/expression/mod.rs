//!
//! The expression statement.
//!

pub mod function_call;
pub mod literal;

use crate::yul::lexer::lexeme::symbol::Symbol;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;

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
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        let identifier = match lexeme {
            Lexeme::Literal(_) => return Ok(Self::Literal(Literal::parse(lexer, Some(lexeme))?)),
            Lexeme::Identifier(identifier) => identifier,
            lexeme => {
                anyhow::bail!(
                    "Expected one of {:?}, found `{}`",
                    ["{literal}", "{identifier}"],
                    lexeme
                );
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
    pub fn into_llvm<'ctx, D>(
        self,
        context: &mut compiler_llvm_context::Context<'ctx, D>,
    ) -> anyhow::Result<Option<compiler_llvm_context::Argument<'ctx>>>
    where
        D: compiler_llvm_context::Dependency,
    {
        match self {
            Self::Literal(inner) => Ok(Some(inner.into_llvm(context))),
            Self::Identifier(inner) => Ok(Some(
                context
                    .build_load(context.function().stack[inner.as_str()], inner.as_str())
                    .into(),
            )),
            Self::FunctionCall(inner) => Ok(inner
                .into_llvm(context)?
                .map(compiler_llvm_context::Argument::new)),
        }
    }
}
