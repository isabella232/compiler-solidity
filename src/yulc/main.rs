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

    let representation = yul_compiler::compile(
        &code,
        target,
        arguments.optimization_level,
        arguments.dump_llvm,
    )?;
    let representation = if arguments.binary {
        let assembly = zkevm_assembly::Assembly::try_from(representation)?;
        Vec::<u8>::from(&assembly)
    } else {
        representation.into_bytes()
    };
    let target_extension = match target {
        yul_compiler::Target::LLVM => compiler_const::extension::LLVM,
        yul_compiler::Target::zkEVM if arguments.binary => "",
        yul_compiler::Target::zkEVM => compiler_const::extension::ASSEMBLY,
    };
    let target_build_path = PathBuf::from(format!(
        "{}{}{}",
        compiler_const::file_name::ASSEMBLY,
        if target_extension.is_empty() { "" } else { "." },
        target_extension,
    ));
    File::create(&target_build_path)
        .expect("File creating error")
        .write_all(representation.as_slice())
        .expect("File writing error");

    Ok(())
}
