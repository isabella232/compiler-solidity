//!
//! The function call subexpression.
//!

pub mod name;

use std::convert::TryInto;

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
use inkwell::types::BasicType;

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
                    .build_alloca(context.integer_type(compiler_const::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[1].into_int_value(),
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                if let Target::LLVM = context.target {
                    let allowed_type = context.integer_type(compiler_const::bitlength::BYTE * 16);
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
                if let Target::LLVM = context.target {
                    result = context.builder.build_int_z_extend_or_bit_cast(
                        result,
                        context.integer_type(compiler_const::bitlength::FIELD),
                        "",
                    );
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(
                    result_pointer,
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                );
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
                    .build_alloca(context.integer_type(compiler_const::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[1].into_int_value(),
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                if let Target::LLVM = context.target {
                    let allowed_type = context.integer_type(compiler_const::bitlength::BYTE * 16);
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
                if let Target::LLVM = context.target {
                    result = context.builder.build_int_z_extend_or_bit_cast(
                        result,
                        context.integer_type(compiler_const::bitlength::FIELD),
                        "",
                    );
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(
                    result_pointer,
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                );
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }
            Name::Not => {
                let arguments = self.pop_arguments::<1>(context);
                let result = context.builder.build_not(arguments[0].into_int_value(), "");
                Some(result.as_basic_value_enum())
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
                    context.integer_type(compiler_const::bitlength::FIELD),
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
                    context.integer_type(compiler_const::bitlength::FIELD),
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
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::IsZero => {
                let arguments = self.pop_arguments::<1>(context);
                let mut result = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[0].into_int_value(),
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                    "",
                );
                result = context.builder.build_int_z_extend_or_bit_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::And => {
                let arguments = self.pop_arguments::<2>(context);
                let result = context.builder.build_and(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Or => {
                let arguments = self.pop_arguments::<2>(context);
                let result = context.builder.build_or(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Xor => {
                let arguments = self.pop_arguments::<2>(context);
                let result = context.builder.build_xor(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::AddMod => {
                let mut arguments = self.pop_arguments::<3>(context);

                let zero_block = context.append_basic_block("zero");
                let non_zero_block = context.append_basic_block("non_zero");
                let join_block = context.append_basic_block("join");

                let result_pointer = context
                    .build_alloca(context.integer_type(compiler_const::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[2].into_int_value(),
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                let mut result = context.builder.build_int_add(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                if let Target::LLVM = context.target {
                    let allowed_type = context.integer_type(compiler_const::bitlength::BYTE * 16);
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
                if let Target::LLVM = context.target {
                    result = context.builder.build_int_z_extend_or_bit_cast(
                        result,
                        context.integer_type(compiler_const::bitlength::FIELD),
                        "",
                    );
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(
                    result_pointer,
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                );
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
                    .build_alloca(context.integer_type(compiler_const::bitlength::FIELD), "");
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[2].into_int_value(),
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                    "",
                );
                context.build_conditional_branch(condition, zero_block, non_zero_block);

                context.set_basic_block(non_zero_block);
                let mut result = context.builder.build_int_mul(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                if let Target::LLVM = context.target {
                    let allowed_type = context.integer_type(compiler_const::bitlength::BYTE * 16);
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
                if let Target::LLVM = context.target {
                    result = context.builder.build_int_z_extend_or_bit_cast(
                        result,
                        context.integer_type(compiler_const::bitlength::FIELD),
                        "",
                    );
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(join_block);

                context.set_basic_block(zero_block);
                context.build_store(
                    result_pointer,
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                );
                context.build_unconditional_branch(join_block);

                context.set_basic_block(join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
            }

            Name::Sdiv => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Smod => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
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
            Name::Slt => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Sgt => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Byte => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Shl => {
                let arguments = self.pop_arguments::<2>(context);

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

                let result_pointer = context.build_alloca(arguments[1].get_type(), "");

                let if_224_block = context.append_basic_block("if.224");
                let if_not_224_block = context.append_basic_block("if.not_224");
                let if_join_block = context.append_basic_block("if.join");
                let loop_condition_block = context.append_basic_block("loop.condition");
                let loop_body_block = context.append_basic_block("loop.body");
                let loop_increment_block = context.append_basic_block("loop.increment");
                let loop_join_block = context.append_basic_block("loop.join");

                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    arguments[0].into_int_value(),
                    arguments[0]
                        .get_type()
                        .into_int_type()
                        .const_int(224, false),
                    "",
                );
                context.build_conditional_branch(condition, if_224_block, if_not_224_block);

                context.set_basic_block(if_not_224_block);
                let intermediate_initial_value = match context.target {
                    Target::LLVM => context
                        .builder
                        .build_int_truncate_or_bit_cast(
                            arguments[1].into_int_value(),
                            context.integer_type(compiler_const::bitlength::BYTE * 16),
                            "",
                        )
                        .as_basic_value_enum(),
                    Target::zkEVM => arguments[1],
                };
                let intermediate_pointer =
                    context.build_alloca(intermediate_initial_value.get_type(), "");
                context.build_store(intermediate_pointer, intermediate_initial_value);
                let index_pointer = context.build_alloca(arguments[0].get_type(), "");
                let index_value = arguments[0]
                    .get_type()
                    .into_int_type()
                    .const_zero()
                    .as_basic_value_enum();
                context.build_store(index_pointer, index_value);
                context.build_unconditional_branch(loop_condition_block);

                context.set_basic_block(loop_condition_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let condition = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    index_value,
                    arguments[0].into_int_value(),
                    "",
                );
                context.build_conditional_branch(condition, loop_body_block, loop_join_block);

                context.set_basic_block(loop_increment_block);
                let index_value = context.build_load(index_pointer, "").into_int_value();
                let incremented = context.builder.build_int_add(
                    index_value,
                    arguments[0].get_type().into_int_type().const_int(1, false),
                    "",
                );
                context.build_store(index_pointer, incremented);
                context.build_unconditional_branch(loop_condition_block);

                context.set_basic_block(loop_body_block);
                let intermediate = context
                    .build_load(intermediate_pointer, "")
                    .into_int_value();
                let divider = intermediate.get_type().const_int(2, false);
                let result = context
                    .builder
                    .build_int_unsigned_div(intermediate, divider, "");
                context.build_store(intermediate_pointer, result);
                context.build_unconditional_branch(loop_increment_block);

                context.set_basic_block(loop_join_block);
                let mut result = context.build_load(intermediate_pointer, "");
                if let Target::LLVM = context.target {
                    result = context
                        .builder
                        .build_int_z_extend_or_bit_cast(
                            result.into_int_value(),
                            arguments[1].get_type().into_int_type(),
                            "",
                        )
                        .as_basic_value_enum();
                }
                context.build_store(result_pointer, result);
                context.build_unconditional_branch(if_join_block);

                context.set_basic_block(if_224_block);
                context.build_store(result_pointer, arguments[1]);
                context.build_unconditional_branch(if_join_block);

                context.set_basic_block(if_join_block);
                let result = context.build_load(result_pointer, "");

                Some(result)
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
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Pc => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::Pop => {
                let _arguments = self.pop_arguments::<1>(context);

                None
            }
            Name::MLoad => {
                let arguments = self.pop_arguments::<1>(context);

                let pointer = context.access_heap(arguments[0].into_int_value(), None);

                let value = context.build_load(pointer, "");

                Some(value)
            }
            Name::MStore => {
                let arguments = self.pop_arguments::<2>(context);

                let pointer = context.access_heap(arguments[0].into_int_value(), None);

                context.build_store(pointer, arguments[1]);

                None
            }
            Name::MStore8 => {
                let arguments = self.pop_arguments::<2>(context);

                let pointer = context.access_heap(
                    arguments[0].into_int_value(),
                    Some(context.integer_type(compiler_const::bitlength::BYTE)),
                );

                let byte_mask = context
                    .integer_type(compiler_const::bitlength::BYTE)
                    .const_int(0xff, false);
                let value = context
                    .builder
                    .build_and(arguments[1].into_int_value(), byte_mask, "");

                context.build_store(pointer, value);

                None
            }

            Name::SLoad => {
                let arguments = self.pop_arguments::<1>(context);
                let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);

                let position = arguments[0];
                let is_external_storage = context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_zero()
                    .as_basic_value_enum();
                let value = context
                    .builder
                    .build_call(intrinsic, &[position, is_external_storage], "")
                    .try_as_basic_value()
                    .expect_left("Contract storage always returns a value");
                Some(value)
            }
            Name::SStore => {
                let arguments = self.pop_arguments::<2>(context);
                let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);

                let position = arguments[0];
                let value = arguments[1];
                let is_external_storage = context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_zero()
                    .as_basic_value_enum();
                context
                    .builder
                    .build_call(intrinsic, &[position, value, is_external_storage], "");
                None
            }

            Name::Caller => Some(
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_zero()
                    .as_basic_value_enum(),
            ),
            Name::CallValue => Some(
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_zero()
                    .as_basic_value_enum(),
            ),
            Name::CallDataLoad => {
                let _arguments = self.pop_arguments::<1>(context);
                let hash = match context.test_entry_hash {
                    Some(ref hash) => context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_int_from_string(hash, inkwell::types::StringRadix::Hexadecimal)
                        .expect(compiler_const::panic::TEST_DATA_VALID),
                    None => context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                }
                .as_basic_value_enum();
                Some(hash)
            }
            Name::CallDataSize => Some(
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_int(4, false)
                    .as_basic_value_enum(),
            ),
            Name::CallDataCopy => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }

            Name::MSize => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Gas => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Address => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Balance => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::SelfBalance => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::ChainId => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Origin => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::GasPrice => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::BlockHash => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::CoinBase => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Timestamp => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Number => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Difficulty => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::GasLimit => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::Create => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Create2 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::Log0 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Log1 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Log2 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Log3 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Log4 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::Call => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::CallCode => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::DelegateCall => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::StaticCall => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::CodeSize => Some(
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_zero()
                    .as_basic_value_enum(),
            ),
            Name::CodeCopy => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }
            Name::ExtCodeSize => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::ExtCodeCopy => {
                let _arguments = self.pop_arguments::<4>(context);
                None
            }
            Name::ReturnDataSize => Some(
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .const_zero()
                    .as_basic_value_enum(),
            ),
            Name::ReturnDataCopy => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }
            Name::ExtCodeHash => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }

            Name::DataSize => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::DataOffset => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::DataCopy => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }

            Name::Stop => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Return => {
                let arguments = self.pop_arguments::<2>(context);

                let function = context.function().to_owned();

                let pointer = context.access_heap(
                    arguments[0].into_int_value(),
                    Some(context.integer_type(compiler_const::bitlength::BYTE)),
                );

                if let Some(return_pointer) = function.return_pointer() {
                    context
                        .builder
                        .build_memcpy(
                            return_pointer,
                            (compiler_const::size::BYTE) as u32,
                            pointer,
                            (compiler_const::size::BYTE) as u32,
                            arguments[1].into_int_value(),
                        )
                        .expect("Return memory copy failed");
                }

                context.build_unconditional_branch(function.return_block);
                None
            }
            Name::Revert => {
                let arguments = self.pop_arguments::<2>(context);

                let function = context.function().to_owned();

                let pointer = context.access_heap(
                    arguments[0].into_int_value(),
                    Some(context.integer_type(compiler_const::bitlength::BYTE)),
                );

                if let Some(return_pointer) = function.return_pointer() {
                    context
                        .builder
                        .build_memcpy(
                            return_pointer,
                            (compiler_const::size::BYTE) as u32,
                            pointer,
                            (compiler_const::size::BYTE) as u32,
                            arguments[1].into_int_value(),
                        )
                        .expect("Revert memory copy failed");
                }

                context.build_unconditional_branch(function.revert_block);
                None
            }
            Name::SelfDestruct => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Invalid => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

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
                            .integer_type(compiler_const::bitlength::FIELD)
                            .as_basic_type_enum();
                        size
                    ]);
                    let pointer = context.build_alloca(r#type, "");
                    context.build_store(pointer, r#type.const_zero());
                    arguments.insert(0, pointer.as_basic_value_enum());
                }
                let return_value = context
                    .builder
                    .build_call(function.value, &arguments, "")
                    .try_as_basic_value();
                if let Some(FunctionReturn::Compound { .. }) = function.r#return {
                    let return_pointer = return_value
                        .expect_left("Always exists")
                        .into_pointer_value();
                    let return_value = context.build_load(return_pointer, "");
                    Some(return_value)
                } else {
                    return_value.left()
                }
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
