//!
//! The YUL object.
//!

use crate::error::Error;
use crate::yul::lexer::lexeme::keyword::Keyword;
use crate::yul::lexer::lexeme::literal::Literal;
use crate::yul::lexer::lexeme::symbol::Symbol;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::error::Error as ParserError;
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
    pub object: Option<Box<Self>>,
    /// The dependency objects, usually related to factory dependencies.
    pub dependencies: Vec<Self>,
}

impl Object {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Keyword(Keyword::Object) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["object"], lexeme, None).into()),
        }

        let identifier = match lexer.next()? {
            Lexeme::Literal(Literal::String(literal)) => literal.inner,
            lexeme => {
                return Err(ParserError::expected_one_of(vec!["{string}"], lexeme, None).into())
            }
        };
        let is_selector = identifier.ends_with("_deployed");

        match lexer.next()? {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }

        let code = Code::parse(lexer, None)?;

        let mut object = None;
        if !is_selector {
            object = match lexer.peek()? {
                Lexeme::Keyword(Keyword::Object) => Some(Self::parse(lexer, None).map(Box::new)?),
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

        let mut dependencies = Vec::new();
        loop {
            match lexer.next()? {
                Lexeme::Symbol(Symbol::BracketCurlyRight) => break,
                lexeme @ Lexeme::Keyword(Keyword::Object) => {
                    let dependency = Self::parse(lexer, Some(lexeme))?;
                    dependencies.push(dependency);
                }
                Lexeme::Identifier(identifier) if identifier.as_str() == "data" => {
                    let _identifier = lexer.next()?;
                    let _metadata = lexer.next()?;
                }
                lexeme => {
                    return Err(
                        ParserError::expected_one_of(vec!["object", "}"], lexeme, None).into(),
                    )
                }
            }
        }

        Ok(Self {
            identifier,
            code,
            object,
            dependencies,
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

        compiler_llvm_context::ConstructorFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;
        compiler_llvm_context::SelectorFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;

        entry.into_llvm(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        if self.identifier.ends_with("_deployed") {
            compiler_llvm_context::SelectorFunction::new(self.code).into_llvm(context)?;
        } else {
            compiler_llvm_context::ConstructorFunction::new(self.code).into_llvm(context)?;
        }

        if let Some(object) = self.object {
            object.into_llvm(context)?;
        }

        Ok(())
    }
}
