//!
//! The function call subexpression.
//!

pub mod name;

use std::convert::TryInto;

use inkwell::values::BasicValue;

use crate::error::Error;
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
        context: &LLVMContext<'ctx>,
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
                let allowed_type = context.integer_type(128);
                arguments[0] = context
                    .builder
                    .build_int_cast(arguments[0].into_int_value(), allowed_type, "")
                    .as_basic_value_enum();
                arguments[1] = context
                    .builder
                    .build_int_cast(arguments[1].into_int_value(), allowed_type, "")
                    .as_basic_value_enum();
                let mut result = context.builder.build_int_unsigned_div(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Mod => {
                let mut arguments = self.pop_arguments::<2>(context);
                let allowed_type = context.integer_type(128);
                arguments[0] = context
                    .builder
                    .build_int_cast(arguments[0].into_int_value(), allowed_type, "")
                    .as_basic_value_enum();
                arguments[1] = context
                    .builder
                    .build_int_cast(arguments[0].into_int_value(), allowed_type, "")
                    .as_basic_value_enum();
                let mut result = context.builder.build_int_unsigned_rem(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Not => {
                let arguments = self.pop_arguments::<1>(context);
                let value = context.builder.build_not(arguments[0].into_int_value(), "");
                Some(value.as_basic_value_enum())
            }
            Name::Lt => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_cast(
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
                result = context.builder.build_int_cast(
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
                result = context.builder.build_int_cast(
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
                result = context.builder.build_int_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::And => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_and(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Or => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_or(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::Xor => {
                let arguments = self.pop_arguments::<2>(context);
                let mut result = context.builder.build_xor(
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    "",
                );
                result = context.builder.build_int_cast(
                    result,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(result.as_basic_value_enum())
            }
            Name::AddMod => {
                let arguments = self.pop_arguments::<3>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::MulMod => {
                let arguments = self.pop_arguments::<3>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
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
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
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
                let arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Shl => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Shr => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Sar => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::SignExtend => {
                let _arguments = self.pop_arguments::<2>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Keccak256 => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Pc => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::Pop => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::MLoad => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::MStore => {
                let _arguments = self.pop_arguments::<2>(context);
                None
            }
            Name::MStore8 => {
                let _arguments = self.pop_arguments::<2>(context);
                None
            }

            Name::SLoad => None,
            Name::SStore => None,

            Name::Caller => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::CallValue => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::CallDataLoad => {
                let _arguments = self.pop_arguments::<1>(context);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::CallDataSize => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::CallDataCopy => {
                panic!("The `{:?}` instruction is unsupported", self.name);
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

            Name::CodeSize => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::CodeCopy => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::ExtCodeSize => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::ExtCodeCopy => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::ReturnCodeSize => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::ReturnCodeCopy => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::ExtCodeHash => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::Stop => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Return => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Revert => {
                let _arguments = self.pop_arguments::<2>(context);
                None
            }
            Name::SelfDestruct => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }
            Name::Invalid => {
                panic!("The `{:?}` instruction is unsupported", self.name);
            }

            Name::UserDefined(name) => {
                let arguments: Vec<inkwell::values::BasicValueEnum> = self
                    .arguments
                    .into_iter()
                    .filter_map(|argument| argument.into_llvm(context))
                    .collect();
                let function = context
                    .module()
                    .get_function(name.as_str())
                    .unwrap_or_else(|| panic!("Undeclared function {}", name));
                let return_value = context
                    .builder
                    .build_call(function, &arguments, "")
                    .try_as_basic_value();
                return_value.left()
            }
        }
    }

    ///
    /// Pops the specified number of arguments.
    ///
    fn pop_arguments<'ctx, const N: usize>(
        &mut self,
        context: &LLVMContext<'ctx>,
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
