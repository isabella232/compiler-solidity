//!
//! Translates the bitwise operations.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the bitwise OR.
///
pub fn or<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if !context.is_native_bitwise_supported {
        return or_loop(context, arguments);
    }

    Some(
        context
            .builder
            .build_or(
                arguments[0].into_int_value(),
                arguments[1].into_int_value(),
                "or_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the bitwise XOR.
///
pub fn xor<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if !context.is_native_bitwise_supported {
        return xor_loop(context, arguments);
    }

    Some(
        context
            .builder
            .build_xor(
                arguments[0].into_int_value(),
                arguments[1].into_int_value(),
                "xor_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the bitwise AND.
///
pub fn and<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if !context.is_native_bitwise_supported {
        return and_loop(context, arguments);
    }

    Some(
        context
            .builder
            .build_and(
                arguments[0].into_int_value(),
                arguments[1].into_int_value(),
                "and_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the bitwise shift left.
///
pub fn shift_left<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if !context.is_native_bitwise_supported {
        return shift_left_loop(context, arguments);
    }

    Some(
        context
            .builder
            .build_left_shift(
                arguments[1].into_int_value(),
                arguments[0].into_int_value(),
                "shift_left_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the bitwise shift right.
///
pub fn shift_right<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if !context.is_native_bitwise_supported {
        return shift_right_loop(context, arguments);
    }

    Some(
        context
            .builder
            .build_right_shift(
                arguments[1].into_int_value(),
                arguments[0].into_int_value(),
                false,
                "shift_right_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the arithmetic bitwise shift right.
///
pub fn shift_right_arithmetic<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if !context.is_native_bitwise_supported {
        return Some(arguments[1]);
    }

    Some(
        context
            .builder
            .build_right_shift(
                arguments[1].into_int_value(),
                arguments[0].into_int_value(),
                true,
                "shift_right_arithmetic_result",
            )
            .as_basic_value_enum(),
    )
}

///
/// Translates the byte extraction.
///
pub fn byte<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let byte_index = context.builder.build_int_sub(
        context.field_const((compiler_common::SIZE_FIELD - 1) as u64),
        arguments[0].into_int_value(),
        "byte_index",
    );
    let byte_bits_offset = context.builder.build_int_mul(
        byte_index,
        context.field_const(compiler_common::BITLENGTH_BYTE as u64),
        "byte_bits_offset",
    );
    let value_shifted = context.builder.build_right_shift(
        arguments[1].into_int_value(),
        byte_bits_offset,
        false,
        "value_shifted",
    );
    let byte_result =
        context
            .builder
            .build_and(value_shifted, context.field_const(0xff), "byte_result");
    Some(byte_result.as_basic_value_enum())
}

///
/// Translates the bitwise OR with loop, when the native operation is not supported.
///
pub fn or_loop<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let condition_block = context.append_basic_block("or_loop_condition");
    let body_block = context.append_basic_block("or_loop_body");
    let increment_block = context.append_basic_block("or_loop_increment");
    let join_block = context.append_basic_block("or_loop_join");

    let result_pointer = context.build_alloca(context.field_type(), "or_loop_result_pointer");
    context.build_store(result_pointer, context.field_const(0));
    let operand_1_pointer = context.build_alloca(context.field_type(), "or_loop_operand_1_pointer");
    context.build_store(operand_1_pointer, arguments[0]);
    let operand_2_pointer = context.build_alloca(context.field_type(), "or_loop_operand_2_pointer");
    context.build_store(operand_2_pointer, arguments[1]);
    let index_pointer = context.build_alloca(context.field_type(), "or_loop_index_pointer");
    context.build_store(index_pointer, context.field_const(0));
    let shift_pointer = context.build_alloca(context.field_type(), "or_loop_shift_pointer");
    context.build_store(shift_pointer, context.field_const(1));
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "or_loop_condition_index_value")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        context.field_const(compiler_common::BITLENGTH_FIELD as u64),
        "or_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "or_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(1),
        "or_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let operand_1 = context
        .build_load(operand_1_pointer, "or_loop_operand_1")
        .into_int_value();
    let operand_2 = context
        .build_load(operand_2_pointer, "or_loop_operand_2")
        .into_int_value();
    let bit_1 =
        context
            .builder
            .build_int_unsigned_rem(operand_1, context.field_const(2), "or_loop_bit_1");
    let bit_2 =
        context
            .builder
            .build_int_unsigned_rem(operand_2, context.field_const(2), "or_loop_bit_2");
    let operand_1 = context.builder.build_int_unsigned_div(
        operand_1,
        context.field_const(2),
        "or_loop_operand_1_divided",
    );
    context.build_store(operand_1_pointer, operand_1);
    let operand_2 = context.builder.build_int_unsigned_div(
        operand_2,
        context.field_const(2),
        "or_loop_operand_2_divided",
    );
    context.build_store(operand_2_pointer, operand_2);
    let bit_result = context
        .builder
        .build_int_add(bit_1, bit_2, "or_loop_bits_added");
    let bit_result = context.builder.build_int_compare(
        inkwell::IntPredicate::UGT,
        bit_result,
        context.field_const(0),
        "or_loop_bits_greater_than_zero",
    );
    let bit_result = context.builder.build_int_z_extend_or_bit_cast(
        bit_result,
        context.field_type(),
        "or_loop_bits_extended",
    );
    let shift_value = context
        .build_load(shift_pointer, "or_loop_shift_value")
        .into_int_value();
    let bit_result = context
        .builder
        .build_int_mul(bit_result, shift_value, "or_loop_bits_shifted");
    let shift_value = context.builder.build_int_mul(
        shift_value,
        context.field_const(2),
        "or_loop_shift_value_doubled",
    );
    context.build_store(shift_pointer, shift_value);
    let result = context
        .build_load(result_pointer, "or_loop_result_value")
        .into_int_value();
    let result = context
        .builder
        .build_int_add(result, bit_result, "or_loop_result_value_added");
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "or_loop_result");

    Some(result)
}

///
/// Translates the bitwise XOR with loop, when the native operation is not supported.
///
pub fn xor_loop<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let condition_block = context.append_basic_block("xor_loop_condition");
    let body_block = context.append_basic_block("xor_loop_body");
    let increment_block = context.append_basic_block("xor_loop_increment");
    let join_block = context.append_basic_block("xor_loop_join");

    let result_pointer = context.build_alloca(context.field_type(), "xor_loop_result_pointer");
    context.build_store(result_pointer, context.field_const(0));
    let operand_1_pointer =
        context.build_alloca(context.field_type(), "xor_loop_operand_1_pointer");
    context.build_store(operand_1_pointer, arguments[0]);
    let operand_2_pointer =
        context.build_alloca(context.field_type(), "xor_loop_operand_2_pointer");
    context.build_store(operand_2_pointer, arguments[1]);
    let index_pointer = context.build_alloca(context.field_type(), "xor_loop_index_pointer");
    context.build_store(index_pointer, context.field_const(0));
    let shift_pointer = context.build_alloca(context.field_type(), "xor_loop_shift_pointer");
    context.build_store(shift_pointer, context.field_const(1));
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "xor_loop_condition_index_value")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        context.field_const(compiler_common::BITLENGTH_FIELD as u64),
        "xor_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "xor_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(1),
        "xor_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let operand_1 = context
        .build_load(operand_1_pointer, "xor_loop_operand_1")
        .into_int_value();
    let operand_2 = context
        .build_load(operand_2_pointer, "xor_loop_operand_2")
        .into_int_value();
    let bit_1 =
        context
            .builder
            .build_int_unsigned_rem(operand_1, context.field_const(2), "xor_loop_bit_1");
    let bit_2 =
        context
            .builder
            .build_int_unsigned_rem(operand_2, context.field_const(2), "xor_loop_bit_2");
    let operand_1 = context.builder.build_int_unsigned_div(
        operand_1,
        context.field_const(2),
        "xor_loop_operand_1_divided",
    );
    context.build_store(operand_1_pointer, operand_1);
    let operand_2 = context.builder.build_int_unsigned_div(
        operand_2,
        context.field_const(2),
        "xor_loop_operand_2_divided",
    );
    context.build_store(operand_2_pointer, operand_2);
    let bit_result = context.builder.build_int_compare(
        inkwell::IntPredicate::NE,
        bit_1,
        bit_2,
        "xor_loop_bits_not_equal",
    );
    let bit_result = context.builder.build_int_z_extend_or_bit_cast(
        bit_result,
        context.field_type(),
        "xor_loop_bits_not_equal_extended",
    );
    let shift_value = context
        .build_load(shift_pointer, "xor_loop_shift_value")
        .into_int_value();
    let bit_result =
        context
            .builder
            .build_int_mul(bit_result, shift_value, "xor_loop_bits_not_equal_shifted");
    let shift_value = context.builder.build_int_mul(
        shift_value,
        context.field_const(2),
        "xor_loop_shift_value_doubled",
    );
    context.build_store(shift_pointer, shift_value);
    let result = context
        .build_load(result_pointer, "xor_loop_result_value")
        .into_int_value();
    let result = context
        .builder
        .build_int_add(result, bit_result, "xor_loop_result_value_added");
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "xor_loop_result");

    Some(result)
}

///
/// Translates the bitwise AND with loop, when the native operation is not supported.
///
pub fn and_loop<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let condition_block = context.append_basic_block("and_loop_condition");
    let body_block = context.append_basic_block("and_loop_body");
    let increment_block = context.append_basic_block("and_loop_increment");
    let join_block = context.append_basic_block("and_loop_join");

    let result_pointer = context.build_alloca(context.field_type(), "and_loop_result_pointer");
    context.build_store(result_pointer, context.field_const(0));
    let operand_1_pointer =
        context.build_alloca(context.field_type(), "and_loop_operand_1_pointer");
    context.build_store(operand_1_pointer, arguments[0]);
    let operand_2_pointer =
        context.build_alloca(context.field_type(), "and_loop_operand_2_pointer");
    context.build_store(operand_2_pointer, arguments[1]);
    let index_pointer = context.build_alloca(context.field_type(), "and_loop_index_pointer");
    context.build_store(index_pointer, context.field_const(0));
    let shift_pointer = context.build_alloca(context.field_type(), "and_loop_shift_pointer");
    context.build_store(shift_pointer, context.field_const(1));
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "and_loop_condition_index_value")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        context.field_const(compiler_common::BITLENGTH_FIELD as u64),
        "and_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "and_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(1),
        "and_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let operand_1 = context
        .build_load(operand_1_pointer, "and_loop_operand_1")
        .into_int_value();
    let operand_2 = context
        .build_load(operand_2_pointer, "and_loop_operand_2")
        .into_int_value();
    let bit_1 =
        context
            .builder
            .build_int_unsigned_rem(operand_1, context.field_const(2), "and_loop_bit_1");
    let bit_2 =
        context
            .builder
            .build_int_unsigned_rem(operand_2, context.field_const(2), "and_loop_bit_2");
    let operand_1 = context.builder.build_int_unsigned_div(
        operand_1,
        context.field_const(2),
        "and_loop_operand_1_divided",
    );
    context.build_store(operand_1_pointer, operand_1);
    let operand_2 = context.builder.build_int_unsigned_div(
        operand_2,
        context.field_const(2),
        "and_loop_operand_2_divided",
    );
    context.build_store(operand_2_pointer, operand_2);
    let bit_result = context
        .builder
        .build_int_mul(bit_1, bit_2, "and_loop_bits_multiplied");
    let bit_result = context.builder.build_int_z_extend_or_bit_cast(
        bit_result,
        context.field_type(),
        "and_loop_bits_extended",
    );
    let shift_value = context
        .build_load(shift_pointer, "and_loop_shift_value")
        .into_int_value();
    let bit_result =
        context
            .builder
            .build_int_mul(bit_result, shift_value, "and_loop_bit_result_doubled");
    let shift_value = context.builder.build_int_mul(
        shift_value,
        context.field_const(2),
        "and_loop_shift_value_doubled",
    );
    context.build_store(shift_pointer, shift_value);
    let result = context
        .build_load(result_pointer, "and_loop_result_value")
        .into_int_value();
    let result = context
        .builder
        .build_int_add(result, bit_result, "and_loop_result_value_added");
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "and_loop_result");

    Some(result)
}

