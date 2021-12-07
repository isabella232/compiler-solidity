//!
//! Translates the mathematics operation.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the modular addition operation.
///
pub fn add_mod<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    Ok(context.build_invoke(
        context.addmod,
        &[arguments[0], arguments[1], arguments[2]],
        "add_mod_call",
    ))
}

///
/// Translates the modular multiplication operation.
///
pub fn mul_mod<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    Ok(context.build_invoke(
        context.mulmod,
        &[arguments[0], arguments[1], arguments[2]],
        "mul_mod_call",
    ))
}

///
/// Translates the exponent operation.
///
pub fn exponent<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let result_pointer = context.build_alloca(context.field_type(), "exponent_result");
    context.build_store(result_pointer, context.field_const(1));

    let condition_block = context.append_basic_block("exponent_loop_condition");
    let body_block = context.append_basic_block("exponent_loop_body");
    let increment_block = context.append_basic_block("exponent_loop_increment");
    let join_block = context.append_basic_block("exponent_loop_join");

    let index_pointer = context.build_alloca(context.field_type(), "exponent_loop_index_pointer");
    let index_value = context.field_const(0).as_basic_value_enum();
    context.build_store(index_pointer, index_value);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "exponent_loop_index_value_condition")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        arguments[1].into_int_value(),
        "exponent_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "exponent_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(1),
        "exponent_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let intermediate = context
        .build_load(result_pointer, "exponent_loop_intermediate_result")
        .into_int_value();
    let result = context.builder.build_int_mul(
        intermediate,
        arguments[0].into_int_value(),
        "exponent_loop_intermediate_result_multiplied",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "exponent_result");

    Ok(Some(result))
}

///
/// Translates the sign extension operation.
///
pub fn sign_extend<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let bitlength = context.builder.build_int_mul(
        arguments[0].into_int_value(),
        context.field_const(compiler_common::BITLENGTH_BYTE as u64),
        "sign_extend_bitlength_multiplied",
    );
    let bitlength = context.builder.build_int_add(
        bitlength,
        context.field_const((compiler_common::BITLENGTH_BYTE - 1) as u64),
        "sign_extend_bitlength",
    );
    let sign_mask = context.builder.build_left_shift(
        context.field_const(1),
        bitlength,
        "sign_extend_sign_mask",
    );
    let sign_bit = context.builder.build_and(
        arguments[1].into_int_value(),
        sign_mask,
        "sign_extend_sign_bit",
    );
    let sign_bit_truncated = context.builder.build_right_shift(
        sign_bit,
        bitlength,
        false,
        "sign_extend_sign_bit_truncated",
    );

    let value_mask =
        context
            .builder
            .build_int_sub(sign_mask, context.field_const(1), "sign_extend_value_mask");
    let value = context.builder.build_and(
        arguments[1].into_int_value(),
        value_mask,
        "sign_extend_value",
    );

    let sign_fill_bits = context.builder.build_xor(
        value_mask,
        context.field_type().const_all_ones(),
        "sign_fill_bits",
    );
    let sign_fill_bits_checked =
        context
            .builder
            .build_int_mul(sign_fill_bits, sign_bit_truncated, "sign_fill_bits_checked");
    let result = context
        .builder
        .build_int_add(value, sign_fill_bits_checked, "sign_extend_result");

    Ok(Some(result.as_basic_value_enum()))
}
