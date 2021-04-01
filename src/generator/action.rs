//!
//! YUL to LLVM compiler action.
//!

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use rand::Rng;

use crate::generator::file_type::FileType;
use crate::generator::llvm::Context;
use crate::lexer::Lexer;
use crate::parser::Module;

///
/// The compilation step.
///
#[derive(Debug)]
pub enum Action {
    /// The `solc` subprocess.
    SolidityCompiler(PathBuf, String),
    /// The YUL compiler.
    CodeGenerator(PathBuf, Option<String>),
}

impl Action {
    ///
    /// Produce sequence of actions required to compile the `file` with specified `options`.
    ///
    pub fn new_list(path: PathBuf, options: String, entry: Option<String>) -> Vec<Action> {
        match FileType::new(&path) {
            FileType::Yul => vec![Action::CodeGenerator(path, entry)],
            FileType::Solidity => {
                let tmp_file = Self::yul_directory(&path);
                let options_string =
                    options.trim().to_owned() + " --ir -o " + tmp_file.to_string_lossy().trim();
                vec![
                    Action::SolidityCompiler(path, options_string.trim().to_owned()),
                    Action::CodeGenerator(tmp_file, entry),
                ]
            }
            FileType::Unknown(extension) => panic!(
                "expected a *.yul or *.sol file, got with extension {}",
                extension
            ),
        }
    }

    ///
    /// Executes an action by calling the corresponding handler.
    ///
    pub fn execute(self) {
        match self {
            Self::SolidityCompiler(input, options) => Self::execute_solc(input, options),
            Self::CodeGenerator(input, entry) => Self::execute_llvm(input, entry),
        }
    }

    ///
    /// Executes the Solidity compiler.
    ///
    pub fn execute_solc(input: PathBuf, options: String) {
        let child = std::process::Command::new("solc")
            .arg(&input)
            .args(options.split(' ').collect::<Vec<&str>>())
            .spawn()
            .expect("The `solc` spawning error. Ensure it's in PATH");
        let output = child.wait_with_output().expect("The `solc` waiting error");
        if !output.status.success() {
            let mut message = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
            message.push_str(String::from_utf8_lossy(output.stderr.as_slice()).as_ref());
            panic!("The `solc` error: {}", message);
        }
    }

    ///
    /// Executes the LLVM generator.
    ///
    pub fn execute_llvm(input: PathBuf, entry: Option<String>) {
        let metadata = fs::metadata(&input).expect("File metadata error");
        let inputs = if metadata.is_file() {
            vec![input]
        } else {
            std::fs::read_dir(input)
                .expect("Directory reading error")
                .map(|entry| entry.expect("Directory entry error").path())
                .collect()
        };

        for input in inputs.into_iter() {
            let input = std::fs::read_to_string(input).expect("Input file reading error");
            let mut lexer = Lexer::new(input);
            // Self::extract_sol_functions(&mut lexemes);

            let llvm = inkwell::context::Context::create();
            let module = Module::parse(&mut lexer, None);
            let entry = entry.clone();
            Context::new(&llvm).compile(module, entry);
        }
    }

    ///
    /// Generates the temporary output directory for a given solidity input.
    ///
    fn yul_directory(input: &Path) -> PathBuf {
        let mut path = std::env::temp_dir();
        let suffix: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let file_stem = input
            .file_stem()
            .expect("File stem always exists")
            .to_string_lossy()
            .to_string();
        let tmp_dir_name = file_stem + "-" + suffix.as_str();
        path.push(tmp_dir_name);
        path
    }

    // fn extract_sol_functions(lexemes: &mut Vec<Lexeme>) {
    //     let pos = lexemes
    //         .iter()
    //         .position(|x| matches!(x, Lexeme::Keyword(Keyword::Function)));
    //     if pos == None {
    //         return;
    //     }
    //     let pos = pos.unwrap();
    //     if lexemes.len() < pos + 2 {
    //         return;
    //     }
    //     if let Lexeme::Identifier(identifier) = &lexemes[pos + 1] {
    //         if !identifier.starts_with("constructor") {
    //             return;
    //         }
    //     }
    //     lexemes.drain(0..pos + 1);
    //     let pos = lexemes
    //         .iter()
    //         .position(|x| matches!(x, Lexeme::Keyword(Keyword::Function)))
    //         .unwrap_or_else(|| panic!("Expected at least one function in the contract"));
    //     lexemes.drain(0..pos);
    //     lexemes.drain(lexemes.len() - 2..);
    //     lexemes.insert(0, Lexeme::Symbol(Symbol::BracketCurlyLeft));
    // }
}
