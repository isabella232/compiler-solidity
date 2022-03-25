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
    /// The constructor code instructions.
    #[serde(rename = ".code")]
    pub code: Vec<Instruction>,
    /// The runtime code.
    #[serde(rename = ".data")]
    pub data: BTreeMap<String, Data>,
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

    fn into_llvm(mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EVM) {
            println!("Constructor EVM:");
            println!("{}", self);
        }
        let mut constructor_ethereal_ir =
            EtherealIR::try_from_instructions(self.code, compiler_llvm_context::CodeType::Deploy)?;
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EthIR) {
            println!("Constructor Ethereal IR:\n\n{}", constructor_ethereal_ir);
        }
        let constructor = compiler_llvm_context::ConstructorFunction::new(EntryLink::new(
            compiler_llvm_context::CodeType::Deploy,
        ));
        constructor_ethereal_ir.declare(context)?;
        constructor.into_llvm(context)?;
        constructor_ethereal_ir.into_llvm(context)?;

        let data = self.data.remove("0").expect("Always exists");
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EVM) {
            println!("Runtime EVM:");
            println!("{}", data);
        };
        let mut runtime_ethereal_ir = EtherealIR::try_from_instructions(
            data.try_into_instructions()?,
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
        for (index, instruction) in self.code.iter().enumerate() {
            match instruction.name {
                InstructionName::Tag => writeln!(f, "{:03} {}", index, instruction)?,
                _ => writeln!(f, "{:03}     {}", index, instruction)?,
            }
        }

        Ok(())
    }
}
