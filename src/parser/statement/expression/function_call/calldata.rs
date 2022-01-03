//!
//! Translates the calldata instructions.
//!

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
