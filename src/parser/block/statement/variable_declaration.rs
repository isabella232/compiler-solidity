//!
//! The variable declaration statement.
//!

use inkwell::types::BasicType;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;
use crate::parser::block::statement::expression::Expression;
use crate::parser::identifier::Identifier;

///
/// The variable declaration statement.
///
#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    /// The variable bindings list.
    pub bindings: Vec<Identifier>,
    /// The variable initializing expression.
    pub expression: Option<Expression>,
}

impl VariableDeclaration {
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) -> Self {
        let (bindings, next) = Identifier::parse_typed_list(lexer, None);

        match next.unwrap_or_else(|| lexer.next()) {
            Lexeme::Symbol(Symbol::Assignment) => {}
            lexeme => panic!("expected ':=', got {}", lexeme),
        }

        let expression = Expression::parse(lexer, None);

        Self {
            bindings,
            expression: Some(expression),
        }
    }

    pub fn into_llvm(mut self, context: &mut Context) {
        let expression = match self.expression.take() {
            Some(expression) => expression,
            None => return,
        };

        if self.bindings.len() == 1 {
            let value = expression.into_llvm(context);
            let identifier = self.bindings.remove(0);
            let pointer = context
                .builder
                .build_alloca(value.get_type(), identifier.name.as_str());
            context.variables.insert(identifier.name, pointer);
            context.builder.build_store(pointer, value);
            return;
        }

        let value = expression.into_llvm(context);
        let llvm_type = value.into_struct_value().get_type();
        let pointer = context.builder.build_alloca(llvm_type, "");
        context.builder.build_store(pointer, value);

        for (index, binding) in self.bindings.into_iter().enumerate() {
            let pointer = unsafe {
                context.builder.build_gep(
                    pointer,
                    &[
                        context.integer_type(64).const_zero(),
                        context.integer_type(32).const_int(index as u64, false),
                    ],
                    "",
                )
            };

            let value = context.builder.build_load(pointer, binding.name.as_str());

            let yul_type = binding
                .yul_type
                .to_owned()
                .unwrap_or_default()
                .into_llvm(context);
            let pointer = context
                .builder
                .build_alloca(yul_type.as_basic_type_enum(), binding.name.as_str());
            context.variables.insert(binding.name, pointer);
            context.builder.build_store(pointer, value);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_parse_boolean_false() {
        let input = r#"{
            let x := false
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_boolean_true() {
        let input = r#"{
            let x := true
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_integer_decimal() {
        let input = r#"{
            let x := 42
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_integer_hexadecimal() {
        let input = r#"{
            let x := 0x42
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_string() {
        let input = r#"{
            let x := "abc"
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_identifier() {
        let input = r#"{
            let x := y
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_function_call() {
        let input = r#"{
            let x := foo()
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_function_with_arguments() {
        let input = r#"{
            let x := foo(x, y)
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_function_with_arguments_nested() {
        let input = r#"{
            let x := foo(bar(x, baz()))
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile_literal_decimal() {
        let input = r#"{
            function foo() -> x {
                let y := 5
                x := y
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 5);
    }

    #[test]
    fn ok_compile_literal_decimal_subtraction() {
        let input = r#"{
            function foo() -> x {
                let y := 1234567890123456789012345678
                let z := 1234567890123456789012345679
                x := sub(z, y)
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 1);
    }

    #[test]
    fn ok_compile_literal_hexadecimal() {
        let input = r#"{
            function foo() -> x {
                let y := 0x2a
                x := y
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 42);
    }

    #[test]
    fn ok_compile_literal_hexadecimal_subtraction() {
        let input = r#"{
            function foo() -> x {
                let y := 0xffffffffffffffff
                let z := 0xfffffffffffffffe
                x := sub(y, z)
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 1);
    }

    #[test]
    fn ok_compile_multiple() {
        let input = r#"{
            function bar() -> x, y {
                x := 25
                y := 42
            }

            function foo() {
                let x, y := bar()
            }
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    fn ok_compile_type_cast() {
        let input = r#"{
            function foo() {
                let x: uint64 := 42
                x := 25
            }
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    fn ok_compile_type_inference() {
        let input = r#"{
            function foo() {
                let x := true
                x := false
            }
        }"#;

        crate::tests::compile(input, None);
    }
}
