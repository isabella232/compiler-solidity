//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() {
    let arguments = Arguments::new();

    let output = yul_compiler::Action::execute_llvm(arguments.input);
    println!("{}", output);
}
