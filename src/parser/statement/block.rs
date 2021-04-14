//!
//! The source code block.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::assignment::Assignment;
use crate::parser::statement::expression::Expression;
use crate::parser::statement::Statement;

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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let mut statements = Vec::new();

        match lexeme {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }

        loop {
            match lexer.next()? {
                lexeme @ Lexeme::Keyword(_) => {
                    statements.push(Statement::parse(lexer, Some(lexeme))?)
                }
                lexeme @ Lexeme::Literal(_) => {
                    statements
                        .push(Expression::parse(lexer, Some(lexeme)).map(Statement::Expression)?);
                }
                lexeme @ Lexeme::Identifier(_) => match lexer.peek()? {
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
                lexeme @ Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    statements.push(Block::parse(lexer, Some(lexeme)).map(Statement::Block)?)
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
                Statement::FunctionDefinition(mut statement) => {
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
    use crate::parser::statement::block::Block;
    use crate::parser::statement::Statement;

    #[test]
    fn ok_nested() {
        let input = r#"{
            {}
        }"#;

        let expected = Ok(Block {
            statements: vec![Statement::Block(Block { statements: vec![] })],
        });

        let mut lexer = crate::lexer::Lexer::new(input.to_owned());
        let result = super::Block::parse(&mut lexer, None);
        assert_eq!(expected, result);
    }

    #[test]
    fn error_expected_bracket_curly_right() {
        let input = r#"{
            {}{}{{
        }"#;

        let mut lexer = crate::lexer::Lexer::new(input.to_owned());
        assert!(super::Block::parse(&mut lexer, None).is_err());
    }
}
