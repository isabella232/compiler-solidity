//!
//! Translates the transaction return operations.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the normal return.
///
pub fn r#return<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let function = context.function().to_owned();

    if let Target::x86 = context.target {
        let source = context.access_heap(
            arguments[0].into_int_value(),
            Some(context.integer_type(compiler_common::bitlength::BYTE)),
        );
        if let Some(return_pointer) = function.return_pointer() {
            context
                .builder
                .build_memcpy(
                    return_pointer,
                    (compiler_common::size::BYTE) as u32,
                    source,
                    (compiler_common::size::BYTE) as u32,
                    arguments[1].into_int_value(),
                )
                .expect("Return memory copy failed");
        }
        context.build_unconditional_branch(function.return_block);
        return None;
    }

    let source = context.builder.build_int_to_ptr(
        arguments[0].into_int_value(),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Heap.into()),
        "return_source_pointer",
    );

    if context.test_entry_hash.is_some() {
        if let Some(return_pointer) = function.return_pointer() {
            let result = context.build_load(source, "return_result");
            context.build_store(return_pointer, result);
        }
        context.build_unconditional_branch(function.return_block);
        return None;
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyToParent);

    let destination = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Parent.into()),
        "return_destination_pointer",
    );

    let size = arguments[1].into_int_value();

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
pub fn revert<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let function = context.function().to_owned();

    if let Target::x86 = context.target {
        return None;
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyToParent);

    let source = context.builder.build_int_to_ptr(
        arguments[0].into_int_value(),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Heap.into()),
        "revert_source_pointer",
    );

    let destination = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Parent.into()),
        "revert_destination_pointer",
    );

    let size = arguments[1].into_int_value();
    let size_remainder = context.builder.build_int_unsigned_rem(
        size,
        context.field_const(compiler_common::size::FIELD as u64),
        "revert_size_remainder",
    );
    let size_padding = context.builder.build_int_sub(
        context.field_const(compiler_common::size::FIELD as u64),
        size_remainder,
        "revert_size_padding",
    );
    let size_padded = context
        .builder
        .build_int_add(size, size_padding, "revert_size_padded");

    context.build_call(
        intrinsic,
        &[
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            size_padded.as_basic_value_enum(),
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
