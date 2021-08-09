//!
//! YUL to LLVM compiler library.
//!

pub mod error;
pub mod generator;
pub mod lexer;
pub mod parser;
pub mod target;

pub use self::error::Error;
pub use self::generator::llvm::Context as LLVMContext;
pub use self::generator::ILLVMWritable;
pub use self::lexer::Lexer;
pub use self::parser::statement::object::Object;
pub use self::target::Target;

///
/// Parses the source code and returns the AST.
///
pub fn parse(input: &str) -> Result<Object, Error> {
    let mut lexer = Lexer::new(input.to_owned());
    Object::parse(&mut lexer, None)
}

///
/// Parses and compiles the source code.
///
pub fn compile(
    input: &str,
    target: Target,
    optimization_level: usize,
    dump_llvm: bool,
) -> Result<String, Error> {
    let object = parse(input)?;

    let optimization_level = match optimization_level {
        0 => inkwell::OptimizationLevel::None,
        1 => inkwell::OptimizationLevel::Less,
        2 => inkwell::OptimizationLevel::Default,
        _ => inkwell::OptimizationLevel::Aggressive,
    };

    let llvm = inkwell::context::Context::create();
    let target_machine = match target {
        Target::X86 => None,
        Target::zkEVM => {
            inkwell::targets::Target::initialize_syncvm(
                &inkwell::targets::InitializationConfig::default(),
            );
            let target =
                inkwell::targets::Target::from_name(compiler_common::virtual_machine::TARGET_NAME)
                    .ok_or_else(|| {
                        Error::LLVM(format!(
                            "Target `{}` not found",
                            compiler_common::virtual_machine::TARGET_NAME
                        ))
                    })?;
            let target_machine = target
                .create_target_machine(
                    &inkwell::targets::TargetTriple::create(
                        compiler_common::virtual_machine::TARGET_NAME,
                    ),
                    "",
                    "",
                    optimization_level,
                    inkwell::targets::RelocMode::Default,
                    inkwell::targets::CodeModel::Default,
                )
                .ok_or_else(|| {
                    Error::LLVM(format!(
                        "Target machine `{}` creation error",
                        compiler_common::virtual_machine::TARGET_NAME
                    ))
                })?;
            Some(target_machine)
        }
    };
    let mut context =
        LLVMContext::new_with_optimizer(&llvm, target_machine.as_ref(), optimization_level);

    let function_type = match context.target {
        Target::X86 => context
            .integer_type(compiler_common::bitlength::WORD)
            .fn_type(&[], false),
        Target::zkEVM if context.test_entry_hash.is_some() => context
            .integer_type(compiler_common::bitlength::FIELD)
            .fn_type(&[], false),
        Target::zkEVM => context.void_type().fn_type(&[], false),
    };
    context.add_function(
        compiler_common::identifier::FUNCTION_SELECTOR,
        function_type,
        Some(inkwell::module::Linkage::External),
        false,
    );

    object.into_llvm(&mut context);
    context.optimize();
    context
        .verify()
        .map_err(|error| Error::LLVM(error.to_string()))?;
    if dump_llvm || matches!(target, Target::X86) {
        let llvm_code = context.module().print_to_string().to_string();
        if let Target::X86 = target {
            return Ok(llvm_code);
        }
        if dump_llvm {
            eprintln!("The LLVM IR code:\n");
            println!("{}", llvm_code);
        }
    }

    let buffer = target_machine
        .expect("Always exists")
        .write_to_memory_buffer(context.module(), inkwell::targets::FileType::Assembly)
        .map_err(|error| Error::LLVM(format!("Code compiling error: {}", error)))?;
    let assembly = String::from_utf8_lossy(buffer.as_slice()).to_string();

    Ok(assembly)
}
