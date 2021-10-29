//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use std::collections::HashMap;
use std::io::Read;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() {
    std::process::exit(match main_inner() {
        Ok(()) => compiler_common::exit_code::SUCCESS,
        Err(error) => {
            eprintln!("{:?}", error);
            compiler_common::exit_code::FAILURE
        }
    })
}

///
/// The auxiliary `main` function to facilitate the `?` error conversion operator.
///
fn main_inner() -> Result<(), compiler_yul::Error> {
    let arguments = Arguments::new();

    let input_string = if arguments.input.to_string_lossy() == "-" {
        let mut buffer = String::with_capacity(16384);
        std::io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        std::fs::read_to_string(&arguments.input)?
    };

    let optimization_level = match arguments.optimization_level {
        0 => inkwell::OptimizationLevel::None,
        1 => inkwell::OptimizationLevel::Less,
        2 => inkwell::OptimizationLevel::Default,
        _ => inkwell::OptimizationLevel::Aggressive,
    };

    let libraries: HashMap<String, String> = arguments
        .libraries
        .into_iter()
        .enumerate()
        .map(|(index, library)| {
            let mut parts = library.split('=');
            let path = parts
                .next()
                .unwrap_or_else(|| panic!("The library #{} path is missing", index));
            let address = parts
                .next()
                .unwrap_or_else(|| panic!("The library #{} address is missing", index));
            (path.to_owned(), address.to_owned())
        })
        .collect();

    let input: compiler_yul::Input = serde_json::from_str(input_string.as_str())?;
    let mut project = input.try_into_project(libraries, arguments.dump_yul, true)?;
    project.compile_all(
        arguments.output,
        optimization_level,
        optimization_level,
        arguments.dump_llvm,
    )?;

    Ok(())
}
