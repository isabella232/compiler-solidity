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

    let child_pointer_input = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
        "contract_call_child_pointer_input",
    );
    context.build_store(
        child_pointer_input,
        context.builder.build_int_unsigned_div(
            input_size,
            context.field_const(compiler_common::size::FIELD as u64),
            "contract_call_input_size_cells",
        ),
    );
    let child_pointer_output = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_RETURN_DATA_SIZE * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
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

    let entry_data_pointer = context.access_heap(input_offset, "contract_call_entry_data_pointer");
    let entry_data = context.build_load(entry_data_pointer, "contract_call_entry_data");
    let child_pointer_entry_data = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_ENTRY_DATA * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
        "contract_call_child_pointer_entry_data",
    );
    context.build_store(child_pointer_entry_data, entry_data.as_basic_value_enum());

    let destination = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
        "contract_call_child_input_destination",
    );
    let input_offset_adjusted = context.builder.build_int_add(
        input_offset,
        context.field_const((compiler_common::size::FIELD) as u64),
        "contract_call_input_offset_adjusted",
    );
    let source = context.access_heap(input_offset_adjusted, "contract_call_child_input_source");
    let input_size_adjusted = context.builder.build_int_sub(
        input_size,
        context.field_const(4),
        "contract_call_input_size_adjusted",
    );

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyToChild);
    context.build_call(
        intrinsic,
        &[
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            input_size_adjusted.as_basic_value_enum(),
            context
                .integer_type(compiler_common::bitlength::BOOLEAN)
                .const_zero()
                .as_basic_value_enum(),
        ],
        "contract_call_memcpy_to_child",
    );

    let intrinsic = context.get_intrinsic_function(call_type);
    let call_definition = context.builder.build_left_shift(
        address,
        context.field_const((compiler_common::bitlength::BYTE * 4) as u64),
        "",
    );
    context.build_call(
        intrinsic,
        &[call_definition.as_basic_value_enum()],
        "contract_call_external",
    );

    let intrinsic = context.get_intrinsic_function(Intrinsic::LesserFlag);
    let overflow_flag = context
        .build_call(intrinsic, &[], "")
        .expect("Intrinsic always returns a flag")
        .into_int_value();
    let is_overflow_flag_zero = context.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        overflow_flag,
        context.field_const(0),
        "contract_call_is_overflow_flag_zero",
    );
    let is_call_successful = context.builder.build_int_z_extend_or_bit_cast(
        is_overflow_flag_zero,
        context.field_type(),
        "contract_call_is_successfull",
    );

    let source = context.builder.build_int_to_ptr(
        context.field_const(
            (compiler_common::abi::OFFSET_CALL_RETURN_DATA * compiler_common::size::FIELD) as u64,
        ),
        context
            .field_type()
            .ptr_type(compiler_common::AddressSpace::Child.into()),
        "contract_call_output_source",
    );
    let destination = context.access_heap(output_offset, "contract_call_output_pointer");

    let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromChild);
    context.build_call(
        intrinsic,
        &[
            destination.as_basic_value_enum(),
            source.as_basic_value_enum(),
            output_size.as_basic_value_enum(),
            context
                .integer_type(compiler_common::bitlength::BOOLEAN)
                .const_zero()
                .as_basic_value_enum(),
        ],
        "contract_call_memcpy_from_child",
    );

    Some(is_call_successful.as_basic_value_enum())
}

///
/// Translates a linker symbol.
///
pub fn linker_symbol<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 1],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    let path = arguments[0].original.take().unwrap_or_default();

    match context.get_library_address(path.as_str()) {
        Some(address) => Some(address.as_basic_value_enum()),
        None => panic!("Linker symbol `{}` not found", path),
    }
}
