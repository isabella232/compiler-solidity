//!
//! The Ethereal IR block.
//!

pub mod element;

use std::collections::HashSet;

use crate::evm::assembly::instruction::name::Name as InstructionName;
use crate::evm::assembly::instruction::Instruction;

use self::element::stack::Stack as ElementStack;
use self::element::Element;

///
/// The Ethereal IR block.
///
#[derive(Debug, Clone)]
pub struct Block {
    /// The block unique identifier, represented by a tag in the bytecode.
    /// Is 0 in the initial block.
    pub tag: usize,
    /// The block elements relevant to the stack consistency.
    pub elements: Vec<Element>,
    /// The block predecessors.
    pub predecessors: HashSet<usize>,
    /// The initial stack state.
    pub initial_stack: ElementStack,
    /// The stack.
    pub stack: ElementStack,
}

impl Block {
    /// The elements vector initial capacity.
    pub const ELEMENTS_VECTOR_DEFAULT_CAPACITY: usize = 64;
    /// The predecessors hashset initial capacity.
    pub const PREDECESSORS_HASHSET_DEFAULT_CAPACITY: usize = 4;

    ///
    /// Assembles a block from the sequence of instructions.
    ///
    pub fn try_from_instructions(slice: &[Instruction]) -> anyhow::Result<(Self, usize)> {
        let mut cursor = 0;

        let tag: usize = match slice[cursor].name {
            InstructionName::Tag => {
                let tag = slice[cursor]
                    .value
                    .as_deref()
                    .expect("Always exists")
                    .parse()
                    .expect("Always valid");
                cursor += 1;
                tag
            }
            _ => 0,
        };

        let mut block = Self {
            tag,
            elements: Vec::with_capacity(Self::ELEMENTS_VECTOR_DEFAULT_CAPACITY),
            predecessors: HashSet::with_capacity(Self::PREDECESSORS_HASHSET_DEFAULT_CAPACITY),
            initial_stack: ElementStack::new(),
            stack: ElementStack::new(),
        };

        while cursor < slice.len() {
            let element: Element = slice[cursor].to_owned().into();
            block.elements.push(element);

            match slice[cursor].name {
                InstructionName::RETURN
                | InstructionName::REVERT
                | InstructionName::STOP
                | InstructionName::INVALID
                | InstructionName::JUMP => {
                    cursor += 1;
                    break;
                }
                InstructionName::Tag => {
                    break;
                }
                _ => {
                    cursor += 1;
                }
            }
        }

        Ok((block, cursor))
    }

    ///
    /// Inserts a predecessor tag.
    ///
    pub fn insert_predecessor(&mut self, tag: usize) {
        self.predecessors.insert(tag);
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Block
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        for element in self.elements.into_iter() {
            element.into_llvm(context)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for element in self.elements.iter() {
            write!(f, "    {}", element)?;
        }

        Ok(())
    }
}
