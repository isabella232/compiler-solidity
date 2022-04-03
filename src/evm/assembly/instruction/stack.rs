//!
//! Translates the stack memory operations.
//!

use inkwell::values::BasicValue;

///
/// Translates the ordinar value push.
///
pub fn push<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    value: String,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let value = context
        .field_type()
        .const_int_from_string(value.as_str(), inkwell::types::StringRadix::Hexadecimal)
        .expect("Always valid")
        .as_basic_value_enum();
    Ok(Some(value))
}

///
/// Translates the block tag label push.
///
pub fn push_tag<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    value: String,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let value: usize = value.parse().expect("Always valid");
    Ok(Some(
        context.field_const(value as u64).as_basic_value_enum(),
    ))
}

///
/// Translates the contract hash size push.
///
pub fn push_contract_hash_size<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    value: String,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let mut identifier = value.trim_start_matches('0');
    if identifier.is_empty() {
        identifier = "0";
    }

    let parent = context.module().get_name().to_str().expect("Always valid");

    if identifier == parent {
        return Ok(Some(context.field_const(0).as_basic_value_enum()));
    }

    Ok(Some(
        context
            .field_const(compiler_common::SIZE_FIELD as u64)
            .as_basic_value_enum(),
    ))
}

///
/// Translates the contract hash push.
///
pub fn push_contract_hash<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    value: String,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let mut identifier = value.trim_start_matches('0');
    if identifier.is_empty() {
        identifier = "0";
    }

    let parent = context.module().get_name().to_str().expect("Always valid");

    if identifier == parent {
        return Ok(Some(context.field_const(0).as_basic_value_enum()));
    }

    let hash_value = context
        .compile_dependency(identifier)
        .map(|hash| context.field_const_str(hash.as_str()))
        .map(inkwell::values::BasicValueEnum::IntValue)?;

    Ok(Some(hash_value))
}

///
/// Translates the stack memory duplicate.
///
pub fn dup<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    offset: usize,
    height: usize,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let pointer = context.evm().stack[height - offset - 1]
        .to_llvm()
        .into_pointer_value();
    let value = context.build_load(pointer, format!("dup{}", offset).as_str());
    Ok(Some(value))
}

///
/// Translates the stack memory swap.
///
pub fn swap<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    offset: usize,
    height: usize,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let top_pointer = context.evm().stack[height - 1]
        .to_llvm()
        .into_pointer_value();
    let top_value = context.build_load(top_pointer, format!("swap{}_top_value", offset).as_str());

    let swap_pointer = context.evm().stack[height - offset - 1]
        .to_llvm()
        .into_pointer_value();
    let swap_value =
        context.build_load(swap_pointer, format!("swap{}_swap_value", offset).as_str());

    context.build_store(top_pointer, swap_value);
    context.build_store(swap_pointer, top_value);

    Ok(None)
}

///
/// Translates the stack memory pop.
///
pub fn pop<'ctx, 'dep, D>(
    _context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    Ok(None)
}
