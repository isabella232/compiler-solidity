//!
//! The variable declaration statement.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::yul::lexer::lexeme::symbol::Symbol;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::identifier::Identifier;
use crate::yul::parser::statement::expression::Expression;

///
/// The variable declaration statement.
///
#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclaration {
    /// The variable bindings list.
    pub bindings: Vec<Identifier>,
    /// The variable initializing expression.
    pub expression: Option<Expression>,
}

impl VariableDeclaration {
    ///
    /// The element parser.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Lexeme>,
    ) -> anyhow::Result<(Self, Option<Lexeme>)> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        let (bindings, next) = Identifier::parse_typed_list(lexer, Some(lexeme))?;

        match crate::yul::parser::take_or_next(next, lexer)? {
            Lexeme::Symbol(Symbol::Assignment) => {}
            lexeme => {
                return Ok((
                    Self {
                        bindings,
                        expression: None,
                    },
                    Some(lexeme),
                ))
            }
        }

        let expression = Expression::parse(lexer, None)?;

        Ok((
            Self {
                bindings,
                expression: Some(expression),
            },
            None,
        ))
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for VariableDeclaration
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        if self.bindings.len() == 1 {
            let identifier = self.bindings.remove(0);
            let r#type = identifier.yul_type.unwrap_or_default().into_llvm(context);
            let pointer = context.build_alloca(r#type, identifier.name.as_str());
            context
                .function_mut()
                .stack
                .insert(identifier.name, pointer);
            let value = if let Some(expression) = self.expression {
                match expression.into_llvm(context)? {
                    Some(value) => value.to_llvm(),
                    None => r#type.const_zero().as_basic_value_enum(),
                }
            } else {
                r#type.const_zero().as_basic_value_enum()
            };
            context.build_store(pointer, value);
            return Ok(());
        }

        let llvm_type = context.structure_type(
            self.bindings
                .iter()
                .map(|binding| {
                    binding
                        .yul_type
                        .to_owned()
                        .unwrap_or_default()
                        .into_llvm(context)
                        .as_basic_type_enum()
                })
                .collect(),
        );
        let pointer = context.build_alloca(llvm_type, "bindings_pointer");
        for (index, binding) in self.bindings.iter().enumerate() {
            let yul_type = binding
                .yul_type
                .to_owned()
                .unwrap_or_default()
                .into_llvm(context);
            let pointer = context.build_alloca(
                yul_type.as_basic_type_enum(),
                format!("binding_{}_pointer", index).as_str(),
            );
            context
                .function_mut()
                .stack
                .insert(binding.name.to_owned(), pointer);
        }

        match self.expression.take() {
            Some(expression) => {
                if let Some(value) = expression.into_llvm(context)? {
                    context.build_store(pointer, value.to_llvm());

                    for (index, binding) in self.bindings.into_iter().enumerate() {
                        let pointer = unsafe {
                            context.builder().build_gep(
                                pointer,
                                &[
                                    context.field_const(0),
                                    context
                                        .integer_type(compiler_common::BITLENGTH_X32)
                                        .const_int(index as u64, false),
                                ],
                                format!("binding_{}_gep_pointer", index).as_str(),
                            )
                        };

                        let value = context
                            .build_load(pointer, format!("binding_{}_value", index).as_str());
                        let pointer = context
                            .function_mut()
                            .stack
                            .get(binding.name.as_str())
                            .cloned()
                            .expect("Always exists");
                        context.build_store(pointer, value);
                    }
                }
            }
            None => {
                context.build_store(pointer, llvm_type.const_zero());
            }
        }

        Ok(())
    }
}
