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
pub use self::lexer::lexeme::keyword::Keyword;
pub use self::lexer::lexeme::Lexeme;
pub use self::lexer::Lexer;
pub use self::parser::error::Error as ParserError;
pub use self::parser::statement::object::Object;

use std::collections::HashMap;

///
/// Parses the source code and returns the AST.
///
pub fn parse(input: &str) -> Result<(Object, HashMap<String, Object>), Error> {
    parse_contract(input, None)
}

///
/// Parses the source code and returns the AST.
///
/// If `contract` is specified, the object of that contract is returned. Otherwise, the last object
/// is returned.
///
pub fn parse_contract(
    input: &str,
    contract_path: Option<&str>,
) -> Result<(Object, HashMap<String, Object>), Error> {
    let mut lexer = Lexer::new(input.to_owned());

    let mut main = None;
    let mut dependencies = HashMap::new();
    while let Lexeme::Keyword(Keyword::Object) = lexer.peek()? {
        let object = Object::parse(&mut lexer, None)?;

        if let Some(contract_path) = contract_path
            .map(|path| &path[(path.find(':').map(|offset| offset + 1).unwrap_or_default())..])
        {
            if let Some(position) = object.identifier.rfind('_') {
                if &object.identifier[..position] == contract_path {
                    main = Some(object);
                    continue;
                }
            }
        }

        dependencies.insert(object.identifier.clone(), object);
    }

    if contract_path.is_none() && dependencies.len() == 1 {
        main = dependencies.remove(
            dependencies
                .keys()
                .next()
                .cloned()
                .expect("Always exists")
                .as_str(),
        );
    }

    match (main, dependencies.is_empty(), contract_path) {
        (None, _, _) => Err(ParserError::ContractNotFound.into()),
        (Some(_), false, None) => Err(ParserError::ContractNotSpecified.into()),
        (Some(main), _, _) => Ok((main, dependencies)),
    }
}

///
/// Parses and compiles the source code.
///
pub fn compile(
    object: Object,
    dependencies: HashMap<String, Object>,
    opt_level_llvm_middle: inkwell::OptimizationLevel,
    opt_level_llvm_back: inkwell::OptimizationLevel,
    dump_llvm: bool,
) -> Result<String, Error> {
    let llvm = inkwell::context::Context::create();
    let target_machine =
        compiler_common::vm::target_machine(opt_level_llvm_back).ok_or_else(|| {
            Error::LLVM(format!(
                "Target machine `{}` creation error",
                compiler_common::vm::TARGET_NAME
            ))
        })?;
    let mut context = LLVMContext::new_with_optimizer(
        &llvm,
        &target_machine,
        opt_level_llvm_middle,
        object.identifier.as_str(),
        dependencies,
    );

    object.into_llvm(&mut context);
    context
        .verify()
        .map_err(|error| Error::LLVM(error.to_string()))?;
    context.optimize();
    context
        .verify()
        .map_err(|error| Error::LLVM(error.to_string()))?;
    if dump_llvm {
        let llvm_code = context.module().print_to_string().to_string();
        eprintln!("The LLVM IR code:\n");
        println!("{}", llvm_code);
    }

    let buffer = target_machine
        .write_to_memory_buffer(context.module(), inkwell::targets::FileType::Assembly)
        .map_err(|error| Error::LLVM(format!("Code compiling error: {}", error)))?;
    let llvm_ir = String::from_utf8_lossy(buffer.as_slice()).to_string();

    Ok(llvm_ir)
}
