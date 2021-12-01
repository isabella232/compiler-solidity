//!
//! The YUL source code literal.
//!

use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::argument::Argument;
use crate::generator::llvm::Context as LLVMContext;
use crate::lexer::lexeme::literal::boolean::Boolean as BooleanLiteral;
use crate::lexer::lexeme::literal::integer::Integer as IntegerLiteral;
use crate::lexer::lexeme::literal::Literal as LexicalLiteral;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::r#type::Type;

///
/// Represents a literal in YUL without differentiating its type.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    /// The lexical literal.
    pub inner: LexicalLiteral,
    /// The type, if it has been explicitly specified.
    pub yul_type: Option<Type>,
}

impl Literal {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let literal = match lexeme {
            Lexeme::Literal(literal) => literal,
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["{literal}"], lexeme, None).into())
            }
        };

        let yul_type = match lexer.peek()? {
            Lexeme::Symbol(Symbol::Colon) => {
                lexer.next()?;
                Some(Type::parse(lexer, None)?)
            }
            _ => None,
        };

        Ok(Self {
            inner: literal,
            yul_type,
        })
    }

    ///
    /// Converts the literal into its LLVM representation.
    ///
    pub fn into_llvm<'ctx, 'src>(self, context: &LLVMContext<'ctx, 'src>) -> Argument<'ctx> {
        match self.inner {
            LexicalLiteral::Boolean(inner) => {
                let value = self
                    .yul_type
                    .unwrap_or_default()
                    .into_llvm(context)
                    .const_int(
                        match inner {
                            BooleanLiteral::False => 0,
                            BooleanLiteral::True => 1,
                        },
                        false,
                    )
                    .as_basic_value_enum();
                Argument::new(value)
            }
            LexicalLiteral::Integer(inner) => {
                let r#type = self.yul_type.unwrap_or_default().into_llvm(context);
                let value = match inner {
                    IntegerLiteral::Decimal { inner } => r#type.const_int_from_string(
                        inner.as_str(),
                        inkwell::types::StringRadix::Decimal,
                    ),
                    IntegerLiteral::Hexadecimal { inner } => r#type.const_int_from_string(
                        &inner["0x".len()..],
                        inkwell::types::StringRadix::Hexadecimal,
                    ),
                }
                .expect("The value is valid")
                .as_basic_value_enum();
                Argument::new(value)
            }
            LexicalLiteral::String(inner) => {
                let string = inner.to_string();
                let r#type = self.yul_type.unwrap_or_default().into_llvm(context);

                let mut hex_string = String::with_capacity(compiler_common::SIZE_FIELD * 2);
                let mut index = 0;
                loop {
                    if index >= string.len() {
                        break;
                    }

                    if string[index..].starts_with("\\x") {
                        hex_string.push_str(&string[index + 2..index + 4]);
                        index += 4;
                    } else if string[index..].starts_with("\\t") {
                        hex_string.push_str("09");
                        index += 2;
                    } else if string[index..].starts_with("\\n") {
                        hex_string.push_str("0a");
                        index += 2;
                    } else if string[index..].starts_with("\\r") {
                        hex_string.push_str("0d");
                        index += 2;
                    } else {
                        hex_string.push_str(format!("{:02x}", string.as_bytes()[index]).as_str());
                        index += 1;
                    }
                }

                if hex_string.len() > compiler_common::SIZE_FIELD * 2 {
                    return Argument::new_with_original(
                        r#type.const_zero().as_basic_value_enum(),
                        string,
                    );
                }
                if string.len() < compiler_common::SIZE_FIELD {
                    hex_string.push_str(
                        "00".repeat(compiler_common::SIZE_FIELD - string.len())
                            .as_str(),
                    );
                }

                let value = r#type
                    .const_int_from_string(
                        hex_string.as_str(),
                        inkwell::types::StringRadix::Hexadecimal,
                    )
                    .expect("The value is valid")
                    .as_basic_value_enum();
                Argument::new_with_original(value, string)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_false() {
        let input = r#"object "Test" { code {
            false
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_true() {
        let input = r#"object "Test" { code {
            true
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_parse() {
        let input = r#"object "Test" { code {
            function foo() {
                let x := 42
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_with_type() {
        let input = r#"object "Test" { code {
            function foo() {
                let x := 42:uint64
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }
}
