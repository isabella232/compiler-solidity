//!
//! The source code block.
//!

use inkwell::values::BasicValue;

use crate::error::Error;
use crate::generator::llvm::function::r#return::Return as FunctionReturn;
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
    /// Translates the constructor code block into LLVM.
    ///
    pub fn into_llvm_constructor(mut self, context: &mut LLVMContext) {
        let mut functions = Vec::with_capacity(self.statements.len());
        let mut local_statements = Vec::with_capacity(self.statements.len());

        let function_type = context.function_type(&[], vec![]);
        context.add_function(
            compiler_common::identifier::FUNCTION_CONSTRUCTOR,
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
            .get(compiler_common::identifier::FUNCTION_CONSTRUCTOR)
            .cloned()
            .expect("Function always exists");
        context.set_function(compiler_common::identifier::FUNCTION_CONSTRUCTOR);
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
    pub fn into_llvm_selector(mut self, context: &mut LLVMContext) {
        let function = match context
            .functions
            .get(compiler_common::identifier::FUNCTION_SELECTOR)
            .cloned()
        {
            Some(function) => function,
            None => return,
        };

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

        context.set_function(compiler_common::identifier::FUNCTION_SELECTOR);
        context.set_basic_block(function.entry_block);
        let slots_to_zero = vec![
            compiler_common::abi::OFFSET_SOLIDITY_HASH_SLOT_FIRST,
            compiler_common::abi::OFFSET_SOLIDITY_HASH_SLOT_SECOND,
            compiler_common::abi::OFFSET_SOLIDITY_ZERO_SLOT,
        ];
        for slot_offset in slots_to_zero.into_iter() {
            let slot_pointer = context.access_heap(
                context.field_const((slot_offset * compiler_common::size::FIELD) as u64),
                None,
            );
            context.build_store(slot_pointer, context.field_const(0));
        }

        let return_pointer = context.build_alloca(context.field_type(), "result");
        let r#return = FunctionReturn::primitive(return_pointer);
        let function = context.update_function(r#return);

        self.statements = local_statements;
        self.constructor_call(context);
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
            Target::x86 => {
                let mut return_value = context.build_load(return_pointer, "return_value");
                return_value = context
                    .builder
                    .build_int_truncate_or_bit_cast(
                        return_value.into_int_value(),
                        context.integer_type(compiler_common::bitlength::WORD),
                        "return_value_truncated",
                    )
                    .as_basic_value_enum();
                context.build_return(Some(&return_value));
            }
            Target::zkEVM if context.test_entry_hash.is_some() => {
                let return_value = context.build_load(return_pointer, "return_value");
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
    fn constructor_call(&self, context: &mut LLVMContext) {
        let constructor = match context
            .functions
            .get(compiler_common::identifier::FUNCTION_CONSTRUCTOR)
            .cloned()
        {
            Some(constructor) => constructor,
            None => return,
        };

        let call_block = context.append_basic_block("constructor_call");
        let join_block = context.append_basic_block("constructor_join");

        let is_entry_hash_zero = Self::is_entry_hash_zero(context);
        let is_executed_flag_unset = Self::is_executed_flag_unset(context);
        let constructor_call_condition = context.builder.build_and(
            is_entry_hash_zero,
            is_executed_flag_unset,
            "constructor_call_condition",
        );
        context.build_conditional_branch(constructor_call_condition, call_block, join_block);

        context.set_basic_block(call_block);
        context.build_invoke(constructor.value, &[], "constructor_call");
        Self::set_is_executed_flag(context);
        context.build_unconditional_branch(context.function().return_block);

        context.set_basic_block(join_block);
    }

    ///
    /// Return the condition whether the entry hash is zero.
    ///
    fn is_entry_hash_zero<'ctx>(
        context: &mut LLVMContext<'ctx>,
    ) -> inkwell::values::IntValue<'ctx> {
        let hash_pointer = context.builder.build_int_to_ptr(
            context.field_const(
                (compiler_common::abi::OFFSET_ENTRY_HASH * compiler_common::size::FIELD) as u64,
            ),
            context
                .field_type()
                .ptr_type(compiler_common::AddressSpace::Parent.into()),
            "entry_hash_pointer",
        );
        let hash = context.build_load(hash_pointer, "entry_hash_value");
        context.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            hash.into_int_value(),
            context.field_const(compiler_common::abi::CONSTRUCTOR_ENTRY_HASH as u64),
            "entry_hash_is_zero_condition",
        )
    }

    ///
    /// Return the condition whether the executed flag is unset.
    ///
    fn is_executed_flag_unset<'ctx>(
        context: &mut LLVMContext<'ctx>,
    ) -> inkwell::values::IntValue<'ctx> {
        let storage_key_string = compiler_common::hashes::keccak256(
            compiler_common::abi::CONSTRUCTOR_EXECUTED_FLAG_KEY_PREIMAGE,
        );
        let storage_key_value = context
            .field_type()
            .const_int_from_string(
                storage_key_string.as_str(),
                inkwell::types::StringRadix::Hexadecimal,
            )
            .expect("Always valid");

        let intrinsic = context.get_intrinsic_function(Intrinsic::StorageLoad);
        let is_executed_flag = context
            .build_call(
                intrinsic,
                &[
                    storage_key_value.as_basic_value_enum(),
                    context.field_const(0).as_basic_value_enum(),
                ],
                "is_executed_flag_load",
            )
            .expect("Contract storage always returns a value");
        context.builder.build_int_compare(
            inkwell::IntPredicate::EQ,
            is_executed_flag.into_int_value(),
            context.field_const(0),
            "is_executed_flag_is_zero_condition",
        )
    }

    ///
    /// Sets the contract constructor executed flag.
    ///
    fn set_is_executed_flag(context: &mut LLVMContext) {
        let storage_key_string = compiler_common::hashes::keccak256(
            compiler_common::abi::CONSTRUCTOR_EXECUTED_FLAG_KEY_PREIMAGE,
        );
        let storage_key_value = context
            .field_type()
            .const_int_from_string(
                storage_key_string.as_str(),
                inkwell::types::StringRadix::Hexadecimal,
            )
            .expect("Always valid");

        let intrinsic = context.get_intrinsic_function(Intrinsic::StorageStore);
        context.build_call(
            intrinsic,
            &[
                context.field_const(1).as_basic_value_enum(),
                storage_key_value.as_basic_value_enum(),
                context.field_const(0).as_basic_value_enum(),
            ],
            "is_executed_flag_store",
        );
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
