//!
//! The if-conditional statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;

///
/// The if-conditional statement.
///
#[derive(Debug, PartialEq)]
pub struct IfConditional {
    pub condition: Expression,
    pub block: Block,
}

impl IfConditional {
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) -> Self {
        let condition = Expression::parse(lexer, None);

        match lexer.next() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let block = Block::parse(lexer, None);

        Self { condition, block }
    }

    pub fn into_llvm(self, context: &mut Context) {
        let condition = context.builder.build_int_cast(
            self.condition.into_llvm(context).into_int_value(),
            context.llvm.bool_type(),
            "",
        );
        let main_block = context
            .llvm
            .append_basic_block(context.function.unwrap(), "if.then");
        let join_block = context
            .llvm
            .append_basic_block(context.function.unwrap(), "if.join");
        context
            .builder
            .build_conditional_branch(condition, main_block, join_block);
        context.builder.position_at_end(main_block);
        self.block.into_llvm_local(context);
        context.builder.build_unconditional_branch(join_block);
        context.builder.position_at_end(join_block);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_parse() {
        let input = r#"{
            if expr {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile_lesser_than() {
        let input = r#"{
            function foo() -> x {
                x := 42
                let y := 1
                if lt(x, y) {
                    x := add(y, 1)
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 42);
    }

    #[test]
    fn ok_compile_equals() {
        let input = r#"{
            function foo() -> x {
                x := 42
                let y := 1
                if eq(x, y) {
                    x := add(y, 1)
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 42);
    }

    #[test]
    fn ok_compile_greater_than() {
        let input = r#"{
            function foo() -> x {
                x := 42
                let y := 1
                if gt(x, y) {
                    x := add(y, 1)
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 2);
    }
}
