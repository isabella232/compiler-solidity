//!
//! Translates the contract storage operations.
//!

use inkwell::values::BasicValue;

///
/// Translates the contract storage immutable push.
///
pub fn push_immutable<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    value: String,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let key_hash = compiler_common::keccak256(value.as_bytes());
    let key = context.field_const_str_hex(key_hash.as_str());

    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::StorageLoad);
    let is_external_storage = context.field_const(0);
    let value = context
        .build_call(
            intrinsic,
            &[
                key.as_basic_value_enum(),
                is_external_storage.as_basic_value_enum(),
            ],
            "push_immutable_storage_load",
        )
        .expect("Contract storage always returns a value");
    Ok(Some(value))
}

///
/// Translates the contract storage immutable assign.
///
pub fn assign_immutable<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
    value: String,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let key_hash = compiler_common::keccak256(value.as_bytes());
    let key = context.field_const_str_hex(key_hash.as_str());

    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::StorageStore);
    let value = arguments[0];
    let is_external_storage = context.field_const(0);
    context.build_call(
        intrinsic,
        &[
            value,
            key.as_basic_value_enum(),
            is_external_storage.as_basic_value_enum(),
        ],
        "assign_immutable_storage_store",
    );
    Ok(None)
}
