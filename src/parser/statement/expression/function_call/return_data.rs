//!
//! Translates the return data instructions.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the return data size.
///
pub fn size<'ctx>(
    context: &mut LLVMContext<'ctx>,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return Some(context.field_const(0).as_basic_value_enum());
    }

    let pointer = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
        "return_data_size_pointer",
    );
    let value = context.build_load(pointer, "return_data_size_value_cells");
    let value = context.builder.build_int_mul(
        value.into_int_value(),
        context.field_const(compiler_common::size::FIELD as u64),
        "return_data_size_value_bytes",
    );
    Some(value.as_basic_value_enum())
}

///
/// Translates the return data copy.
///
pub fn copy<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return None;
    }

    let destination = context.builder.build_int_to_ptr(
        arguments[0].into_int_value(),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Heap.into()),
        "return_data_copy_destination_pointer",
    );

    let source_offset_shift =
        compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD - 4;
    let source_offset = context.builder.build_int_add(
        arguments[1].into_int_value(),
        context.field_const(source_offset_shift as u64),
        "return_data_copy_source_offset",
    );
    let source = context.builder.build_int_to_ptr(
        source_offset,
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
        "return_data_copy_source_pointer",
    );

    let size = arguments[2].into_int_value();

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromChild);
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
        "return_data_copy_memcpy_from_child",
    );

    None
}
