//!
//! Translates the hash instruction.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the hash instruction.
///
pub fn keccak256<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return Some(context.field_const(0).as_basic_value_enum());
    }

    let condition_block = context.append_basic_block("keccak256_condition");
    let body_block = context.append_basic_block("keccak256_body");
    let increment_block = context.append_basic_block("keccak256_increment");
    let join_block = context.append_basic_block("keccak256_join");

    let index_pointer = context.build_alloca(context.field_type(), "keccak256_index_pointer");
    let index_value = context
        .build_load(index_pointer, "keccak256_index_value")
        .into_int_value();
    let pointer = context.access_heap(index_value, None);
    let value = context.build_load(pointer, "keccak256_first_value");
    let intrinsic = context.get_intrinsic_function(Intrinsic::HashAbsorbReset);
    context.build_call(intrinsic, &[value], "keccak256_call_hash_absorb_reset");
    let range_start = context.builder.build_int_add(
        arguments[0].into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "keccak256_range_start",
    );
    let length = context.builder.build_int_sub(
        arguments[1].into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "keccak256_range_length",
    );
    let range_end = context
        .builder
        .build_int_add(range_start, length, "keccak256_range_end");
    context.build_store(index_pointer, range_start);

    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(index_pointer, "keccak256_index_value_condition")
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        range_end,
        "keccak256_condition_comparison",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "keccak256_index_value")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(compiler_common::size::FIELD as u64),
        "keccak256_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let index_value = context
        .build_load(index_pointer, "keccak256_body_index_value")
        .into_int_value();
    let pointer = context.access_heap(index_value, None);
    let value = context.build_load(pointer, "keccak256_next_value");
    let intrinsic = context.get_intrinsic_function(Intrinsic::HashAbsorb);
    context.build_call(intrinsic, &[value], "keccak256_call_hash_absorb");
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    let intrinsic = context.get_intrinsic_function(Intrinsic::HashOutput);
    let result = context
        .build_call(intrinsic, &[], "keccak256_call_hash_output")
        .expect("Hash output function always returns a value");

    Some(result)
}
