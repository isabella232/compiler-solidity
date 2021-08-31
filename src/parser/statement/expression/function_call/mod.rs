//!
//! The function call subexpression.
//!

pub mod arithmetic;
pub mod bitwise;
pub mod calldata;
pub mod comparison;
pub mod context;
pub mod contract;
pub mod event;
pub mod hash;
pub mod mathematic;
pub mod memory;
pub mod name;
pub mod r#return;
pub mod return_data;
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
            Name::Sdiv => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::division_signed(context, arguments)
            }
            Name::Smod => {
                let arguments = self.pop_arguments::<2>(context);
                arithmetic::remainder_signed(context, arguments)
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
                comparison::compare(context, arguments, inkwell::IntPredicate::SLT)
            }
            Name::Sgt => {
                let arguments = self.pop_arguments::<2>(context);
                comparison::compare(context, arguments, inkwell::IntPredicate::SGT)
            }

            Name::Or => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::or(context, arguments)
            }
            Name::Xor => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::xor(context, arguments)
            }
            Name::Not => {
                let arguments = self.pop_arguments::<1>(context);
                bitwise::xor(
                    context,
                    [
                        arguments[0],
                        context.field_type().const_all_ones().as_basic_value_enum(),
                    ],
                )
            }
            Name::And => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::and(context, arguments)
            }
            Name::Shl => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::shift_left(context, arguments)
            }
            Name::Shr => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::shift_right(context, arguments)
            }
            Name::Sar => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::shift_right_arithmetic(context, arguments)
            }
            Name::Byte => {
                let arguments = self.pop_arguments::<2>(context);
                bitwise::byte(context, arguments)
            }
            Name::Pop => {
                let _arguments = self.pop_arguments::<1>(context);
                None
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
            Name::SignExtend => {
                let arguments = self.pop_arguments::<2>(context);
                mathematic::sign_extend(context, arguments)
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
            Name::CodeSize => calldata::size(context),
            Name::CodeCopy => {
                let arguments = self.pop_arguments::<3>(context);
                calldata::codecopy(context, arguments)
            }
            Name::ExtCodeSize => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(context.field_const(0xffff).as_basic_value_enum())
            }
            Name::ReturnDataSize => return_data::size(context),
            Name::ReturnDataCopy => {
                let arguments = self.pop_arguments::<3>(context);
                return_data::copy(context, arguments)
            }

            Name::Return => {
                let arguments = self.pop_arguments::<2>(context);
                r#return::r#return(context, arguments)
            }
            Name::Revert => {
                let arguments = self.pop_arguments::<2>(context);
                r#return::revert(context, arguments)
            }

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

            Name::Address => context::get(context, compiler_common::ContextValue::Address),
            Name::Caller => context::get(context, compiler_common::ContextValue::MessageSender),
            Name::Timestamp => context::get(context, compiler_common::ContextValue::BlockTimestamp),
            Name::Number => context::get(context, compiler_common::ContextValue::BlockNumber),
            Name::Gas => context::get(context, compiler_common::ContextValue::GasLeft),

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
                    Intrinsic::FarCall,
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
                    Intrinsic::CallCode,
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
                    Intrinsic::StaticCall,
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
                    Intrinsic::DelegateCall,
                )
            }
            Name::SetImmutable => {
                let _arguments = self.pop_arguments::<3>(context);
                None
            }
            Name::LoadImmutable => {
                let _arguments = self.pop_arguments::<1>(context);
                context::get(context, compiler_common::ContextValue::Address)
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

            Name::Pc => Some(context.field_const(0).as_basic_value_enum()),
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
