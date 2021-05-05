//!
//! The LLVM generator function return entity.
//!

///
/// The LLVM generator function return entity.
///
#[derive(Debug, Clone)]
pub enum Return<'ctx> {
    /// The function does not return a value.
    None,
    /// The function returns a primitive value.
    Primitive {
        /// The pointer allocated within the function.
        pointer: inkwell::values::PointerValue<'ctx>,
    },
    /// The function returns a compound value.
    /// In this case, the return pointer is allocated on the stack by the callee.
    Compound {
        /// The pointer passed as the first function argument.
        pointer: inkwell::values::PointerValue<'ctx>,
        /// The function return type size.
        size: usize,
    },
}

impl<'ctx> Return<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn none() -> Self {
        Self::None
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn primitive(pointer: inkwell::values::PointerValue<'ctx>) -> Self {
        Self::Primitive { pointer }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn compound(pointer: inkwell::values::PointerValue<'ctx>, size: usize) -> Self {
        Self::Compound { pointer, size }
    }

    ///
    /// Returns the pointer to the function return value.
    ///
    pub fn return_pointer(&self) -> Option<inkwell::values::PointerValue<'ctx>> {
        match self {
            Return::None => None,
            Return::Primitive { pointer } => Some(*pointer),
            Return::Compound { pointer, .. } => Some(*pointer),
        }
    }
}
