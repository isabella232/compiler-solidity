//!
//! The YUL source code type.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;

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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Keyword(Keyword::Bool) => Ok(Self::Bool),
            Lexeme::Keyword(Keyword::Int(bitlength)) => Ok(Self::Int(bitlength)),
            Lexeme::Keyword(Keyword::Uint(bitlength)) => Ok(Self::UInt(bitlength)),
            Lexeme::Identifier(identifier) => Ok(Self::Custom(identifier)),
            lexeme => Err(ParserError::expected_one_of(vec!["{type}"], lexeme, None).into()),
        }
    }

    ///
    /// Converts the type into its LLVM representation.
    ///
    pub fn into_llvm<'ctx, 'src>(
        self,
        context: &LLVMContext<'ctx, 'src>,
    ) -> inkwell::types::IntType<'ctx> {
        match self {
            Self::Bool => context.integer_type(compiler_common::BITLENGTH_BOOLEAN),
            Self::Int(bitlength) => context.integer_type(bitlength),
            Self::UInt(bitlength) => context.integer_type(bitlength),
            Self::Custom(_) => context.field_type(),
        }
    }
}
