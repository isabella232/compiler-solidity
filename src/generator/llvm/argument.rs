//!
//! The LLVM argument with metadata.
//!

///
/// The LLVM argument with metadata.
///
#[derive(Debug, Clone)]
pub struct Argument<'ctx> {
    /// The actual LLVM operand.
    pub value: inkwell::values::BasicValueEnum<'ctx>,
    /// The original AST value. Used mostly for string literals.
    pub original: Option<String>,
}

impl<'ctx> Argument<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(value: inkwell::values::BasicValueEnum<'ctx>) -> Self {
        Self {
            value,
            original: None,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_with_original(
        value: inkwell::values::BasicValueEnum<'ctx>,
        original: String,
    ) -> Self {
        Self {
            value,
            original: Some(original),
        }
    }

    ///
    /// Returns the inner LLVM value.
    ///
    pub fn to_llvm(&self) -> inkwell::values::BasicValueEnum<'ctx> {
        self.value
    }
}

impl<'ctx> From<inkwell::values::BasicValueEnum<'ctx>> for Argument<'ctx> {
    fn from(value: inkwell::values::BasicValueEnum<'ctx>) -> Self {
        Self::new(value)
    }
}
