//!
//! Translates a contract call.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::address_space::AddressSpace;
use crate::generator::llvm::argument::Argument;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates a contract call.
///
#[allow(clippy::too_many_arguments)]
pub fn call<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    call_type: Intrinsic,
    address: inkwell::values::IntValue<'ctx>,
    value: Option<inkwell::values::IntValue<'ctx>>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_size: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    if let Some(value) = value {
        check_value_zero(context, value);
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::SwitchContext);
    context.build_call(intrinsic, &[], "contract_call_switch_context");

    let child_pointer_header = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_HEADER * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "contract_call_child_pointer_header",
    );
    context.build_store(child_pointer_header, input_size);

    let destination = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "contract_call_child_input_destination",
    );
    let source = context.access_memory(
        input_offset,
        AddressSpace::Heap,
        "contract_call_child_input_source",
    );

    context.build_memcpy(
        Intrinsic::MemoryCopyToChild,
        destination,
        source,
        input_size,
        "contract_call_memcpy_to_child",
    );

    let address = map_precompile_address(context, address);

    let intrinsic = context.get_intrinsic_function(call_type);
    let call_definition = context.builder.build_left_shift(
        address,
        context.field_const((compiler_common::BITLENGTH_X32) as u64),
        "",
    );
    let is_call_successful = context
        .build_call(
            intrinsic,
            &[call_definition.as_basic_value_enum()],
            "contract_call_external",
        )
        .expect("Intrinsic always returns a flag");

    let source = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "contract_call_output_source",
    );
    let destination = context.access_memory(
        output_offset,
        AddressSpace::Heap,
        "contract_call_output_pointer",
    );

    context.build_memcpy(
        Intrinsic::MemoryCopyFromChild,
        destination,
        source,
        output_size,
        "contract_call_memcpy_from_child",
    );

    Ok(Some(is_call_successful))
}

///
/// Translates a linker symbol.
///
pub fn linker_symbol<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let path = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("Linker symbol literal is missing"))?;

    match context.get_library_address(path.as_str()) {
        Some(address) => Ok(Some(address.as_basic_value_enum())),
        None => anyhow::bail!("Linker symbol `{}` not found", path),
    }
}

///
/// Remaps the precompiled contract addresses. Currently supported:
/// - keccak256 (eponymous instruction => 0x10)
/// - ecrecover (0x01 => 0x12)
/// - sha256 (0x02 => 0x11)
///
fn map_precompile_address<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    address: inkwell::values::IntValue<'ctx>,
) -> inkwell::values::IntValue<'ctx> {
    let block_ecrecover = context.append_basic_block("contract_call_address_map_ecrecover");
    let block_sha256 = context.append_basic_block("contract_call_address_map_sha256");
    let block_default = context.append_basic_block("contract_call_address_map_default");
    let block_join = context.append_basic_block("contract_call_address_map_join");

    let result_pointer = context.build_alloca(
        context.field_type(),
        "contract_call_address_map_result_pointer",
    );
    context.builder.build_switch(
        address,
        block_default,
        &[
            (
                context.field_const_str(compiler_common::SOLIDITY_ADDRESS_ECRECOVER),
                block_ecrecover,
            ),
            (
                context.field_const_str(compiler_common::SOLIDITY_ADDRESS_SHA256),
                block_sha256,
            ),
        ],
    );

    context.set_basic_block(block_ecrecover);
    context.build_store(
        result_pointer,
        context.field_const_str(compiler_common::ABI_ADDRESS_ECRECOVER),
    );
    context.build_unconditional_branch(block_join);

    context.set_basic_block(block_sha256);
    context.build_store(
        result_pointer,
        context.field_const_str(compiler_common::ABI_ADDRESS_SHA256),
    );
    context.build_unconditional_branch(block_join);

    context.set_basic_block(block_default);
    context.build_store(result_pointer, address);
    context.build_unconditional_branch(block_join);

    context.set_basic_block(block_join);
    context
        .build_load(result_pointer, "contract_call_address_map_result")
        .into_int_value()
}

///
/// Throws an exception if the call is a send/transfer.
///
/// Sends and transfers have their `value` non-zero.
///
fn check_value_zero<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    value: inkwell::values::IntValue<'ctx>,
) {
    let value_zero_block = context.append_basic_block("contract_call_value_zero_block");
    let value_non_zero_block = context.append_basic_block("contract_call_value_non_zero_block");

    let is_value_zero = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        value,
        context.field_const(0),
        "contract_call_is_value_zero",
    );

    context.build_conditional_branch(is_value_zero, value_zero_block, value_non_zero_block);

    context.set_basic_block(value_non_zero_block);
    context.write_error(compiler_common::ABI_ERROR_FORBIDDEN_SEND_TRANSFER);
    context.build_unconditional_branch(context.function().throw_block);

    context.set_basic_block(value_zero_block);
}
