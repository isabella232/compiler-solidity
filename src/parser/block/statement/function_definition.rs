//!
//! The function definition statement.
//!

use crate::generator::llvm::Context;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::block::Block;
use crate::parser::identifier::Identifier;

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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Self {
        let lexeme = initial.unwrap_or_else(|| lexer.next());

        let name = match lexeme {
            Lexeme::Identifier(name) => name,
            lexeme => panic!("Expected function identifier, got {}", lexeme),
        };

        match lexer.next() {
            Lexeme::Symbol(Symbol::ParenthesisLeft) => {}
            lexeme => panic!(
                "Expected '(' in {} function definition, got {}",
                name, lexeme
            ),
        }

        let (arguments, next) = Identifier::parse_typed_list(lexer, None);

        match next.unwrap_or_else(|| lexer.next()) {
            Lexeme::Symbol(Symbol::ParenthesisRight) => {}
            lexeme => panic!(
                "Expected ')' in {} function definition, got {}",
                name, lexeme
            ),
        }

        let (result, _next) = match lexer.peek() {
            Lexeme::Symbol(Symbol::Arrow) => {
                lexer.next();
                Identifier::parse_typed_list(lexer, None)
            }
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                lexer.next();
                (vec![], None)
            }
            lexeme => panic!("Expected -> or {{, got {}", lexeme),
        };

        let body = Block::parse(lexer, None);

        Self {
            name,
            arguments,
            result,
            body,
        }
    }

    ///
    /// Hoists a function to allow calls before translating the body.
    ///
    pub fn declare(&self, context: &mut Context) {
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

    pub fn into_llvm(self, context: &mut Context) {
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
    fn ok_parse_void() {
        let input = r#"{
            function foo(a: A, b) {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_non_void_typed() {
        let input = r#"{
            function foo(a: A, b) -> x: T, z: Y {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile_void() {
        let input = r#"{
            function foo() {}
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    fn ok_compile_i256() {
        let input = r#"{
            function foo() -> x {}
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    fn ok_compile_multiple_i256() {
        let input = r#"{
            function foo() -> x, y {}
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    #[should_panic]
    fn error_parse_invalid_name() {
        let input = r#"{
            function 42() {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    #[should_panic]
    fn error_parse_invalid_argument() {
        let input = r#"{
            function foo(42) {}
        }"#;

        crate::tests::parse(input);
    }
}
