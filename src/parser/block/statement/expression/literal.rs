//!
//! The YUL source code literal.
//!

use inkwell::values::BasicValue;

use crate::generator::llvm::Context as LLVMContext;
use crate::lexer::lexeme::literal::boolean::Boolean as BooleanLiteral;
use crate::lexer::lexeme::literal::integer::Integer as IntegerLiteral;
use crate::lexer::lexeme::literal::Literal as LexicalLiteral;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let lexeme = initial.unwrap_or_else(|| lexer.next());

        let literal = match lexeme {
            Lexeme::Literal(literal) => literal,
            lexeme => panic!("Expected literal, got {}", lexeme),
        };

        let yul_type = match lexer.peek() {
            Lexeme::Symbol(Symbol::Colon) => {
                lexer.next();
                lexer.next();
                None
            }
            _ => None,
        };

        Self {
            inner: literal,
            yul_type,
        }
    }

    ///
    /// Converts the literal into its LLVM representation.
    ///
    pub fn into_llvm<'ctx>(
        self,
        context: &LLVMContext<'ctx>,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        match self.inner {
            LexicalLiteral::Boolean(inner) => self
                .yul_type
                .unwrap_or(Type::Bool)
                .into_llvm(context)
                .const_int(
                    match inner {
                        BooleanLiteral::False => 0,
                        BooleanLiteral::True => 1,
                    },
                    false,
                )
                .as_basic_value_enum(),
            LexicalLiteral::Integer(inner) => {
                let r#type = self.yul_type.unwrap_or_default().into_llvm(context);
                match inner {
                    IntegerLiteral::Decimal { inner } => r#type.const_int_from_string(
                        inner.as_str(),
                        inkwell::types::StringRadix::Decimal,
                    ),
                    IntegerLiteral::Hexadecimal { inner } => r#type.const_int_from_string(
                        &inner[2..],
                        inkwell::types::StringRadix::Hexadecimal,
                    ),
                }
                .expect("The value is valid")
                .as_basic_value_enum()
            }
            LexicalLiteral::String(_inner) => context
                .integer_type(crate::BITLENGTH_DEFAULT)
                .const_int(0, false)
                .as_basic_value_enum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lexeme::literal::boolean::Boolean as LexicalBooleanLiteral;
    use crate::lexer::lexeme::literal::Literal as LexicalLiteral;
    use crate::parser::block::statement::expression::literal::Literal;
    use crate::parser::block::statement::expression::Expression;
    use crate::parser::block::statement::Statement;
    use crate::parser::block::Block;
    use crate::parser::Module;

    #[test]
    fn ok_parse_false() {
        let input = r#"{
            false
        }"#;

        let result = crate::parse(input);
        assert_eq!(
            result,
            Module {
                block: Block {
                    statements: vec![Statement::Expression(Expression::Literal(Literal {
                        inner: LexicalLiteral::Boolean(LexicalBooleanLiteral::False),
                        yul_type: None,
                    }))]
                }
            }
        );
    }

    #[test]
    fn ok_parse_true() {
        let input = r#"{
            true
        }"#;

        let result = crate::parse(input);
        assert_eq!(
            result,
            Module {
                block: Block {
                    statements: vec![Statement::Expression(Expression::Literal(Literal {
                        inner: LexicalLiteral::Boolean(LexicalBooleanLiteral::True),
                        yul_type: None,
                    }))]
                }
            }
        );
    }

    #[test]
    fn ok_compile() {
        let input = r#"{
            function foo() {
                let x := 42
            }
        }"#;

        crate::compile(input);
    }

    #[test]
    fn ok_compile_with_type() {
        let input = r#"{
            function foo() {
                let x := 42:uint64
            }
        }"#;

        crate::compile(input);
    }
}
