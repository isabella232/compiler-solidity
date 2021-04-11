//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use self::arguments::Arguments;

/// The common application success exit code.
pub const SUCCESS: i32 = 0;

/// The common application failure exit code.
pub const FAILURE: i32 = 1;

///
/// The application entry point.
///
fn main() {
    std::process::exit(match main_inner() {
        Ok(()) => SUCCESS,
        Err(error) => {
            eprintln!("{:?}", error);
            FAILURE
        }
    })
}

///
/// The auxiliary `main` function to facilitate the `?` error conversion operator.
///
fn main_inner() -> Result<(), yul_compiler::Error> {
    let arguments = Arguments::new();

    let code = std::fs::read_to_string(&arguments.input).expect("Input file reading error");
    let output = yul_compiler::compile(&code, arguments.optimization_level)?;
    println!("{}", output);

    Ok(())
}
