//!
//! Translates the calldata instructions.
//!

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the calldata load.
///
pub fn load<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let offset_shift = compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD;
    let offset = context.builder.build_int_add(
        arguments[0].into_int_value(),
        context.field_const(offset_shift as u64),
        "calldata_offset",
    );

    let pointer = context.access_memory(
        offset,
        compiler_common::AddressSpace::Parent,
        "calldata_pointer",
    );
    let value = context.build_load(pointer, "calldata_value");

    Some(value)
}

///
/// Translates the calldata size.
///
pub fn size<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Parent,
        "calldata_size_pointer",
    );
    let value = context.build_load(pointer, "calldata_size_value_cells");

    Some(value)
}

///
/// Translates the calldata copy.
///
pub fn copy<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let copy_block = context.append_basic_block("calldata_if_copy");
    let zero_block = context.append_basic_block("calldata_if_zero");
    let join_block = context.append_basic_block("calldata_if_join");

    let pointer = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Parent,
        "calldata_size_pointer",
    );
    let calldata_size = context
        .build_load(pointer, "calldata_size_value_cells")
        .into_int_value();

    let range_end_bytes = context.builder.build_int_add(
        arguments[1].into_int_value(),
        arguments[2].into_int_value(),
        "calldata_range_end_bytes",
    );
    let range_end = context.builder.build_int_unsigned_div(
        range_end_bytes,
        context.field_const(compiler_common::size::FIELD as u64),
        "calldata_range_end",
    );

    let is_calldata_available = context.builder.build_int_compare(
        inkwell::IntPredicate::UGE,
        calldata_size,
        range_end,
        "calldata_is_available",
    );
    context.build_conditional_branch(is_calldata_available, copy_block, zero_block);

    context.set_basic_block(copy_block);
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "calldata_copy_destination_pointer",
    );

    let source_offset_shift =
        compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD;
    let source_offset = context.builder.build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "calldata_copy_source_offset",
    );
    let source = context.access_memory(
        source_offset,
        compiler_common::AddressSpace::Parent,
        "calldata_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    context.build_memcpy(
        Intrinsic::MemoryCopyFromParent,
        destination,
        source,
        size,
        "calldata_copy_memcpy_from_parent",
    );
    context.build_unconditional_branch(join_block);

    // TODO: remove if VM provides zeros after actual calldata
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
    let pointer = context.access_memory(
        index_value,
        compiler_common::AddressSpace::Heap,
        "calldata_copy_zero_pointer_body",
    );
    context.build_store(pointer, context.field_const(0));
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);
    None
}

///
/// Translates the calldata copy from the `codecopy` instruction.
///
pub fn codecopy<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "calldata_codecopy_destination_pointer",
    );

    let source = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Parent,
        "calldata_codecopy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    context.build_memcpy(
        Intrinsic::MemoryCopyFromParent,
        destination,
        source,
        size,
        "calldata_codecopy_memcpy_from_parent",
    );

    None
}
