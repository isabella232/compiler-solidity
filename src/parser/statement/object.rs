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
use crate::target::Target;

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

        Ok(Self {
            identifier,
            code,
            object,
        })
    }
}

impl ILLVMWritable for Object {
    fn into_llvm(self, context: &mut LLVMContext) {
        let is_selector = self.identifier.ends_with("_deployed");
        let is_constructor = !is_selector;

        context.allocate_heap(2048 * compiler_common::size::FIELD);
        context.allocate_storage(256);
        context.allocate_calldata(64);

        let selector_type = match context.target {
            Target::X86 => context
                .integer_type(compiler_common::bitlength::WORD)
                .fn_type(&[], false),
            Target::zkEVM if context.test_entry_hash.is_some() => context
                .integer_type(compiler_common::bitlength::FIELD)
                .fn_type(&[], false),
            Target::zkEVM => context.void_type().fn_type(&[], false),
        };

        if is_selector {
            self.code.into_llvm_deployed(context);
        } else if is_constructor
            && matches!(context.target, Target::zkEVM)
            && context.test_entry_hash.is_none()
        {
            context.add_function(
                compiler_common::identifier::FUNCTION_SELECTOR,
                selector_type,
                Some(inkwell::module::Linkage::External),
                false,
            );

            self.code.into_llvm_constructor(context);
        }

        if let Some(object) = self.object {
            object.into_llvm(context);
        }
    }
}
