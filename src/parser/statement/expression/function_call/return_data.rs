//!
//! Translates the return data instructions.
//!

use inkwell::values::BasicValue;

///
/// Translates the return data size.
///
pub fn size<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let header = context.read_header(compiler_llvm_context::AddressSpace::Child);
    let value = context.builder().build_and(
        header,
        context.field_const(0x00000000ffffffff),
        "calldata_size",
    );

    Ok(Some(value.as_basic_value_enum()))
}

///
/// Translates the return data copy.
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
        "return_data_copy_destination_pointer",
    );

    let source_offset_shift = compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD;
    let source_offset = context.builder().build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "return_data_copy_source_offset",
    );
    let source = context.access_memory(
        source_offset,
        compiler_llvm_context::AddressSpace::Child,
        "return_data_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    context.build_memcpy(
        compiler_llvm_context::IntrinsicFunction::MemoryCopyFromChild,
        destination,
        source,
        size,
        "return_data_copy_memcpy_from_child",
    );

    Ok(None)
}
