//!
//! The for-loop statement.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;
use crate::parser::error::Error as ParserError;

///
/// The for-loop statement.
///
#[derive(Debug, PartialEq, Clone)]
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
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }

        let initializer = Block::parse(lexer, None)?;

        let condition = Expression::parse(lexer, None)?;

        match lexer.next()? {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }

        let finalizer = Block::parse(lexer, None)?;

        match lexer.next()? {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }

        let body = Block::parse(lexer, None)?;

        Ok(Self {
            initializer,
            condition,
            finalizer,
            body,
        })
    }
}

impl ILLVMWritable for ForLoop {
    fn into_llvm(self, context: &mut LLVMContext) {
        self.initializer.into_llvm_local(context);
        let condition_block = context
            .llvm
            .append_basic_block(context.function(), "for.cond");
        let body = context
            .llvm
            .append_basic_block(context.function(), "for.body");
        let increment_block = context
            .llvm
            .append_basic_block(context.function(), "for.inc");
        let exit = context
            .llvm
            .append_basic_block(context.function(), "for.exit");
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
        context.break_block = Some(exit);
        context.continue_block = Some(increment_block);
        context.builder.position_at_end(body);
        self.body.into_llvm_local(context);
        context.builder.build_unconditional_branch(increment_block);
        context.builder.position_at_end(increment_block);
        self.finalizer.into_llvm_local(context);
        context.builder.build_unconditional_branch(condition_block);
        context.break_block = None;
        context.continue_block = None;
        context.builder.position_at_end(exit);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_empty() {
        let input = r#"{
            for {} expr {} {}
        }"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_complex() {
        let input = r#"{
            function foo() -> x {
                x := 0
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    x := add(i, x)
                }
            }
        }"#;

        assert!(crate::parse(input).is_ok());
    }
}
