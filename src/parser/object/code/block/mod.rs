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
    /// Translates an object block into LLVM.
    ///
    pub fn into_llvm_object(self, context: &mut LLVMContext) {
        let mut functions = Vec::with_capacity(self.statements.len());
        for statement in self.statements.into_iter() {
            match statement {
                Statement::Object(object) => object.into_llvm(context),
                Statement::Code(code) => code.into_llvm(context),
                Statement::FunctionDefinition(statement) => {
                    statement.declare(context);
                    functions.push(statement);
                }
                _ => {}
            }
        }

        for function in functions.into_iter() {
            function.into_llvm(context);
        }
    }

    ///
    /// Translates a function or ordinar block into LLVM.
    ///
    pub fn into_llvm_local(self, context: &mut LLVMContext) {
        for statement in self.statements.into_iter() {
            match statement {
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
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::object::code::block::statement::Statement;
    use crate::parser::object::code::block::Block;
    use crate::parser::object::code::Code;

    #[test]
    fn ok_nested() {
        let input = r#"{
            {}
        }"#;

        let expected = Ok(Code {
            block: Block {
                statements: vec![Statement::Block(Block { statements: vec![] })],
            },
        }
        .into_test_object());

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
