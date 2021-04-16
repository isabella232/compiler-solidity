//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use std::convert::TryFrom;
use std::io::Read;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() {
    std::process::exit(match main_inner() {
        Ok(()) => compiler_const::exit_code::SUCCESS,
        Err(error) => {
            eprintln!("{:?}", error);
            compiler_const::exit_code::FAILURE
        }
    })
}

///
/// The auxiliary `main` function to facilitate the `?` error conversion operator.
///
fn main_inner() -> Result<(), yul_compiler::Error> {
    let arguments = Arguments::new();

    let target = yul_compiler::Target::try_from(arguments.target.as_str())
        .map_err(yul_compiler::Error::Target)?;

    let code = if arguments.input.to_string_lossy() == "-" {
        let mut buffer = String::with_capacity(16384);
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        std::fs::read_to_string(&arguments.input)?
    };

    let output = yul_compiler::compile(
        &code,
        target,
        arguments.optimization_level,
        arguments.dump_llvm,
    )?;
    println!("{}", output);

    Ok(())
}
