//!
//! Translates the heap memory operations.
//!

use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the heap memory load.
///
pub fn load<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if matches!(context.target, Target::x86) || context.unaligned_memory_access_supported {
        let pointer = context.access_heap(arguments[0].into_int_value(), None);
        let result = context.build_load(pointer, "memory_load_result");
        return Some(result);
    }

    let aligned_block = context.append_basic_block("memory_load_aligned");
    let unaligned_block = context.append_basic_block("memory_load_unaligned");
    let join_block = context.append_basic_block("memory_load_join");

    let result_pointer = context.build_alloca(context.field_type(), "memory_load_result_pointer");

    let second_value_bytes = context.builder.build_int_unsigned_rem(
        arguments[0].into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "memory_load_second_value_bytes",
    );
    let is_aligned = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        second_value_bytes,
        context.field_const(0),
        "memory_load_is_aligned",
    );
    context.build_conditional_branch(is_aligned, aligned_block, unaligned_block);

    context.set_basic_block(aligned_block);
    let pointer = context.access_heap(arguments[0].into_int_value(), None);
    let value = context.build_load(pointer, "memory_load_value_aligned");
    context.build_store(result_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(unaligned_block);
    let second_value_bits = context.builder.build_int_mul(
        second_value_bytes,
        context.field_const(compiler_common::bitlength::BYTE as u64),
        "memory_load_second_value_bits",
    );
    let first_value_bits = context.builder.build_int_sub(
        context.field_const(compiler_common::bitlength::FIELD as u64),
        second_value_bits,
        "memory_load_first_value_bits",
    );

    let first_value_offset = context.builder.build_int_sub(
        arguments[0].into_int_value(),
        second_value_bytes,
        "memory_load_first_value_offset",
    );
    let first_value_pointer = context.access_heap(first_value_offset, None);
    let first_value = context
        .build_load(first_value_pointer, "memory_load_first_value")
        .into_int_value();
    let first_value_shifted = context.builder.build_right_shift(
        first_value,
        second_value_bits,
        false,
        "memory_load_first_value_shifted",
    );

    let second_value_offset = context.builder.build_int_add(
        first_value_offset,
        context.field_const(compiler_common::size::FIELD as u64),
        "memory_load_second_value_offset",
    );
    let second_value_pointer = context.access_heap(second_value_offset, None);
    let second_value = context
        .build_load(second_value_pointer, "memory_load_second_value")
        .into_int_value();
    let second_value_shifted = context.builder.build_left_shift(
        second_value,
        first_value_bits,
        "memory_load_second_value_shifted",
    );

    let value = context.builder.build_int_add(
        first_value_shifted,
        second_value_shifted,
        "memory_load_value_unaligned",
    );
    context.build_store(result_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "memory_load_result");
    Some(result)
}

///
/// Translates the heap memory store.
///
pub fn store<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if matches!(context.target, Target::x86) || context.unaligned_memory_access_supported {
        let pointer = context.access_heap(arguments[0].into_int_value(), None);
        context.build_store(pointer, arguments[1]);
        return None;
    }

    let aligned_block = context.append_basic_block("memory_store_aligned");
    let unaligned_block = context.append_basic_block("memory_store_unaligned");
    let join_block = context.append_basic_block("memory_store_join");

    let second_value_bytes = context.builder.build_int_unsigned_rem(
        arguments[0].into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "memory_store_second_value_bytes",
    );
    let is_aligned = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        second_value_bytes,
        context.field_const(0),
        "memory_store_is_aligned",
    );
    context.build_conditional_branch(is_aligned, aligned_block, unaligned_block);

    context.set_basic_block(aligned_block);
    let pointer = context.access_heap(arguments[0].into_int_value(), None);
    context.build_store(pointer, arguments[1]);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(unaligned_block);
    let second_value_bits = context.builder.build_int_mul(
        second_value_bytes,
        context.field_const(compiler_common::bitlength::BYTE as u64),
        "memory_store_second_value_bits",
    );
    let first_value_bits = context.builder.build_int_sub(
        context.field_const(compiler_common::bitlength::FIELD as u64),
        second_value_bits,
        "memory_store_first_value_bits",
    );

    let first_value = context.builder.build_left_shift(
        arguments[1].into_int_value(),
        second_value_bits,
        "memory_store_first_value",
    );
    let first_value_offset = context.builder.build_int_sub(
        arguments[0].into_int_value(),
        second_value_bytes,
        "memory_store_first_value_offset",
    );
    let first_value_pointer = context.access_heap(first_value_offset, None);
    context.build_store(first_value_pointer, first_value);

    let second_value = context.builder.build_right_shift(
        arguments[1].into_int_value(),
        first_value_bits,
        false,
        "memory_store_second_value",
    );
    let second_value_offset = context.builder.build_int_add(
        first_value_offset,
        context.field_const(compiler_common::size::FIELD as u64),
        "memory_store_second_value_offset",
    );
    let second_value_pointer = context.access_heap(second_value_offset, None);
    context.build_store(second_value_pointer, second_value);

    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
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

    let pointer = context.access_heap(offset, None);

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
