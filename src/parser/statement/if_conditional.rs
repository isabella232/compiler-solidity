//!
//! The if-conditional statement.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::statement::block::Block;
use crate::parser::statement::expression::Expression;

///
/// The if-conditional statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct IfConditional {
    /// The condition expression.
    pub condition: Expression,
    /// The conditional block.
    pub block: Block,
}

impl IfConditional {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let condition = Expression::parse(lexer, Some(lexeme))?;

        let block = Block::parse(lexer, None)?;

        Ok(Self { condition, block })
    }
}

impl ILLVMWritable for IfConditional {
    fn into_llvm(self, context: &mut LLVMContext) -> anyhow::Result<()> {
        let condition = self
            .condition
            .into_llvm(context)?
            .expect("Always exists")
            .to_llvm()
            .into_int_value();
        let condition = context.builder.build_int_z_extend_or_bit_cast(
            condition,
            context.field_type(),
            "if_condition_extended",
        );
        let condition = context.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            condition,
            context.field_const(0),
            "if_condition_compared",
        );
        let main_block = context.append_basic_block("if_main");
        let join_block = context.append_basic_block("if_join");
        context.build_conditional_branch(condition, main_block, join_block);
        context.set_basic_block(main_block);
        self.block.into_llvm_local(context)?;
        context.build_unconditional_branch(join_block);
        context.set_basic_block(join_block);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_empty() {
        let input = r#"object "Test" { code {
            if expr {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_lesser_than() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                x := 42
                let y := 1
                if lt(x, y) {
                    x := add(y, 1)
                }
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_equals() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                x := 42
                let y := 1
                if eq(x, y) {
                    x := add(y, 1)
                }
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_greater_than() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                x := 42
                let y := 1
                if gt(x, y) {
                    x := add(y, 1)
                }
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }
}
