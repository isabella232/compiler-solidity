//!
//! Translates the return data instructions.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the return data size.
///
pub fn size<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let pointer = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Child,
        "return_data_size_pointer",
    );
    let value = context.build_load(pointer, "return_data_size_value");

    Some(value.as_basic_value_enum())
}

///
/// Translates the return data copy.
///
pub fn copy<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let destination = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "return_data_copy_destination_pointer",
    );

    let source_offset_shift =
        compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD;
    let source_offset = context.builder.build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "return_data_copy_source_offset",
    );
    let source = context.access_memory(
        source_offset,
        compiler_common::AddressSpace::Child,
        "return_data_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    context.build_memcpy(
        Intrinsic::MemoryCopyFromChild,
        destination,
        source,
        size,
        "return_data_copy_memcpy_from_child",
    );

    None
}
