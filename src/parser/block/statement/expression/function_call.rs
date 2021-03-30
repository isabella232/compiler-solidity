//!
//! The function call subexpression.
//!

use inkwell::values::BasicValue;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;
use crate::parser::block::statement::expression::Expression;

///
/// The function call subexpression.
///
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    /// The function name.
    pub name: String,
    /// The function arguments expression list.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let name = match initial.unwrap_or_else(|| lexer.next()) {
            Lexeme::Identifier(identifier) => identifier,
            lexeme => panic!("expected an identifier, found {}", lexeme),
        };

        let mut arguments = Vec::new();
        loop {
            let argument = match lexer.next() {
                Lexeme::Symbol(Symbol::ParenthesisRight) => break,
                lexeme => Expression::parse(lexer, Some(lexeme)),
            };

            arguments.push(argument);

            match lexer.peek() {
                Lexeme::Symbol(Symbol::Comma) => {
                    lexer.next();
                    continue;
                }
                Lexeme::Symbol(Symbol::ParenthesisRight) => {
                    lexer.next();
                    break;
                }
                _ => break,
            }
        }

        Self { name, arguments }
    }

    pub fn into_llvm<'ctx>(
        self,
        context: &Context<'ctx>,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        if let Some(value) = self.clone().builtin(context) {
            return Some(value);
        }

        let name = self.name.clone();
        let arguments: Vec<inkwell::values::BasicValueEnum> = self
            .arguments
            .into_iter()
            .filter_map(|argument| argument.into_llvm(context))
            .collect();
        let function = context
            .module
            .get_function(self.name.as_str())
            .unwrap_or_else(|| panic!("Undeclared function {}", name));
        let return_value = context
            .builder
            .build_call(function, &arguments, "")
            .try_as_basic_value();
        return_value.left()
    }

    fn builtin<'ctx>(
        mut self,
        context: &Context<'ctx>,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        // TODO: Figure out how to use high-order functions to reduce code duplication.
        match self.name.as_str() {
            "add" => {
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
            "sub" => {
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
            "mul" => {
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
            "div" => {
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
            "sdiv" => {
                let value = context.builder.build_int_signed_div(
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
            "mod" => {
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
            "smod" => {
                let value = context.builder.build_int_signed_rem(
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
            "lt" => {
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
                    context.integer_type(crate::BITLENGTH_DEFAULT),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "slt" => {
                let value = context.builder.build_int_compare(
                    inkwell::IntPredicate::SLT,
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
                    context.integer_type(crate::BITLENGTH_DEFAULT),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "gt" => {
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
                    context.integer_type(crate::BITLENGTH_DEFAULT),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "sgt" => {
                let value = context.builder.build_int_compare(
                    inkwell::IntPredicate::SGT,
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
                    context.integer_type(crate::BITLENGTH_DEFAULT),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "eq" => {
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
                    context.integer_type(crate::BITLENGTH_DEFAULT),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "and" => {
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
                Some(value.as_basic_value_enum())
            }
            "or" => {
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
                Some(value.as_basic_value_enum())
            }
            "xor" => {
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
                Some(value.as_basic_value_enum())
            }
            "shl" => {
                let value = context.builder.build_left_shift(
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
            "shr" => {
                let value = context.builder.build_right_shift(
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
                    false,
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "sar" => {
                let value = context.builder.build_right_shift(
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
                    true,
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            "iszero" => {
                let value = context.builder.build_right_shift(
                    self.arguments
                        .remove(0)
                        .into_llvm(context)
                        .expect("Always exists")
                        .into_int_value(),
                    context
                        .integer_type(crate::BITLENGTH_DEFAULT)
                        .const_int(0, false),
                    true,
                    "",
                );
                let value = context.builder.build_int_cast(
                    value,
                    context.integer_type(crate::BITLENGTH_DEFAULT),
                    "",
                );
                Some(value.as_basic_value_enum())
            }
            // TODO: implement once we support it
            "revert" => Some(
                context
                    .integer_type(crate::BITLENGTH_DEFAULT)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            "mstore" => Some(
                context
                    .integer_type(crate::BITLENGTH_DEFAULT)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            "mload" => Some(
                context
                    .integer_type(crate::BITLENGTH_DEFAULT)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            "selfdataload" => Some(
                context
                    .integer_type(crate::BITLENGTH_DEFAULT)
                    .const_int(0, false)
                    .as_basic_value_enum(),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_compile_void() {
        let input = r#"{
            function bar() {}

            function foo() -> x {
                x := 42
                bar()
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 42);
    }

    #[test]
    fn ok_compile_non_void() {
        let input = r#"{
            function bar() -> x {
                x:= 42
            }

            function foo() -> x {
                x := bar()
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 42);
    }

    #[test]
    fn ok_compile_with_arguments() {
        let input = r#"{
            function foo(z) -> x {
                let y := 3
                x := add(3, y)
            }
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    fn ok_compile_builtin_add() {
        let input = r#"{
            function foo() -> x {let y := 3 x := add(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 6);
    }

    #[test]
    fn ok_compile_builtin_sub() {
        let input = r#"{
            function foo() -> x {let y := 3 x := sub(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 0);
    }

    #[test]
    fn ok_compile_builtin_mul() {
        let input = r#"{
            function foo() -> x {let y := 3 x := mul(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 9);
    }

    #[test]
    fn ok_compile_builtin_div() {
        let input = r#"{
            function foo() -> x {let y := 3 x := div(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 1);
    }

    #[test]
    fn ok_compile_builtin_sdiv() {
        let input = r#"{
            function foo() -> x {let y := 3 x := sdiv(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 1);
    }

    #[test]
    fn ok_compile_builtin_mod() {
        let input = r#"{
            function foo() -> x {let y := 3 x := mod(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 0);
    }

    #[test]
    fn ok_compile_builtin_smod() {
        let input = r#"{
            function foo() -> x {let y := 3 x := smod(3, y)}
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 0);
    }
}
