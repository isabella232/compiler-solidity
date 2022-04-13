//!
//! Solidity to zkEVM compiler arguments.
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

    /// Path to the `solc` executable.
    /// By default, the one in $PATH is used.
    #[structopt(long = "solc")]
    pub solc: Option<String>,

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

    /// Switch to Standard JSON input / output mode.
    /// Reads from stdin, result is written to stdout.
    #[structopt(long = "standard-json")]
    pub standard_json: bool,

    /// Switch to Yul mode.
    #[structopt(long = "yul")]
    pub yul: bool,

    /// Output ABI specification of the contracts.
    #[structopt(long = "abi")]
    pub output_abi: bool,

    /// Output function signature hashes of the contracts.
    #[structopt(long = "hashes")]
    pub output_hashes: bool,

    /// Output zkEVM assembly of the contracts.
    #[structopt(long = "asm")]
    pub output_assembly: bool,

    /// Output zkEVM bytecode of the contracts.
    #[structopt(long = "bin")]
    pub output_binary: bool,

    /// Dump the Yul Intermediate Representation (IR) of all contracts.
    #[structopt(long = "dump-yul")]
    pub dump_yul: bool,

    /// Dump the EVM legacy assembly Intermediate Representation (IR) of all contracts.
    #[structopt(long = "dump-evm")]
    pub dump_evm: bool,

    /// Dump the Ethereal Intermediate Representation (IR) of all contracts.
    #[structopt(long = "dump-ethir")]
    pub dump_ethir: bool,

    /// Dump the LLVM Intermediate Representation (IR) of all contracts.
    #[structopt(long = "dump-llvm")]
    pub dump_llvm: bool,

    /// Dump the zkEVM assembly of all contracts.
    #[structopt(long = "dump-assembly")]
    pub dump_assembly: bool,
}

impl Arguments {
    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self::from_args()
    }

    ///
    /// Validates the arguments.
    ///
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.yul {
            if self.combined_json.is_some() {
                anyhow::bail!("The following options are invalid in Yul mode: --combined-json.");
            }
            if self.standard_json {
                anyhow::bail!("The following options are invalid in Yul mode: --standard-json.");
            }
            if self.output_abi {
                anyhow::bail!("The following options are invalid in Yul mode: --abi.");
            }
            if self.output_hashes {
                anyhow::bail!("The following options are invalid in Yul mode: --hashes.");
            }
        }

        Ok(())
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self::new()
    }
}
