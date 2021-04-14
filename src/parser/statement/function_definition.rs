//!
//! The function definition statement.
//!

use crate::error::Error;
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
                inkwell::types::BasicTypeEnum::IntType(yul_type.into_llvm(context))
            })
            .collect();

        let function_type =
            context.function_type(self.result.as_slice(), argument_types.as_slice());

        context.create_function(self.name.as_str(), function_type);
    }
}

impl ILLVMWritable for FunctionDefinition {
    fn into_llvm(self, context: &mut LLVMContext) {
        let argument_types: Vec<_> = self
            .arguments
            .iter()
            .map(|argument| {
                let yul_type = argument.yul_type.to_owned().unwrap_or_default();
                yul_type.into_llvm(context)
            })
            .collect();

        let function = context
            .module
            .get_function(self.name.as_str())
            .expect("Function always exists");
        let return_types = function.get_type().get_return_type();
        context.function = Some(function);

        let entry = context.llvm.append_basic_block(function, "entry");
        context.builder.position_at_end(entry);

        for (index, argument) in self.arguments.iter().enumerate() {
            let pointer = context
                .builder
                .build_alloca(argument_types[index], argument.name.as_str());
            context.variables.insert(argument.name.clone(), pointer);
            context.builder.build_store(
                pointer,
                function.get_nth_param(index as u32).expect("Always exists"),
            );
        }

        let return_pointer = match return_types {
            Some(r#type) => Some(context.builder.build_alloca(r#type, "result")),
            None => None,
        };

        let mut return_values: Vec<_> = self
            .result
            .into_iter()
            .map(|identifier| {
                let r#type = identifier.yul_type.unwrap_or_default();
                let value = context
                    .builder
                    .build_alloca(r#type.into_llvm(context), identifier.name.as_str());
                context.variables.insert(identifier.name.clone(), value);
                value
            })
            .collect();

        let exit = context.llvm.append_basic_block(function, "exit");
        context.leave_block = Some(exit);

        self.body.into_llvm_local(context);

        let last_instruction = context
            .builder
            .get_insert_block()
            .expect("Always exists")
            .get_last_instruction();

        match last_instruction {
            None => {
                context.builder.build_unconditional_branch(exit);
            }
            Some(instruction) => match instruction.get_opcode() {
                inkwell::values::InstructionOpcode::Br => (),
                inkwell::values::InstructionOpcode::Switch => (),
                _ => {
                    context.builder.build_unconditional_branch(exit);
                }
            },
        };

        context.builder.position_at_end(exit);

        match return_pointer {
            Some(return_pointer) => {
                if return_values.len() == 1 {
                    let return_value = context.builder.build_load(return_values.remove(0), "");
                    context.builder.build_return(Some(&return_value))
                } else {
                    for (index, value) in return_values.iter().enumerate() {
                        context.builder.build_store(
                            context
                                .builder
                                .build_struct_gep(return_pointer, index as u32, "")
                                .expect("Always exists"),
                            context.builder.build_load(*value, ""),
                        );
                    }
                    let return_value = context.builder.build_load(return_pointer, "");
                    context.builder.build_return(Some(&return_value))
                }
            }
            None => context.builder.build_return(None),
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_void() {
        let input = r#"object "Test" { code {
            function foo() {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_void_with_arguments() {
        let input = r#"object "Test" { code {
            function foo(a: A, b) {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_non_void_typed() {
        let input = r#"object "Test" { code {
            function foo(a: A, b) -> x: T, z: Y {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_i256() {
        let input = r#"object "Test" { code {
            function foo() -> x {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_multiple_i256() {
        let input = r#"object "Test" { code {
            function foo() -> x, y {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn error_invalid_name() {
        let input = r#"object "Test" { code {
            function 42() {}
        }}"#;

        assert!(crate::parse(input).is_err());
    }

    #[test]
    fn error_invalid_argument() {
        let input = r#"object "Test" { code {
            function foo(42) {}
        }}"#;

        assert!(crate::parse(input).is_err());
    }
}
