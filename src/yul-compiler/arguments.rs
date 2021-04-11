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
