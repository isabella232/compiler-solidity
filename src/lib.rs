//!
//! YUL to LLVM compiler library.
//!

pub mod generator;
pub mod lexer;
pub mod parser;

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
pub fn parse(input: &str) -> Module {
    Module::parse(&mut Lexer::new(input.to_owned()), None)
}

///
/// Parses and compiles the source code.
///
pub fn compile(input: &str) -> String {
    let llvm = inkwell::context::Context::create();
    let module = parse(input);
    LLVMContext::new(&llvm).compile(module)
}
