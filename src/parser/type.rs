//!
//! Datatype for a lexeme for further analysis and translation.
//!

use crate::llvm::Generator;

///
/// Datatype for a lexeme for further analysis and translation.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Bool,
    Int(u32),
    UInt(u32),
    Unknown(String),
}

impl Default for Type {
    fn default() -> Self {
        Self::UInt(256)
    }
}

impl Type {
    #[allow(clippy::wrong_self_convention)]
    pub fn into_llvm<'a, 'ctx>(
        self,
        context: &Generator<'a, 'ctx>,
    ) -> inkwell::types::IntType<'ctx> {
        match self {
            Self::Bool => context.llvm.bool_type(),
            Self::Int(bitlength) => context.llvm.custom_width_int_type(bitlength),
            Self::UInt(bitlength) => context.llvm.custom_width_int_type(bitlength),
            Self::Unknown(_) => unreachable!(),
        }
    }
}
