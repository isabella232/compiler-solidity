//!
//! YUL to LLVM compiler arguments.
//!

use std::path::PathBuf;

use structopt::StructOpt;

///
/// YUL to LLVM compiler arguments.
///
#[derive(Debug, StructOpt)]
#[structopt(name = "The YUL-to-LLVM compiler")]
pub struct Arguments {
    /// The input file path.
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    /// The output directory path.
    #[structopt(short = "o", long = "output", default_value = "./build/")]
    pub output: PathBuf,

    /// Sets the LLVM optimization level.
    #[structopt(short = "O", long = "opt-level", default_value = "3")]
    pub optimization_level: usize,

    /// Whether to dump the Yul code.
    #[structopt(long = "dump-yul")]
    pub dump_yul: bool,

    /// Whether to dump the LLVM code to the terminal.
    #[structopt(long = "dump-llvm")]
    pub dump_llvm: bool,

    /// The hashmap of colon-separated library paths and their ETH addresses.
    #[structopt(short = "l", long = "library")]
    pub libraries: Vec<String>,
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
