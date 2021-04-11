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

    /// Sets the LLVM optimization level.
    #[structopt(short = "O", long = "opt-level", default_value = "0")]
    pub optimization_level: usize,
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
