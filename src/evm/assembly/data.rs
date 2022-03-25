//!
//! The JSON assembly runtime code representation.
//!

use serde::Deserialize;
use serde::Serialize;

use crate::evm::assembly::instruction::name::Name as InstructionName;
use crate::evm::assembly::instruction::Instruction;

///
/// The JSON assembly runtime code representation.
///
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Data {
    /// The assembly code wrapper.
    Code {
        /// The runtime code metadata.
        #[serde(rename = ".auxdata")]
        auxdata: Option<String>,
        /// The runtime code instructions.
        #[serde(rename = ".code")]
        code: Vec<Instruction>,
    },
    /// The hash representation.
    Hash(String),
}

impl Data {
    ///
    /// Converts the data into code.
    ///
    pub fn try_into_instructions(self) -> anyhow::Result<Vec<Instruction>> {
        match self {
            Self::Code { code, .. } => Ok(code),
            Self::Hash(value) => {
                anyhow::bail!("Assembly code is missing, found hash `{}`", value)
            }
        }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code { code, .. } => {
                for (index, instruction) in code.iter().enumerate() {
                    match instruction.name {
                        InstructionName::Tag => writeln!(f, "{:03} {}", index, instruction)?,
                        _ => writeln!(f, "{:03}     {}", index, instruction)?,
                    }
                }
            }
            Self::Hash(value) => writeln!(f, "{}", value)?,
        }

        Ok(())
    }
}
