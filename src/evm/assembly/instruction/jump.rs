//!
//! Translates the jump operations.
//!

///
/// Translates the unconditional jump.
///
pub fn unconditional<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    destination: num::BigUint,
    stack_hash: md5::Digest,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let code_type = context.code_type();
    let block_key = compiler_llvm_context::FunctionBlockKey::new(code_type, destination);

    let block = context
        .function()
        .evm()
        .find_block(&block_key, &stack_hash)?;
    context.build_unconditional_branch(block.inner);

    Ok(None)
}

///
/// Translates the conditional jump.
///
pub fn conditional<'ctx, 'dep, D>(
    context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    destination: num::BigUint,
    stack_hash: md5::Digest,
    stack_height: usize,
) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
where
    D: compiler_llvm_context::Dependency,
{
    let code_type = context.code_type();
    let block_key = compiler_llvm_context::FunctionBlockKey::new(code_type, destination);

    let condition_pointer = context.evm().stack[stack_height]
        .to_llvm()
        .into_pointer_value();
    let condition = context.build_load(
        condition_pointer,
        format!("conditional_{}_condition", block_key).as_str(),
    );
    let condition = context.builder().build_int_compare(
        inkwell::IntPredicate::NE,
        condition.into_int_value(),
        context.field_const(0),
        format!("conditional_{}_condition_compared", block_key).as_str(),
    );

    let then_block = context
        .function()
        .evm()
        .find_block(&block_key, &stack_hash)?;
    let join_block =
        context.append_basic_block(format!("conditional_{}_join_block", block_key).as_str());

    context.build_conditional_branch(condition, then_block.inner, join_block);

    context.set_basic_block(join_block);

    Ok(None)
}
