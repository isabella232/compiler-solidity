//!
//! Translates the contract storage operations.
//!

use inkwell::values::BasicValue;

///
/// Translates the contract storage immutable load.
///
pub fn load_immutable<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    mut arguments: [compiler_llvm_context::Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let literal = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`load_immutable` literal is missing"))?;

    let position = context.field_const_str(compiler_common::keccak256(literal.as_bytes()).as_str());

    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::StorageLoad);
    let is_external_storage = context.field_const(0);
    let value = context
        .build_call(
            intrinsic,
            &[
                position.as_basic_value_enum(),
                is_external_storage.as_basic_value_enum(),
            ],
            "load_immutable_storage_load",
        )
        .expect("Contract storage always returns a value");
    Ok(Some(value))
}

///
/// Translates the contract storage immutable set.
///
pub fn set_immutable<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    mut arguments: [compiler_llvm_context::Argument<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let literal = arguments[1]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`set_immutable` literal is missing"))?;

    let position = context.field_const_str(compiler_common::keccak256(literal.as_bytes()).as_str());

    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::StorageStore);
    let value = arguments[2].value;
    let is_external_storage = context.field_const(0);
    context.build_call(
        intrinsic,
        &[
            value,
            position.as_basic_value_enum(),
            is_external_storage.as_basic_value_enum(),
        ],
        "set_immutable_storage_store",
    );
    Ok(None)
}
