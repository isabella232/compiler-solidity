//!
//! Translates a contract call.
//!

use inkwell::values::BasicValue;

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
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Some(value) = value {
        check_value_zero(context, value);
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::SwitchContext);
    context.build_call(intrinsic, &[], "contract_call_switch_context");

    let child_pointer_header = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_HEADER * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Child,
        "contract_call_child_pointer_header",
    );
    context.build_store(child_pointer_header, input_size);

    let destination = context.access_memory(
        context
            .field_const((compiler_common::abi::OFFSET_DATA * compiler_common::size::FIELD) as u64),
        compiler_common::AddressSpace::Child,
        "contract_call_child_input_destination",
    );
    let source = context.access_memory(
        input_offset,
        compiler_common::AddressSpace::Heap,
        "contract_call_child_input_source",
    );

    context.build_memcpy(
        Intrinsic::MemoryCopyToChild,
        destination,
        source,
        input_size,
        "contract_call_memcpy_to_child",
    );

    let intrinsic = context.get_intrinsic_function(call_type);
    let call_definition = context.builder.build_left_shift(
        address,
        context.field_const((compiler_common::bitlength::BYTE * 4) as u64),
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
        context
            .field_const((compiler_common::abi::OFFSET_DATA * compiler_common::size::FIELD) as u64),
        compiler_common::AddressSpace::Child,
        "contract_call_output_source",
    );
    let destination = context.access_memory(
        output_offset,
        compiler_common::AddressSpace::Heap,
        "contract_call_output_pointer",
    );

    context.build_memcpy(
        Intrinsic::MemoryCopyFromChild,
        destination,
        source,
        output_size,
        "contract_call_memcpy_from_child",
    );

    Some(is_call_successful)
}

///
/// Translates a linker symbol.
///
pub fn linker_symbol<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let path = arguments[0].original.take().expect("Always exists");

    match context.get_library_address(path.as_str()) {
        Some(address) => Some(address.as_basic_value_enum()),
        None => panic!("Linker symbol `{}` not found", path),
    }
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
    context.write_error(compiler_common::abi::ERROR_FORBIDDEN_SEND_TRANSFER);
    context.build_unconditional_branch(context.function().throw_block);

    context.set_basic_block(value_zero_block);
}
