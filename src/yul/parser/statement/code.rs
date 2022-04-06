//!
//! The YUL code.
//!

use crate::yul::lexer::lexeme::keyword::Keyword;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::block::Block;

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
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        match lexeme {
            Lexeme::Keyword(Keyword::Code) => {}
            lexeme => anyhow::bail!("Expected one of {:?}, found `{}`", ["code"], lexeme),
        }

        let block = Block::parse(lexer, None)?;

        Ok(Self { block })
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Code
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.block.into_llvm(context)?;

        Ok(())
    }
}
