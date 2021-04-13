//!
//! The YUL code.
//!

pub mod block;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::object::Object;

use self::block::Block;

///
/// The YUL code.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Code {
    /// The main block.
    pub block: Block,
}

impl Code {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => Ok(Self {
                block: Block::parse(lexer, None)?,
            }),
            lexeme => Err(ParserError::expected_one_of(vec!["{"], lexeme, None).into()),
        }
    }

    ///
    /// Converts the element into a test object.
    ///
    pub fn into_test_object(self) -> Object {
        Object {
            identifier: "test".to_owned(),
            code: self,
        }
    }
}

impl ILLVMWritable for Code {
    fn into_llvm(self, context: &mut LLVMContext) {
        self.block.into_llvm_object(context);
    }
}
