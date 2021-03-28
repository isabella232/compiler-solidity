//!
//! YUL to LLVM compiler library.
//!

use std::fs;
use std::path::Path;

use rand::Rng;

use self::file_type::FileType;
use self::lexer::lexeme::keyword::Keyword;
use self::lexer::lexeme::symbol::Symbol;
use self::lexer::lexeme::Lexeme;
use self::lexer::Lexer;
use self::parser::Parser;

#[cfg(test)]
mod tests;

pub mod file_type;
pub mod generator;
pub mod lexer;
pub mod parser;

/// Abstract compilation step
#[derive(Debug)]
pub enum Action<'a> {
    SolidityCompiler(&'a str, String),
    CodeGenerator(String, &'a Option<&'a str>),
}

/// Generate temporary output directory for a given solidity input
/// Precondition: input must exist
fn tmp_yul(input: &str) -> String {
    let mut path = std::env::temp_dir();
    let suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let file_stem = std::path::Path::new(input).file_stem().unwrap();
    let tmp_dir_name = String::from(file_stem.to_str().unwrap()) + "-" + suffix.as_str();
    path.push(tmp_dir_name);
    String::from(path.to_str().unwrap())
}

/// Produce sequence of actions required to compile file with specified options
pub fn generate_actions<'a>(
    file: &'a Path,
    options: &'a str,
    run: &'a Option<&'a str>,
) -> Vec<Action<'a>> {
    match FileType::new(file) {
        FileType::Yul => vec![Action::CodeGenerator(
            String::from(file.to_str().unwrap()),
            run,
        )],
        FileType::Solidity => {
            let tmp_file = tmp_yul(file.to_str().unwrap());
            let options_string = String::from(options) + " --ir -o " + tmp_file.as_str();
            let options_string = String::from(options_string.trim());
            vec![
                Action::SolidityCompiler(file.to_str().unwrap(), options_string),
                Action::CodeGenerator(tmp_file, run),
            ]
        }
        _ => vec![],
    }
}

/// Wrap Solidity compiler invocation
pub fn invoke_solidity(input: &str, options: &str) {
    let mut child = std::process::Command::new("solc")
        .arg(input)
        .args(options.split(' ').collect::<Vec<&str>>())
        .spawn()
        .expect("Unable to run solidity. Ensure it's in PATH");
    let ecode = child
        .wait()
        .expect("failed to wait on Solidity compiler run");
    if !ecode.success() {
        panic!("{}", "Solidity compiler terminated with a failure");
    }
}

fn extract_sol_functions(lexemes: &mut Vec<Lexeme>) {
    let pos = lexemes
        .iter()
        .position(|x| matches!(x, Lexeme::Keyword(Keyword::Function)));
    if pos == None {
        return;
    }
    let pos = pos.unwrap();
    if lexemes.len() < pos + 2 {
        return;
    }
    if let Lexeme::Identifier(identifier) = &lexemes[pos + 1] {
        if !identifier.starts_with("constructor") {
            return;
        }
    }
    lexemes.drain(0..pos + 1);
    let pos = lexemes
        .iter()
        .position(|x| matches!(x, Lexeme::Keyword(Keyword::Function)))
        .unwrap_or_else(|| panic!("Expected at least one function in the contract"));
    lexemes.drain(0..pos);
    lexemes.drain(lexemes.len() - 2..);
    lexemes.insert(0, Lexeme::Symbol(Symbol::BracketCurlyLeft));
}

/// Wrap Zinc generator invocation
pub fn invoke_codegen<'a>(input: &str, run: &'a Option<&'a str>) {
    let meta = fs::metadata(input).unwrap();
    let filenames = if meta.is_file() {
        vec![input.to_string()]
    } else {
        std::fs::read_dir(input)
            .unwrap()
            .map(|x| x.unwrap().path().to_str().unwrap().to_string())
            .collect()
    };
    for in_file in filenames {
        let input = std::fs::read_to_string(in_file.as_str()).unwrap();
        let mut lexer = Lexer::new(input);
        let mut lexemes = lexer.get_lexemes();
        extract_sol_functions(&mut lexemes);
        let statements = Parser::parse(lexemes.into_iter());
        generator::Compiler::compile(&statements[0], run);
    }
}

/// Execute an action by calling corresponding handler
pub fn execute_action(action: &Action) {
    match action {
        Action::SolidityCompiler(input, options) => invoke_solidity(input, options.as_str()),
        Action::CodeGenerator(input, run) => invoke_codegen(input.as_str(), run),
    }
}

pub trait PeekableIterator: std::iter::Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;
}

impl<I: std::iter::Iterator> PeekableIterator for std::iter::Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        std::iter::Peekable::peek(self)
    }
}
