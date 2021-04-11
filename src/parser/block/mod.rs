//!
//! The source code block.
//!

pub mod statement;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;

use self::statement::assignment::Assignment;
use self::statement::expression::Expression;
use self::statement::Statement;

///
/// The source code block.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    /// The block statements.
    pub statements: Vec<Statement>,
}

impl Block {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) -> Result<Self, Error> {
        let mut statements = Vec::new();

        loop {
            let lexeme = crate::parser::take_or_next(initial.take(), lexer)?;

            match lexeme {
                Lexeme::Keyword(_) => statements.push(Statement::parse(lexer, Some(lexeme))?),
                Lexeme::Literal(_) => {
                    statements
                        .push(Expression::parse(lexer, Some(lexeme)).map(Statement::Expression)?);
                }
                Lexeme::Identifier(_) => match lexer.peek()? {
                    Lexeme::Symbol(Symbol::Assignment) => {
                        statements.push(
                            Assignment::parse(lexer, Some(lexeme)).map(Statement::Assignment)?,
                        );
                    }
                    Lexeme::Symbol(Symbol::Comma) => {
                        statements.push(
                            Assignment::parse(lexer, Some(lexeme)).map(Statement::Assignment)?,
                        );
                    }
                    _ => {
                        statements.push(
                            Expression::parse(lexer, Some(lexeme)).map(Statement::Expression)?,
                        );
                    }
                },
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    statements.push(Block::parse(lexer, None).map(Statement::Block)?)
                }
                Lexeme::Symbol(Symbol::BracketCurlyRight) => break,
                lexeme => {
                    return Err(ParserError::expected_one_of(
                        vec!["{keyword}", "{expression}", "{identifier}", "{", "}"],
                        lexeme,
                        None,
                    )
                    .into())
                }
            }
        }

        Ok(Self { statements })
    }

    ///
    /// Translates a module block into LLVM.
    ///
    pub fn into_llvm_module(self, context: &mut LLVMContext) {
        context.create_module("main");

        for statement in self.statements.iter() {
            match statement {
                Statement::FunctionDefinition(statement) => {
                    statement.declare(context);
                }
                _ => panic!("Cannot appear in local blocks"),
            }
        }
        for statement in self.statements.into_iter() {
            match statement {
                Statement::FunctionDefinition(statement) => statement.into_llvm(context),
                _ => panic!("Cannot appear in local blocks"),
            }
        }
    }

    ///
    /// Translates a function or ordinar block into LLVM.
    ///
    pub fn into_llvm_local(self, context: &mut LLVMContext) {
        for statement in self.statements.into_iter() {
            match statement {
                // The scope can be cleaned up on exit, but let's LLVM do the job. We can also rely
                // on YUL renaming so we don't need to track scope.
                Statement::Block(block) => block.into_llvm_local(context),
                Statement::Expression(expression) => {
                    expression.into_llvm(context);
                }
                Statement::VariableDeclaration(statement) => statement.into_llvm(context),
                Statement::Assignment(statement) => statement.into_llvm(context),
                Statement::IfConditional(statement) => statement.into_llvm(context),
                Statement::Switch(statement) => statement.into_llvm(context),
                Statement::ForLoop(statement) => statement.into_llvm(context),
                Statement::Leave => {
                    context
                        .builder
                        .build_unconditional_branch(context.leave_block.expect("Always exists"));
                }
                Statement::Break => {
                    context
                        .builder
                        .build_unconditional_branch(context.break_block.expect("Always exists"));
                }
                Statement::Continue => {
                    context
                        .builder
                        .build_unconditional_branch(context.continue_block.expect("Always exists"));
                }
                Statement::FunctionDefinition(_) => panic!("Cannot appear in local blocks"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::statement::Statement;
    use crate::parser::block::Block;
    use crate::parser::Module;

    #[test]
    fn ok_nested() {
        let input = r#"{
            {}
        }"#;

        let expected = Ok(Module {
            block: Block {
                statements: vec![Statement::Block(Block { statements: vec![] })],
            },
        });

        let result = crate::parse(input);
        assert_eq!(expected, result);
    }

    #[test]
    fn error_expected_bracket_curly_right() {
        let input = r#"{
            {}{}{{
        }"#;

        assert!(crate::parse(input).is_err());
    }
}
