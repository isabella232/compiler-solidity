//!
//! Translates the contract creation instructions.
//!

use inkwell::values::BasicValue;

///
/// Translates the `datasize` instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
pub fn datasize<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    mut arguments: [compiler_llvm_context::Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let identifier = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`datasize` object identifier is missing"))?;

    let parent = context.module().get_name().to_str().expect("Always valid");

    if identifier.ends_with("_deployed") || identifier.as_str() == parent {
        return Ok(Some(context.field_const(0).as_basic_value_enum()));
    }

    Ok(Some(
        context
            .field_const(compiler_common::SIZE_FIELD as u64)
            .as_basic_value_enum(),
    ))
}

///
/// Translates the `dataoffset` instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
pub fn dataoffset<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    mut arguments: [compiler_llvm_context::Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let identifier = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`dataoffset` object identifier is missing"))?;

    let parent = context.module().get_name().to_str().expect("Always valid");

    if identifier.ends_with("_deployed") || identifier.as_str() == parent {
        return Ok(Some(context.field_const(0).as_basic_value_enum()));
    }

    let hash_value = context
        .compile_dependency(identifier.as_str())
        .map(|hash| context.field_const_str(hash.as_str()))
        .map(inkwell::values::BasicValueEnum::IntValue)?;

    Ok(Some(hash_value))
}

///
/// Translates the `datacopy` instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
pub fn datacopy<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let pointer = context.access_memory(
        arguments[0].into_int_value(),
        compiler_llvm_context::AddressSpace::Heap,
        "datacopy_pointer",
    );
    context.build_store(pointer, arguments[1]);

    Ok(None)
}
