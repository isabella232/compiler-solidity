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
    /// The x86 target.
    X86,
    /// The zkEVM assembly target.
    zkEVM,
}

impl TryFrom<&str> for Target {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(match input {
            "llvm" => Self::X86,
            "x86" => Self::X86,
            "zkevm" => Self::zkEVM,

            _ => return Err(input.to_owned()),
        })
    }
}

impl From<Option<&inkwell::targets::TargetMachine>> for Target {
    fn from(machine: Option<&inkwell::targets::TargetMachine>) -> Self {
        match machine {
            Some(machine) => {
                if machine.get_target().get_name().to_string_lossy().as_ref()
                    == compiler_const::virtual_machine::TARGET_NAME
                {
                    Self::zkEVM
                } else {
                    Self::X86
                }
            }
            None => Self::X86,
        }
    }
}