///
/// Translates the bitwise shift left, when the native operation is not supported.
///
pub fn shift_left_loop<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let result_pointer = context.build_alloca(context.field_type(), "shift_left_result_pointer");
    context.build_store(result_pointer, arguments[1]);

    let condition_block = context.append_basic_block("shift_left_loop_condition");
    let body_block = context.append_basic_block("shift_left_loop_body");
    let increment_block = context.append_basic_block("shift_left_loop_increment");
    let join_block = context.append_basic_block("shift_left_loop_join");

    let index_pointer = context.build_alloca(context.field_type(), "shift_left_loop_index_pointer");
    let index_value = context.field_const(0).as_basic_value_enum();
    context.build_store(index_pointer, index_value);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "shift_left_loop_condition_index_value")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        arguments[0].into_int_value(),
        "shift_left_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "shift_left_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(1),
        "shift_left_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let intermediate = context
        .build_load(result_pointer, "shift_left_loop_intermediate")
        .into_int_value();
    let multiplier = context.field_const(2);
    let result = context.builder.build_int_mul(
        intermediate,
        multiplier,
        "shift_left_loop_intermediate_multiplied",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "shift_left_loop_result");

    Some(result)
}

///
/// Translates the bitwise shift right, when the native operation is not supported.
///
pub fn shift_right_loop<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let result_pointer = context.build_alloca(context.field_type(), "shift_right_result_pointer");
    context.build_store(result_pointer, arguments[1]);

    let condition_block = context.append_basic_block("shift_right_loop_condition");
    let body_block = context.append_basic_block("shift_right_loop_body");
    let increment_block = context.append_basic_block("shift_right_loop_increment");
    let join_block = context.append_basic_block("shift_right_loop_join");

    let index_pointer =
        context.build_alloca(context.field_type(), "shift_right_loop_index_pointer");
    let index_value = context.field_const(0).as_basic_value_enum();
    context.build_store(index_pointer, index_value);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "shift_right_loop_condition_index_value")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        arguments[0].into_int_value(),
        "shift_right_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "shift_right_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(1),
        "shift_right_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let intermediate = context
        .build_load(result_pointer, "shift_right_loop_intermediate")
        .into_int_value();
    let divider = context.field_const(2);
    let result = context.builder.build_int_unsigned_div(
        intermediate,
        divider,
        "shift_right_loop_intermediate_divided",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "shift_right_loop_result");

    Some(result)
}
