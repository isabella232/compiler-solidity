//!
//! The assignment expression statement.
//!

use inkwell::types::BasicType;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Generator;
use crate::parser::block::statement::expression::identifier::Identifier;
use crate::parser::block::statement::expression::Expression;
use crate::parser::r#type::Type;

///
/// The assignment expression statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    /// The variable bindings.
    pub bindings: Vec<Identifier>,
    /// The initializing expression.
    pub initializer: Expression,
}

impl Assignment {
    pub fn parse(lexer: &mut Lexer, initial: Lexeme) -> Self {
        match lexer.peek() {
            Lexeme::Symbol(Symbol::Assignment) => {
                lexer.next();
                Self {
                    bindings: vec![Identifier {
                        name: initial.to_string(),
                        yul_type: None,
                    }],
                    initializer: Expression::parse(lexer, None),
                }
            }
            Lexeme::Symbol(Symbol::Comma) => {
                let (identifiers, next) = Identifier::parse_list(lexer, Some(initial));

                match next.unwrap_or_else(|| lexer.next()) {
                    Lexeme::Symbol(Symbol::Assignment) => {}
                    lexeme => panic!("expected ':=', got {}", lexeme),
                }

                Self {
                    bindings: identifiers,
                    initializer: Expression::parse(lexer, None),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn into_llvm(mut self, context: &Generator) {
        if self.bindings.len() == 1 {
            let value = self.initializer.into_llvm(context);
            let name = self.bindings.remove(0).name;
            context
                .builder
                .build_store(context.variables[name.as_str()], value);
            return;
        }

        let types =
            vec![Type::default().into_llvm(context).as_basic_type_enum(); self.bindings.len()];
        let llvm_type = context.llvm.struct_type(types.as_slice(), false);
        let pointer = context.builder.build_alloca(llvm_type, "");
        let value = self.initializer.into_llvm(context);
        context.builder.build_store(pointer, value);

        for (index, binding) in self.bindings.into_iter().enumerate() {
            let pointer = unsafe {
                context.builder.build_gep(
                    pointer,
                    &[
                        context.integer_type(64).const_zero(),
                        context.integer_type(32).const_int(index as u64, false),
                    ],
                    "",
                )
            };

            let value = context.builder.build_load(pointer, binding.name.as_str());

            context
                .builder
                .build_store(context.variables[binding.name.as_str()], value);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_parse_single() {
        let input = r#"{
            x := foo(x)
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_multiple() {
        let input = r#"{
            x, y := foo(x)
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile() {
        let input = r#"{
            function foo() -> x {
                let y := x
            }
        }"#;

        crate::tests::compile(input, None);
    }

    #[test]
    #[should_panic]
    fn error_parse_expected_expression() {
        let input = r#"{
            x :=
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    #[should_panic]
    fn error_parse_expected_symbol_assignment() {
        let input = r#"{
            x, y
        }"#;

        crate::tests::parse(input);
    }
}
