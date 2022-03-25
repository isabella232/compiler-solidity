//!
//! Translates the stack memory operations.
//!

use inkwell::values::BasicValue;

///
/// Translates the stack memory push.
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
/// Translates the tag constant push.
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
/// Translates the stack memory duplicate.
///
pub fn dup<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    index: usize,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let pointer = context.evm().stack_pointer(index);
    let value = context.build_load(pointer, format!("dup{}", index).as_str());
    Ok(Some(value))
}

///
/// Translates the stack memory swap.
///
pub fn swap<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    index: usize,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let top_pointer = context.evm().stack_pointer(1);
    let top_value = context.build_load(top_pointer, format!("swap{}_top_value", index).as_str());

    let swap_pointer = context.evm().stack_pointer(index + 1);
    let swap_value = context.build_load(swap_pointer, format!("swap{}_swap_value", index).as_str());

    context.build_store(top_pointer, swap_value);
    context.build_store(swap_pointer, top_value);

    Ok(None)
}

///
/// Translates the stack memory pop.
///
pub fn pop<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    context.evm_mut().decrease_stack_pointer(1);

    Ok(None)
}
