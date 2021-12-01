//!
//! Translates the contract creation instructions.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::address_space::AddressSpace;
use crate::generator::llvm::argument::Argument;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the contract `create` instruction.
///
pub fn create<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    create2(
        context,
        [
            arguments[0],
            arguments[1],
            arguments[2],
            context.field_const(0).as_basic_value_enum(),
        ],
    )
}

///
/// Translates the contract `create2` instruction.
///
pub fn create2<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 4],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let input_offset = context.builder.build_int_add(
        arguments[1].into_int_value(),
        context.field_const(compiler_common::SIZE_FIELD as u64),
        "create_input_offset",
    );
    let input_size = context.builder.build_int_sub(
        arguments[2].into_int_value(),
        context.field_const(compiler_common::SIZE_FIELD as u64),
        "create_input_size",
    );

    let intrinsic = context.get_intrinsic_function(Intrinsic::SwitchContext);
    context.build_call(intrinsic, &[], "create_switch_context");

    let child_header_data = context.builder.build_or(
        input_size,
        context.field_const_str("00000000000000010000000000000000"),
        "child_header_data",
    );

    let child_pointer_header = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_HEADER * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "create_child_pointer_header",
    );
    context.build_store(child_pointer_header, child_header_data);

    let destination = context.access_memory(
        context.field_const(
            (compiler_common::ABI_MEMORY_OFFSET_DATA * compiler_common::SIZE_FIELD) as u64,
        ),
        AddressSpace::Child,
        "create_child_input_destination",
    );
    let source = context.access_memory(
        input_offset,
        AddressSpace::Heap,
        "create_child_input_source",
    );

    context.build_memcpy(
        Intrinsic::MemoryCopyToChild,
        destination,
        source,
        input_size,
        "create_memcpy_to_child",
    );

    let intrinsic = context.get_intrinsic_function(Intrinsic::FarCall);
    let address = context.field_const_str("1234567812345678123456781234567812345678"); // TODO: get from the special event call
    let call_definition = context.builder.build_left_shift(
        address,
        context.field_const((compiler_common::BITLENGTH_X32) as u64),
        "",
    );
    let is_call_successful = context
        .build_call(
            intrinsic,
            &[call_definition.as_basic_value_enum()],
            "create_call",
        )
        .expect("Intrinsic always returns a flag");

    Ok(Some(is_call_successful))
}

///
/// Translates the `datasize` instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
pub fn datasize<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let literal = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`datasize` object identifier is missing"))?;

    if literal.ends_with("_deployed") || literal.as_str() == context.object() {
        return Ok(Some(context.field_const(0).as_basic_value_enum()));
    }

    Ok(Some(
        context
            .field_const(compiler_common::SIZE_FIELD as u64)
            .as_basic_value_enum(),
    ))
}

///
/// Translates the `dataoffset` instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
pub fn dataoffset<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    mut arguments: [Argument<'ctx>; 1],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let literal = arguments[0]
        .original
        .take()
        .ok_or_else(|| anyhow::anyhow!("`dataoffset` object identifier is missing"))?;

    if literal.ends_with("_deployed") {
        return Ok(Some(context.field_const(0).as_basic_value_enum()));
    }

    let hash_string = context.compile_dependency(literal.as_str());
    let hash_value = context.field_const_str(hash_string.as_str());

    Ok(Some(hash_value.as_basic_value_enum()))
}

///
/// Translates the `datacopy` instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
pub fn datacopy<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 3],
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let pointer = context.access_memory(
        arguments[0].into_int_value(),
        AddressSpace::Heap,
        "datacopy_pointer",
    );
    context.build_store(pointer, arguments[1]);

    Ok(None)
}
