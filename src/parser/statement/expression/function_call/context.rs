//!
//! Translates the contract context getter calls.
//!

use inkwell::values::BasicValue;

///
/// Translates the contract context getter calls.
///
pub fn get<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    context_value: compiler_common::ContextValue,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let intrinsic =
        context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::GetFromContext);
    let value = context
        .build_call(
            intrinsic,
            &[context
                .field_const(context_value.into())
                .as_basic_value_enum()],
            "context_get_call",
        )
        .expect("Contract context always returns a value");
    Ok(Some(value))
}
