//!
//! The assignment expression statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;
use crate::parser::block::statement::expression::Expression;
use crate::parser::identifier::Identifier;

///
/// The assignment expression statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    /// The variable bindings.
    pub bindings: Vec<String>,
    /// The initializing expression.
    pub initializer: Expression,
}

impl Assignment {
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let lexeme = match initial {
            Some(lexeme) => lexeme,
            None => lexer.next(),
        };

        match lexer.peek() {
            Lexeme::Symbol(Symbol::Assignment) => {
                lexer.next();

                let identifier = match lexeme {
                    Lexeme::Identifier(identifier) => identifier,
                    lexeme => panic!("expected identifier, got {}", lexeme),
                };

                Self {
                    bindings: vec![identifier],
                    initializer: Expression::parse(lexer, None),
                }
            }
            Lexeme::Symbol(Symbol::Comma) => {
                let (identifiers, next) = Identifier::parse_list(lexer, Some(lexeme));

                match next.unwrap_or_else(|| lexer.next()) {
                    Lexeme::Symbol(Symbol::Assignment) => {}
                    lexeme => panic!("expected ':=', got {}", lexeme),
                }

                Self {
                    bindings: identifiers,
                    initializer: Expression::parse(lexer, None),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn into_llvm(mut self, context: &Context) {
        let value = match self.initializer.into_llvm(context) {
            Some(value) => value,
            None => return,
        };

        if self.bindings.len() == 1 {
            let name = self.bindings.remove(0);
            context
                .builder
                .build_store(context.variables[name.as_str()], value);
            return;
        }

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

            let value = context.builder.build_load(pointer, binding.as_str());

            context
                .builder
                .build_store(context.variables[binding.as_str()], value);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_parse_single() {
        let input = r#"{
            x := foo(x)
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_multiple() {
        let input = r#"{
            x, y := foo(x)
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile_single() {
        let input = r#"{
            function foo() -> x {
                let y := x
            }
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    fn ok_compile_multiple() {
        let input = r#"{
            function bar() -> x, y {
                x := 25
                y := 42
            }

            function foo() {
                let x := 1
                let y := 2
                x, y := bar()
            }
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    #[should_panic]
    fn error_parse_expected_expression() {
        let input = r#"{
            x :=
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    #[should_panic]
    fn error_parse_expected_symbol_assignment() {
        let input = r#"{
            x, y
        }"#;

        crate::tests::parse(input);
    }
}
