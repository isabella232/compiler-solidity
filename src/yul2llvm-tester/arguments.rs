//!
//! The YUL to LLVM tester arguments.
//!

use std::path::PathBuf;
use structopt::StructOpt;

///
/// The YUL to LLVM tester arguments.
///
#[derive(Debug, StructOpt)]
#[structopt(
    name = "Solidity tester",
    about = "The integration test runner for Solidity to SyncVM compiler"
)]
pub struct Arguments {
    /// Input file or directory
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
