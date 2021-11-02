//!
//! Translates the transaction return operations.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the normal return.
///
pub fn r#return<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let function = context.function().to_owned();

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyToParent);

    let source = context.access_heap(arguments[0].into_int_value(), "return_source_pointer");

    let destination = context.access_calldata(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        "return_destination_pointer",
    );

    let size = arguments[1].into_int_value();
    let size_adjusted = context.ceil32(size, "return_size_adjusted");

    let parent_pointer_return_data_size = context.access_calldata(
        context.field_const(
            (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        "return_destination_size_pointer",
    );
    context.build_store(
        parent_pointer_return_data_size,
        context.builder.build_int_unsigned_div(
            size_adjusted,
            context.field_const(compiler_common::size::FIELD as u64),
            "return_destination_size_cells",
        ),
    );

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
        "return_memcpy_to_parent",
    );

    context.build_unconditional_branch(function.return_block);
    None
}

///
/// Translates the revert.
///
pub fn revert<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let function = context.function().to_owned();

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyToParent);

    let source = context.access_heap(arguments[0].into_int_value(), "revert_source_pointer");

    let destination = context.access_calldata(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        "revert_destination_pointer",
    );

    let size = arguments[1].into_int_value();
    let size_adjusted = context.ceil32(size, "revert_size_adjusted");

    let parent_pointer_return_data_size = context.access_calldata(
        context.field_const(
            (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        "revert_parent_pointer_return_data_size",
    );
    context.build_store(
        parent_pointer_return_data_size,
        context.builder.build_int_unsigned_div(
            size_adjusted,
            context.field_const(compiler_common::size::FIELD as u64),
            "revert_parent_return_data_size_cells",
        ),
    );

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
        "revert_memcpy_to_parent",
    );

    context.build_unconditional_branch(function.throw_block);
    None
}
