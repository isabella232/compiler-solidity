//!
//! Translates a log or event call.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::target::Target;

///
/// Translates a log or event call.
///
pub fn log<'ctx>(
    context: &mut LLVMContext<'ctx>,
    range_start: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
    mut topics: Vec<inkwell::values::IntValue<'ctx>>,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    if let Target::x86 = context.target {
        return None;
    }

    let intrinsic = context.get_intrinsic_function(Intrinsic::Event);

    if topics.is_empty() {
        topics = vec![context.field_const(0)];
    }

    for topic in topics.into_iter() {
        let pointer = context.access_heap(range_start, None);
        let value = context.build_load(pointer, "");
        context.build_call(
            intrinsic,
            &[
                value,
                topic.as_basic_value_enum(),
                context.field_const(1).as_basic_value_enum(),
            ],
            "",
        );

        let condition_block = context.append_basic_block("condition");
        let body_block = context.append_basic_block("body");
        let increment_block = context.append_basic_block("increment");
        let join_block = context.append_basic_block("join");

        let index_pointer = context.build_alloca(context.field_type(), "");
        let range_start = context.builder.build_int_add(
            range_start,
            context.field_const(compiler_common::size::FIELD as u64),
            "",
        );
        let length = context.builder.build_int_sub(
            length,
            context.field_const(compiler_common::size::FIELD as u64),
            "",
        );
        let range_end = context.builder.build_int_add(range_start, length, "");
        context.build_store(index_pointer, range_start);
        context.build_unconditional_branch(condition_block);

        context.set_basic_block(condition_block);
        let index_value = context.build_load(index_pointer, "").into_int_value();
        let condition = context.builder.build_int_compare(
            inkwell::IntPredicate::ULT,
            index_value,
            range_end,
            "",
        );
        context.build_conditional_branch(condition, body_block, join_block);

        context.set_basic_block(increment_block);
        let index_value = context.build_load(index_pointer, "").into_int_value();
        let incremented = context.builder.build_int_add(
            index_value,
            context.field_const(compiler_common::size::FIELD as u64),
            "",
        );
        context.build_store(index_pointer, incremented);
        context.build_unconditional_branch(condition_block);

        context.set_basic_block(body_block);
        let index_value = context.build_load(index_pointer, "").into_int_value();
        let pointer = context.access_heap(index_value, None);
        let value = context.build_load(pointer, "");
        context.build_call(
            intrinsic,
            &[
                value,
                topic.as_basic_value_enum(),
                context.field_const(0).as_basic_value_enum(),
            ],
            "",
        );
        context.build_unconditional_branch(increment_block);

        context.set_basic_block(join_block);
    }

    None
}
