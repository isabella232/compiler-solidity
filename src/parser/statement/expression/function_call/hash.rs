//!
//! Translates the hash instruction.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::address_space::AddressSpace;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the hash instruction.
///
pub fn keccak256<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let intrinsic = context.get_intrinsic_function(Intrinsic::SwitchContext);
    context.build_call(intrinsic, &[], "keccak256_switch_context");

    let child_pointer_header = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_HEADER * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "keccak256_child_pointer_header",
    );
    context.build_store(child_pointer_header, input_size);

    let child_pointer_data = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "keccak256_child_input_destination",
    );
    let heap_pointer = context.access_memory(
        input_offset,
        AddressSpace::Heap,
        "keccak256_child_input_source",
    );

    context.build_memcpy(
        Intrinsic::MemoryCopyToChild,
        child_pointer_data,
        heap_pointer,
        input_size,
        "keccak256_memcpy_to_child",
    );

    let intrinsic = context.get_intrinsic_function(Intrinsic::DelegateCall);
    let call_definition = context.builder.build_left_shift(
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
