//!
//! The function call subexpression.
//!

pub mod contract;
pub mod create;
pub mod name;
pub mod storage;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::error::Error;
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
    pub fn into_llvm<'ctx, 'dep, D>(
        mut self,
        context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
    where
        D: compiler_llvm_context::Dependency,
    {
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

                if let Some(compiler_llvm_context::FunctionReturn::Compound { size, .. }) =
                    function.r#return
                {
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

                if let Some(compiler_llvm_context::FunctionReturn::Compound { .. }) =
                    function.r#return
                {
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
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::addition(context, arguments)
            }
            Name::Sub => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::subtraction(context, arguments)
            }
            Name::Mul => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::multiplication(context, arguments)
            }
            Name::Div => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::division(context, arguments)
            }
            Name::Mod => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::remainder(context, arguments)
            }
            Name::Sdiv => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::division_signed(context, arguments)
            }
            Name::Smod => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::remainder_signed(context, arguments)
            }

            Name::Lt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::ULT,
                )
            }
            Name::Gt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::UGT,
                )
            }
            Name::Eq => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::EQ,
                )
            }
            Name::IsZero => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    [arguments[0], context.field_const(0).as_basic_value_enum()],
                    inkwell::IntPredicate::EQ,
                )
            }
            Name::Slt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::SLT,
                )
            }
            Name::Sgt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::SGT,
                )
            }

            Name::Or => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::or(context, arguments)
            }
            Name::Xor => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::xor(context, arguments)
            }
            Name::Not => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::bitwise::xor(
                    context,
                    [
                        arguments[0],
                        context.field_type().const_all_ones().as_basic_value_enum(),
                    ],
                )
            }
            Name::And => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::and(context, arguments)
            }
            Name::Shl => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::shift_left(context, arguments)
            }
            Name::Shr => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::shift_right(context, arguments)
            }
            Name::Sar => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::shift_right_arithmetic(context, arguments)
            }
            Name::Byte => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::byte(context, arguments)
            }
            Name::Pop => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(None)
            }

            Name::AddMod => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::math::add_mod(context, arguments)
            }
            Name::MulMod => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::math::mul_mod(context, arguments)
            }
            Name::Exp => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::math::exponent(context, arguments)
            }
            Name::SignExtend => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::math::sign_extend(context, arguments)
            }

            Name::Keccak256 => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::hash::keccak256(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                )
            }

            Name::MLoad => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::memory::load(context, arguments)
            }
            Name::MStore => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::memory::store(context, arguments)
            }
            Name::MStore8 => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::memory::store_byte(context, arguments)
            }

            Name::SLoad => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::storage::load(context, arguments)
            }
            Name::SStore => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::storage::store(context, arguments)
            }
            Name::LoadImmutable => {
                let arguments = self.pop_arguments::<D, 1>(context)?;
                storage::load_immutable(context, arguments)
            }
            Name::SetImmutable => {
                let arguments = self.pop_arguments::<D, 3>(context)?;
                storage::set_immutable(context, arguments)
            }

            Name::CallDataLoad => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::calldata::load(context, arguments)
            }
            Name::CallDataSize => compiler_llvm_context::calldata::size(context),
            Name::CallDataCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            Name::CodeSize => compiler_llvm_context::calldata::size(context),
            Name::CodeCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::calldata::codecopy(context, arguments)
            }
            Name::ExtCodeSize => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(Some(context.field_const(0xffff).as_basic_value_enum()))
            }
            Name::ReturnDataSize => compiler_llvm_context::return_data::size(context),
            Name::ReturnDataCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::return_data::copy(context, arguments)
            }

            Name::Return => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::r#return::r#return(context, arguments)
            }
            Name::Revert => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::r#return::revert(context, arguments)
            }
            Name::Stop => compiler_llvm_context::r#return::stop(context),
            Name::Invalid => compiler_llvm_context::r#return::invalid(context),

            Name::Log0 => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    vec![],
                )
            }
            Name::Log1 => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::event::log(
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
                let arguments = self.pop_arguments_llvm::<D, 4>(context)?;
                compiler_llvm_context::event::log(
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
                let arguments = self.pop_arguments_llvm::<D, 5>(context)?;
                compiler_llvm_context::event::log(
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
                let arguments = self.pop_arguments_llvm::<D, 6>(context)?;
                compiler_llvm_context::event::log(
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
                let arguments = self.pop_arguments_llvm::<D, 7>(context)?;

                let address = arguments[1].into_int_value();
                let value = arguments[2].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::FarCall,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::CallCode => {
                let arguments = self.pop_arguments_llvm::<D, 7>(context)?;

                let address = arguments[1].into_int_value();
                let value = arguments[2].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::CallCode,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::StaticCall => {
                let arguments = self.pop_arguments_llvm::<D, 6>(context)?;

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::StaticCall,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::DelegateCall => {
                let arguments = self.pop_arguments_llvm::<D, 6>(context)?;

                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::DelegateCall,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::LinkerSymbol => {
                let arguments = self.pop_arguments::<D, 1>(context)?;
                contract::linker_symbol(context, arguments)
            }

            Name::Create => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;

                let value = arguments[0].into_int_value();
                let input_offset = arguments[1].into_int_value();
                let input_size = arguments[2].into_int_value();

                compiler_llvm_context::create::create(context, value, input_offset, input_size)
            }
            Name::Create2 => {
                let arguments = self.pop_arguments_llvm::<D, 4>(context)?;

                let value = arguments[0].into_int_value();
                let input_offset = arguments[1].into_int_value();
                let input_size = arguments[2].into_int_value();
                let salt = arguments[3].into_int_value();

                compiler_llvm_context::create::create2(
                    context,
                    value,
                    input_offset,
                    input_size,
                    Some(salt),
                )
            }
            Name::DataSize => {
                let arguments = self.pop_arguments::<D, 1>(context)?;
                create::datasize(context, arguments)
            }
            Name::DataOffset => {
                let arguments = self.pop_arguments::<D, 1>(context)?;
                create::dataoffset(context, arguments)
            }
            Name::DataCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                create::datacopy(context, arguments)
            }

            Name::MemoryGuard => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(Some(arguments[0]))
            }

            Name::Address => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::Address,
            ),
            Name::Caller => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::MessageSender,
            ),
            Name::Timestamp => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::BlockTimestamp,
            ),
            Name::Number => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::BlockNumber,
            ),
            Name::Gas => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::GasLeft,
            ),

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
            Name::Origin => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::ChainId => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::BlockHash => Ok(Some(context.field_const(0).as_basic_value_enum())),

            Name::Difficulty => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::Pc => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::Balance => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::SelfBalance => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::CoinBase => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::ExtCodeCopy => {
                let _arguments = self.pop_arguments_llvm::<D, 4>(context)?;
                Ok(None)
            }
            Name::ExtCodeHash => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            Name::SelfDestruct => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(None)
            }
        }
    }

    ///
    /// Pops the specified number of arguments, converted into their LLVM values.
    ///
    fn pop_arguments_llvm<'ctx, 'dep, D, const N: usize>(
        &mut self,
        context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> anyhow::Result<[inkwell::values::BasicValueEnum<'ctx>; N]>
    where
        D: compiler_llvm_context::Dependency,
    {
        let mut arguments = Vec::with_capacity(N);
        for expression in self.arguments.drain(0..N) {
            arguments.push(expression.into_llvm(context)?.expect("Always exists").value);
        }

        Ok(arguments.try_into().expect("Always successful"))
    }

    ///
    /// Pops the specified number of arguments.
    ///
    fn pop_arguments<'ctx, 'dep, D, const N: usize>(
        &mut self,
        context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> anyhow::Result<[compiler_llvm_context::Argument<'ctx>; N]>
    where
        D: compiler_llvm_context::Dependency,
    {
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
