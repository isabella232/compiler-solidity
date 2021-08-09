//!
//! The source code block.
//!

use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::function::r#return::Return as FunctionReturn;
use crate::generator::llvm::function::Function;
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
    /// Translates the constructor code block into LLVM.
    ///
    pub fn into_llvm_constructor(mut self, context: &mut LLVMContext) {
        let mut functions = Vec::with_capacity(self.statements.len());
        let mut local_statements = Vec::with_capacity(self.statements.len());

        let function_name = context.object().to_owned();
        let function_type = context.function_type(&[], vec![]);
        context.add_function(
            function_name.as_str(),
            function_type,
            Some(inkwell::module::Linkage::External),
            true,
        );

        for statement in self.statements.into_iter() {
            match statement {
                Statement::FunctionDefinition(mut statement) => {
                    statement.declare(context);
                    functions.push(statement);
                }
                statement => local_statements.push(statement),
            }
        }

        let function = context
            .functions
            .get(function_name.as_str())
            .cloned()
            .expect("Function always exists");
        context.set_function(function_name.as_str());
        context.set_basic_block(function.entry_block);
        context.update_function(FunctionReturn::none());

        self.statements = local_statements;
        self.into_llvm_local(context);
        match context.basic_block().get_last_instruction() {
            Some(instruction) => match instruction.get_opcode() {
                inkwell::values::InstructionOpcode::Br => {}
                inkwell::values::InstructionOpcode::Switch => {}
                _ => {
                    context.build_unconditional_branch(function.return_block);
                }
            },
            None => {
                context.build_unconditional_branch(function.return_block);
            }
        };

        context.set_basic_block(function.catch_block);
        context.build_catch_block();
        context.build_unreachable();

        context.set_basic_block(function.throw_block);
        context.build_throw_block();
        context.build_unreachable();

        context.set_basic_block(function.return_block);
        context.build_return(None);

        for function in functions.into_iter() {
            function.into_llvm(context);
        }
    }

    ///
    /// Translates the main deployed code block into LLVM.
    ///
    pub fn into_llvm_deployed(mut self, context: &mut LLVMContext) {
        let mut functions = Vec::with_capacity(self.statements.len());
        let mut local_statements = Vec::with_capacity(self.statements.len());

        let function_name = context.object().to_owned();
        let function_type = match context.target {
            Target::X86 => context
                .integer_type(compiler_common::bitlength::WORD)
                .fn_type(&[], false),
            Target::zkEVM if context.test_entry_hash.is_some() => context
                .integer_type(compiler_common::bitlength::FIELD)
                .fn_type(&[], false),
            Target::zkEVM => context.void_type().fn_type(&[], false),
        };
        context.add_function(
            function_name.as_str(),
            function_type,
            Some(inkwell::module::Linkage::External),
            false,
        );

        for statement in self.statements.into_iter() {
            match statement {
                Statement::FunctionDefinition(mut statement) => {
                    statement.declare(context);
                    functions.push(statement);
                }
                statement => local_statements.push(statement),
            }
        }

        let function = context
            .functions
            .get(function_name.as_str())
            .cloned()
            .expect("Always exists");
        context.set_function(function_name.as_str());
        context.set_basic_block(function.entry_block);
        let return_pointer = context.build_alloca(
            context.integer_type(compiler_common::bitlength::FIELD),
            "result",
        );
        let r#return = FunctionReturn::primitive(return_pointer);
        let function = context.update_function(r#return);

        self.statements = local_statements;
        if let Some(constructor) = context
            .functions
            .get(compiler_common::identifier::FUNCTION_CONSTRUCTOR)
            .cloned()
        {
            self.constructor_call(context, constructor);
        }
        self.into_llvm_local(context);
        context.build_unconditional_branch(function.return_block);

        context.set_basic_block(function.throw_block);
        context.build_throw_block();
        context.build_unreachable();

        context.set_basic_block(function.catch_block);
        context.build_catch_block();
        context.build_unreachable();

        context.set_basic_block(function.return_block);
        match context.target {
            Target::X86 => {
                let mut return_value = context.build_load(return_pointer, "");
                return_value = context
                    .builder
                    .build_int_truncate_or_bit_cast(
                        return_value.into_int_value(),
                        context.integer_type(compiler_common::bitlength::WORD),
                        "",
                    )
                    .as_basic_value_enum();
                context.build_return(Some(&return_value));
            }
            Target::zkEVM if context.test_entry_hash.is_some() => {
                let return_value = context.build_load(return_pointer, "");
                context.build_return(Some(&return_value));
            }
            Target::zkEVM => {
                context.build_return(None);
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

    ///
    /// Writes a conditional constructor call at the beginning of the selector.
    ///
    fn constructor_call<'ctx>(&self, context: &mut LLVMContext<'ctx>, constructor: Function<'ctx>) {
        let call_block = context.append_basic_block("call_block");
        let join_block = context.append_basic_block("join_block");

        let hash_pointer = context.builder.build_int_to_ptr(
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .const_int(
                    (compiler_common::contract::ABI_OFFSET_ENTRY_HASH
                        * compiler_common::size::FIELD) as u64,
                    false,
                ),
            context
                .integer_type(compiler_common::bitlength::FIELD)
                .ptr_type(compiler_common::AddressSpace::Parent.into()),
            "",
        );
        let hash = context.build_load(hash_pointer, "");
        let is_zero = context.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            hash.into_int_value(),
            context.field_const(0),
            "",
        );
        context.build_conditional_branch(is_zero, call_block, join_block);

        context.set_basic_block(call_block);
        context.build_invoke(constructor.value, &[], "");
        context.build_unconditional_branch(context.function().return_block);

        context.set_basic_block(join_block);
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
