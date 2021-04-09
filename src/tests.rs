//!
//! The compiler test tools.
//!

use crate::generator::llvm::Context;
use crate::lexer::Lexer;
use crate::parser::Module;

///
/// Parses the source code and returns the AST.
///
pub(crate) fn parse(input: &str) -> Module {
    Module::parse(&mut Lexer::new(input.to_owned()), None)
}

///
/// Parses and compiles the source code.
///
pub(crate) fn compile(input: &str) {
    let llvm = inkwell::context::Context::create();
    let module = parse(input);
    Context::new(&llvm).compile(module);
}
