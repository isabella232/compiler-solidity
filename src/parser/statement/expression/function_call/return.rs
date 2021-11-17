//!
//! Translates the transaction return operations.
//!

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

    let source = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "return_source_pointer",
    );

    let destination = context.access_memory(
        context
            .field_const((compiler_common::abi::OFFSET_DATA * compiler_common::size::FIELD) as u64),
        compiler_common::AddressSpace::Parent,
        "return_destination_pointer",
    );

    let size = arguments[1].into_int_value();

    context.write_header(size, compiler_common::AddressSpace::Parent);
    context.build_memcpy(
        Intrinsic::MemoryCopyToParent,
        destination,
        source,
        size,
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

    let source = context.access_memory(
        arguments[0].into_int_value(),
        compiler_common::AddressSpace::Heap,
        "revert_source_pointer",
    );

    let destination = context.access_memory(
        context
            .field_const((compiler_common::abi::OFFSET_DATA * compiler_common::size::FIELD) as u64),
        compiler_common::AddressSpace::Parent,
        "revert_destination_pointer",
    );

    let size = arguments[1].into_int_value();

    context.write_header(size, compiler_common::AddressSpace::Parent);
    context.build_memcpy(
        Intrinsic::MemoryCopyToParent,
        destination,
        source,
        size,
        "revert_memcpy_to_parent",
    );

    context.build_unconditional_branch(function.throw_block);
    None
}
