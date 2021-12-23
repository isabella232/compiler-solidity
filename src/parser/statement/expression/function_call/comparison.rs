//!
//! Translates the comparison operations.
//!

use inkwell::values::BasicValue;

///
/// Translates the comparison operations.
///
pub fn compare<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    arguments: [inkwell::values::BasicValueEnum<'ctx>; 2],
    operation: inkwell::IntPredicate,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let result = context.builder().build_int_compare(
        operation,
        arguments[0].into_int_value(),
        arguments[1].into_int_value(),
        "comparison_result",
    );
    let result = context.builder().build_int_z_extend_or_bit_cast(
        result,
        context.field_type(),
        "comparison_result_extended",
    );
    Ok(Some(result.as_basic_value_enum()))
}
