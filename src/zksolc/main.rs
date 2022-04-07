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
        Ok(()) => compiler_common::EXIT_CODE_SUCCESS,
        Err(error) => {
            eprintln!("{}", error);
            compiler_common::EXIT_CODE_FAILURE
        }
    })
}

///
/// The auxiliary `main` function to facilitate the `?` error conversion operator.
///
fn main_inner() -> anyhow::Result<()> {
    let mut arguments = Arguments::new();
    arguments.validate()?;

    let dump_flags = compiler_solidity::DumpFlag::initialize(
        arguments.dump_yul,
        arguments.dump_ethir,
        arguments.dump_evm,
        arguments.dump_llvm,
        arguments.dump_assembly,
    );

    for path in arguments.input_files.iter_mut() {
        *path = path.canonicalize()?;
    }

    let solc =
        compiler_solidity::SolcCompiler::new(arguments.solc.unwrap_or_else(|| {
            compiler_solidity::SolcCompiler::DEFAULT_EXECUTABLE_NAME.to_owned()
        }));
    let solc_version = solc.version()?;

    let output_selection = compiler_solidity::SolcStandardJsonInputSettings::get_output_selection(
        arguments
            .input_files
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        compiler_solidity::SolcPipeline::Yul,
    );
    let solc_input = if arguments.standard_json {
        let mut input: compiler_solidity::SolcStandardJsonInput =
            serde_json::from_reader(std::io::BufReader::new(std::io::stdin()))?;
        input.settings.output_selection = output_selection;
        input
    } else {
        let language = if arguments.yul {
            compiler_solidity::SolcStandardJsonInputLanguage::Yul
        } else {
            compiler_solidity::SolcStandardJsonInputLanguage::Solidity
        };
        compiler_solidity::SolcStandardJsonInput::try_from_paths(
            language,
            arguments.input_files.as_slice(),
            arguments.libraries,
            output_selection,
            true,
        )?
    };

    let libraries = solc_input.settings.libraries.clone().unwrap_or_default();
    let mut solc_output = solc.standard_json(
        solc_input,
        arguments.base_path,
        arguments.include_paths,
        arguments.allow_paths,
    )?;

    if let Some(errors) = solc_output.errors.as_deref() {
        let mut cannot_compile = false;
        for error in errors.iter() {
            if error.severity.as_str() == "error" {
                cannot_compile = true;
                if arguments.standard_json {
                    serde_json::to_writer(std::io::stdout(), &solc_output)?;
                    return Ok(());
                }
            }

            eprintln!("{}", error);
        }

        if cannot_compile {
            anyhow::bail!("Error(s) found. Compilation aborted");
        }
    }

    compiler_solidity::initialize_target();
    let mut project = solc_output.try_into_project(
        libraries,
        compiler_solidity::SolcPipeline::Yul,
        solc_version,
        dump_flags.as_slice(),
    )?;
    project.compile_all(arguments.optimize, dump_flags)?;

    if arguments.standard_json {
        project.write_to_standard_json(&mut solc_output)?;
        serde_json::to_writer(std::io::stdout(), &solc_output)?;
        return Ok(());
    }

    let combined_json = if let Some(combined_json) = arguments.combined_json {
        Some(solc.combined_json(arguments.input_files.as_slice(), combined_json.as_str())?)
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
    } else if arguments.output_assembly
        || arguments.output_binary
        || arguments.output_hashes
        || arguments.output_abi
    {
        for (path, contract) in project.contracts.into_iter() {
            if arguments.output_assembly {
                println!(
                    "Contract `{}` assembly:\n\n{}",
                    path,
                    contract.assembly.expect("Always exists")
                );
            }
            if arguments.output_binary {
                println!(
                    "Contract `{}` bytecode: 0x{}",
                    path,
                    hex::encode(contract.bytecode.expect("Always exists").as_slice())
                );
            }
        }

        if arguments.output_abi || arguments.output_hashes {
            let extra_output = solc.extra_output(
                arguments.input_files.as_slice(),
                arguments.output_abi,
                arguments.output_hashes,
            )?;
            print!("{}", extra_output);
        }
    } else {
        eprintln!("Compiler run successful. No output requested. Use --asm and --bin flags.");
    }

    Ok(())
}
