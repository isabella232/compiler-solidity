//!
//! Translates the calldata instructions.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the calldata load.
///
pub fn load<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Some(ref test_entry_hash) = context.test_entry_hash {
        let hash = context
            .field_type()
            .const_int_from_string(test_entry_hash, inkwell::types::StringRadix::Hexadecimal)
            .expect("Always valid");
        let hash = context.builder.build_left_shift(
            hash,
            context.field_const(
                ((compiler_common::size::FIELD - 4) * compiler_common::bitlength::BYTE) as u64,
            ),
            "test_entry_hash_shifted",
        );
        return Some(hash.as_basic_value_enum());
    }

    let if_zero_block = context.append_basic_block("calldata_if_zero");
    let if_non_zero_block = context.append_basic_block("calldata_if_not_zero");
    let join_block = context.append_basic_block("calldata_if_join");

    let value_pointer = context.build_alloca(context.field_type(), "calldata_value_pointer");
    context.build_store(value_pointer, context.field_const(0));
    let is_zero = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        arguments[0].into_int_value(),
        context.field_const(0),
        "calldata_if_zero_condition",
    );
    context.build_conditional_branch(is_zero, if_zero_block, if_non_zero_block);

    context.set_basic_block(if_zero_block);
    let offset = context.field_const(
        (compiler_common::contract::ABI_OFFSET_ENTRY_HASH * compiler_common::size::FIELD) as u64,
    );
    let pointer = context.access_calldata(offset);
    let value = context.build_load(pointer, "calldata_entry_hash_value");
    context.build_store(value_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(if_non_zero_block);
    let offset = match context.target {
        Target::x86 => arguments[0].into_int_value(),
        Target::zkEVM => context.builder.build_int_add(
            arguments[0].into_int_value(),
            context.field_const(
                (compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                    * compiler_common::size::FIELD
                    - 4) as u64,
            ),
            "calldata_value_offset",
        ),
    };
    let pointer = context.access_calldata(offset);
    let value = context.build_load(pointer, "calldata_value_non_zero");
    context.build_store(value_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let value = context.build_load(value_pointer, "calldata_value_result");
    Some(value)
}

///
/// Translates the calldata size.
///
pub fn size<'ctx>(
    context: &mut LLVMContext<'ctx>,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return Some(context.field_const(4).as_basic_value_enum());
    }

    if context.test_entry_hash.is_some() {
        return Some(context.field_const(4).as_basic_value_enum());
    }

    let pointer = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::contract::ABI_OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD)
                as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Parent.into()),
        "calldata_size_pointer",
    );
    let value = context.build_load(pointer, "calldata_size_value_cells");
    let value = context.builder.build_int_mul(
        value.into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "calldata_size_value_bytes",
    );
    let value = context.builder.build_int_add(
        value,
        context.field_const(4),
        "calldata_size_value_bytes_with_selector",
    );
    Some(value.as_basic_value_enum())
}

///
/// Translates the calldata copy.
///
pub fn copy<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let copy_block = context.append_basic_block("calldata_if_copy");
    let zero_block = context.append_basic_block("calldata_if_zero");
    let join_block = context.append_basic_block("calldata_if_join");

    let is_calldata_available = match context.target {
        Target::x86 => context
            .integer_type(compiler_common::bitlength::BOOLEAN)
            .const_int(0, false),
        Target::zkEVM if context.test_entry_hash.is_some() => context
            .integer_type(compiler_common::bitlength::BOOLEAN)
            .const_int(0, false),
        Target::zkEVM => {
            let pointer = context.builder.build_int_to_ptr(
                context.field_const(
                    (compiler_common::contract::ABI_OFFSET_CALLDATA_SIZE
                        * compiler_common::size::FIELD) as u64,
                ),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Parent.into()),
                "calldata_size_pointer",
            );
            let calldata_size = context
                .build_load(pointer, "calldata_size_value_cells")
                .into_int_value();
            let calldata_size = context.builder.build_int_mul(
                calldata_size,
                context.field_const(compiler_common::size::FIELD as u64),
                "calldata_size_value_bytes",
            );

            let range_end = context.builder.build_int_sub(
                arguments[1].into_int_value(),
                context.field_const(4),
                "calldata_range_end",
            );

            context.builder.build_int_compare(
                inkwell::IntPredicate::UGT,
                range_end,
                calldata_size,
                "calldata_is_available",
            )
        }
    };
    context.build_conditional_branch(is_calldata_available, copy_block, zero_block);

    context.set_basic_block(copy_block);
    let destination = context.builder.build_int_to_ptr(
        arguments[0].into_int_value(),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Heap.into()),
        "calldata_copy_destination_pointer",
    );

    let source_offset_shift =
        compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD - 4;
    let source_offset = context.builder.build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "calldata_copy_source_offset",
    );
    let source = context.builder.build_int_to_ptr(
        source_offset,
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Parent.into()),
        "calldata_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromParent);
    context.build_call(
        intrinsic,
        &[
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            size.as_basic_value_enum(),
            context
                .integer_type(compiler_common::bitlength::BOOLEAN)
                .const_zero()
                .as_basic_value_enum(),
        ],
        "calldata_copy_memcpy_from_parent",
    );
    context.build_unconditional_branch(join_block);

    context.set_basic_block(zero_block);
    let condition_block = context.append_basic_block("calldata_copy_zero_loop_condition");
    let body_block = context.append_basic_block("calldata_copy_zero_loop_body");
    let increment_block = context.append_basic_block("calldata_copy_zero_loop_increment");

    let index_pointer = context.build_alloca(
        context.field_type(),
        "calldata_copy_zero_loop_index_pointer",
    );
    context.build_store(index_pointer, arguments[0]);
    let range_end = context.builder.build_int_add(
        arguments[0].into_int_value(),
        arguments[2].into_int_value(),
        "calldata_copy_zero_loop_range_end",
    );
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context
        .build_load(
            index_pointer,
            "calldata_copy_zero_loop_index_value_condition",
        )
        .into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        range_end,
        "calldata_copy_zero_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(
            index_pointer,
            "calldata_copy_zero_loop_index_value_increment",
        )
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const(compiler_common::size::FIELD as u64),
        "calldata_copy_zero_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let index_value = context
        .build_load(index_pointer, "calldata_copy_zero_loop_index_value_body")
        .into_int_value();
    let pointer = context.access_heap(index_value, None);
    context.build_store(pointer, context.field_const(0));
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    None
}
