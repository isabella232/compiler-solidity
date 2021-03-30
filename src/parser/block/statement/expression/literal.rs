//!
//! The YUL source code literal.
//!

use inkwell::values::BasicValue;

use crate::llvm::Generator;

///
/// Represents a literal in YUL without differentiating its type.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    /// The stringified value.
    pub value: String,
}

impl Literal {
    pub fn into_llvm<'ctx, 'a>(
        self,
        context: &'ctx Generator<'ctx, 'a>,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let decimal = regex::Regex::new("^[0-9]+$").unwrap();
        let hex = regex::Regex::new("^0x[0-9a-fA-F]+$").unwrap();
        let i256_ty = context.llvm.custom_width_int_type(256);

        if decimal.is_match(self.value.as_str()) {
            i256_ty
                .const_int_from_string(self.value.as_str(), inkwell::types::StringRadix::Decimal)
                .unwrap()
                .as_basic_value_enum()
        } else if hex.is_match(self.value.as_str()) {
            i256_ty
                .const_int_from_string(&self.value[2..], inkwell::types::StringRadix::Hexadecimal)
                .unwrap()
                .as_basic_value_enum()
        } else {
            unreachable!();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::statement::expression::literal::Literal;
    use crate::parser::block::statement::expression::Expression;
    use crate::parser::block::statement::Statement;
    use crate::parser::block::Block;

    #[test]
    fn ok_false() {
        let input = r#"{
            false
        }"#;

        let result = crate::tests::parse(input);
        assert_eq!(
            result,
            [Statement::Block(Block {
                statements: vec![Statement::Expression(Expression::Literal(Literal {
                    value: "false".to_string()
                }))]
            })]
        );
    }

    #[test]
    fn ok_true() {
        let input = r#"{
            true
        }"#;

        let result = crate::tests::parse(input);
        assert_eq!(
            result,
            [Statement::Block(Block {
                statements: vec![Statement::Expression(Expression::Literal(Literal {
                    value: "true".to_string()
                }))]
            })]
        );
    }
}
