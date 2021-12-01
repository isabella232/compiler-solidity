//!
//! The YUL object.
//!

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::literal::Literal;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::code::Code;

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
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

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

        match lexer.next()? {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
            lexeme => return Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }

        let code = Code::parse(lexer, None)?;

        let object = match lexer.peek()? {
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

        let mut dependencies = Vec::new();
        loop {
            match lexer.next()? {
                Lexeme::Symbol(Symbol::BracketCurlyRight) => break,
                lexeme @ Lexeme::Keyword(Keyword::Object) => {
                    let dependency = Self::parse(lexer, Some(lexeme))?;
                    dependencies.push(dependency);
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

impl ILLVMWritable for Object {
    fn into_llvm(self, context: &mut LLVMContext) -> anyhow::Result<()> {
        let is_selector = self.identifier.ends_with("_deployed");
        if !is_selector {
            context.set_object(self.identifier.as_str());
        }
        let is_constructor = !is_selector;

        if is_constructor {
            context.add_function(
                compiler_common::LLVM_FUNCTION_SELECTOR,
                context.void_type().fn_type(&[], false),
                Some(inkwell::module::Linkage::External),
                false,
            );

            self.code.into_llvm_constructor(context)?;
        } else if is_selector {
            self.code.into_llvm_selector(context)?;
        }

        if let Some(object) = self.object {
            object.into_llvm(context)?;
        }

        Ok(())
    }
}
