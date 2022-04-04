//!
//! The `solc --asm-json` output representation.
//!

pub mod data;
pub mod instruction;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::evm::ethereal_ir::entry_link::EntryLink;
use crate::evm::ethereal_ir::EtherealIR;

use self::data::Data;
use self::instruction::name::Name as InstructionName;
use self::instruction::Instruction;

///
/// The JSON assembly representation.
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Assembly {
    /// The metadata string.
    #[serde(rename = ".auxdata")]
    pub auxdata: Option<String>,
    /// The constructor code instructions.
    #[serde(rename = ".code")]
    pub code: Option<Vec<Instruction>>,
    /// The runtime code.
    #[serde(rename = ".data")]
    pub data: Option<BTreeMap<String, Data>>,
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Assembly
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let mut entry = compiler_llvm_context::EntryFunction::default();
        entry.declare(context)?;

        compiler_llvm_context::ConstructorFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;
        compiler_llvm_context::SelectorFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;

        entry.into_llvm(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EVM) {
            println!("Constructor EVM:");
            println!("{}", self);
        }
        let mut constructor_ethereal_ir = EtherealIR::new(
            context.evm().version.to_owned(),
            self.code
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Constructor instructions not found"))?,
            compiler_llvm_context::CodeType::Deploy,
        )?;
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EthIR) {
            println!("Constructor Ethereal IR:\n\n{}", constructor_ethereal_ir);
        }
        let constructor = compiler_llvm_context::ConstructorFunction::new(EntryLink::new(
            compiler_llvm_context::CodeType::Deploy,
        ));
        constructor_ethereal_ir.declare(context)?;
        constructor.into_llvm(context)?;
        constructor_ethereal_ir.into_llvm(context)?;

        let data = self
            .data
            .ok_or_else(|| anyhow::anyhow!("Runtime data not found"))?
            .remove("0")
            .expect("Always exists");
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EVM) {
            println!("Runtime EVM:");
            println!("{}", data);
        };
        let runtime_instructions = match data {
            Data::Assembly(assembly) => assembly
                .code
                .ok_or_else(|| anyhow::anyhow!("Runtime instructions not found"))?,
            Data::Hash(hash) => {
                anyhow::bail!("Expected runtime instructions, found hash `{}`", hash)
            }
        };
        let mut runtime_ethereal_ir = EtherealIR::new(
            context.evm().version.to_owned(),
            runtime_instructions.as_slice(),
            compiler_llvm_context::CodeType::Runtime,
        )?;
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EthIR) {
            println!("Runtime Ethereal IR:\n\n{}", runtime_ethereal_ir);
        }
        let selector = compiler_llvm_context::SelectorFunction::new(EntryLink::new(
            compiler_llvm_context::CodeType::Runtime,
        ));
        runtime_ethereal_ir.declare(context)?;
        selector.into_llvm(context)?;
        runtime_ethereal_ir.into_llvm(context)?;

        Ok(())
    }
}

impl std::fmt::Display for Assembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(instructions) = self.code.as_ref() {
            for (index, instruction) in instructions.iter().enumerate() {
                match instruction.name {
                    InstructionName::Tag => writeln!(f, "{:03} {}", index, instruction)?,
                    _ => writeln!(f, "{:03}     {}", index, instruction)?,
                }
            }
        }

        Ok(())
    }
}
