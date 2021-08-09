//!
//! The function call subexpression.
//!

pub mod name;

use std::convert::TryInto;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::function::r#return::Return as FunctionReturn;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::expression::Expression;
use crate::target::Target;

use self::name::Name;

///
/// The function call subexpression.
///
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    /// The function name.
    pub name: Name,
    /// The function arguments expression list.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let name = match lexeme {
            Lexeme::Identifier(identifier) => Name::from(identifier.as_str()),
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["{identifier}"], lexeme, None).into())
            }
        };

        let mut arguments = Vec::new();
        loop {
            let argument = match lexer.next()? {
                Lexeme::Symbol(Symbol::ParenthesisRight) => break,
                lexeme => Expression::parse(lexer, Some(lexeme))?,
            };

            arguments.push(argument);

            match lexer.peek()? {
                Lexeme::Symbol(Symbol::Comma) => {
                    lexer.next()?;
                    continue;
                }
                Lexeme::Symbol(Symbol::ParenthesisRight) => {
                    lexer.next()?;
                    break;
                }
                _ => break,
            }
        }

        Ok(Self { name, arguments })
    }

    ///
    /// Converts the function call into an LLVM value.
    ///
    pub fn into_llvm<'ctx>(
        mut self,
        context: &mut LLVMContext<'ctx>,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        match self.name {
            Name::UserDefined(name) => {
                let mut arguments: Vec<inkwell::values::BasicValueEnum> = self
                    .arguments
                    .into_iter()
                    .filter_map(|argument| argument.into_llvm(context))
                    .collect();
                let function = context
                    .functions
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| panic!("Undeclared function {}", name));

                if let Some(FunctionReturn::Compound { size, .. }) = function.r#return {
                    let r#type = context.structure_type(vec![
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .as_basic_type_enum();
                        size
                    ]);
                    let pointer = context.build_alloca(r#type, "");
                    context.build_store(pointer, r#type.const_zero());
                    arguments.insert(0, pointer.as_basic_value_enum());
                }

                let return_value = context.build_invoke(function.value, arguments.as_slice(), "");

                if let Some(FunctionReturn::Compound { .. }) = function.r#return {
                    let return_pointer = return_value.expect("Always exists").into_pointer_value();
                    let return_value = context.build_load(return_pointer, "");
                    Some(return_value)
                } else {
                    return_value
                }
            }

            Name::Add => {
                let arguments = self.pop_arguments::<2>(context);
                let result = context
                    .builder
                    .build_int_add(
                        arguments[0].into_int_value(),
                        arguments[1].into_int_value(),
                        "",
                    )
                    .as_basic_value_enum();
                Some(result)
            }
            Name::Sub => {
                let arguments = self.pop_arguments::<2>(context);
                let result = context
                    .builder
                    .build_int_sub(
                        arguments[0].into_int_value(),
                        arguments[1].into_int_value(),
                        "",
                    )
                    .as_basic_value_enum();
                Some(result)
            }
            Name::Mul => {
                let arguments = self.pop_arguments::<2>(context);
                let result = context
                    .builder
                    .build_int_mul(
                        arguments[0].into_int_value(),
                        arguments[1].into_int_value(),
                        "",
                    )
                    .as_basic_value_enum();
                Some(result)
            }
            Name::Div => {
                let mut arguments = self.pop_arguments::<2>(context);

                let zero_block = context.append_basic_block("zero");
                let non_zero_block = context.append_basic_block("non_zero");
                let join_block = context.append_basic_block("join");

                let result_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[1].into_int_value(),
                    context.field_const(0),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                let initial_type = arguments[0].get_type().into_int_type();
                if let Target::X86 = context.target {
                    let allowed_type = context.integer_type(compiler_common::bitlength::BYTE * 16);
                    arguments[0] = context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[0].into_int_value(),
                            allowed_type,
                            "",
                        )
                        .as_basic_value_enum();
                    arguments[1] = context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[1].into_int_value(),
                            allowed_type,
                            "",
                        )
                        .as_basic_value_enum();
                }
                let mut result = context.builder.build_int_unsigned_div(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                if let Target::X86 = context.target {
                    result =
                        context
                            .builder
                            .build_int_z_extend_or_bit_cast(result, initial_type, "");
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(result_pointer, context.field_const(0));
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Mod => {
                let mut arguments = self.pop_arguments::<2>(context);

                let zero_block = context.append_basic_block("zero");
                let non_zero_block = context.append_basic_block("non_zero");
                let join_block = context.append_basic_block("join");

                let result_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[1].into_int_value(),
                    context.field_const(0),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                let initial_type = arguments[0].get_type().into_int_type();
                if let Target::X86 = context.target {
                    let allowed_type = context.integer_type(compiler_common::bitlength::BYTE * 16);
                    arguments[0] = context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[0].into_int_value(),
                            allowed_type,
                            "",
                        )
                        .as_basic_value_enum();
                    arguments[1] = context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[1].into_int_value(),
                            allowed_type,
                            "",
                        )
                        .as_basic_value_enum();
                }
                let mut result = context.builder.build_int_unsigned_rem(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                if let Target::X86 = context.target {
                    result =
                        context
                            .builder
                            .build_int_z_extend_or_bit_cast(result, initial_type, "");
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(result_pointer, context.field_const(0));
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }

            Name::Lt => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_z_extend_or_bit_cast(
                    result,
                    context.integer_type(compiler_common::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Gt => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_int_compare(
                    inkwell::IntPredicate::UGT,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_z_extend_or_bit_cast(
                    result,
                    context.integer_type(compiler_common::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Eq => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_z_extend_or_bit_cast(
                    result,
                    context.integer_type(compiler_common::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::IsZero => {
                let arguments = self.pop_arguments::<1>(context);
                let mut result = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[0].into_int_value(),
                    context.field_const(0),
                    "",
                );
                result = context.builder.build_int_z_extend_or_bit_cast(
                    result,
                    context.integer_type(compiler_common::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }

            Name::And => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::X86)
                    || (arguments[0].into_int_value().is_const()
                        && arguments[1].into_int_value().is_const())
                {
                    return Some(
                        context
                            .builder
                            .build_and(
                                arguments[0].into_int_value(),
                                arguments[1].into_int_value(),
                                "",
                            )
                            .as_basic_value_enum(),
                    );
                }

                let llvm_type = context.integer_type(compiler_common::bitlength::FIELD);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let result_pointer = context.build_alloca(llvm_type, "");
                context.build_store(result_pointer, llvm_type.const_zero());
                let operand_1_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_1_pointer, arguments[0]);
                let operand_2_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_2_pointer, arguments[1]);
                let index_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                context.build_store(index_pointer, context.field_const(0));
                let shift_pointer = context.build_alloca(llvm_type, "");
                context.build_store(shift_pointer, llvm_type.const_int(1, false));
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    context.field_const(compiler_common::bitlength::FIELD as u64),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented =
                    context
                        .builder
                        .build_int_add(index_value, context.field_const(1), "");
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let operand_1 = context.build_load(operand_1_pointer, "").into_int_value();
                let operand_2 = context.build_load(operand_2_pointer, "").into_int_value();
                let bit_1 = context.builder.build_int_unsigned_rem(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                let bit_2 = context.builder.build_int_unsigned_rem(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                let operand_1 = context.builder.build_int_unsigned_div(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_1_pointer, operand_1);
                let operand_2 = context.builder.build_int_unsigned_div(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_2_pointer, operand_2);
                let bit_result = context.builder.build_int_mul(bit_1, bit_2, "");
                let bit_result = context
                    .builder
                    .build_int_z_extend_or_bit_cast(bit_result, llvm_type, "");
                let shift_value = context.build_load(shift_pointer, "").into_int_value();
                let bit_result = context.builder.build_int_mul(bit_result, shift_value, "");
                let shift_value =
                    context
                        .builder
                        .build_int_mul(shift_value, llvm_type.const_int(2, false), "");
                context.build_store(shift_pointer, shift_value);
                let result = context.build_load(result_pointer, "").into_int_value();
                let result = context.builder.build_int_add(result, bit_result, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Or => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::X86)
                    || (arguments[0].into_int_value().is_const()
                        && arguments[1].into_int_value().is_const())
                {
                    return Some(
                        context
                            .builder
                            .build_or(
                                arguments[0].into_int_value(),
                                arguments[1].into_int_value(),
                                "",
                            )
                            .as_basic_value_enum(),
                    );
                }

                let llvm_type = context.integer_type(compiler_common::bitlength::FIELD);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let result_pointer = context.build_alloca(llvm_type, "");
                context.build_store(result_pointer, llvm_type.const_zero());
                let operand_1_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_1_pointer, arguments[0]);
                let operand_2_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_2_pointer, arguments[1]);
                let index_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                context.build_store(index_pointer, context.field_const(0));
                let shift_pointer = context.build_alloca(llvm_type, "");
                context.build_store(shift_pointer, llvm_type.const_int(1, false));
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    context.field_const(compiler_common::bitlength::FIELD as u64),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented =
                    context
                        .builder
                        .build_int_add(index_value, context.field_const(1), "");
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let operand_1 = context.build_load(operand_1_pointer, "").into_int_value();
                let operand_2 = context.build_load(operand_2_pointer, "").into_int_value();
                let bit_1 = context.builder.build_int_unsigned_rem(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                let bit_2 = context.builder.build_int_unsigned_rem(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                let operand_1 = context.builder.build_int_unsigned_div(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_1_pointer, operand_1);
                let operand_2 = context.builder.build_int_unsigned_div(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_2_pointer, operand_2);
                let bit_result = context.builder.build_int_add(bit_1, bit_2, "");
                let bit_result = context.builder.build_int_compare(
                    inkwell::IntPredicate::UGT,
                    bit_result,
                    llvm_type.const_zero(),
                    "",
                );
                let bit_result = context
                    .builder
                    .build_int_z_extend_or_bit_cast(bit_result, llvm_type, "");
                let shift_value = context.build_load(shift_pointer, "").into_int_value();
                let bit_result = context.builder.build_int_mul(bit_result, shift_value, "");
                let shift_value =
                    context
                        .builder
                        .build_int_mul(shift_value, llvm_type.const_int(2, false), "");
                context.build_store(shift_pointer, shift_value);
                let result = context.build_load(result_pointer, "").into_int_value();
                let result = context.builder.build_int_add(result, bit_result, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Xor => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::X86)
                    || (arguments[0].into_int_value().is_const()
                        && arguments[1].into_int_value().is_const())
                {
                    return Some(
                        context
                            .builder
                            .build_xor(
                                arguments[0].into_int_value(),
                                arguments[1].into_int_value(),
                                "",
                            )
                            .as_basic_value_enum(),
                    );
                }

                let llvm_type = context.integer_type(compiler_common::bitlength::FIELD);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let result_pointer = context.build_alloca(llvm_type, "");
                context.build_store(result_pointer, llvm_type.const_zero());
                let operand_1_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_1_pointer, arguments[0]);
                let operand_2_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_2_pointer, arguments[1]);
                let index_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                context.build_store(index_pointer, context.field_const(0));
                let shift_pointer = context.build_alloca(llvm_type, "");
                context.build_store(shift_pointer, llvm_type.const_int(1, false));
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    context.field_const(compiler_common::bitlength::FIELD as u64),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented =
                    context
                        .builder
                        .build_int_add(index_value, context.field_const(1), "");
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let operand_1 = context.build_load(operand_1_pointer, "").into_int_value();
                let operand_2 = context.build_load(operand_2_pointer, "").into_int_value();
                let bit_1 = context.builder.build_int_unsigned_rem(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                let bit_2 = context.builder.build_int_unsigned_rem(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                let operand_1 = context.builder.build_int_unsigned_div(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_1_pointer, operand_1);
                let operand_2 = context.builder.build_int_unsigned_div(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_2_pointer, operand_2);
                let bit_result =
                    context
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, bit_1, bit_2, "");
                let bit_result = context
                    .builder
                    .build_int_z_extend_or_bit_cast(bit_result, llvm_type, "");
                let shift_value = context.build_load(shift_pointer, "").into_int_value();
                let bit_result = context.builder.build_int_mul(bit_result, shift_value, "");
                let shift_value =
                    context
                        .builder
                        .build_int_mul(shift_value, llvm_type.const_int(2, false), "");
                context.build_store(shift_pointer, shift_value);
                let result = context.build_load(result_pointer, "").into_int_value();
                let result = context.builder.build_int_add(result, bit_result, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Not => {
                let arguments = self.pop_arguments::<1>(context);

                if matches!(context.target, Target::X86) || arguments[0].into_int_value().is_const()
                {
                    return Some(
                        context
                            .builder
                            .build_not(arguments[0].into_int_value(), "")
                            .as_basic_value_enum(),
                    );
                }

                let llvm_type = context.integer_type(compiler_common::bitlength::FIELD);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let result_pointer = context.build_alloca(llvm_type, "");
                context.build_store(result_pointer, llvm_type.const_zero());
                let operand_1_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_1_pointer, arguments[0]);
                let operand_2_pointer = context.build_alloca(llvm_type, "");
                context.build_store(operand_2_pointer, llvm_type.const_all_ones());
                let index_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                context.build_store(index_pointer, context.field_const(0));
                let shift_pointer = context.build_alloca(llvm_type, "");
                context.build_store(shift_pointer, llvm_type.const_int(1, false));
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    context.field_const(compiler_common::bitlength::FIELD as u64),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented =
                    context
                        .builder
                        .build_int_add(index_value, context.field_const(1), "");
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let operand_1 = context.build_load(operand_1_pointer, "").into_int_value();
                let operand_2 = context.build_load(operand_2_pointer, "").into_int_value();
                let bit_1 = context.builder.build_int_unsigned_rem(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                let bit_2 = context.builder.build_int_unsigned_rem(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                let operand_1 = context.builder.build_int_unsigned_div(
                    operand_1,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_1_pointer, operand_1);
                let operand_2 = context.builder.build_int_unsigned_div(
                    operand_2,
                    llvm_type.const_int(2, false),
                    "",
                );
                context.build_store(operand_2_pointer, operand_2);
                let bit_result =
                    context
                        .builder
                        .build_int_compare(inkwell::IntPredicate::NE, bit_1, bit_2, "");
                let bit_result = context
                    .builder
                    .build_int_z_extend_or_bit_cast(bit_result, llvm_type, "");
                let shift_value = context.build_load(shift_pointer, "").into_int_value();
                let bit_result = context.builder.build_int_mul(bit_result, shift_value, "");
                let shift_value =
                    context
                        .builder
                        .build_int_mul(shift_value, llvm_type.const_int(2, false), "");
                context.build_store(shift_pointer, shift_value);
                let result = context.build_load(result_pointer, "").into_int_value();
                let result = context.builder.build_int_add(result, bit_result, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Shl => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::X86) || arguments[0].into_int_value().is_const()
                {
                    return Some(
                        context
                            .builder
                            .build_left_shift(
                                arguments[1].into_int_value(),
                                arguments[0].into_int_value(),
                                "",
                            )
                            .as_basic_value_enum(),
                    );
                }

                let result_pointer = context.build_alloca(arguments[1].get_type(), "");
                context.build_store(result_pointer, arguments[1]);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let index_pointer = context.build_alloca(arguments[0].get_type(), "");
                let index_value = arguments[0]
                    .get_type()
                    .into_int_type()
                    .const_zero()
                    .as_basic_value_enum();
                context.build_store(index_pointer, index_value);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    arguments[0].into_int_value(),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented = context.builder.build_int_add(
                    index_value,
                    arguments[0].get_type().into_int_type().const_int(1, false),
                    "",
                );
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let intermediate = context.build_load(result_pointer, "").into_int_value();
                let multiplier = arguments[1].get_type().into_int_type().const_int(2, false);
                let result = context.builder.build_int_mul(intermediate, multiplier, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Shr => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::X86) || arguments[0].into_int_value().is_const()
                {
                    return Some(
                        context
                            .builder
                            .build_right_shift(
                                arguments[1].into_int_value(),
                                arguments[0].into_int_value(),
                                false,
                                "",
                            )
                            .as_basic_value_enum(),
                    );
                }

                let result_pointer = context.build_alloca(arguments[1].get_type(), "");
                context.build_store(result_pointer, arguments[1]);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let index_pointer = context.build_alloca(arguments[0].get_type(), "");
                let index_value = arguments[0]
                    .get_type()
                    .into_int_type()
                    .const_zero()
                    .as_basic_value_enum();
                context.build_store(index_pointer, index_value);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    arguments[0].into_int_value(),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented = context.builder.build_int_add(
                    index_value,
                    arguments[0].get_type().into_int_type().const_int(1, false),
                    "",
                );
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let intermediate = context.build_load(result_pointer, "").into_int_value();
                let divider = arguments[1].get_type().into_int_type().const_int(2, false);
                let result = context
                    .builder
                    .build_int_unsigned_div(intermediate, divider, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }

            Name::AddMod => {
                let mut arguments = self.pop_arguments::<3>(context);

                let zero_block = context.append_basic_block("zero");
                let non_zero_block = context.append_basic_block("non_zero");
                let join_block = context.append_basic_block("join");

                let result_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[2].into_int_value(),
                    context.field_const(0),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                let mut result = context.builder.build_int_add(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                let initial_type = arguments[0].get_type().into_int_type();
                if let Target::X86 = context.target {
                    let allowed_type = context.integer_type(compiler_common::bitlength::BYTE * 16);
                    result =
                        context
                            .builder
                            .build_int_truncate_or_bit_cast(result, allowed_type, "");
                    arguments[2] = context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[2].into_int_value(),
                            allowed_type,
                            "",
                        )
                        .as_basic_value_enum();
                }
                let mut result = context.builder.build_int_unsigned_rem(
                    result,
                    arguments[2].into_int_value(),
                    "",
                );
                if let Target::X86 = context.target {
                    result =
                        context
                            .builder
                            .build_int_z_extend_or_bit_cast(result, initial_type, "");
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(result_pointer, context.field_const(0));
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::MulMod => {
                let mut arguments = self.pop_arguments::<3>(context);

                let zero_block = context.append_basic_block("zero");
                let non_zero_block = context.append_basic_block("non_zero");
                let join_block = context.append_basic_block("join");

                let result_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[2].into_int_value(),
                    context.field_const(0),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                let mut result = context.builder.build_int_mul(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                let initial_type = arguments[0].get_type().into_int_type();
                if let Target::X86 = context.target {
                    let allowed_type = context.integer_type(compiler_common::bitlength::BYTE * 16);
                    result =
                        context
                            .builder
                            .build_int_truncate_or_bit_cast(result, allowed_type, "");
                    arguments[2] = context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[2].into_int_value(),
                            allowed_type,
                            "",
                        )
                        .as_basic_value_enum();
                }
                let mut result = context.builder.build_int_unsigned_rem(
                    result,
                    arguments[2].into_int_value(),
                    "",
                );
                if let Target::X86 = context.target {
                    result =
                        context
                            .builder
                            .build_int_z_extend_or_bit_cast(result, initial_type, "");
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(result_pointer, context.field_const(0));
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Exp => {
                let arguments = self.pop_arguments::<2>(context);

                let result_pointer = context.build_alloca(arguments[0].get_type(), "");
                context.build_store(result_pointer, arguments[0]);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let index_pointer = context.build_alloca(arguments[1].get_type(), "");
                let index_value = arguments[1]
                    .get_type()
                    .into_int_type()
                    .const_zero()
                    .as_basic_value_enum();
                context.build_store(index_pointer, index_value);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    arguments[1].into_int_value(),
                    "",
                );
                context.build_conditional_branch(condition, body_block, join_block);

                context.set_basic_block(increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented = context.builder.build_int_add(
                    index_value,
                    arguments[1].get_type().into_int_type().const_int(1, false),
                    "",
                );
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let intermediate = context.build_load(result_pointer, "").into_int_value();
                let result =
                    context
                        .builder
                        .build_int_mul(intermediate, arguments[0].into_int_value(), "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }

            Name::Sdiv => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::Smod => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::Slt => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::Sgt => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::Sar => {
                let arguments = self.pop_arguments::<2>(context);
                Some(arguments[1])
            }
            Name::SignExtend => {
                let arguments = self.pop_arguments::<2>(context);
                Some(arguments[1])
            }

            Name::Keccak256 => {
                let arguments = self.pop_arguments::<2>(context);

                if let Target::X86 = context.target {
                    return Some(context.field_const(0).as_basic_value_enum());
                }

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let index_pointer = context.build_alloca(arguments[0].get_type(), "");
                context.build_store(index_pointer, arguments[0]);
                let range_end = context.builder.build_int_add(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
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
                    arguments[1]
                        .get_type()
                        .into_int_type()
                        .const_int(compiler_common::size::FIELD as u64, false),
                    "",
                );
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let pointer = context.access_heap(index_value, None);
                let value = context.build_load(pointer, "");
                let call_ordinar_block = context.append_basic_block("call_ordinar");
                let call_reset_block = context.append_basic_block("call_reset");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    index_value,
                    arguments[0].into_int_value(),
                    "",
                );
                context.build_conditional_branch(condition, call_reset_block, call_ordinar_block);

                context.set_basic_block(call_ordinar_block);
                let intrinsic = context.get_intrinsic_function(Intrinsic::HashAbsorb);
                context.build_call(intrinsic, &[value], "");
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(call_reset_block);
                let intrinsic = context.get_intrinsic_function(Intrinsic::HashAbsorbReset);
                context.build_call(intrinsic, &[value], "");
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let intrinsic = context.get_intrinsic_function(Intrinsic::HashOutput);
                let result = context
                    .build_call(intrinsic, &[], "")
                    .expect("Hash output function always returns a value");

                Some(result)
            }

            Name::MLoad => {
                let arguments = self.pop_arguments::<1>(context);

                if let Some(value) = arguments[0].into_int_value().get_zero_extended_constant() {
                    if value % (compiler_common::size::FIELD as u64) != 0 {
                        return None;
                    }
                }

                let pointer = context.access_heap(arguments[0].into_int_value(), None);

                let value = context.build_load(pointer, "");

                Some(value)
            }
            Name::MStore => {
                let arguments = self.pop_arguments::<2>(context);

                let offset = context.builder.build_int_truncate_or_bit_cast(
                    arguments[0].into_int_value(),
                    context.integer_type(compiler_common::bitlength::WORD),
                    "",
                );
                if let Some(value) = offset.get_zero_extended_constant() {
                    if value == 0 || value % (compiler_common::size::FIELD as u64) != 0 {
                        return None;
                    }
                }

                let pointer = context.access_heap(offset, None);

                context.build_store(pointer, arguments[1]);

                None
            }
            Name::MStore8 => {
                let arguments = self.pop_arguments::<2>(context);

                let offset = context.builder.build_int_truncate_or_bit_cast(
                    arguments[0].into_int_value(),
                    context.integer_type(compiler_common::bitlength::WORD),
                    "",
                );
                if let Some(value) = offset.get_zero_extended_constant() {
                    if value == 0 || value % (compiler_common::size::FIELD as u64) != 0 {
                        return None;
                    }
                }

                let pointer = context.access_heap(
                    offset,
                    Some(context.integer_type(compiler_common::bitlength::BYTE)),
                );

                let byte_mask = context
                    .integer_type(compiler_common::bitlength::BYTE)
                    .const_int(0xff, false);
                let value = context
                    .builder
                    .build_and(arguments[1].into_int_value(), byte_mask, "");

                context.build_store(pointer, value);

                None
            }

            Name::SLoad => {
                let arguments = self.pop_arguments::<1>(context);

                let value = match context.target {
                    Target::X86 => {
                        let pointer = context.access_storage(arguments[0].into_int_value());
                        context.build_load(pointer, "")
                    }
                    Target::zkEVM => {
                        let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);
                        let position = arguments[0];
                        let is_external_storage = context.field_const(0).as_basic_value_enum();
                        context
                            .build_call(intrinsic, &[position, is_external_storage], "")
                            .expect("Contract storage always returns a value")
                    }
                };

                Some(value)
            }
            Name::SStore => {
                let arguments = self.pop_arguments::<2>(context);

                match context.target {
                    Target::X86 => {
                        let pointer = context.access_storage(arguments[0].into_int_value());
                        context.build_store(pointer, arguments[1]);
                    }
                    Target::zkEVM => {
                        let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);
                        let value = arguments[0];
                        let position = arguments[1];
                        let is_external_storage = context.field_const(0).as_basic_value_enum();
                        context.build_call(intrinsic, &[position, value, is_external_storage], "");
                    }
                }

                None
            }

            Name::Caller => {
                let intrinsic = context.get_intrinsic_function(Intrinsic::GetFromContext);
                let value = context
                    .build_call(
                        intrinsic,
                        &[context
                            .field_const(compiler_common::ContextValue::MessageSender.into())
                            .as_basic_value_enum()],
                        "",
                    )
                    .expect("Context always returns a value");
                Some(value)
            }
            Name::CallDataLoad => {
                let arguments = self.pop_arguments::<1>(context);

                if let Some(ref test_entry_hash) = context.test_entry_hash {
                    let hash = context
                        .integer_type(compiler_common::bitlength::FIELD)
                        .const_int_from_string(
                            test_entry_hash,
                            inkwell::types::StringRadix::Hexadecimal,
                        )
                        .expect("Always valid");
                    let hash = context.builder.build_left_shift(
                        hash,
                        context.field_const(
                            ((compiler_common::size::FIELD - 4) * compiler_common::bitlength::BYTE)
                                as u64,
                        ),
                        "",
                    );
                    return Some(hash.as_basic_value_enum());
                }

                let if_zero_block = context.append_basic_block("if_zero");
                let if_non_zero_block = context.append_basic_block("if_not_zero");
                let join_block = context.append_basic_block("join");

                let value_pointer = context
                    .build_alloca(context.integer_type(compiler_common::bitlength::FIELD), "");
                context.build_store(value_pointer, context.field_const(0));
                let is_zero = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[0].into_int_value(),
                    context.field_const(0),
                    "",
                );
                context.build_conditional_branch(is_zero, if_zero_block, if_non_zero_block);

                context.set_basic_block(if_zero_block);
                let offset = context.field_const(
                    (compiler_common::contract::ABI_OFFSET_ENTRY_HASH
                        * compiler_common::size::FIELD) as u64,
                );
                let pointer = context.access_calldata(offset);
                let value = context.build_load(pointer, "");
                context.build_store(value_pointer, value);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(if_non_zero_block);
                let offset = match context.target {
                    Target::X86 => arguments[0].into_int_value(),
                    Target::zkEVM => context.builder.build_int_add(
                        arguments[0].into_int_value(),
                        context.field_const(
                            (compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                                * compiler_common::size::FIELD
                                - 4) as u64,
                        ),
                        "",
                    ),
                };
                let pointer = context.access_calldata(offset);
                let value = context.build_load(pointer, "");
                context.build_store(value_pointer, value);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let value = context.build_load(value_pointer, "");
                Some(value)
            }
            Name::CallDataSize => match context.target {
                Target::X86 => Some(context.field_const(4).as_basic_value_enum()),
                Target::zkEVM if context.test_entry_hash.is_some() => {
                    Some(context.field_const(4).as_basic_value_enum())
                }
                Target::zkEVM => {
                    let pointer = context.builder.build_int_to_ptr(
                        context.field_const(
                            (compiler_common::contract::ABI_OFFSET_CALLDATA_SIZE
                                * compiler_common::size::FIELD) as u64,
                        ),
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .ptr_type(compiler_common::AddressSpace::Parent.into()),
                        "",
                    );
                    let value = context.build_load(pointer, "");
                    let value = context.builder.build_int_mul(
                        value.into_int_value(),
                        context.field_const(compiler_common::size::FIELD as u64),
                        "",
                    );
                    let value = context
                        .builder
                        .build_int_add(value, context.field_const(4), "");
                    Some(value.as_basic_value_enum())
                }
            },
            Name::CallDataCopy => {
                let arguments = self.pop_arguments::<3>(context);

                match context.target {
                    Target::X86 => return None,
                    Target::zkEVM if context.test_entry_hash.is_some() => return None,
                    Target::zkEVM => {}
                }

                let destination = context.builder.build_int_to_ptr(
                    arguments[0].into_int_value(),
                    context
                        .integer_type(compiler_common::bitlength::FIELD)
                        .ptr_type(compiler_common::AddressSpace::Heap.into()),
                    "",
                );

                let source_offset_shift = compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                    * compiler_common::size::FIELD
                    - 4;
                let source_offset = context.builder.build_int_add(
                    arguments[1].into_int_value(),
                    context.field_const(source_offset_shift as u64),
                    "",
                );
                let source = context.builder.build_int_to_ptr(
                    source_offset,
                    context
                        .integer_type(compiler_common::bitlength::FIELD)
                        .ptr_type(compiler_common::AddressSpace::Parent.into()),
                    "",
                );

                let size = arguments[2].into_int_value();

                let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromParent);
                context.build_call(
                    intrinsic,
                    &[
                        destination.as_basic_value_enum(),
                        source.as_basic_value_enum(),
                        size.as_basic_value_enum(),
                        context
                            .integer_type(compiler_common::bitlength::BOOLEAN)
                            .const_zero()
                            .as_basic_value_enum(),
                    ],
                    "",
                );

                None
            }

            Name::Gas => {
                let intrinsic = context.get_intrinsic_function(Intrinsic::GetFromContext);
                let value = context
                    .build_call(
                        intrinsic,
                        &[context
                            .field_const(compiler_common::ContextValue::GasLeft.into())
                            .as_basic_value_enum()],
                        "",
                    )
                    .expect("Context always returns a value");
                Some(value)
            }
            Name::Address => {
                let intrinsic = context.get_intrinsic_function(Intrinsic::GetFromContext);
                let value = context
                    .build_call(
                        intrinsic,
                        &[context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_int(compiler_common::ContextValue::Address.into(), false)
                            .as_basic_value_enum()],
                        "",
                    )
                    .expect("Context always returns a value");
                Some(value)
            }

            Name::Timestamp => {
                let intrinsic = context.get_intrinsic_function(Intrinsic::GetFromContext);
                let value = context
                    .build_call(
                        intrinsic,
                        &[context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_int(compiler_common::ContextValue::BlockTimestamp.into(), false)
                            .as_basic_value_enum()],
                        "",
                    )
                    .expect("Context always returns a value");
                Some(value)
            }
            Name::Number => {
                let intrinsic = context.get_intrinsic_function(Intrinsic::GetFromContext);
                let value = context
                    .build_call(
                        intrinsic,
                        &[context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_int(compiler_common::ContextValue::BlockNumber.into(), false)
                            .as_basic_value_enum()],
                        "",
                    )
                    .expect("Context always returns a value");
                Some(value)
            }

            Name::Log0 => {
                let arguments = self.pop_arguments::<2>(context);
                Self::log_call(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    vec![],
                );
                None
            }
            Name::Log1 => {
                let arguments = self.pop_arguments::<3>(context);
                Self::log_call(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                );
                None
            }
            Name::Log2 => {
                let arguments = self.pop_arguments::<4>(context);
                Self::log_call(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                );
                None
            }
            Name::Log3 => {
                let arguments = self.pop_arguments::<5>(context);
                Self::log_call(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                );
                None
            }
            Name::Log4 => {
                let arguments = self.pop_arguments::<6>(context);
                Self::log_call(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                );
                None
            }

            Name::Call => {
                let arguments = self.pop_arguments::<7>(context);

                if let Target::X86 = context.target {
                    return Some(
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_zero()
                            .as_basic_value_enum(),
                    );
                }

                let address = arguments[1].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                Some(Self::contract_call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                ))
            }
            Name::CallCode => {
                let arguments = self.pop_arguments::<7>(context);

                if let Target::X86 = context.target {
                    return Some(
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_zero()
                            .as_basic_value_enum(),
                    );
                }

                let address = arguments[1].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                Some(Self::contract_call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                ))
            }
            Name::DelegateCall => {
                let arguments = self.pop_arguments::<6>(context);

                if let Target::X86 = context.target {
                    return Some(
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_zero()
                            .as_basic_value_enum(),
                    );
                }

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                Some(Self::contract_call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                ))
            }
            Name::StaticCall => {
                let arguments = self.pop_arguments::<6>(context);

                if let Target::X86 = context.target {
                    return Some(
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .const_zero()
                            .as_basic_value_enum(),
                    );
                }

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                Some(Self::contract_call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                ))
            }

            Name::Return => {
                let arguments = self.pop_arguments::<2>(context);

                let function = context.function().to_owned();

                match context.target {
                    Target::X86 => {
                        let heap_type = match context.target {
                            Target::X86 => {
                                Some(context.integer_type(compiler_common::bitlength::BYTE))
                            }
                            Target::zkEVM => None,
                        };

                        let source = context.access_heap(arguments[0].into_int_value(), heap_type);

                        if let Some(return_pointer) = function.return_pointer() {
                            context
                                .builder
                                .build_memcpy(
                                    return_pointer,
                                    (compiler_common::size::BYTE) as u32,
                                    source,
                                    (compiler_common::size::BYTE) as u32,
                                    arguments[1].into_int_value(),
                                )
                                .expect("Return memory copy failed");
                        }
                    }
                    Target::zkEVM => {
                        let intrinsic =
                            context.get_intrinsic_function(Intrinsic::MemoryCopyToParent);

                        let source = context.builder.build_int_to_ptr(
                            arguments[0].into_int_value(),
                            context
                                .integer_type(compiler_common::bitlength::FIELD)
                                .ptr_type(compiler_common::AddressSpace::Heap.into()),
                            "",
                        );

                        if context.test_entry_hash.is_some() {
                            if let Some(return_pointer) = function.return_pointer() {
                                let result = context.build_load(source, "");
                                context.build_store(return_pointer, result);
                            }
                        } else {
                            let destination = context.builder.build_int_to_ptr(
                                context
                                    .integer_type(compiler_common::bitlength::FIELD)
                                    .const_int(
                                        (compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                                            * compiler_common::size::FIELD)
                                            as u64,
                                        false,
                                    ),
                                context
                                    .integer_type(compiler_common::bitlength::FIELD)
                                    .ptr_type(compiler_common::AddressSpace::Parent.into()),
                                "",
                            );

                            let size = arguments[1].into_int_value();

                            context.build_call(
                                intrinsic,
                                &[
                                    destination.as_basic_value_enum(),
                                    source.as_basic_value_enum(),
                                    size.as_basic_value_enum(),
                                    context
                                        .integer_type(compiler_common::bitlength::BOOLEAN)
                                        .const_zero()
                                        .as_basic_value_enum(),
                                ],
                                "",
                            );
                        }
                    }
                }

                context.build_unconditional_branch(function.return_block);
                None
            }
            Name::ReturnDataSize => match context.target {
                Target::X86 => Some(
                    context
                        .integer_type(compiler_common::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                ),
                Target::zkEVM => {
                    let pointer = context.builder.build_int_to_ptr(
                        context.field_const(
                            (compiler_common::contract::ABI_OFFSET_RETURN_DATA_SIZE
                                * compiler_common::size::FIELD) as u64,
                        ),
                        context
                            .integer_type(compiler_common::bitlength::FIELD)
                            .ptr_type(compiler_common::AddressSpace::Child.into()),
                        "",
                    );
                    let value = context.build_load(pointer, "");
                    let value = context.builder.build_int_mul(
                        value.into_int_value(),
                        context.field_const(compiler_common::size::FIELD as u64),
                        "",
                    );
                    Some(value.as_basic_value_enum())
                }
            },
            Name::ReturnDataCopy => {
                let arguments = self.pop_arguments::<3>(context);

                if !matches!(context.target, Target::zkEVM) {
                    return None;
                }

                let destination = context.builder.build_int_to_ptr(
                    arguments[0].into_int_value(),
                    context
                        .integer_type(compiler_common::bitlength::FIELD)
                        .ptr_type(compiler_common::AddressSpace::Heap.into()),
                    "",
                );

                let source_offset_shift = compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                    * compiler_common::size::FIELD
                    - 4;
                let source_offset = context.builder.build_int_add(
                    arguments[1].into_int_value(),
                    context.field_const(source_offset_shift as u64),
                    "",
                );
                let source = context.builder.build_int_to_ptr(
                    source_offset,
                    context
                        .integer_type(compiler_common::bitlength::FIELD)
                        .ptr_type(compiler_common::AddressSpace::Child.into()),
                    "",
                );

                let size = arguments[2].into_int_value();

                let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromChild);
                context.build_call(
                    intrinsic,
                    &[
                        destination.as_basic_value_enum(),
                        source.as_basic_value_enum(),
                        size.as_basic_value_enum(),
                        context
                            .integer_type(compiler_common::bitlength::BOOLEAN)
                            .const_zero()
                            .as_basic_value_enum(),
                    ],
                    "",
                );

                None
            }

            Name::Revert => {
                let _arguments = self.pop_arguments::<2>(context);

                let function = context.function().to_owned();

                context.build_unconditional_branch(function.throw_block);
                None
            }
            Name::Stop => {
                let function = context.function().to_owned();

                context.build_unconditional_branch(function.throw_block);
                None
            }
            Name::SelfDestruct => {
                let _arguments = self.pop_arguments::<1>(context);

                let function = context.function().to_owned();

                context.build_unconditional_branch(function.throw_block);
                None
            }
            Name::Invalid => {
                let function = context.function().to_owned();

                context.build_unconditional_branch(function.throw_block);
                None
            }

            Name::Byte => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::Pc => Some(context.field_const(0).as_basic_value_enum()),
            Name::Pop => {
                let _arguments = self.pop_arguments::<1>(context);
                None
            }
            Name::CallValue => Some(context.field_const(0).as_basic_value_enum()),
            Name::MSize => Some(context.field_const(0).as_basic_value_enum()),
            Name::Balance => Some(context.field_const(0).as_basic_value_enum()),
            Name::SelfBalance => Some(context.field_const(0).as_basic_value_enum()),
            Name::ChainId => Some(context.field_const(0).as_basic_value_enum()),
            Name::Origin => Some(context.field_const(0).as_basic_value_enum()),
            Name::GasPrice => Some(context.field_const(0).as_basic_value_enum()),
            Name::BlockHash => Some(context.field_const(0).as_basic_value_enum()),
            Name::CoinBase => Some(context.field_const(0).as_basic_value_enum()),
            Name::Difficulty => Some(context.field_const(0).as_basic_value_enum()),
            Name::GasLimit => Some(context.field_const(0).as_basic_value_enum()),
            Name::CodeSize => Some(context.field_const(0).as_basic_value_enum()),
            Name::CodeCopy => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }
            Name::ExtCodeSize => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::ExtCodeCopy => {
                let _arguments = self.pop_arguments::<4>(context);
                None
            }
            Name::ExtCodeHash => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::DataSize => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::DataOffset => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::DataCopy => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }
            Name::Create => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }
            Name::Create2 => {
                let _arguments = self.pop_arguments::<4>(context);
                None
            }
        }
    }

    ///
    /// Translates a contract call.
    ///
    fn contract_call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        address: inkwell::values::IntValue<'ctx>,
        input_offset: inkwell::values::IntValue<'ctx>,
        input_size: inkwell::values::IntValue<'ctx>,
        output_offset: inkwell::values::IntValue<'ctx>,
        output_size: inkwell::values::IntValue<'ctx>,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let intrinsic = context.get_intrinsic_function(Intrinsic::SwitchContext);
        context.build_call(intrinsic, &[], "");

        let input_offset = context.builder.build_int_unsigned_div(
            input_offset,
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .const_int(compiler_common::size::FIELD as u64, false),
            "",
        );
        let output_offset = context.builder.build_int_unsigned_div(
            output_offset,
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .const_int(compiler_common::size::FIELD as u64, false),
            "",
        );

        let stack_pointer = context.builder.build_int_to_ptr(
            input_offset,
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .ptr_type(compiler_common::AddressSpace::Heap.into()),
            "",
        );

        let child_pointer_input = context.builder.build_int_to_ptr(
            context.field_const(
                (compiler_common::contract::ABI_OFFSET_CALLDATA_SIZE * compiler_common::size::FIELD)
                    as u64,
            ),
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .ptr_type(compiler_common::AddressSpace::Child.into()),
            "",
        );
        context.build_store(child_pointer_input, input_size);
        let child_pointer_output = context.builder.build_int_to_ptr(
            context.field_const(
                (compiler_common::contract::ABI_OFFSET_RETURN_DATA_SIZE
                    * compiler_common::size::FIELD) as u64,
            ),
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .ptr_type(compiler_common::AddressSpace::Child.into()),
            "",
        );
        context.build_store(child_pointer_output, output_size);

        let destination = context.builder.build_int_to_ptr(
            context.field_const(
                (compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                    * compiler_common::size::FIELD) as u64,
            ),
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .ptr_type(compiler_common::AddressSpace::Child.into()),
            "",
        );
        let source = stack_pointer;

        let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyToChild);
        context.build_call(
            intrinsic,
            &[
                destination.as_basic_value_enum(),
                source.as_basic_value_enum(),
                input_size.as_basic_value_enum(),
                context
                    .integer_type(compiler_common::bitlength::BOOLEAN)
                    .const_zero()
                    .as_basic_value_enum(),
            ],
            "",
        );

        let intrinsic = context.get_intrinsic_function(Intrinsic::FarCall);
        context.build_call(intrinsic, &[address.as_basic_value_enum()], "");

        let source = destination;
        let destination = unsafe {
            context
                .builder
                .build_gep(stack_pointer, &[output_offset], "")
        };

        let intrinsic = context.get_intrinsic_function(Intrinsic::MemoryCopyFromChild);
        context.build_call(
            intrinsic,
            &[
                destination.as_basic_value_enum(),
                source.as_basic_value_enum(),
                input_size.as_basic_value_enum(),
                context
                    .integer_type(compiler_common::bitlength::BOOLEAN)
                    .const_zero()
                    .as_basic_value_enum(),
            ],
            "",
        );

        context
            .integer_type(compiler_common::bitlength::FIELD)
            .const_int(1, false)
            .as_basic_value_enum()
    }

    ///
    /// Translates a log or event call.
    ///
    fn log_call<'ctx>(
        context: &mut LLVMContext<'ctx>,
        range_start: inkwell::values::IntValue<'ctx>,
        length: inkwell::values::IntValue<'ctx>,
        topics: Vec<inkwell::values::IntValue<'ctx>>,
    ) {
        let condition_block = context.append_basic_block("condition");
        let body_block = context.append_basic_block("body");
        let increment_block = context.append_basic_block("increment");
        let join_block = context.append_basic_block("join");

        let index_pointer = context.build_alloca(range_start.get_type(), "");
        context.build_store(index_pointer, range_start);
        let range_end = context.builder.build_int_add(range_start, length, "");
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
            length
                .get_type()
                .const_int(compiler_common::size::FIELD as u64, false),
            "",
        );
        context.build_store(index_pointer, incremented);
        context.build_unconditional_branch(condition_block);

        context.set_basic_block(body_block);
        let index_value = context.build_load(index_pointer, "").into_int_value();
        let pointer = context.access_heap(index_value, None);
        let value = context.build_load(pointer, "");
        let condition = context.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            index_value,
            range_start,
            "",
        );
        let intrinsic = context.get_intrinsic_function(Intrinsic::Event);
        if topics.is_empty() {
            context.build_call(
                intrinsic,
                &[
                    value,
                    context.field_const(0).as_basic_value_enum(),
                    condition.as_basic_value_enum(),
                ],
                "",
            );
        } else {
            for topic in topics.into_iter() {
                context.build_call(
                    intrinsic,
                    &[
                        value,
                        topic.as_basic_value_enum(),
                        condition.as_basic_value_enum(),
                    ],
                    "",
                );
            }
        }
        context.build_unconditional_branch(increment_block);

        context.set_basic_block(join_block);
    }

    ///
    /// Pops the specified number of arguments.
    ///
    fn pop_arguments<'ctx, const N: usize>(
        &mut self,
        context: &mut LLVMContext<'ctx>,
    ) -> [inkwell::values::BasicValueEnum<'ctx>; N] {
        self.arguments
            .drain(0..N)
            .map(|argument| argument.into_llvm(context).expect("Always exists"))
            .collect::<Vec<inkwell::values::BasicValueEnum<'ctx>>>()
            .try_into()
            .expect("Always successful")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_void() {
        let input = r#"object "Test" { code {
            function bar() {}

            function foo() -> x {
                x := 42
                bar()
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_non_void() {
        let input = r#"object "Test" { code {
            function bar() -> x {
                x:= 42
            }

            function foo() -> x {
                x := bar()
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_with_arguments() {
        let input = r#"object "Test" { code {
            function foo(z) -> x {
                let y := 3
                x := add(3, y)
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_add() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := add(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_sub() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := sub(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_mul() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := mul(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_div() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := div(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_sdiv() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := sdiv(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_mod() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := mod(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_smod() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := smod(3, y)}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }
}
