//!
//! The assignment expression statement.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::identifier::Identifier;
use crate::parser::statement::expression::Expression;

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
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let identifier = match lexeme {
            Lexeme::Identifier(identifier) => identifier,
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["{identifier}"], lexeme, None).into())
            }
        };

        match lexer.peek()? {
            Lexeme::Symbol(Symbol::Assignment) => {
                lexer.next()?;

                Ok(Self {
                    bindings: vec![identifier],
                    initializer: Expression::parse(lexer, None)?,
                })
            }
            Lexeme::Symbol(Symbol::Comma) => {
                let (identifiers, next) =
                    Identifier::parse_list(lexer, Some(Lexeme::Identifier(identifier)))?;

                match crate::parser::take_or_next(next, lexer)? {
                    Lexeme::Symbol(Symbol::Assignment) => {}
                    lexeme => {
                        return Err(ParserError::expected_one_of(vec![":="], lexeme, None).into())
                    }
                }

                Ok(Self {
                    bindings: identifiers,
                    initializer: Expression::parse(lexer, None)?,
                })
            }
            lexeme => Err(ParserError::expected_one_of(vec![":=", ","], lexeme, None).into()),
        }
    }
}

impl ILLVMWritable for Assignment {
    fn into_llvm(mut self, context: &mut LLVMContext) -> anyhow::Result<()> {
        let value = match self.initializer.into_llvm(context)? {
            Some(value) => value,
            None => return Ok(()),
        };

        if self.bindings.len() == 1 {
            let name = self.bindings.remove(0);
            context.build_store(context.function().stack[name.as_str()], value.to_llvm());
            return Ok(());
        }

        let llvm_type = value.to_llvm().into_struct_value().get_type();
        let pointer = context.build_alloca(llvm_type, "assignment_pointer");
        context.build_store(pointer, value.to_llvm());

        for (index, binding) in self.bindings.into_iter().enumerate() {
            let pointer = unsafe {
                context.builder.build_gep(
                    pointer,
                    &[
                        context.field_const(0),
                        context
                            .integer_type(compiler_common::BITLENGTH_X32)
                            .const_int(index as u64, false),
                    ],
                    format!("assignment_binding_{}_gep_pointer", index).as_str(),
                )
            };

            let value = context.build_load(
                pointer,
                format!("assignment_binding_{}_value", index).as_str(),
            );

            context.build_store(context.function().stack[binding.as_str()], value);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_single() {
        let input = r#"x := foo(x)"#;

        let mut lexer = crate::lexer::Lexer::new(input.to_owned());
        assert!(super::Assignment::parse(&mut lexer, None).is_ok());
    }

    #[test]
    fn ok_multiple() {
        let input = r#"x, y := foo(x)"#;

        let mut lexer = crate::lexer::Lexer::new(input.to_owned());
        assert!(super::Assignment::parse(&mut lexer, None).is_ok());
    }

    #[test]
    fn ok_multiple_return_values() {
        let input = r#"object "Test" { code {
            function bar() -> x, y {
                x := 25
                y := 42
            }

            function foo() {
                let x := 1
                let y := 2
                x, y := bar()
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn error_expected_expression() {
        let input = r#"x :="#;

        let mut lexer = crate::lexer::Lexer::new(input.to_owned());
        assert!(super::Assignment::parse(&mut lexer, None).is_err());
    }

    #[test]
    fn error_expected_symbol_assignment() {
        let input = r#"x, y"#;

        let mut lexer = crate::lexer::Lexer::new(input.to_owned());
        assert!(super::Assignment::parse(&mut lexer, None).is_err());
    }
}
