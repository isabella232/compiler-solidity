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
pub use self::lexer::lexeme::keyword::Keyword;
pub use self::lexer::lexeme::Lexeme;
pub use self::lexer::Lexer;
pub use self::parser::error::Error as ParserError;
pub use self::parser::statement::object::Object;
pub use self::target::Target;

///
/// Parses the source code and returns the AST.
///
pub fn parse(input: &str) -> Result<Object, Error> {
    let mut lexer = Lexer::new(input.to_owned());

    let mut objects = Vec::with_capacity(1);
    while let Lexeme::Keyword(Keyword::Object) = lexer.peek()? {
        objects.push(Object::parse(&mut lexer, None)?);
    }
    Ok(objects.pop().expect("No objects found in the input"))
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
        Target::x86 => None,
        Target::zkEVM => {
            let target_machine = compiler_common::vm::target_machine(optimization_level)
                .ok_or_else(|| {
                    Error::LLVM(format!(
                        "Target machine `{}` creation error",
                        compiler_common::vm::TARGET_NAME
                    ))
                })?;
            Some(target_machine)
        }
    };
    let mut context =
        LLVMContext::new_with_optimizer(&llvm, target_machine.as_ref(), optimization_level);

    object.into_llvm(&mut context);
    context.optimize();
    context
        .verify()
        .map_err(|error| Error::LLVM(error.to_string()))?;
    if dump_llvm || matches!(target, Target::x86) {
        let llvm_code = context.module().print_to_string().to_string();
        if let Target::x86 = target {
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
