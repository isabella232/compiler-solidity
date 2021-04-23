//!
//! The variable declaration statement.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::identifier::Identifier;
use crate::parser::r#type::Type;
use crate::parser::statement::expression::Expression;

///
/// The variable declaration statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration {
    /// The variable bindings list.
    pub bindings: Vec<Identifier>,
    /// The variable initializing expression.
    pub expression: Option<Expression>,
}

impl VariableDeclaration {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Lexeme>,
    ) -> Result<(Self, Option<Lexeme>), Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let (bindings, next) = Identifier::parse_typed_list(lexer, Some(lexeme))?;

        match crate::parser::take_or_next(next, lexer)? {
            Lexeme::Symbol(Symbol::Assignment) => {}
            lexeme => {
                return Ok((
                    Self {
                        bindings,
                        expression: None,
                    },
                    Some(lexeme),
                ))
            }
        }

        let expression = Expression::parse(lexer, None)?;

        Ok((
            Self {
                bindings,
                expression: Some(expression),
            },
            None,
        ))
    }
}

impl ILLVMWritable for VariableDeclaration {
    fn into_llvm(mut self, context: &mut LLVMContext) {
        if self.bindings.len() == 1 {
            let identifier = self.bindings.remove(0);
            let r#type = identifier
                .yul_type
                .unwrap_or_else(Type::default)
                .into_llvm(context);
            let pointer = context
                .builder
                .build_alloca(r#type, identifier.name.as_str());
            context
                .function_mut()
                .stack
                .insert(identifier.name, pointer);
            let value = if let Some(expression) = self.expression {
                match expression.into_llvm(context) {
                    Some(value) => value,
                    None => r#type.const_zero().as_basic_value_enum(),
                }
            } else {
                r#type.const_zero().as_basic_value_enum()
            };
            context.builder.build_store(pointer, value);
            return;
        }

        let llvm_type = context.structure_type(
            self.bindings
                .iter()
                .map(|binding| {
                    binding
                        .yul_type
                        .to_owned()
                        .unwrap_or_default()
                        .into_llvm(context)
                        .as_basic_type_enum()
                })
                .collect(),
        );
        let pointer = context.builder.build_alloca(llvm_type, "");
        match self.expression.take() {
            Some(expression) => {
                if let Some(value) = expression.into_llvm(context) {
                    context.builder.build_store(pointer, value);

                    for (index, binding) in self.bindings.into_iter().enumerate() {
                        let pointer = unsafe {
                            context.builder.build_gep(
                                pointer,
                                &[
                                    context
                                        .integer_type(compiler_const::bitlength::WORD)
                                        .const_zero(),
                                    context
                                        .integer_type(compiler_const::bitlength::BYTE * 4)
                                        .const_int(index as u64, false),
                                ],
                                "",
                            )
                        };

                        let value = context.builder.build_load(pointer, binding.name.as_str());

                        let yul_type = binding.yul_type.unwrap_or_default().into_llvm(context);
                        let pointer = context
                            .builder
                            .build_alloca(yul_type.as_basic_type_enum(), binding.name.as_str());
                        context.function_mut().stack.insert(binding.name, pointer);
                        context.builder.build_store(pointer, value);
                    }
                }
            }
            None => {
                context.builder.build_store(pointer, llvm_type.const_zero());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_boolean_false() {
        let input = r#"object "Test" { code {
            let x := false
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_boolean_true() {
        let input = r#"object "Test" { code {
            let x := true
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_integer_decimal() {
        let input = r#"object "Test" { code {
            let x := 42
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_integer_hexadecimal() {
        let input = r#"object "Test" { code {
            let x := 0x42
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_string() {
        let input = r#"object "Test" { code {
            let x := "abc"
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_identifier() {
        let input = r#"object "Test" { code {
            let x := y
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_function_call() {
        let input = r#"object "Test" { code {
            let x := foo()
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_function_with_arguments() {
        let input = r#"object "Test" { code {
            let x := foo(x, y)
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_function_with_arguments_nested() {
        let input = r#"object "Test" { code {
            let x := foo(bar(x, baz()))
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_literal_decimal() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                let y := 5
                x := y
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_literal_decimal_subtraction() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                let y := 1234567890123456789012345678
                let z := 1234567890123456789012345679
                x := sub(z, y)
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_literal_hexadecimal() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                let y := 0x2a
                x := y
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_literal_hexadecimal_subtraction() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                let y := 0xffffffffffffffff
                let z := 0xfffffffffffffffe
                x := sub(y, z)
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_multiple() {
        let input = r#"object "Test" { code {
            function bar() -> x, y {
                x := 25
                y := 42
            }

            function foo() {
                let x, y := bar()
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_type_cast() {
        let input = r#"object "Test" { code {
            function foo() {
                let x: uint64 := 42
                x := 25
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_type_inference() {
        let input = r#"object "Test" { code {
            function foo() {
                let x := true
                x := false
            }
        }}"#;

        assert!(crate::parse(input).is_ok());
    }
}
