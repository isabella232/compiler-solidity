//!
//! The function call subexpression.
//!

pub mod name;

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
                let value = context.builder.build_int_add(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Sub => {
                let value = context.builder.build_int_sub(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Mul => {
                let value = context.builder.build_int_mul(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Div => {
                let value = context.builder.build_int_unsigned_div(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Mod => {
                let value = context.builder.build_int_unsigned_rem(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Not => {
                let value = context.builder.build_not(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Lt => {
                let value = context.builder.build_int_compare(
                    inkwell::IntPredicate::ULT,
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Gt => {
                let value = context.builder.build_int_compare(
                    inkwell::IntPredicate::UGT,
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Eq => {
                let value = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::IsZero => {
                let value = context.builder.build_int_compare(
                    inkwell::IntPredicate::EQ,
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::And => {
                let value = context.builder.build_and(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Or => {
                let value = context.builder.build_or(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::Xor => {
                let value = context.builder.build_xor(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(compiler_const::bitlength::FIELD),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            Name::AddMod => {
                // println!("{:?}", self.name);
                None
            }
            Name::MulMod => {
                // println!("{:?}", self.name);
                None
            }

            Name::Sdiv => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_int_signed_div(
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::Smod => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_int_signed_rem(
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::Exp => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Slt => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_int_compare(
                //     inkwell::IntPredicate::SLT,
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     "",
                // );
                // let value = context.builder.build_int_cast(
                //     value,
                //     context.integer_type(compiler_const::bitlength::FIELD),
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::Sgt => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_int_compare(
                //     inkwell::IntPredicate::SGT,
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     "",
                // );
                // let value = context.builder.build_int_cast(
                //     value,
                //     context.integer_type(compiler_const::bitlength::FIELD),
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::Byte => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Shl => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_left_shift(
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::Shr => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_right_shift(
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     false,
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::Sar => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
                // let value = context.builder.build_right_shift(
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     self.arguments
                //         .remove(0)
                //         .into_llvm(context)
                //         .expect("Always exists")
                //         .into_int_value(),
                //     true,
                //     "",
                // );
                // Some(value.as_basic_value_enum())
            }
            Name::SignExtend => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::Keccak256 => {
                // println!("{:?}", self.name);
                None
            }
            Name::Pc => {
                // println!("{:?}", self.name);
                None
            }

            Name::Pop => {
                // println!("{:?}", self.name);
                None
            }
            Name::MLoad => {
                self.arguments.remove(0);
                Some(
                    context
                        .integer_type(compiler_const::bitlength::FIELD)
                        .const_zero()
                        .as_basic_value_enum(),
                )
            }
            Name::MStore => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                None
            }
            Name::MStore8 => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                None
            }

            Name::SLoad => {
                // println!("{:?}", self.name);
                None
            }
            Name::SStore => {
                // println!("{:?}", self.name);
                None
            }

            Name::Caller => {
                // println!("{:?}", self.name);
                None
            }
            Name::CallValue => {
                // println!("{:?}", self.name);
                None
            }
            Name::CallDataLoad => {
                // println!("{:?}", self.name);
                None
            }
            Name::CallDataSize => {
                // println!("{:?}", self.name);
                None
            }
            Name::CallDataCopy => {
                // println!("{:?}", self.name);
                None
            }

            Name::MSize => {
                // println!("{:?}", self.name);
                None
            }
            Name::Gas => {
                // println!("{:?}", self.name);
                None
            }
            Name::Address => {
                // println!("{:?}", self.name);
                None
            }
            Name::Balance => {
                // println!("{:?}", self.name);
                None
            }
            Name::SelfBalance => {
                // println!("{:?}", self.name);
                None
            }

            Name::ChainId => {
                // println!("{:?}", self.name);
                None
            }
            Name::Origin => {
                // println!("{:?}", self.name);
                None
            }
            Name::GasPrice => {
                // println!("{:?}", self.name);
                None
            }
            Name::BlockHash => {
                // println!("{:?}", self.name);
                None
            }
            Name::CoinBase => {
                // println!("{:?}", self.name);
                None
            }
            Name::Timestamp => {
                // println!("{:?}", self.name);
                None
            }
            Name::Number => {
                // println!("{:?}", self.name);
                None
            }
            Name::Difficulty => {
                // println!("{:?}", self.name);
                None
            }
            Name::GasLimit => {
                // println!("{:?}", self.name);
                None
            }

            Name::Create => {
                // println!("{:?}", self.name);
                None
            }
            Name::Create2 => {
                // println!("{:?}", self.name);
                None
            }

            Name::Log0 => {
                // println!("{:?}", self.name);
                None
            }
            Name::Log1 => {
                // println!("{:?}", self.name);
                None
            }
            Name::Log2 => {
                // println!("{:?}", self.name);
                None
            }
            Name::Log3 => {
                // println!("{:?}", self.name);
                None
            }
            Name::Log4 => {
                // println!("{:?}", self.name);
                None
            }

            Name::Call => {
                // println!("{:?}", self.name);
                None
            }
            Name::CallCode => {
                // println!("{:?}", self.name);
                None
            }
            Name::DelegateCall => {
                // println!("{:?}", self.name);
                None
            }
            Name::StaticCall => {
                // println!("{:?}", self.name);
                None
            }

            Name::CodeSize => {
                // println!("{:?}", self.name);
                None
            }
            Name::CodeCopy => {
                // println!("{:?}", self.name);
                None
            }
            Name::ExtCodeSize => {
                // println!("{:?}", self.name);
                None
            }
            Name::ExtCodeCopy => {
                // println!("{:?}", self.name);
                None
            }
            Name::ReturnCodeSize => {
                // println!("{:?}", self.name);
                None
            }
            Name::ReturnCodeCopy => {
                // println!("{:?}", self.name);
                None
            }
            Name::ExtCodeHash => {
                // println!("{:?}", self.name);
                None
            }

            Name::Stop => {
                // println!("{:?}", self.name);
                None
            }
            Name::Return => {
                // println!("{:?}", self.name);
                None
            }
            Name::Revert => {
                self.arguments.remove(0);
                self.arguments.remove(0);
                None
            }
            Name::SelfDestruct => {
                // println!("{:?}", self.name);
                None
            }
            Name::Invalid => {
                // println!("{:?}", self.name);
                None
            }

            Name::UserDefined(name) => {
                let arguments: Vec<inkwell::values::BasicValueEnum> = self
                    .arguments
                    .into_iter()
                    .filter_map(|argument| argument.into_llvm(context))
                    .collect();
                let function = context
                    .module
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_void() {
        let input = r#"{
            function bar() {}

            function foo() -> x {
                x := 42
                bar()
            }
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_non_void() {
        let input = r#"{
            function bar() -> x {
                x:= 42
            }

            function foo() -> x {
                x := bar()
            }
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_with_arguments() {
        let input = r#"{
            function foo(z) -> x {
                let y := 3
                x := add(3, y)
            }
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_add() {
        let input = r#"{
            function foo() -> x {let y := 3 x := add(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_sub() {
        let input = r#"{
            function foo() -> x {let y := 3 x := sub(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_mul() {
        let input = r#"{
            function foo() -> x {let y := 3 x := mul(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_div() {
        let input = r#"{
            function foo() -> x {let y := 3 x := div(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_sdiv() {
        let input = r#"{
            function foo() -> x {let y := 3 x := sdiv(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_mod() {
        let input = r#"{
            function foo() -> x {let y := 3 x := mod(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_builtin_smod() {
        let input = r#"{
            function foo() -> x {let y := 3 x := smod(3, y)}
        }"#;

        assert!(crate::parse(input).is_ok());
    }
}
