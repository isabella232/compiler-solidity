//!
//! The for-loop statement.
//!

use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::expression::Expression;

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
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        let initializer = Block::parse(lexer, Some(lexeme))?;

        let condition = Expression::parse(lexer, None)?;

        let finalizer = Block::parse(lexer, None)?;

        let body = Block::parse(lexer, None)?;

        Ok(Self {
            initializer,
            condition,
            finalizer,
            body,
        })
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for ForLoop
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.initializer.into_llvm(context)?;

        let condition_block = context.append_basic_block("for_condition");
        let body_block = context.append_basic_block("for_body");
        let increment_block = context.append_basic_block("for_increment");
        let join_block = context.append_basic_block("for_join");

        context.build_unconditional_branch(condition_block);
        context.set_basic_block(condition_block);
        let condition = self
            .condition
            .into_llvm(context)?
            .expect("Always exists")
            .to_llvm()
            .into_int_value();
        let condition = context.builder().build_int_z_extend_or_bit_cast(
            condition,
            context.field_type(),
            "for_condition_extended",
        );
        let condition = context.builder().build_int_compare(
            inkwell::IntPredicate::NE,
            condition,
            context.field_const(0),
            "for_condition_compared",
        );
        context.build_conditional_branch(condition, body_block, join_block);

        context.push_loop(body_block, increment_block, join_block);

        context.set_basic_block(body_block);
        self.body.into_llvm(context)?;
        context.build_unconditional_branch(increment_block);

        context.set_basic_block(increment_block);
        self.finalizer.into_llvm(context)?;
        context.build_unconditional_branch(condition_block);

        context.pop_loop();
        context.set_basic_block(join_block);

        Ok(())
    }
}
