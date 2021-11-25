//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use std::str::FromStr;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() {
    std::process::exit(match main_inner() {
        Ok(()) => compiler_common::EXIT_CODE_SUCCESS,
        Err(error) => {
            eprintln!("{:?}", error);
            compiler_common::EXIT_CODE_FAILURE
        }
    })
}

///
/// The auxiliary `main` function to facilitate the `?` error conversion operator.
///
fn main_inner() -> Result<(), compiler_solidity::Error> {
    let mut arguments = Arguments::new();

    for path in arguments.input_files.iter_mut() {
        *path = path.canonicalize()?;
    }

    let solc = compiler_solidity::SolcCompiler::new(
        "solc".to_owned(),
        semver::Version::from_str(env!("CARGO_PKG_VERSION")).expect("Always valid"),
    );

    let is_output_requested = arguments.combined_json.is_some()
        || arguments.output_assembly
        || arguments.output_binary
        || arguments.output_hashes
        || arguments.output_abi;

    if arguments.output_abi || arguments.output_hashes {
        match solc.extra_output(
            arguments.input_files.as_slice(),
            arguments.output_abi,
            arguments.output_hashes,
        ) {
            Ok(stdout) => {
                print!("{}", stdout);
            }
            Err(stderr) => {
                eprint!("{}", stderr);
                std::process::exit(compiler_common::EXIT_CODE_FAILURE);
            }
        }
    }

    let solc_input = compiler_solidity::SolcStandardJsonInput::try_from_paths(
        arguments.input_files.as_slice(),
        arguments.libraries,
        true,
    )?;
    let libraries = solc_input.settings.libraries.clone();
    let solc_output = match solc.standard_json(
        solc_input,
        arguments.base_path,
        arguments.include_paths,
        arguments.allow_paths,
    ) {
        Ok(standard_json) => standard_json,
        Err(stderr) => {
            eprint!("{}", stderr);
            std::process::exit(compiler_common::EXIT_CODE_FAILURE);
        }
    };

    compiler_solidity::initialize_target();
    let mut project = solc_output.try_into_project(libraries, arguments.dump_yul, true)?;
    project.compile_all(arguments.optimize, arguments.dump_llvm)?;

    let combined_json = if let Some(combined_json) = arguments.combined_json {
        match solc.combined_json(arguments.input_files.as_slice(), combined_json.as_str()) {
            Ok(combined_json) => Some(combined_json),
            Err(stderr) => {
                eprint!("{}", stderr);
                std::process::exit(compiler_common::EXIT_CODE_FAILURE);
            }
        }
    } else {
        None
    };

    if let Some(output_directory) = arguments.output_directory {
        std::fs::create_dir_all(&output_directory)?;

        if let Some(mut combined_json) = combined_json {
            project.write_to_combined_json(&mut combined_json)?;
            combined_json.write_to_directory(&output_directory, arguments.overwrite)?;
        } else {
            project.write_to_directory(
                &output_directory,
                arguments.output_assembly,
                arguments.output_binary,
                arguments.overwrite,
            )?;
        }

        eprintln!(
            "Compiler run successful. Artifact(s) can be found in directory {:?}.",
            output_directory
        );
    } else if let Some(mut combined_json) = combined_json {
        project.write_to_combined_json(&mut combined_json)?;
        println!(
            "{}",
            serde_json::to_string(&combined_json).expect("Always valid")
        );
    } else if !is_output_requested {
        eprintln!("Compiler run successful. No output requested. Use --asm and --bin flags.");
    }

    Ok(())
}
