//!
//! Translates the hash instruction.
//!

use inkwell::values::BasicValue;

///
/// Translates the hash instruction.
///
pub fn keccak256<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::SwitchContext);
    context.build_call(intrinsic, &[], "keccak256_switch_context");

    let child_pointer_header = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_HEADER * compiler_common::SIZE_FIELD) as u64,
        ),
        compiler_llvm_context::AddressSpace::Child,
        "keccak256_child_pointer_header",
    );
    context.build_store(child_pointer_header, input_size);

    let child_pointer_data = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        compiler_llvm_context::AddressSpace::Child,
        "keccak256_child_input_destination",
    );
    let heap_pointer = context.access_memory(
        input_offset,
        compiler_llvm_context::AddressSpace::Heap,
        "keccak256_child_input_source",
    );

    context.build_memcpy(
        compiler_llvm_context::IntrinsicFunction::MemoryCopyToChild,
        child_pointer_data,
        heap_pointer,
        input_size,
        "keccak256_memcpy_to_child",
    );

    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::StaticCall);
    let call_definition = context.builder().build_left_shift(
        context.field_const_str(compiler_common::ABI_ADDRESS_KECCAK256),
        context.field_const((compiler_common::BITLENGTH_X32) as u64),
        "",
    );
    context.build_call(
        intrinsic,
        &[call_definition.as_basic_value_enum()],
        "keccak256_call_external",
    );

    let result = context.build_load(child_pointer_data, "keccak256_result");

    Ok(Some(result))
}
