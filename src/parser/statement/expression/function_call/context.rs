//!
//! Translates the contract context getter calls.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;

///
/// Translates the contract context getter calls.
///
pub fn get<'ctx, 'src>(
    context: &mut LLVMContext<'ctx, 'src>,
    context_value: compiler_common::ContextValue,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
    let intrinsic = context.get_intrinsic_function(Intrinsic::GetFromContext);
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
