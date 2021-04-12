//!
//! YUL to LLVM compiler library.
//!

pub mod error;
pub mod generator;
pub mod lexer;
pub mod parser;

pub use self::error::Error;
pub use self::generator::action::Action;
pub use self::generator::llvm::Context as LLVMContext;
pub use self::generator::ILLVMWritable;
pub use self::lexer::Lexer;
pub use self::parser::Module;

///
/// Parses the source code and returns the AST.
///
pub fn parse(input: &str) -> Result<Module, Error> {
    Module::parse(&mut Lexer::new(input.to_owned()), None)
}

///
/// Parses and compiles the source code.
///
pub fn compile(input: &str, optimization_level: usize) -> Result<String, Error> {
    let module = parse(input)?;

    let optimization_level = match optimization_level {
        0 => inkwell::OptimizationLevel::None,
        1 => inkwell::OptimizationLevel::Less,
        2 => inkwell::OptimizationLevel::Default,
        _ => inkwell::OptimizationLevel::Aggressive,
    };
    let llvm = inkwell::context::Context::create();
    let mut context = LLVMContext::new_with_optimizer(&llvm, optimization_level);
    module.into_llvm(&mut context);
    context.optimize();
    context.verify().expect("Verification error");

    inkwell::targets::Target::initialize_syncvm(&inkwell::targets::InitializationConfig::default());
    let target = inkwell::targets::Target::from_name(compiler_const::virtual_machine::TARGET_NAME)
        .ok_or_else(|| {
            Error::LLVM(format!(
                "Target `{}` not found",
                compiler_const::virtual_machine::TARGET_NAME
            ))
        })?;
    let target_machine = target
        .create_target_machine(
            &inkwell::targets::TargetTriple::create(compiler_const::virtual_machine::TARGET_NAME),
            "",
            "",
            optimization_level,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .ok_or_else(|| {
            Error::LLVM(format!(
                "Target machine `{}` creation error",
                compiler_const::virtual_machine::TARGET_NAME
            ))
        })?;
    let buffer = target_machine
        .write_to_memory_buffer(&context.module, inkwell::targets::FileType::Assembly)
        .map_err(|error| Error::LLVM(format!("Module compiling error: {}", error)))?;
    let assembly = String::from_utf8_lossy(buffer.as_slice()).to_string();

    Ok(assembly)
}
