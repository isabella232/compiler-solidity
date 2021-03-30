//!
//! The variable declaration statement.
//!

use inkwell::types::BasicType;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Generator;
use crate::parser::block::statement::assignment::Assignment;
use crate::parser::block::statement::expression::identifier::Identifier;
use crate::parser::block::statement::expression::Expression;

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
        let (bindings, next) = Identifier::parse_list(lexer, None);

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

    pub fn into_llvm(mut self, context: &mut Generator) {
        let mut types = Vec::with_capacity(self.bindings.len());
        for identifier in self.bindings.iter() {
            let yul_type = identifier
                .yul_type
                .to_owned()
                .unwrap_or_default()
                .into_llvm(context);
            let pointer = context
                .builder
                .build_alloca(yul_type.as_basic_type_enum(), identifier.name.as_str());
            types.push(yul_type.as_basic_type_enum());
            context.variables.insert(identifier.name.clone(), pointer);
        }

        let expression = match self.expression.take() {
            Some(expression) => expression,
            None => return,
        };

        let assignment = Assignment {
            bindings: self.bindings,
            initializer: expression,
        };
        assignment.into_llvm(context);
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
}
