//!
//! YUL to LLVM compiler library.
//!

pub mod error;
pub mod generator;
pub mod lexer;
pub mod parser;

pub use self::error::Error;
pub use self::generator::action::Action;
pub use self::generator::llvm::Context as LLVMContext;
pub use self::generator::ILLVMWritable;
pub use self::lexer::Lexer;
pub use self::parser::Module;

/// The `bool` type bitlength.
pub const BITLENGTH_BOOLEAN: usize = 1;

/// The `uint` / `uint256` type bitlength.
pub const BITLENGTH_DEFAULT: usize = 256;

///
/// Parses the source code and returns the AST.
///
pub fn parse(input: &str) -> Result<Module, Error> {
    Module::parse(&mut Lexer::new(input.to_owned()), None)
}

///
/// Parses and compiles the source code.
///
pub fn compile(input: &str, optimization_level: usize) -> Result<String, Error> {
    let llvm = inkwell::context::Context::create();
    let module = parse(input)?;
    let mut context = LLVMContext::new_with_optimizer(&llvm, optimization_level);
    module.into_llvm(&mut context);
    context.optimize();
    context.verify().expect("Verification error");
    Ok(context.module.print_to_string().to_string())
}
