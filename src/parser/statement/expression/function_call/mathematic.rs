//!
//! Translates the mathematics operation.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the modular addition operation.
///
pub fn add_mod<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return add_mod_x86(context, arguments);
    }

    let zero_block = context.append_basic_block("add_mod_if_zero");
    let non_zero_block = context.append_basic_block("add_mod_if_not_zero");
    let join_block = context.append_basic_block("add_mod_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "add_mod_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[2].into_int_value(),
        context.field_const(0),
        "add_mod_if_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let result = context.builder.build_int_add(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "add_mod_addition",
    );
    let result = context.builder.build_int_unsigned_rem(
        result,
        arguments[2].into_int_value(),
        "add_mod_modulo",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "add_mod_result");

    Some(result)
}

///
/// Translates the modular multiplication operation.
///
pub fn mul_mod<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return mul_mod_x86(context, arguments);
    }

    let zero_block = context.append_basic_block("mul_mod_if_zero");
    let non_zero_block = context.append_basic_block("mul_mod_if_not_zero");
    let join_block = context.append_basic_block("mul_mod_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "mul_mod_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[2].into_int_value(),
        context.field_const(0),
        "mul_mod_if_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let result = context.builder.build_int_mul(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "mul_mod_mulition",
    );
    let result = context.builder.build_int_unsigned_rem(
        result,
        arguments[2].into_int_value(),
        "mul_mod_modulo",
    );
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "mul_mod_result");

    Some(result)
}

///
/// Translates the exponent operation.
///
pub fn exponent<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
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

    Some(result)
}

///
/// Translates the sign extension operation.
///
pub fn sign_extend<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let bitlength = context.builder.build_int_mul(
        arguments[0].into_int_value(),
        context.field_const(compiler_common::bitlength::BYTE as u64),
        "sign_extend_bitlength_multiplied",
    );
    let bitlength = context.builder.build_int_add(
        bitlength,
        context.field_const((compiler_common::bitlength::BYTE - 1) as u64),
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

    Some(result.as_basic_value_enum())
}

///
/// Translates the modular addition operation on the `x86` target.
///
pub fn add_mod_x86<'ctx>(
    context: &mut LLVMContext<'ctx>,
    mut arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("add_mod_if_zero");
    let non_zero_block = context.append_basic_block("add_mod_if_not_zero");
    let join_block = context.append_basic_block("add_mod_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "add_mod_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[2].into_int_value(),
        context.field_const(0),
        "add_mod_if_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let mut result = context.builder.build_int_add(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "add_mod_addition",
    );
    let initial_type = context.field_type();
    if let Target::x86 = context.target {
        let allowed_type = context.integer_type(compiler_common::bitlength::BYTE * 16);
        result = context.builder.build_int_truncate_or_bit_cast(
            result,
            allowed_type,
            "add_mod_addition_truncated",
        );
        arguments[2] = context
            .builder
            .build_int_truncate_or_bit_cast(
                arguments[2].into_int_value(),
                allowed_type,
                "add_mod_modulo_truncated",
            )
            .as_basic_value_enum();
    }
    let mut result = context.builder.build_int_unsigned_rem(
        result,
        arguments[2].into_int_value(),
        "add_mod_result_truncated",
    );
    if let Target::x86 = context.target {
        result = context.builder.build_int_z_extend_or_bit_cast(
            result,
            initial_type,
            "add_mod_result_extended",
        );
    }
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "add_mod_result");

    Some(result)
}

///
/// Translates the modular multiplication operation on the `x86` target.
///
pub fn mul_mod_x86<'ctx>(
    context: &mut LLVMContext<'ctx>,
    mut arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let zero_block = context.append_basic_block("mul_mod_if_zero");
    let non_zero_block = context.append_basic_block("mul_mod_if_not_zero");
    let join_block = context.append_basic_block("mul_mod_if_join");

    let result_pointer = context.build_alloca(context.field_type(), "mul_mod_result_pointer");
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[2].into_int_value(),
        context.field_const(0),
        "mul_mod_if_condition",
    );
    context.build_conditional_branch(condition, zero_block, non_zero_block);

    context.set_basic_block(non_zero_block);
    let mut result = context.builder.build_int_mul(
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "mul_mod_multiplication",
    );
    let initial_type = context.field_type();
    if let Target::x86 = context.target {
        let allowed_type = context.integer_type(compiler_common::bitlength::BYTE * 16);
        result = context.builder.build_int_truncate_or_bit_cast(
            result,
            allowed_type,
            "mul_mod_multiplication_truncated",
        );
        arguments[2] = context
            .builder
            .build_int_truncate_or_bit_cast(
                arguments[2].into_int_value(),
                allowed_type,
                "mul_mod_modulo_truncated",
            )
            .as_basic_value_enum();
    }
    let mut result = context.builder.build_int_unsigned_rem(
        result,
        arguments[2].into_int_value(),
        "mul_mod_result_truncated",
    );
    if let Target::x86 = context.target {
        result = context.builder.build_int_z_extend_or_bit_cast(
            result,
            initial_type,
            "mul_mod_result_extended",
        );
    }
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    context.build_store(result_pointer, context.field_const(0));
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "mul_mod_result");

    Some(result)
}
