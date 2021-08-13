//!
//! The function call subexpression.
//!

pub mod arithmetic;
pub mod calldata;
pub mod comparison;
pub mod context;
pub mod contract;
pub mod event;
pub mod hash;
pub mod mathematic;
pub mod memory;
pub mod name;
pub mod storage;

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
                    let r#type =
                        context
                            .structure_type(vec![context.field_type().as_basic_type_enum(); size]);
                    let pointer = context
                        .build_alloca(r#type, format!("{}_return_pointer_argument", name).as_str());
                    context.build_store(pointer, r#type.const_zero());
                    arguments.insert(0, pointer.as_basic_value_enum());
                }

                let return_value = context.build_invoke(
                    function.value,
                    arguments.as_slice(),
                    format!("{}_return_value", name).as_str(),
                );

                if let Some(FunctionReturn::Compound { .. }) = function.r#return {
                    let return_pointer = return_value.expect("Always exists").into_pointer_value();
                    let return_value = context.build_load(
                        return_pointer,
                        format!("{}_return_value_loaded", name).as_str(),
                    );
                    Some(return_value)
                } else {
                    return_value
                }
            }

            Name::Add => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::addition(context, arguments)
            }
            Name::Sub => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::subtraction(context, arguments)
            }
            Name::Mul => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::multiplication(context, arguments)
            }
            Name::Div => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::division(context, arguments)
            }
            Name::Mod => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::remainder(context, arguments)
            }

            Name::Lt => {
                let arguments = self.pop_arguments::<2>(context);
                comparison::compare(context, arguments, inkwell::IntPredicate::ULT)
            }
            Name::Gt => {
                let arguments = self.pop_arguments::<2>(context);
                comparison::compare(context, arguments, inkwell::IntPredicate::UGT)
            }
            Name::Eq => {
                let arguments = self.pop_arguments::<2>(context);
                comparison::compare(context, arguments, inkwell::IntPredicate::EQ)
            }
            Name::IsZero => {
                let arguments = self.pop_arguments::<1>(context);
                comparison::compare(
                    context,
                    [arguments[0], context.field_const(0).as_basic_value_enum()],
                    inkwell::IntPredicate::EQ,
                )
            }
            Name::Slt => {
                let arguments = self.pop_arguments::<2>(context);
                comparison::compare(context, arguments, inkwell::IntPredicate::ULT)
            }
            Name::Sgt => {
                let arguments = self.pop_arguments::<2>(context);
                comparison::compare(context, arguments, inkwell::IntPredicate::UGT)
            }

            Name::And => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::x86)
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

                let llvm_type = context.field_type();

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
                let index_pointer = context.build_alloca(context.field_type(), "");
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

                if matches!(context.target, Target::x86)
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

                let llvm_type = context.field_type();

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
                let index_pointer = context.build_alloca(context.field_type(), "");
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

                if matches!(context.target, Target::x86)
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

                let llvm_type = context.field_type();

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
                let index_pointer = context.build_alloca(context.field_type(), "");
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

                if matches!(context.target, Target::x86) || arguments[0].into_int_value().is_const()
                {
                    return Some(
                        context
                            .builder
                            .build_not(arguments[0].into_int_value(), "")
                            .as_basic_value_enum(),
                    );
                }

                let llvm_type = context.field_type();

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
                let index_pointer = context.build_alloca(context.field_type(), "");
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

                if matches!(context.target, Target::x86) || arguments[0].into_int_value().is_const()
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

                let result_pointer = context.build_alloca(context.field_type(), "");
                context.build_store(result_pointer, arguments[1]);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let index_pointer = context.build_alloca(context.field_type(), "");
                let index_value = context.field_const(0).as_basic_value_enum();
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
                let incremented =
                    context
                        .builder
                        .build_int_add(index_value, context.field_const(1), "");
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let intermediate = context.build_load(result_pointer, "").into_int_value();
                let multiplier = context.field_const(2);
                let result = context.builder.build_int_mul(intermediate, multiplier, "");
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(increment_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Shr => {
                let arguments = self.pop_arguments::<2>(context);

                if matches!(context.target, Target::x86) || arguments[0].into_int_value().is_const()
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

                let result_pointer = context.build_alloca(context.field_type(), "");
                context.build_store(result_pointer, arguments[1]);

                let condition_block = context.append_basic_block("condition");
                let body_block = context.append_basic_block("body");
                let increment_block = context.append_basic_block("increment");
                let join_block = context.append_basic_block("join");

                let index_pointer = context.build_alloca(context.field_type(), "");
                let index_value = context.field_const(0).as_basic_value_enum();
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
                let incremented =
                    context
                        .builder
                        .build_int_add(index_value, context.field_const(1), "");
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(condition_block);

                context.set_basic_block(body_block);
                let intermediate = context.build_load(result_pointer, "").into_int_value();
                let divider = context.field_const(2);
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
                let arguments = self.pop_arguments::<3>(context);
                mathematic::add_mod(context, arguments)
            }
            Name::MulMod => {
                let arguments = self.pop_arguments::<3>(context);
                mathematic::mul_mod(context, arguments)
            }
            Name::Exp => {
                let arguments = self.pop_arguments::<2>(context);
                mathematic::exponent(context, arguments)
            }

            Name::Sdiv => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(context.field_const(0).as_basic_value_enum())
            }
            Name::Smod => {
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
                hash::keccak256(context, arguments)
            }

            Name::MLoad => {
                let arguments = self.pop_arguments::<1>(context);
                memory::load(context, arguments)
            }
            Name::MStore => {
                let arguments = self.pop_arguments::<2>(context);
                memory::store(context, arguments)
            }
            Name::MStore8 => {
                let arguments = self.pop_arguments::<2>(context);
                memory::store_byte(context, arguments)
            }

            Name::SLoad => {
                let arguments = self.pop_arguments::<1>(context);
                storage::load(context, arguments)
            }
            Name::SStore => {
                let arguments = self.pop_arguments::<2>(context);
                storage::store(context, arguments)
            }

            Name::CallDataLoad => {
                let arguments = self.pop_arguments::<1>(context);
                calldata::load(context, arguments)
            }
            Name::CallDataSize => calldata::size(context),
            Name::CallDataCopy => {
                let arguments = self.pop_arguments::<3>(context);
                calldata::copy(context, arguments)
            }

            Name::Address => context::get(context, compiler_common::ContextValue::Address),
            Name::Caller => context::get(context, compiler_common::ContextValue::MessageSender),
            Name::Timestamp => context::get(context, compiler_common::ContextValue::BlockTimestamp),
            Name::Number => context::get(context, compiler_common::ContextValue::BlockNumber),
            Name::Gas => context::get(context, compiler_common::ContextValue::GasLeft),

            Name::Log0 => {
                let arguments = self.pop_arguments::<2>(context);
                event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    vec![],
                )
            }
            Name::Log1 => {
                let arguments = self.pop_arguments::<3>(context);
                event::log(
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
                event::log(
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
                event::log(
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
                event::log(
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

                let address = arguments[1].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                contract::call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::CallCode => {
                let arguments = self.pop_arguments::<7>(context);

                let address = arguments[1].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                contract::call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::DelegateCall => {
                let arguments = self.pop_arguments::<6>(context);

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                contract::call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::StaticCall => {
                let arguments = self.pop_arguments::<6>(context);

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                contract::call(
                    context,
                    address,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }

            Name::Return => {
                let arguments = self.pop_arguments::<2>(context);

                let function = context.function().to_owned();

                match context.target {
                    Target::x86 => {
                        let heap_type = match context.target {
                            Target::x86 => {
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
                                .field_type()
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
                                context.field_const(
                                    (compiler_common::contract::ABI_OFFSET_CALL_RETURN_DATA
                                        * compiler_common::size::FIELD)
                                        as u64,
                                ),
                                context
                                    .field_type()
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
                Target::x86 => Some(context.field_const(0).as_basic_value_enum()),
                Target::zkEVM => {
                    let pointer = context.builder.build_int_to_ptr(
                        context.field_const(
                            (compiler_common::contract::ABI_OFFSET_RETURN_DATA_SIZE
                                * compiler_common::size::FIELD) as u64,
                        ),
                        context
                            .field_type()
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
                        .field_type()
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
                        .field_type()
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
