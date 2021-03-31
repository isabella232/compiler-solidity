//!
//! The compiler test tools.
//!

use crate::generator::llvm::Context;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::Module;

///
/// Consumes the source code and returns the vector of lexems.
///
pub(crate) fn tokenize(input: &str) -> Vec<Lexeme> {
    Lexer::new(input.to_owned()).tokenize()
}

///
/// Parses the source code and returns the AST.
///
pub(crate) fn parse(input: &str) -> Module {
    Module::parse(&mut Lexer::new(input.to_owned()), None)
}

///
/// Parses and compiles the source code.
/// Also executes it and returns the result, if `entry` is specified.
///
pub(crate) fn compile(input: &str, entry: Option<&str>) -> u64 {
    let llvm = inkwell::context::Context::create();
    let module = parse(input);
    let entry = entry.map(|entry| entry.to_owned());
    Context::new(&llvm)
        .compile(module, entry)
        .unwrap_or_default()
}
