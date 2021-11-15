//!
//! Translates the heap memory operations.
//!

use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the heap memory load.
///
pub fn load<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "memory_load_pointer",
    );
    let result = context.build_load(pointer, "memory_load_result");
    Some(result)
}

///
/// Translates the heap memory store.
///
pub fn store<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let offset = arguments[0].into_int_value();
    let pointer = context.access_memory(
        offset,
        compiler_common::AddressSpace::Heap,
        "memory_store_pointer",
    );
    context.build_store(pointer, arguments[1]);

    None
}

///
/// Translates the heap memory byte store.
///
pub fn store_byte<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "memory_store_byte_original_value_pointer",
    );

    let original_value = context
        .build_load(pointer, "memory_store_byte_original_value")
        .into_int_value();
    let original_value_shifted_left = context.builder.build_left_shift(
        original_value,
        context.field_const(compiler_common::bitlength::BYTE as u64),
        "memory_store_byte_original_value_shifted_left",
    );
    let original_value_shifted_right = context.builder.build_right_shift(
        original_value_shifted_left,
        context.field_const(compiler_common::bitlength::BYTE as u64),
        false,
        "memory_store_byte_original_value_shifted_right",
    );

    let value_shifted = context.builder.build_left_shift(
        arguments[1].into_int_value(),
        context.field_const(
            ((compiler_common::size::FIELD - 1) * compiler_common::bitlength::BYTE) as u64,
        ),
        "memory_store_byte_value_shifted",
    );
    let result = context.builder.build_or(
        original_value_shifted_right,
        value_shifted,
        "memory_store_byte_result",
    );

    context.build_store(pointer, result);

    None
}
