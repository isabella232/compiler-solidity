//!
//! Translates the transaction return operations.
//!

use crate::generator::llvm::address_space::AddressSpace;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the normal return.
///
pub fn r#return<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let function = context.function().to_owned();

    let source = context.access_memory(
        arguments[0].into_int_value(),
        AddressSpace::Heap,
        "return_source_pointer",
    );

    let destination = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Parent,
        "return_destination_pointer",
    );

    let size = arguments[1].into_int_value();

    context.write_header(size, AddressSpace::Parent);
    context.build_memcpy(
        Intrinsic::MemoryCopyToParent,
        destination,
        source,
        size,
        "return_memcpy_to_parent",
    );

    if context.function().name == compiler_common::LLVM_FUNCTION_SELECTOR
        || context.function().name == compiler_common::LLVM_FUNCTION_CONSTRUCTOR
    {
        context.build_unconditional_branch(function.return_block);
    } else {
        let long_return_flag_pointer = context.access_memory(
            context.field_const(
                (compiler_common::SOLIDITY_MEMORY_OFFSET_EMPTY_SLOT * compiler_common::SIZE_FIELD)
                    as u64,
            ),
            AddressSpace::Heap,
            "long_return_flag_pointer",
        );
        context.build_store(long_return_flag_pointer, context.field_const(1));
        context.build_unconditional_branch(function.throw_block);
    }

    Ok(None)
}

///
/// Translates the revert.
///
pub fn revert<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let function = context.function().to_owned();

    let source = context.access_memory(
        arguments[0].into_int_value(),
        AddressSpace::Heap,
        "revert_source_pointer",
    );

    let destination = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Parent,
        "revert_destination_pointer",
    );

    let size = arguments[1].into_int_value();

    context.write_header(size, AddressSpace::Parent);
    context.build_memcpy(
        Intrinsic::MemoryCopyToParent,
        destination,
        source,
        size,
        "revert_memcpy_to_parent",
    );

    context.build_unconditional_branch(function.throw_block);
    Ok(None)
}

///
/// Translates the stop.
///
pub fn stop<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let function = context.function().to_owned();

    context.write_header(context.field_const(0), AddressSpace::Parent);

    context.build_unconditional_branch(function.return_block);
    Ok(None)
}

///
/// Translates the invalid.
///
pub fn invalid<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let function = context.function().to_owned();

    context.write_header(context.field_const(0), AddressSpace::Parent);

    context.build_unconditional_branch(function.throw_block);
    Ok(None)
}
