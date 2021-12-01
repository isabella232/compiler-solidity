//!
//! Translates the contract storage operations.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::argument::Argument;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the contract storage load.
///
pub fn load<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);
    let position = arguments[0];
    let is_external_storage = context.field_const(0);
    let value = context
        .build_call(
            intrinsic,
            &[position, is_external_storage.as_basic_value_enum()],
            "storage_value",
        )
        .expect("Contract storage always returns a value");
    Ok(Some(value))
}

///
/// Translates the contract storage store.
///
pub fn store<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);
    let position = arguments[0];
    let value = arguments[1];
    let is_external_storage = context.field_const(0);
    context.build_call(
        intrinsic,
        &[value, position, is_external_storage.as_basic_value_enum()],
        "storage_store",
    );
    Ok(None)
}

///
/// Translates the contract storage immutable load.
///
pub fn load_immutable<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let literal = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`load_immutable` literal is missing"))?;

    let position = context.field_const_str(compiler_common::keccak256(literal.as_bytes()).as_str());

    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);
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
pub fn set_immutable<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let literal = arguments[1]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`set_immutable` literal is missing"))?;

    let position = context.field_const_str(compiler_common::keccak256(literal.as_bytes()).as_str());

    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);
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
