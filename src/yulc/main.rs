//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

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

    let target = compiler_yul::Target::try_from(arguments.target.as_str())
        .map_err(compiler_yul::Error::Target)?;

    let code = if arguments.input.to_string_lossy() == "-" {
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

    let (object, dependencies) =
        compiler_yul::parse_contract(code.as_str(), arguments.contract.as_deref())?;
    let representation = compiler_yul::compile(
        object,
        dependencies,
        target,
        optimization_level,
        optimization_level,
        arguments.dump_llvm,
    )?;

    let text = representation.clone().into_bytes();
    let binary = zkevm_assembly::Assembly::try_from(representation)?;
    let binary = Vec::<u8>::from(&binary);

    let text_file_name = match target {
        compiler_yul::Target::x86 => compiler_common::file_name::LLVM_SOURCE,
        compiler_yul::Target::zkEVM => compiler_common::file_name::ZKEVM_ASSEMBLY,
    };
    let text_file_extension = match target {
        compiler_yul::Target::x86 => compiler_common::extension::LLVM_SOURCE,
        compiler_yul::Target::zkEVM => compiler_common::extension::ZKEVM_ASSEMBLY,
    };
    let text_file_path = PathBuf::from(format!("{}.{}", text_file_name, text_file_extension,));
    File::create(&text_file_path)
        .expect("Text file creating error")
        .write_all(text.as_slice())
        .expect("Text file writing error");

    let binary_file_path = PathBuf::from(compiler_common::file_name::ZKEVM_BINARY);
    File::create(&binary_file_path)
        .expect("Binary file creating error")
        .write_all(binary.as_slice())
        .expect("Binary file writing error");

    Ok(())
}
