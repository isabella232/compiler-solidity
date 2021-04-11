//!
//! The YUL syntax tree.
//!

pub mod block;
pub mod comment;
pub mod error;
pub mod identifier;
pub mod r#type;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;

use self::block::Block;
use self::comment::Comment;

///
/// The upper module.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    /// The main block.
    pub block: Block,
}

impl Module {
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) -> Result<Self, Error> {
        loop {
            let lexeme = crate::parser::take_or_next(initial.take(), lexer)?;

            match lexeme {
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    return Ok(Self {
                        block: Block::parse(lexer, None)?,
                    });
                }
                Lexeme::Symbol(Symbol::CommentStart) => {
                    Comment::parse(lexer, None)?;
                }
                lexeme => {
                    return Err(ParserError::expected_one_of(vec!["/*", "{"], lexeme, None).into())
                }
            }
        }
    }
}

impl ILLVMWritable for Module {
    fn into_llvm(self, context: &mut LLVMContext) {
        self.block.into_llvm_module(context);
    }
}

///
/// Returns the `token` value if it is `Some(_)`, otherwise takes the next token from the `stream`.
///
pub fn take_or_next(mut lexeme: Option<Lexeme>, lexer: &mut Lexer) -> Result<Lexeme, Error> {
    match lexeme.take() {
        Some(lexeme) => Ok(lexeme),
        None => Ok(lexer.next()?),
    }
}
