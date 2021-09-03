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
    let pointer = context.access_heap(arguments[0].into_int_value(), "memory_load_pointer");
    let result = context.build_load(pointer, "memory_load_result");
    Some(result)
}

///
/// Translates the heap memory store.
///
pub fn store<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let offset = context.adjust_offset(arguments[0].into_int_value(), "memory_store_offset");
    let pointer = context.access_heap(offset, "memory_store_pointer");
    context.build_store(pointer, arguments[1]);

    None
}

///
/// Translates the heap memory byte store.
///
pub fn store_byte<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let offset_remainder_bytes = context.builder.build_int_unsigned_rem(
        arguments[0].into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "memory_store_byte_offset_remainder_bytes",
    );
    let offset_remainder_bits = context.builder.build_int_mul(
        offset_remainder_bytes,
        context.field_const(compiler_common::bitlength::BYTE as u64),
        "memory_store_byte_offset_remainder_bits",
    );
    let offset = context.builder.build_int_sub(
        arguments[0].into_int_value(),
        offset_remainder_bytes,
        "memory_store_byte_offset",
    );

    let pointer = context.access_heap(offset, "original_value_pointer");

    let original_value = context
        .build_load(pointer, "original_value")
        .into_int_value();
    let original_value_mask = context.builder.build_left_shift(
        context.field_const(0xff),
        offset_remainder_bits,
        "memory_store_byte_original_value_mask",
    );
    let original_value_mask_inverted = context.builder.build_xor(
        original_value_mask,
        context.field_type().const_all_ones(),
        "memory_store_byte_original_value_mask_inverted",
    );
    let original_value_with_empty_byte = context.builder.build_and(
        original_value,
        original_value_mask_inverted,
        "original_value_with_empty_byte",
    );

    let value_shifted = context.builder.build_left_shift(
        arguments[1].into_int_value(),
        offset_remainder_bits,
        "memory_store_byte_value_shifted",
    );
    let result = context.builder.build_or(
        original_value_with_empty_byte,
        value_shifted,
        "memory_store_byte_result",
    );

    context.build_store(pointer, result);

    None
}
