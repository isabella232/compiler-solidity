//!
//! Translates the heap memory operations.
//!

use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the heap memory load.
///
pub fn load<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Some(value) = arguments[0].into_int_value().get_zero_extended_constant() {
        if value % (compiler_common::size::FIELD as u64) != 0 {
            return None;
        }
    }

    let pointer = context.access_heap(arguments[0].into_int_value(), None);

    let value = context.build_load(pointer, "heap_value");

    Some(value)
}

///
/// Translates the heap memory store.
///
pub fn store<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let offset = context.builder.build_int_truncate_or_bit_cast(
        arguments[0].into_int_value(),
        context.integer_type(compiler_common::bitlength::WORD),
        "heap_offset",
    );
    if let Some(value) = offset.get_zero_extended_constant() {
        if value == 0 || value % (compiler_common::size::FIELD as u64) != 0 {
            return None;
        }
    }

    let pointer = context.access_heap(offset, None);

    context.build_store(pointer, arguments[1]);

    None
}

///
/// Translates the heap memory byte store.
///
pub fn store_byte<'ctx>(
    _context: &mut LLVMContext<'ctx>,
    _arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    None
}
