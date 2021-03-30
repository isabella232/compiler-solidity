//!
//! The compiler test tools.
//!

use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::block::statement::Statement;
use crate::parser::Module;

pub(crate) fn tokenize(input: &str) -> Vec<Lexeme> {
    Lexer::new(input.to_owned()).tokenize()
}

pub(crate) fn parse(input: &str) -> Vec<Statement> {
    Module::parse(&mut Lexer::new(input.to_owned()), None).statements
}

pub(crate) fn compile(input: &str, entry: Option<&str>) -> u64 {
    let mut statements = parse(input);
    crate::llvm::Generator::compile(statements.remove(0), entry.map(|entry| entry.to_owned()))
}
