//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

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
fn main_inner() -> Result<(), compiler_solidity::Error> {
    let arguments = Arguments::new();

    let optimization_level = if arguments.optimize {
        inkwell::OptimizationLevel::Aggressive
    } else {
        inkwell::OptimizationLevel::None
    };

    let solc = compiler_solidity::SolcCompiler::default();
    let solc_input =
        compiler_solidity::SolcInput::try_from_paths(arguments.input_files, arguments.libraries)?;
    let libraries = solc_input.settings.libraries.clone();
    let solc_output: compiler_solidity::SolcOutput = solc
        .standard_json(
            solc_input,
            arguments.base_path,
            arguments.include_paths,
            arguments.allow_paths,
        )
        .map_err(compiler_solidity::Error::Solidity)?;

    compiler_common::vm::initialize_target();

    let mut project = solc_output.try_into_project(libraries, arguments.dump_yul, true)?;
    project.compile_all(
        arguments.output_directory,
        optimization_level,
        optimization_level,
        arguments.dump_llvm,
        arguments.output_assembly,
        arguments.output_binary,
        arguments.overwrite,
    )?;

    Ok(())
}
