//!
//! Datatype for a lexeme for further analysis and translation.
//!

use crate::generator::llvm::Context;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

///
/// Datatype for a lexeme for further analysis and translation.
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
        Self::UInt(256)
    }
}

impl Type {
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let lexeme = initial.unwrap_or_else(|| lexer.next());

        match lexeme {
            Lexeme::Keyword(Keyword::Bool) => Self::Bool,
            Lexeme::Keyword(Keyword::Int(bitlength)) => Self::Int(bitlength),
            Lexeme::Keyword(Keyword::Uint(bitlength)) => Self::UInt(bitlength),
            Lexeme::Identifier(identifier) => Self::Custom(identifier),
            lexeme => panic!("Expected type, got {}", lexeme),
        }
    }

    pub fn into_llvm<'ctx>(self, context: &Context<'ctx>) -> inkwell::types::IntType<'ctx> {
        match self {
            Self::Bool => context.llvm.bool_type(),
            Self::Int(bitlength) => context.integer_type(bitlength),
            Self::UInt(bitlength) => context.integer_type(bitlength),
            Self::Custom(_) => todo!(),
        }
    }
}
