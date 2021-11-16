//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

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

    let solc = compiler_solidity::SolcCompiler::new(
        "solc".to_owned(),
        semver::Version::from_str(env!("CARGO_PKG_VERSION")).expect("Always valid"),
    );

    let mut is_output_requested = false;

    if let Some(combined_json) = arguments.combined_json {
        let stdout = solc.combined_json(arguments.input_files.as_slice(), combined_json)?;
        if let Some(ref output_directory) = arguments.output_directory {
            let mut file_path = output_directory.to_owned();
            file_path.push(format!("combined.{}", compiler_common::extension::JSON));
            if file_path.exists() && !arguments.overwrite {
                println!(
                    "Refusing to overwrite existing file {:?} (use --overwrite to force).",
                    file_path
                );
            } else {
                File::create(&file_path)
                    .map_err(compiler_solidity::Error::FileSystem)?
                    .write_all(stdout.as_bytes())
                    .map_err(compiler_solidity::Error::FileSystem)?;

                println!(
                    "Compiler run successful. Artifact(s) can be found in directory {:?}.",
                    output_directory
                );
            }
            return Ok(());
        }

        print!("{}", stdout);
        is_output_requested = true;
    }

    if arguments.output_abi || arguments.output_hashes {
        print!(
            "{}",
            solc.extra_output(
                arguments.input_files.as_slice(),
                arguments.output_abi,
                arguments.output_hashes
            )?
        );
        is_output_requested = true;
    }

    let solc_input = compiler_solidity::SolcInput::try_from_paths(
        arguments.input_files,
        arguments.libraries,
        false,
    )?;
    let libraries = solc_input.settings.libraries.clone();
    let solc_output: compiler_solidity::SolcOutput = solc.standard_json(
        solc_input,
        arguments.base_path,
        arguments.include_paths,
        arguments.allow_paths,
    )?;

    compiler_common::vm::initialize_target();

    let output_directory = arguments
        .output_directory
        .clone()
        .unwrap_or_else(|| PathBuf::from("./build/"));
    let mut project = solc_output.try_into_project(libraries, arguments.dump_yul, true)?;
    project.compile_all(
        &output_directory,
        optimization_level,
        optimization_level,
        arguments.dump_llvm,
        arguments.output_assembly,
        arguments.output_binary,
        arguments.overwrite,
    )?;

    if arguments.output_assembly || arguments.output_binary {
        println!(
            "Compiler run successful. Artifact(s) can be found in directory {:?}.",
            output_directory
        );
    } else if !is_output_requested {
        println!("Compiler run successful. No output requested. Use --asm and --bin flags.");
    }

    Ok(())
}
