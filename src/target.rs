//!
//! The Yul compiler target.
//!

use std::convert::TryFrom;

///
/// The Yul compiler target.
///
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    /// The middle-end LLVM IR target.
    LLVM,
    /// The zkEVM assembly target.
    zkEVM,
}

impl TryFrom<&str> for Target {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(match input {
            "llvm" => Self::LLVM,
            "zkevm" => Self::zkEVM,

            _ => return Err(input.to_owned()),
        })
    }
}
