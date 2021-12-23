//!
//! Translates the calldata instructions.
//!

use inkwell::values::BasicValue;

///
/// Translates the calldata load.
///
pub fn load<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let offset_shift = compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD;
    let offset = context.builder().build_int_add(
        arguments[0].into_int_value(),
        context.field_const(offset_shift as u64),
        "calldata_offset",
    );

    let pointer = context.access_memory(
        offset,
        compiler_llvm_context::AddressSpace::Parent,
        "calldata_pointer",
    );
    let value = context.build_load(pointer, "calldata_value");

    Ok(Some(value))
}

///
/// Translates the calldata size.
///
pub fn size<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let header = context.read_header(compiler_llvm_context::AddressSpace::Parent);
    let value = context.builder().build_and(
        header,
        context.field_const(0x00000000ffffffff),
        "calldata_size",
    );

    Ok(Some(value.as_basic_value_enum()))
}

///
/// Translates the calldata copy.
///
pub fn copy<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_llvm_context::AddressSpace::Heap,
        "calldata_copy_destination_pointer",
    );

    let source_offset_shift = compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD;
    let source_offset = context.builder().build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "calldata_copy_source_offset",
    );
    let source = context.access_memory(
        source_offset,
        compiler_llvm_context::AddressSpace::Parent,
        "calldata_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    context.build_memcpy(
        compiler_llvm_context::IntrinsicFunction::MemoryCopyFromParent,
        destination,
        source,
        size,
        "calldata_copy_memcpy_from_child",
    );

    Ok(None)
}

///
/// Translates the calldata copy from the `codecopy` instruction.
///
pub fn codecopy<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_llvm_context::AddressSpace::Heap,
        "calldata_codecopy_destination_pointer",
    );

    let source = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        compiler_llvm_context::AddressSpace::Parent,
        "calldata_codecopy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    context.build_memcpy(
        compiler_llvm_context::IntrinsicFunction::MemoryCopyFromParent,
        destination,
        source,
        size,
        "calldata_codecopy_memcpy_from_parent",
    );

    Ok(None)
}
