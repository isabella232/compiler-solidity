//!
//! Solidity to LLVM compiler arguments.
//!

use std::path::PathBuf;

use structopt::StructOpt;

///
/// Compiles the given Solidity input files (or the standard input if none given or
/// "-" is used as a file name) and outputs the components specified in the options
/// at standard output or in files in the output directory, if specified.
/// Imports are automatically read from the filesystem.
///
/// Example: zksolc ERC20.sol --optimize --output-dir './build/'
///
#[derive(Debug, StructOpt)]
#[structopt(name = "The zkEVM Solidity compiler")]
pub struct Arguments {
    /// The input file paths.
    #[structopt(parse(from_os_str))]
    pub input_files: Vec<PathBuf>,

    /// Use the given path as the root of the source tree
    /// instead of the root of the filesystem.
    #[structopt(long = "base-path")]
    pub base_path: Option<String>,

    /// Make an additional source directory available to the
    /// default import callback. Use this option if you want to
    /// import contracts whose location is not fixed in relation
    /// to your main source tree, e.g. third-party libraries
    /// installed using a package manager. Can be used multiple
    /// times. Can only be used if base path has a non-empty
    /// value.
    #[structopt(long = "include-path")]
    pub include_paths: Vec<String>,

    /// Allow a given path for imports. A list of paths can be
    /// supplied by separating them with a comma.
    #[structopt(long = "allow-paths")]
    pub allow_paths: Option<String>,

    /// If given, creates one file per component and
    /// contract/file at the specified directory.
    #[structopt(short = "o", long = "output-dir")]
    pub output_directory: Option<PathBuf>,

    /// Overwrite existing files (used together with -o).
    #[structopt(long = "overwrite")]
    pub overwrite: bool,

    /// Enable the LLVM bytecode optimizer.
    #[structopt(long = "optimize")]
    pub optimize: bool,

    /// Direct string or file containing library addresses.
    /// Syntax: <libraryName>=<address> [, or whitespace] ...
    /// Address is interpreted as a hex string prefixed by 0x.
    #[structopt(short = "l", long = "libraries")]
    pub libraries: Vec<String>,

    /// Output a single json document containing the specified information.
    /// Available arguments: abi, hashes
    /// Example: solc --combined-json abi,hashes
    #[structopt(long = "combined-json")]
    pub combined_json: Option<String>,

    /// Output ABI specification of the contracts.
    #[structopt(long = "abi")]
    pub output_abi: bool,

    /// Output function signature hashes of the contracts.
    #[structopt(long = "hashes")]
    pub output_hashes: bool,

    /// Dump Yul Intermediate Representation (IR) of all contracts.
    #[structopt(long = "ir")]
    pub dump_yul: bool,

    /// Dump LLVM Intermediate Representation (IR) of all contracts.
    #[structopt(long = "llvm")]
    pub dump_llvm: bool,

    /// Output zkEVM assembly of the contracts.
    #[structopt(long = "asm")]
    pub output_assembly: bool,

    /// Output zkEVM bytecode of the contracts.
    #[structopt(long = "bin")]
    pub output_binary: bool,
}

impl Arguments {
    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self::from_args()
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self::new()
    }
}
