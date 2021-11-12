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
pub fn call<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    address: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_size: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_size: inkwell::values::IntValue<'ctx>,
    call_type: Intrinsic,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let intrinsic = context.get_intrinsic_function(Intrinsic::SwitchContext);
    context.build_call(intrinsic, &[], "contract_call_switch_context");

    let child_pointer_input = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Child,
        "contract_call_child_pointer_input",
    );
    let input_size_without_selector = context.builder.build_int_sub(
        input_size,
        context.field_const(4),
        "contract_call_input_size_without_selector",
    );
    let input_size_adjusted = context.ceil32(
        input_size_without_selector,
        "contract_call_input_size_adjusted",
    );
    context.build_store(
        child_pointer_input,
        context.builder.build_int_unsigned_div(
            input_size_adjusted,
            context.field_const(compiler_common::size::FIELD as u64),
            "contract_call_input_size_cells",
        ),
    );
    let child_pointer_output = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        compiler_common::AddressSpace::Child,
        "contract_call_child_pointer_output",
    );
    context.build_store(
        child_pointer_output,
        context.builder.build_int_unsigned_div(
            output_size,
            context.field_const(compiler_common::size::FIELD as u64),
            "contract_call_output_size_cells",
        ),
    );

    let destination = context.access_memory(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD - 4)
                as u64,
        ),
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
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
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
