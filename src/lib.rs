//!
//! YUL to LLVM compiler library.
//!

pub(crate) mod error;
pub(crate) mod generator;
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod project;
pub(crate) mod solc;

pub use self::error::Error;
pub use self::generator::llvm::Context as LLVMContext;
pub use self::generator::ILLVMWritable;
pub use self::lexer::lexeme::keyword::Keyword;
pub use self::lexer::lexeme::Lexeme;
pub use self::lexer::Lexer;
pub use self::parser::error::Error as ParserError;
pub use self::parser::statement::object::Object;
pub use self::project::contract::Contract as ProjectContract;
pub use self::project::Project;
pub use self::solc::combined_json::contract::Contract as SolcCombinedJsonContract;
pub use self::solc::combined_json::CombinedJson as SolcCombinedJson;
pub use self::solc::standard_json::input::settings::Settings as SolcStandardJsonInputSettings;
pub use self::solc::standard_json::input::source::Source as SolcStandardJsonInputSource;
pub use self::solc::standard_json::input::Input as SolcStandardJsonInput;
pub use self::solc::standard_json::output::contract::Contract as SolcStandardJsonOutputContract;
pub use self::solc::standard_json::output::Output as SolcStandardJsonOutput;
pub use self::solc::Compiler as SolcCompiler;

///
/// Initializes the zkEVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_syncvm(&inkwell::targets::InitializationConfig::default());
}

///
/// Returns the zkEVM target machine instance.
///
pub fn target_machine(
    optimization_level: inkwell::OptimizationLevel,
) -> Option<inkwell::targets::TargetMachine> {
    inkwell::targets::Target::from_name(compiler_common::VM_TARGET_NAME)?.create_target_machine(
        &inkwell::targets::TargetTriple::create(compiler_common::VM_TARGET_NAME),
        "",
        "",
        optimization_level,
        inkwell::targets::RelocMode::Default,
        inkwell::targets::CodeModel::Default,
    )
}
