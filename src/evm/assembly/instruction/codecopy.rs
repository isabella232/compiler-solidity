//!
//! Translates the CODECOPY use cases.
//!

use inkwell::values::BasicValue;

use crate::evm::ethereal_ir::EtherealIR;

///
/// Translates the contract hash copying.
///
pub fn contract_hash<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let offset = context
        .builder()
        .build_int_add(
            arguments[0].into_int_value(),
            context.field_const((compiler_common::SIZE_X32 + compiler_common::SIZE_FIELD) as u64),
            "datacopy_contract_hash_offset",
        )
        .as_basic_value_enum();
    let value = arguments[1];

    compiler_llvm_context::memory::store(context, [offset, value])?;

    Ok(None)
}

///
/// Translates the library marker copying.
///
pub fn library_marker<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    offset: &str,
    value: &str,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    if let Some(compiler_llvm_context::CodeType::Deploy) = context.code_type {
        let address = context
            .build_call(
                context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::Address),
                &[],
                "address",
            )
            .expect("Always exists");
        compiler_llvm_context::immutable::store(
            context,
            EtherealIR::DEPLOY_ADDRESS_STORAGE_KEY.to_owned(),
            address.into_int_value(),
        )?;
    }

    compiler_llvm_context::memory::store_byte(
        context,
        [
            context.field_const_str_hex(offset).as_basic_value_enum(),
            context.field_const_str_hex(value).as_basic_value_enum(),
        ],
    )?;

    Ok(None)
}

///
/// Translates the static data copying.
///
pub fn static_data<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
    source: &str,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let mut offset = 0;
    for (index, chunk) in source
        .chars()
        .collect::<Vec<char>>()
        .chunks(compiler_common::SIZE_FIELD * 2)
        .enumerate()
    {
        let mut value_string = chunk.iter().collect::<String>();
        value_string.push_str(
            "0".repeat((compiler_common::SIZE_FIELD * 2) - chunk.len())
                .as_str(),
        );

        let datacopy_destination = context.builder().build_int_add(
            arguments[0].into_int_value(),
            context.field_const(offset as u64),
            format!("datacopy_destination_index_{}", index).as_str(),
        );
        let datacopy_value = context.field_const_str(value_string.as_str());
        compiler_llvm_context::memory::store(
            context,
            [
                datacopy_destination.as_basic_value_enum(),
                datacopy_value.as_basic_value_enum(),
            ],
        )?;
        offset += chunk.len() / 2;
    }

    Ok(None)
}
