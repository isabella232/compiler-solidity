//!
//! YUL to LLVM compiler library.
//!

pub mod error;
pub mod generator;
pub mod lexer;
pub mod parser;

pub use self::error::Error;
pub use self::generator::llvm::Context as LLVMContext;
pub use self::generator::ILLVMWritable;
pub use self::lexer::Lexer;
pub use self::parser::object::Object;

///
/// Removes the metadata put by `solc` at the beginning of the Yul input.
///
pub fn clean(mut input: &str) -> &str {
    input = input.strip_prefix("IR:").unwrap_or(input);
    input = input.strip_prefix("Optimized IR:").unwrap_or(input);
    input
}

///
/// Parses the source code and returns the AST.
///
pub fn parse(input: &str) -> Result<Object, Error> {
    let input = clean(input).to_owned();
    let mut lexer = Lexer::new(input);
    Object::parse(&mut lexer, None)
}

///
/// Parses and compiles the source code.
///
pub fn compile(input: &str, optimization_level: usize) -> Result<String, Error> {
    let object = parse(input)?;

    let optimization_level = match optimization_level {
        0 => inkwell::OptimizationLevel::None,
        1 => inkwell::OptimizationLevel::Less,
        2 => inkwell::OptimizationLevel::Default,
        _ => inkwell::OptimizationLevel::Aggressive,
    };
    let llvm = inkwell::context::Context::create();
    let mut context = LLVMContext::new_with_optimizer(&llvm, optimization_level);
    context.create_module(object.identifier.as_str());
    object.into_llvm(&mut context);
    context.optimize();
    context
        .verify()
        .map_err(|error| Error::LLVM(error.to_string()))?;

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
        .map_err(|error| Error::LLVM(format!("Code compiling error: {}", error)))?;
    let assembly = String::from_utf8_lossy(buffer.as_slice()).to_string();

    Ok(assembly)
}
