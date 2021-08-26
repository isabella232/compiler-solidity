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

    let topics_length = context.field_const(topics.len() as u64);
    let data_length_shifted = context.builder.build_left_shift(
        length,
        context.field_const((compiler_common::bitlength::BYTE * 4) as u64),
        "event_data_length_shifted",
    );
    let event_initializer =
        context
            .builder
            .build_int_add(topics_length, data_length_shifted, "event_initializer");
    let is_topics_length_odd = topics.len() % 2 == 0;

    let range_start_pointer =
        context.build_alloca(context.field_type(), "event_range_start_pointer");
    let length_pointer = context.build_alloca(context.field_type(), "event_length_pointer");
    if is_topics_length_odd {
        let data_not_empty_block = context.append_basic_block("event_data_not_empty");
        let data_empty_block = context.append_basic_block("event_data_empty");
        let join_block = context.append_basic_block("event_data_join");

        let data_not_empty_condition = context.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            length,
            context.field_const(0),
            "event_data_empty_condition",
        );
        context.build_conditional_branch(
            data_not_empty_condition,
            data_not_empty_block,
            data_empty_block,
        );

        context.set_basic_block(data_not_empty_block);
        let pointer = context.access_heap(range_start, None);
        let value = context.build_load(pointer, "event_first_value");
        if topics.is_empty() {
            context.build_call(
                intrinsic,
                &[
                    event_initializer.as_basic_value_enum(),
                    value,
                    context.field_const(1).as_basic_value_enum(),
                ],
                "event_call_init_with_value",
            );
        } else {
            context.build_call(
                intrinsic,
                &[
                    event_initializer.as_basic_value_enum(),
                    topics.remove(0).as_basic_value_enum(),
                    context.field_const(1).as_basic_value_enum(),
                ],
                "event_call_init_with_topic",
            );
            while topics.len() >= 2 {
                context.build_call(
                    intrinsic,
                    &[
                        topics.remove(0).as_basic_value_enum(),
                        topics.remove(0).as_basic_value_enum(),
                        context.field_const(0).as_basic_value_enum(),
                    ],
                    "event_call_with_two_topics",
                );
            }
            context.build_call(
                intrinsic,
                &[
                    topics.remove(0).as_basic_value_enum(),
                    value,
                    context.field_const(0).as_basic_value_enum(),
                ],
                "event_call_init_with_topic_and_value",
            );
        }

        context.build_store(
            range_start_pointer,
            context.builder.build_int_add(
                range_start,
                context.field_const(compiler_common::size::FIELD as u64),
                "event_range_start_after_first",
            ),
        );
        context.build_store(
            length_pointer,
            context.builder.build_int_sub(
                length,
                context.field_const(compiler_common::size::FIELD as u64),
                "event_length_without_first",
            ),
        );
        context.build_unconditional_branch(join_block);

        context.set_basic_block(data_empty_block);
        context.build_store(range_start_pointer, range_start);
        context.build_store(length_pointer, length);
        context.build_unconditional_branch(join_block);

        context.set_basic_block(join_block);
    } else {
        context.build_call(
            intrinsic,
            &[
                event_initializer.as_basic_value_enum(),
                topics.remove(0).as_basic_value_enum(),
                context.field_const(1).as_basic_value_enum(),
            ],
            "event_call_init_with_topic",
        );
        while topics.len() >= 2 {
            context.build_call(
                intrinsic,
                &[
                    topics.remove(0).as_basic_value_enum(),
                    topics.remove(0).as_basic_value_enum(),
                    context.field_const(0).as_basic_value_enum(),
                ],
                "event_call_with_two_topics",
            );
        }
    }

    let range_start = context
        .build_load(range_start_pointer, "event_range_start_joined")
        .into_int_value();
    let length = context
        .build_load(length_pointer, "event_length_joined")
        .into_int_value();

    let condition_block = context.append_basic_block("event_loop_condition");
    let body_block = context.append_basic_block("event_loop_body");
    let increment_block = context.append_basic_block("event_loop_increment");
    let join_block = context.append_basic_block("event_loop_join");

    let index_pointer = context.build_alloca(context.field_type(), "event_loop_index_pointer");
    let range_end = context
        .builder
        .build_int_add(range_start, length, "event_loop_range_end");
    context.build_store(index_pointer, range_start);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(condition_block);
    let index_value = context.build_load(index_pointer, "").into_int_value();
    let condition = context.builder.build_int_compare(
        inkwell::IntPredicate::ULT,
        index_value,
        range_end,
        "event_loop_condition",
    );
    context.build_conditional_branch(condition, body_block, join_block);

    context.set_basic_block(increment_block);
    let index_value = context
        .build_load(index_pointer, "event_loop_index_value_increment")
        .into_int_value();
    let incremented = context.builder.build_int_add(
        index_value,
        context.field_const((compiler_common::size::FIELD * 2) as u64),
        "event_loop_index_value_incremented",
    );
    context.build_store(index_pointer, incremented);
    context.build_unconditional_branch(condition_block);

    context.set_basic_block(body_block);
    let two_values_block = context.append_basic_block("event_loop_body_two_values");
    let one_value_block = context.append_basic_block("event_loop_body_one_value");
    let index_value = context
        .build_load(index_pointer, "event_loop_body_index_value")
        .into_int_value();
    let values_remaining =
        context
            .builder
            .build_int_sub(range_end, index_value, "event_loop_values_remaining");
    let has_two_values = context.builder.build_int_compare(
        inkwell::IntPredicate::UGE,
        values_remaining,
        context.field_const((compiler_common::size::FIELD * 2) as u64),
        "event_loop_has_two_values",
    );
    context.build_conditional_branch(has_two_values, two_values_block, one_value_block);

    context.set_basic_block(two_values_block);
    let value_1_pointer = context.access_heap(index_value, None);
    let value_1 = context.build_load(value_1_pointer, "event_loop_value_1");
    let index_value_next = context.builder.build_int_add(
        index_value,
        context.field_const(compiler_common::size::FIELD as u64),
        "event_loop_index_value_next",
    );
    let value_2_pointer = context.access_heap(index_value_next, None);
    let value_2 = context.build_load(value_2_pointer, "event_loop_value_2");
    context.build_call(
        intrinsic,
        &[
            value_1,
            value_2,
            context.field_const(0).as_basic_value_enum(),
        ],
        "event_loop_call_with_two_values",
    );
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(one_value_block);
    let value_1_pointer = context.access_heap(index_value, None);
    let value_1 = context.build_load(value_1_pointer, "event_loop_value_1");
    context.build_call(
        intrinsic,
        &[
            value_1,
            context.field_const(0).as_basic_value_enum(),
            context.field_const(0).as_basic_value_enum(),
        ],
        "event_loop_call_with_value_and_zero",
    );
    context.build_unconditional_branch(increment_block);

    context.set_basic_block(join_block);

    None
}
