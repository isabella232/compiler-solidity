//!
//! The Ethereal IR representation of the EVM bytecode.
//!

pub mod entry_link;
pub mod function;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::evm::assembly::instruction::Instruction;

use self::function::block::exit::Exit as BlockExit;
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
    /// The entry function tag.
    pub const ENTRY_FUNCTION_TAG: usize = 0;

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

        let mut entries: BTreeSet<usize> = blocks
            .iter()
            .filter_map(|(_tag, block)| match block.exit {
                Some(BlockExit::Call { callee, .. }) => Some(callee),
                _ => None,
            })
            .collect();
        entries.insert(Self::ENTRY_FUNCTION_TAG);

        let mut visited = HashSet::with_capacity(blocks.len());
        let mut functions = BTreeMap::new();
        let function = Function::try_from_tag(
            Self::ENTRY_FUNCTION_TAG,
            code_type,
            &blocks,
            &entries,
            &mut visited,
            &mut functions,
        )?;
        functions.insert(Self::ENTRY_FUNCTION_TAG, function);

        if visited.len() != blocks.len() {
            anyhow::bail!(
                "Not all blocks are visited: {} out of {}",
                visited.len(),
                blocks.len()
            );
        }

        let mut call_graph = Vec::with_capacity(functions.len());
        for (tag, function) in functions.iter() {
            call_graph.push((*tag, function.callees.to_owned()));
        }
        for (caller, callees) in call_graph.into_iter() {
            for callee in callees.into_iter() {
                functions
                    .get_mut(&callee)
                    .expect("Always exists")
                    .insert_caller(caller);
            }
        }

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
