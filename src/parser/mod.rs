//!
//! The YUL syntax tree.
//!

pub mod block;
pub mod comment;
pub mod identifier;
pub mod r#type;

use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;

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
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) -> Self {
        loop {
            let lexeme = initial.take().unwrap_or_else(|| lexer.next());

            match lexeme {
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    return Self {
                        block: Block::parse(lexer, None),
                    };
                }
                Lexeme::Symbol(Symbol::CommentStart) => {
                    Comment::parse(lexer, None);
                }
                lexeme => panic!("Expected one of `/*`, `{{`, got {}", lexeme),
            }
        }
    }
}

impl ILLVMWritable for Module {
    fn into_llvm(self, context: &mut LLVMContext) {
        self.block.into_llvm_module(context);
    }
}
