//!
//! The for-loop statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;

///
/// The for-loop statement.
///
#[derive(Debug, PartialEq)]
pub struct ForLoop {
    /// The index variables initialization block.
    pub initializer: Block,
    /// The continue condition block.
    pub condition: Expression,
    /// The index variables mutating block.
    pub finalizer: Block,
    /// The loop body.
    pub body: Block,
}

impl ForLoop {
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) -> Self {
        match lexer.next() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let initializer = Block::parse(lexer, None);

        let condition = Expression::parse(lexer, None);

        match lexer.next() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let finalizer = Block::parse(lexer, None);

        match lexer.next() {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => panic!("expected `{{`, found {}", lexeme),
        }

        let body = Block::parse(lexer, None);

        Self {
            initializer,
            condition,
            finalizer,
            body,
        }
    }

    pub fn into_llvm(self, context: &mut Context) {
        self.initializer.into_llvm_local(context);
        let condition_block = context
            .llvm
            .append_basic_block(context.function.unwrap(), "for.cond");
        let body = context
            .llvm
            .append_basic_block(context.function.unwrap(), "for.body");
        let increment_block = context
            .llvm
            .append_basic_block(context.function.unwrap(), "for.inc");
        let exit = context
            .llvm
            .append_basic_block(context.function.unwrap(), "for.exit");
        context.builder.build_unconditional_branch(condition_block);
        context.builder.position_at_end(condition_block);
        let condition = context.builder.build_int_cast(
            self.condition
                .into_llvm(context)
                .expect("Always exists")
                .into_int_value(),
            context.llvm.bool_type(),
            "",
        );
        context
            .builder
            .build_conditional_branch(condition, body, exit);
        context.break_bb = Some(exit);
        context.continue_bb = Some(increment_block);
        context.builder.position_at_end(body);
        self.body.into_llvm_local(context);
        context.builder.build_unconditional_branch(increment_block);
        context.builder.position_at_end(increment_block);
        self.finalizer.into_llvm_local(context);
        context.builder.build_unconditional_branch(condition_block);
        context.break_bb = None;
        context.continue_bb = None;
        context.builder.position_at_end(exit);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_parse() {
        let input = r#"{
            for {} expr {} {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile() {
        let input = r#"{
            function foo() -> x {
                x := 0
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    x := add(i, x)
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 45);
    }
}
