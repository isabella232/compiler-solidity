//!
//! The function call subexpression.
//!

pub mod arithmetic;
pub mod bitwise;
pub mod calldata;
pub mod comparison;
pub mod context;
pub mod contract;
pub mod create;
pub mod event;
pub mod hash;
pub mod mathematic;
pub mod memory;
pub mod name;
pub mod r#return;
pub mod return_data;
pub mod storage;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::argument::Argument;
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
    pub fn into_llvm<'ctx, 'src>(
        mut self,
        context: &mut LLVMContext<'ctx, 'src>,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        match self.name {
            Name::UserDefined(name) => {
                let mut values = Vec::with_capacity(self.arguments.len());
                for argument in self.arguments.into_iter() {
                    let value = argument.into_llvm(context)?.expect("Always exists").value;
                    values.push(value);
                }
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
                    values.insert(0, pointer.as_basic_value_enum());
                }

                let return_value = context.build_invoke(
                    function.value,
                    values.as_slice(),
                    format!("{}_return_value", name).as_str(),
                );

                if let Some(FunctionReturn::Compound { .. }) = function.r#return {
                    let return_pointer = return_value.expect("Always exists").into_pointer_value();
                    let return_value = context.build_load(
                        return_pointer,
                        format!("{}_return_value_loaded", name).as_str(),
                    );
                    Ok(Some(return_value))
                } else {
                    Ok(return_value)
                }
            }

            Name::Add => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::addition(context, arguments)
            }
            Name::Sub => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::subtraction(context, arguments)
            }
            Name::Mul => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::multiplication(context, arguments)
            }
            Name::Div => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::division(context, arguments)
            }
            Name::Mod => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::remainder(context, arguments)
            }
            Name::Sdiv => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::division_signed(context, arguments)
            }
            Name::Smod => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                arithmetic::remainder_signed(context, arguments)
            }

            Name::Lt => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                comparison::compare(context, arguments, inkwell::IntPredicate::ULT)
            }
            Name::Gt => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                comparison::compare(context, arguments, inkwell::IntPredicate::UGT)
            }
            Name::Eq => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                comparison::compare(context, arguments, inkwell::IntPredicate::EQ)
            }
            Name::IsZero => {
                let arguments = self.pop_arguments_llvm::<1>(context)?;
                comparison::compare(
                    context,
                    [arguments[0], context.field_const(0).as_basic_value_enum()],
                    inkwell::IntPredicate::EQ,
                )
            }
            Name::Slt => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                comparison::compare(context, arguments, inkwell::IntPredicate::SLT)
            }
            Name::Sgt => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                comparison::compare(context, arguments, inkwell::IntPredicate::SGT)
            }

            Name::Or => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::or(context, arguments)
            }
            Name::Xor => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::xor(context, arguments)
            }
            Name::Not => {
                let arguments = self.pop_arguments_llvm::<1>(context)?;
                bitwise::xor(
                    context,
                    [
                        arguments[0],
                        context.field_type().const_all_ones().as_basic_value_enum(),
                    ],
                )
            }
            Name::And => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::and(context, arguments)
            }
            Name::Shl => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::shift_left(context, arguments)
            }
            Name::Shr => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::shift_right(context, arguments)
            }
            Name::Sar => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::shift_right_arithmetic(context, arguments)
            }
            Name::Byte => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                bitwise::byte(context, arguments)
            }
            Name::Pop => {
                let _arguments = self.pop_arguments_llvm::<1>(context)?;
                Ok(None)
            }

            Name::AddMod => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                mathematic::add_mod(context, arguments)
            }
            Name::MulMod => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                mathematic::mul_mod(context, arguments)
            }
            Name::Exp => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                mathematic::exponent(context, arguments)
            }
            Name::SignExtend => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                mathematic::sign_extend(context, arguments)
            }

            Name::Keccak256 => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                hash::keccak256(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                )
            }

            Name::MLoad => {
                let arguments = self.pop_arguments_llvm::<1>(context)?;
                memory::load(context, arguments)
            }
            Name::MStore => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                memory::store(context, arguments)
            }
            Name::MStore8 => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                memory::store_byte(context, arguments)
            }

            Name::SLoad => {
                let arguments = self.pop_arguments_llvm::<1>(context)?;
                storage::load(context, arguments)
            }
            Name::SStore => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                storage::store(context, arguments)
            }
            Name::LoadImmutable => {
                let arguments = self.pop_arguments::<1>(context)?;
                storage::load_immutable(context, arguments)
            }
            Name::SetImmutable => {
                let arguments = self.pop_arguments::<3>(context)?;
                storage::set_immutable(context, arguments)
            }

            Name::CallDataLoad => {
                let arguments = self.pop_arguments_llvm::<1>(context)?;
                calldata::load(context, arguments)
            }
            Name::CallDataSize => calldata::size(context),
            Name::CallDataCopy => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                calldata::copy(context, arguments)
            }
            Name::CodeSize => calldata::size(context),
            Name::CodeCopy => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                calldata::codecopy(context, arguments)
            }
            Name::ExtCodeSize => {
                let _arguments = self.pop_arguments_llvm::<1>(context)?;
                Ok(Some(context.field_const(0xffff).as_basic_value_enum()))
            }
            Name::ReturnDataSize => return_data::size(context),
            Name::ReturnDataCopy => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                return_data::copy(context, arguments)
            }

            Name::Return => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                r#return::r#return(context, arguments)
            }
            Name::Revert => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                r#return::revert(context, arguments)
            }
            Name::Stop => r#return::stop(context),
            Name::Invalid => r#return::invalid(context),

            Name::Log0 => {
                let arguments = self.pop_arguments_llvm::<2>(context)?;
                event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    vec![],
                )
            }
            Name::Log1 => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            Name::Log2 => {
                let arguments = self.pop_arguments_llvm::<4>(context)?;
                event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            Name::Log3 => {
                let arguments = self.pop_arguments_llvm::<5>(context)?;
                event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            Name::Log4 => {
                let arguments = self.pop_arguments_llvm::<6>(context)?;
                event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }

            Name::Call => {
                let arguments = self.pop_arguments_llvm::<7>(context)?;

                let address = arguments[1].into_int_value();
                let value = arguments[2].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                contract::call(
                    context,
                    Intrinsic::FarCall,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::CallCode => {
                let arguments = self.pop_arguments_llvm::<7>(context)?;

                let address = arguments[1].into_int_value();
                let value = arguments[2].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                contract::call(
                    context,
                    Intrinsic::CallCode,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::StaticCall => {
                let arguments = self.pop_arguments_llvm::<6>(context)?;

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                contract::call(
                    context,
                    Intrinsic::StaticCall,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::DelegateCall => {
                let arguments = self.pop_arguments_llvm::<6>(context)?;

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                contract::call(
                    context,
                    Intrinsic::DelegateCall,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::LinkerSymbol => {
                let arguments = self.pop_arguments::<1>(context)?;
                contract::linker_symbol(context, arguments)
            }

            Name::Create => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                create::create(context, arguments)
            }
            Name::Create2 => {
                let arguments = self.pop_arguments_llvm::<4>(context)?;
                create::create2(context, arguments)
            }
            Name::DataSize => {
                let arguments = self.pop_arguments::<1>(context)?;
                create::datasize(context, arguments)
            }
            Name::DataOffset => {
                let arguments = self.pop_arguments::<1>(context)?;
                create::dataoffset(context, arguments)
            }
            Name::DataCopy => {
                let arguments = self.pop_arguments_llvm::<3>(context)?;
                create::datacopy(context, arguments)
            }

            Name::MemoryGuard => {
                let arguments = self.pop_arguments_llvm::<1>(context)?;
                Ok(Some(arguments[0]))
            }

            Name::Address => context::get(context, compiler_common::ContextValue::Address),
            Name::Caller => context::get(context, compiler_common::ContextValue::MessageSender),
            Name::Timestamp => context::get(context, compiler_common::ContextValue::BlockTimestamp),
            Name::Number => context::get(context, compiler_common::ContextValue::BlockNumber),
            Name::Gas => context::get(context, compiler_common::ContextValue::GasLeft),

            Name::GasLimit => Ok(Some(
                context.field_const(u32::MAX as u64).as_basic_value_enum(),
            )),
            Name::GasPrice => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::CallValue => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::MSize => Ok(Some(
                context
                    .field_const(((1 << 16) * compiler_common::SIZE_FIELD) as u64)
                    .as_basic_value_enum(),
            )),
            Name::Origin => Ok(Some(context.field_const(0).as_basic_value_enum())), // TODO
            Name::ChainId => Ok(Some(context.field_const(0).as_basic_value_enum())), // TODO
            Name::BlockHash => Ok(Some(context.field_const(0).as_basic_value_enum())), // TODO

            name @ Name::Difficulty => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            name @ Name::Pc => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            name @ Name::Balance => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            name @ Name::SelfBalance => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            name @ Name::CoinBase => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            name @ Name::ExtCodeCopy => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(None)
            }
            name @ Name::ExtCodeHash => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            name @ Name::SelfDestruct => {
                eprintln!("Warning: instruction {:?} is not supported", name);
                Ok(None)
            }
        }
    }

    ///
    /// Pops the specified number of arguments, converted into their LLVM values.
    ///
    fn pop_arguments_llvm<'ctx, 'src, const N: usize>(
        &mut self,
        context: &mut LLVMContext<'ctx, 'src>,
    ) -> anyhow::Result<[inkwell::values::BasicValueEnum<'ctx>; N]> {
        let mut arguments = Vec::with_capacity(N);
        for expression in self.arguments.drain(0..N) {
            arguments.push(expression.into_llvm(context)?.expect("Always exists").value);
        }

        Ok(arguments.try_into().expect("Always successful"))
    }

    ///
    /// Pops the specified number of arguments.
    ///
    fn pop_arguments<'ctx, 'src, const N: usize>(
        &mut self,
        context: &mut LLVMContext<'ctx, 'src>,
    ) -> anyhow::Result<[Argument<'ctx>; N]> {
        let mut arguments = Vec::with_capacity(N);
        for expression in self.arguments.drain(0..N) {
            arguments.push(expression.into_llvm(context)?.expect("Always exists"));
        }

        Ok(arguments.try_into().expect("Always successful"))
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

        assert!(crate::Project::try_from_test_yul(input).is_ok());
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

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_with_arguments() {
        let input = r#"object "Test" { code {
            function foo(z) -> x {
                let y := 3
                x := add(3, y)
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_add() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := add(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_sub() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := sub(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_mul() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := mul(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_div() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := div(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_sdiv() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := sdiv(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_mod() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := mod(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_builtin_smod() {
        let input = r#"object "Test" { code {
            function foo() -> x {let y := 3 x := smod(3, y)}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }
}
