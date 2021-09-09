//!
//! Translates the contract storage operations.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates the contract storage load.
///
pub fn load<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        let pointer = context.access_storage(arguments[0].into_int_value(), "storage_pointer");
        return Some(context.build_load(pointer, "storage_value"));
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);
    let position = arguments[0];
    let is_external_storage = context.field_const(0).as_basic_value_enum();
    let value = context
        .build_call(intrinsic, &[position, is_external_storage], "storage_value")
        .expect("Contract storage always returns a value");
    Some(value)
}

///
/// Translates the contract storage store.
///
pub fn store<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        let pointer = context.access_storage(arguments[0].into_int_value(), "storage_store");
        context.build_store(pointer, arguments[1]);
        return None;
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);
    let position = arguments[0];
    let value = arguments[1];
    let is_external_storage = context.field_const(0).as_basic_value_enum();
    context.build_call(
        intrinsic,
        &[value, position, is_external_storage],
        "storage_store",
    );
    None
}

///
/// Translates the contract storage immutable load.
///
pub fn load_immutable<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return Some(context.field_const(0).as_basic_value_enum());
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::HashAbsorbReset);
    context.build_call(intrinsic, &[arguments[0]], "load_immutable_hash_absorb");
    let intrinsic = context.get_intrinsic_function(Intrinsic::HashOutput);
    let position = context
        .build_call(intrinsic, &[], "load_immutable_hash_output")
        .expect("Hash always returns a value");

    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);
    let is_external_storage = context.field_const(0).as_basic_value_enum();
    let value = context
        .build_call(
            intrinsic,
            &[position, is_external_storage],
            "load_immutable_storage_load",
        )
        .expect("Contract storage always returns a value");
    Some(value)
}

///
/// Translates the contract storage immutable set.
///
pub fn set_immutable<'ctx>(
    context: &mut LLVMContext<'ctx>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return None;
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::HashAbsorbReset);
    context.build_call(intrinsic, &[arguments[1]], "load_immutable_hash_absorb");
    let intrinsic = context.get_intrinsic_function(Intrinsic::HashOutput);
    let position = context
        .build_call(intrinsic, &[], "load_immutable_hash_output")
        .expect("Hash always returns a value");

    let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);
    let value = arguments[2];
    let is_external_storage = context.field_const(0).as_basic_value_enum();
    context.build_call(
        intrinsic,
        &[value, position, is_external_storage],
        "set_immutable_storage_store",
    );
    None
}
