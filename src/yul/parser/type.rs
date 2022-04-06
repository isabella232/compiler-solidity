//!
//! The YUL source code type.
//!

use crate::yul::lexer::lexeme::keyword::Keyword;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;

///
/// The YUL source code type.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// The `bool` type.
    Bool,
    /// The `int{N}` type.
    Int(usize),
    /// The `uint{N}` type.
    UInt(usize),
    /// The custom user-defined type.
    Custom(String),
}

impl Default for Type {
    fn default() -> Self {
        Self::UInt(compiler_common::BITLENGTH_FIELD)
    }
}

impl Type {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Keyword(Keyword::Bool) => Ok(Self::Bool),
            Lexeme::Keyword(Keyword::Int(bitlength)) => Ok(Self::Int(bitlength)),
            Lexeme::Keyword(Keyword::Uint(bitlength)) => Ok(Self::UInt(bitlength)),
            Lexeme::Identifier(identifier) => Ok(Self::Custom(identifier)),
            lexeme => anyhow::bail!("Expected one of {:?}, found `{}`", ["{type}"], lexeme),
        }
    }

    ///
    /// Converts the type into its LLVM representation.
    ///
    pub fn into_llvm<'ctx, 'dep, D>(
        self,
        context: &compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> inkwell::types::IntType<'ctx>
    where
        D: compiler_llvm_context::Dependency,
    {
        match self {
            Self::Bool => context.integer_type(compiler_common::BITLENGTH_BOOLEAN),
            Self::Int(bitlength) => context.integer_type(bitlength),
            Self::UInt(bitlength) => context.integer_type(bitlength),
            Self::Custom(_) => context.field_type(),
        }
    }
}
