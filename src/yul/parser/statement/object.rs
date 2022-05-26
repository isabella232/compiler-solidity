//!
//! The YUL object.
//!

use std::collections::HashSet;

use crate::yul::lexer::lexeme::keyword::Keyword;
use crate::yul::lexer::lexeme::literal::Literal;
use crate::yul::lexer::lexeme::symbol::Symbol;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::code::Code;

///
/// The YUL object.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    /// The identifier.
    pub identifier: String,
    /// The code.
    pub code: Code,
    /// The optional inner object.
    pub inner_object: Option<Box<Self>>,
    /// The factory dependency objects.
    pub factory_dependencies: HashSet<String>,
}

impl Object {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Keyword(Keyword::Object) => {}
            lexeme => anyhow::bail!("Expected one of {:?}, found `{}`", ["object"], lexeme),
        }

        let identifier = match lexer.next()? {
            Lexeme::Literal(Literal::String(literal)) => literal.inner,
            lexeme => {
                anyhow::bail!("Expected one of {:?}, found `{}`", ["{string}"], lexeme);
            }
        };
        let is_runtime_code = identifier.ends_with("_deployed");

        match lexer.next()? {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => anyhow::bail!("Expected one of {:?}, found `{}`", ["{"], lexeme),
        }

        let code = Code::parse(lexer, None)?;
        let mut inner_object = None;
        let mut factory_dependencies = HashSet::new();

        if !is_runtime_code {
            inner_object = match lexer.peek()? {
                Lexeme::Keyword(Keyword::Object) => {
                    let mut object = Self::parse(lexer, None)?;
                    factory_dependencies.extend(object.factory_dependencies.drain());
                    Some(Box::new(object))
                }
                _ => None,
            };

            if let Lexeme::Identifier(identifier) = lexer.peek()? {
                if identifier.as_str() == "data" {
                    let _data = lexer.next()?;
                    let _identifier = lexer.next()?;
                    let _metadata = lexer.next()?;
                }
            };
        }

        loop {
            match lexer.next()? {
                Lexeme::Symbol(Symbol::BracketCurlyRight) => break,
                lexeme @ Lexeme::Keyword(Keyword::Object) => {
                    let dependency = Self::parse(lexer, Some(lexeme))?;
                    factory_dependencies.insert(dependency.identifier);
                }
                Lexeme::Identifier(identifier) if identifier.as_str() == "data" => {
                    let _identifier = lexer.next()?;
                    let _metadata = lexer.next()?;
                }
                lexeme => {
                    anyhow::bail!("Expected one of {:?}, found `{}`", ["object", "}"], lexeme);
                }
            }
        }

        Ok(Self {
            identifier,
            code,
            inner_object,
            factory_dependencies,
        })
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Object
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let mut entry = compiler_llvm_context::EntryFunction::default();
        entry.declare(context)?;

        compiler_llvm_context::DeployCodeFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;
        compiler_llvm_context::RuntimeCodeFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;

        entry.into_llvm(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        if self.identifier.ends_with("_deployed") {
            compiler_llvm_context::RuntimeCodeFunction::new(self.code).into_llvm(context)?;
        } else {
            compiler_llvm_context::DeployCodeFunction::new(self.code).into_llvm(context)?;
        }

        if let Some(object) = self.inner_object {
            object.into_llvm(context)?;
        }

        Ok(())
    }
}
