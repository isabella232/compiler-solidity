//!
//! The Ethereal IR representation of the EVM bytecode.
//!

pub mod entry_link;
pub mod function;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::evm::assembly::instruction::Instruction;

use self::function::block::Block;
use self::function::Function;

///
/// The Ethereal IR representation of the EVM bytecode.
///
#[derive(Debug)]
pub struct EtherealIR {
    /// The function representations.
    pub functions: BTreeMap<usize, Function>,
    /// The contract code part type.
    pub code_type: compiler_llvm_context::CodeType,
}

impl EtherealIR {
    /// The blocks hashmap initial capacity.
    pub const BLOCKS_HASHMAP_DEFAULT_CAPACITY: usize = 64;

    ///
    /// Assembles a sequence of functions from the sequence of instructions.
    ///
    pub fn try_from_instructions(
        instructions: Vec<Instruction>,
        code_type: compiler_llvm_context::CodeType,
    ) -> anyhow::Result<Self> {
        let mut blocks = HashMap::with_capacity(Self::BLOCKS_HASHMAP_DEFAULT_CAPACITY);
        let mut offset = 0;

        while offset < instructions.len() {
            let (block, size) = Block::try_from_instructions(&instructions[offset..])?;
            blocks.insert(block.tag, block);
            offset += size;
        }

        let mut visited = HashSet::with_capacity(blocks.len());
        let mut functions = BTreeMap::new();
        let function = Function::try_from_blocks(code_type, &blocks, &mut visited, &mut functions)?;
        let function = function.finalize();
        functions.insert(0, function);

        Ok(Self {
            functions,
            code_type,
        })
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for EtherealIR
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        context.code_type = Some(self.code_type);

        for (_tag, function) in self.functions.iter_mut() {
            function.declare(context)?;
        }

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        context.evm_mut().stack = vec![];

        for (_tag, function) in self.functions.into_iter() {
            function.into_llvm(context)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for EtherealIR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_tag, function) in self.functions.iter() {
            writeln!(f, "{}", function)?;
        }

        Ok(())
    }
}
