//!
//! The Ethereal IR representation of the EVM bytecode.
//!

pub mod entry_link;
pub mod function;

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
    /// The Solidity compiler version.
    pub solc_version: semver::Version,
    /// The contract code part type.
    pub code_type: compiler_llvm_context::CodeType,
    /// The all-inlined function representation.
    pub function: Function,
}

impl EtherealIR {
    /// The `deploy address` immutable storage key.
    pub const DEPLOY_ADDRESS_STORAGE_KEY: &'static str = "deploy_address";

    /// The blocks hashmap initial capacity.
    pub const BLOCKS_HASHMAP_DEFAULT_CAPACITY: usize = 64;

    ///
    /// Assembles a sequence of functions from the sequence of instructions.
    ///
    pub fn new(
        solc_version: semver::Version,
        instructions: &[Instruction],
        code_type: compiler_llvm_context::CodeType,
    ) -> anyhow::Result<Self> {
        let mut blocks = HashMap::with_capacity(Self::BLOCKS_HASHMAP_DEFAULT_CAPACITY);
        let mut offset = 0;

        while offset < instructions.len() {
            let (block, size) =
                Block::try_from_instructions(solc_version.clone(), &instructions[offset..])?;
            blocks.insert(block.tag, block);
            offset += size;
        }

        let mut visited = HashSet::with_capacity(blocks.len());
        let function = Function::new(solc_version.clone(), code_type, &blocks, &mut visited)?;

        Ok(Self {
            solc_version,
            code_type,
            function,
        })
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for EtherealIR
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        context.code_type = Some(self.code_type);

        self.function.declare(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        context.evm_mut().stack = vec![];

        self.function.into_llvm(context)?;

        Ok(())
    }
}

impl std::fmt::Display for EtherealIR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.function)?;

        Ok(())
    }
}
