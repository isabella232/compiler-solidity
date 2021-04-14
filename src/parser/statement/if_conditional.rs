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
    fn into_llvm(self, context: &mut LLVMContext) {
        let condition = context.builder.build_int_cast(
            self.condition
                .into_llvm(context)
                .expect("Always exists")
                .into_int_value(),
            context.llvm.bool_type(),
            "",
        );
        let main_block = context
            .llvm
            .append_basic_block(context.function(), "if.main");
        let join_block = context
            .llvm
            .append_basic_block(context.function(), "if.join");
        context
            .builder
            .build_conditional_branch(condition, main_block, join_block);
        context.builder.position_at_end(main_block);
        self.block.into_llvm_local(context);
        context.build_unconditional_branch(join_block);
        context.builder.position_at_end(join_block);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_empty() {
        let input = r#"object "Test" { code {
            if expr {}
        }}"#;

        assert!(crate::parse(input).is_ok());
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

        assert!(crate::parse(input).is_ok());
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

        assert!(crate::parse(input).is_ok());
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

        assert!(crate::parse(input).is_ok());
    }
}
