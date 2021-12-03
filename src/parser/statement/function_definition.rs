//!
//! The function definition statement.
//!

use inkwell::types::BasicType;

use crate::error::Error;
use crate::generator::llvm::address_space::AddressSpace;
use crate::generator::llvm::function::r#return::Return as FunctionReturn;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::identifier::Identifier;
use crate::parser::statement::block::Block;

///
/// The function definition statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinition {
    /// The function name.
    pub name: String,
    /// The function formal arguments.
    pub arguments: Vec<Identifier>,
    /// The function return variables.
    pub result: Vec<Identifier>,
    /// The function body block.
    pub body: Block,
}

impl FunctionDefinition {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        let name = match lexeme {
            Lexeme::Identifier(name) => name,
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["{identifier}"], lexeme, None).into())
            }
        };

        match lexer.next()? {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["("], lexeme, None).into()),
        }

        let (arguments, next) = Identifier::parse_typed_list(lexer, None)?;

        match crate::parser::take_or_next(next, lexer)? {
            Lexeme::Symbol(Symbol::ParenthesisRight) => {}
            lexeme => return Err(ParserError::expected_one_of(vec![")"], lexeme, None).into()),
        }

        let (result, next) = match lexer.peek()? {
            Lexeme::Symbol(Symbol::Arrow) => {
                lexer.next()?;
                Identifier::parse_typed_list(lexer, None)?
            }
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => (vec![], None),
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["->", "{"], lexeme, None).into())
            }
        };

        let body = Block::parse(lexer, next)?;

        Ok(Self {
            name,
            arguments,
            result,
            body,
        })
    }

    ///
    /// Hoists a function to allow calls before translating the body.
    ///
    pub fn declare(&mut self, context: &mut LLVMContext) {
        let argument_types: Vec<_> = self
            .arguments
            .iter()
            .map(|argument| {
                let yul_type = argument.yul_type.to_owned().unwrap_or_default();
                yul_type.into_llvm(context).as_basic_type_enum()
            })
            .collect();

        let function_type = context.function_type(self.result.as_slice(), argument_types);

        context.add_function(
            self.name.as_str(),
            function_type,
            Some(inkwell::module::Linkage::Private),
            true,
        );

        if self.result.len() > 1 {
            let function = context.function().value;
            let pointer = function
                .get_first_param()
                .expect("Always exists")
                .into_pointer_value();
            context.set_function_return(FunctionReturn::compound(pointer, self.result.len()));
        }
    }
}

impl ILLVMWritable for FunctionDefinition {
    fn into_llvm(mut self, context: &mut LLVMContext) -> anyhow::Result<()> {
        let function = context
            .functions
            .get(self.name.as_str())
            .cloned()
            .expect("Function always exists");
        context.set_function(function.clone());

        context.set_basic_block(function.entry_block);
        let r#return = match function.r#return {
            Some(r#return) => {
                for (index, identifier) in self.result.into_iter().enumerate() {
                    let pointer = r#return.return_pointer().expect("Always exists");
                    let pointer = unsafe {
                        context.builder.build_gep(
                            pointer,
                            &[
                                context.field_const(0),
                                context
                                    .integer_type(compiler_common::BITLENGTH_X32)
                                    .const_int(index as u64, false),
                            ],
                            format!("return_{}_gep_pointer", index).as_str(),
                        )
                    };
                    let pointer = context.builder.build_pointer_cast(
                        pointer,
                        context.field_type().ptr_type(AddressSpace::Stack.into()),
                        format!("return_{}_gep_pointer_field", index).as_str(),
                    );
                    context
                        .function_mut()
                        .stack
                        .insert(identifier.name.clone(), pointer);
                }
                r#return
            }
            None => {
                if let Some(identifier) = self.result.pop() {
                    let r#type = identifier.yul_type.unwrap_or_default();
                    let pointer =
                        context.build_alloca(r#type.clone().into_llvm(context), "return_pointer");
                    context.build_store(pointer, r#type.into_llvm(context).const_zero());
                    context
                        .function_mut()
                        .stack
                        .insert(identifier.name, pointer);
                    FunctionReturn::primitive(pointer)
                } else {
                    FunctionReturn::none()
                }
            }
        };

        let argument_types: Vec<_> = self
            .arguments
            .iter()
            .map(|argument| {
                let yul_type = argument.yul_type.to_owned().unwrap_or_default();
                yul_type.into_llvm(context)
            })
            .collect();
        for (mut index, argument) in self.arguments.iter().enumerate() {
            let pointer = context.build_alloca(argument_types[index], argument.name.as_str());
            context
                .function_mut()
                .stack
                .insert(argument.name.clone(), pointer);
            if let Some(FunctionReturn::Compound { .. }) = context.function().r#return {
                index += 1;
            }
            context.build_store(
                pointer,
                context
                    .function()
                    .value
                    .get_nth_param(index as u32)
                    .expect("Always exists"),
            );
        }

        self.body.into_llvm_local(context)?;
        match context
            .basic_block()
            .get_last_instruction()
            .map(|instruction| instruction.get_opcode())
        {
            Some(inkwell::values::InstructionOpcode::Br) => {}
            Some(inkwell::values::InstructionOpcode::Switch) => {}
            _ => context.build_unconditional_branch(context.function().return_block),
        }

        match r#return {
            FunctionReturn::None => {
                context.build_throw_block(false);
                context.build_catch_block(false);

                context.set_basic_block(context.function().return_block);
                context.build_return(None);
            }
            FunctionReturn::Primitive { pointer } => {
                context.build_throw_block(false);
                context.build_catch_block(false);

                context.set_basic_block(context.function().return_block);
                let return_value = context.build_load(pointer, "return_value");
                context.build_return(Some(&return_value));
            }
            FunctionReturn::Compound {
                pointer: return_pointer,
                ..
            } => {
                context.build_throw_block(false);
                context.build_catch_block(false);

                context.set_basic_block(context.function().return_block);
                context.build_return(Some(&return_pointer));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_void() {
        let input = r#"object "Test" { code {
            function foo() {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_void_with_arguments() {
        let input = r#"object "Test" { code {
            function foo(a: A, b) {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_non_void_typed() {
        let input = r#"object "Test" { code {
            function foo(a: A, b) -> x: T, z: Y {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_i256() {
        let input = r#"object "Test" { code {
            function foo() -> x {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_multiple_i256() {
        let input = r#"object "Test" { code {
            function foo() -> x, y {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn error_invalid_name() {
        let input = r#"object "Test" { code {
            function 42() {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_err());
    }

    #[test]
    fn error_invalid_argument() {
        let input = r#"object "Test" { code {
            function foo(42) {}
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_err());
    }
}
