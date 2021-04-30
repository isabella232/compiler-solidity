//!
//! The source code block.
//!

use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::intrinsic::Intrinsic;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::assignment::Assignment;
use crate::parser::statement::expression::Expression;
use crate::parser::statement::Statement;
use crate::target::Target;

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

        let mut remaining = None;

        loop {
            match crate::parser::take_or_next(remaining.take(), lexer)? {
                lexeme @ Lexeme::Keyword(_) => {
                    let (statement, next) = Statement::parse(lexer, Some(lexeme))?;
                    remaining = next;
                    statements.push(statement);
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
    pub fn into_llvm_code(mut self, context: &mut LLVMContext) {
        let mut functions = Vec::with_capacity(self.statements.len());
        let mut local_statements = Vec::with_capacity(self.statements.len());

        for statement in self.statements.into_iter() {
            match statement {
                Statement::FunctionDefinition(mut statement) => {
                    statement.declare(context);
                    functions.push(statement);
                }
                statement => local_statements.push(statement),
            }
        }

        let name = context.object().to_owned();

        let return_type = match context.target {
            Target::LLVM => context.integer_type(compiler_const::bitlength::WORD),
            Target::zkEVM => context.integer_type(compiler_const::bitlength::FIELD),
        };
        let function_type = return_type.fn_type(&[], false);
        context.add_function(
            name.as_str(),
            function_type,
            Some(inkwell::module::Linkage::External),
        );
        let function = context.function().to_owned();
        context.set_basic_block(function.entry_block);

        let return_pointer = context.build_alloca(
            context.integer_type(compiler_const::bitlength::FIELD),
            "result",
        );
        let function = context.update_function(Some(return_pointer));

        self.statements = local_statements;
        self.into_llvm_local(context);
        context.build_unconditional_branch(function.return_block);

        context.set_basic_block(function.revert_block);
        let mut return_value = context.build_load(return_pointer, "");
        if let Target::LLVM = context.target {
            return_value = context
                .builder
                .build_int_truncate_or_bit_cast(return_value.into_int_value(), return_type, "")
                .as_basic_value_enum();
        }
        if let Target::zkEVM = context.target {
            let intrinsic = context.get_intrinsic_function(Intrinsic::Throw);
            context.builder.build_call(intrinsic, &[], "");
        }
        context.build_return(Some(&return_value));

        context.set_basic_block(function.return_block);
        let mut return_value = context.build_load(return_pointer, "");
        if let Target::LLVM = context.target {
            return_value = context
                .builder
                .build_int_truncate_or_bit_cast(return_value.into_int_value(), return_type, "")
                .as_basic_value_enum();
        }
        context.build_return(Some(&return_value));

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
                Statement::Continue => {
                    context.build_unconditional_branch(context.r#loop().continue_block);
                }
                Statement::Break => {
                    context.build_unconditional_branch(context.r#loop().join_block);
                }
                Statement::Leave => {
                    context.build_unconditional_branch(context.function().return_block);
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
