//!
//! The YUL object.
//!

pub mod code;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::literal::Literal;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;

use self::code::Code;

///
/// The YUL object.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    /// The identifier.
    pub identifier: String,
    /// The code.
    pub code: Code,
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

        let code = Code::parse(lexer, None)?;

        Ok(Self { identifier, code })
    }
}

impl ILLVMWritable for Object {
    fn into_llvm(self, context: &mut LLVMContext) {
        self.code.into_llvm(context)
    }
}
