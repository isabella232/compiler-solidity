//!
//! Solidity to zkEVM compiler library.
//!

pub(crate) mod dump_flag;
pub(crate) mod evm;
pub(crate) mod project;
pub(crate) mod solc;
pub(crate) mod yul;

pub use self::dump_flag::DumpFlag;
pub use self::project::contract::Contract as ProjectContract;
pub use self::project::Project;
pub use self::solc::combined_json::contract::Contract as SolcCombinedJsonContract;
pub use self::solc::combined_json::CombinedJson as SolcCombinedJson;
pub use self::solc::pipeline::Pipeline as SolcPipeline;
pub use self::solc::standard_json::input::language::Language as SolcStandardJsonInputLanguage;
pub use self::solc::standard_json::input::settings::selection::Selection as SolcStandardJsonInputSettingsSelection;
pub use self::solc::standard_json::input::settings::Settings as SolcStandardJsonInputSettings;
pub use self::solc::standard_json::input::source::Source as SolcStandardJsonInputSource;
pub use self::solc::standard_json::input::Input as SolcStandardJsonInput;
pub use self::solc::standard_json::output::contract::evm::bytecode::Bytecode as SolcStandardJsonOutputContractEVMBytecode;
pub use self::solc::standard_json::output::contract::evm::EVM as SolcStandardJsonOutputContractEVM;
pub use self::solc::standard_json::output::contract::Contract as SolcStandardJsonOutputContract;
pub use self::solc::standard_json::output::Output as SolcStandardJsonOutput;
pub use self::solc::Compiler as SolcCompiler;
