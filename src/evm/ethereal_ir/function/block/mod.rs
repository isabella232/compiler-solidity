//!
//! The Ethereal IR block.
//!

pub mod element;
pub mod exit;
pub mod jump;

use std::collections::HashSet;

use crate::evm::assembly::instruction::name::Name as InstructionName;
use crate::evm::assembly::instruction::Instruction;

use self::element::kind::Kind as ElementKind;
use self::element::stack_data::StackData as ElementStackData;
use self::element::Element;
use self::exit::Exit;
use self::jump::Jump;

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
    /// The block exit data, if needed.
    pub exit: Option<Exit>,
    /// The conditional jumps.
    pub jumps: Vec<Jump>,
    /// The unused tags, which are usually return addresses.
    pub tags: Vec<usize>,
    /// The block predecessors.
    pub predecessors: HashSet<usize>,
    /// The block call trace, that is, all function call return addresses on the way to it.
    pub call_trace: Vec<usize>,
    /// The block vertical tags buffer.
    pub vertical_tags_buffer: Vec<usize>,
    /// The initial stack offset.
    pub initial_stack_offset: Option<isize>,
    /// The final stack offset.
    pub final_stack_offset: Option<isize>,
    /// The deepest stack offset.
    pub deepest_stack_offset: isize,
    /// The highest stack size.
    pub highest_stack_size: usize,
}

impl Block {
    /// The elements vector initial capacity.
    pub const ELEMENTS_VECTOR_DEFAULT_CAPACITY: usize = 64;
    /// The jumps vector initial capacity.
    pub const JUMPS_VECTOR_DEFAULT_CAPACITY: usize = 4;
    /// The tags vector initial capacity.
    pub const TAGS_VECTOR_DEFAULT_CAPACITY: usize = 4;
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
            exit: None,
            jumps: Vec::with_capacity(Self::JUMPS_VECTOR_DEFAULT_CAPACITY),
            tags: Vec::with_capacity(Self::TAGS_VECTOR_DEFAULT_CAPACITY),
            predecessors: HashSet::with_capacity(Self::PREDECESSORS_HASHSET_DEFAULT_CAPACITY),
            call_trace: vec![],
            vertical_tags_buffer: vec![],
            initial_stack_offset: None,
            final_stack_offset: None,
            deepest_stack_offset: 0,
            highest_stack_size: 0,
        };

        while cursor < slice.len() {
            match slice[cursor].name {
                InstructionName::JUMPDEST => {}
                InstructionName::PUSH_Tag => {
                    let tag = slice[cursor]
                        .value
                        .as_deref()
                        .expect("Always exists")
                        .parse()
                        .expect("Always valid");

                    match slice[cursor + 1].name {
                        InstructionName::JUMP
                            if slice[cursor + 1].value.as_deref() == Some("[in]") =>
                        {
                            block.exit = Some(Exit::call(tag));
                            cursor += 2;
                            break;
                        }
                        InstructionName::JUMPI => {
                            block
                                .elements
                                .push(ElementKind::conditional_jump(tag, vec![]).into());
                            block.jumps.push(Jump::new(
                                tag,
                                block.tags.clone(),
                                block.elements.len() - 1,
                            ));
                            cursor += 1;
                        }
                        _ => {
                            block.tags.push(tag);
                            block
                                .elements
                                .push(ElementKind::Instruction(slice[cursor].to_owned()).into());
                        }
                    }
                }
                InstructionName::JUMP => {
                    block.exit = match slice[cursor].value.as_deref() {
                        Some("[out]") => Some(Exit::Return),
                        Some("[in]") => {
                            block
                                .elements
                                .push(ElementKind::Instruction(InstructionName::POP.into()).into());
                            let tag = block
                                .tags
                                .pop()
                                .ok_or_else(|| anyhow::anyhow!("Function call tag is missing"))?;
                            Some(Exit::call(tag))
                        }
                        _ => Some(Exit::Unconditional),
                    };
                    cursor += 1;
                    break;
                }
                InstructionName::RETURN
                | InstructionName::REVERT
                | InstructionName::STOP
                | InstructionName::INVALID => {
                    block
                        .elements
                        .push(ElementKind::Instruction(slice[cursor].to_owned()).into());
                    cursor += 1;
                    break;
                }
                InstructionName::Tag => {
                    let tag = slice[cursor]
                        .value
                        .as_deref()
                        .expect("Always exists")
                        .parse()
                        .expect("Always valid");
                    block.exit = Some(Exit::fallthrough(tag));
                    break;
                }
                _ => block
                    .elements
                    .push(ElementKind::Instruction(slice[cursor].to_owned()).into()),
            }
            cursor += 1;
        }

        Ok((block, cursor))
    }

    ///
    /// Sets the stack data.
    ///
    pub fn set_stack_data(&mut self, initial_offset: isize) -> isize {
        if let Some(final_stack_offset) = self.final_stack_offset {
            return final_stack_offset;
        }

        self.initial_stack_offset = Some(initial_offset);
        let mut offset = initial_offset;
        for element in self.elements.iter_mut() {
            let difference = offset - (element.kind.stack_depth() as isize);
            if difference < self.deepest_stack_offset {
                self.deepest_stack_offset = difference;
            }
            offset -= element.kind.input_size() as isize;
            if let Some(output_size) = element.kind.output_size() {
                offset += output_size as isize;
            }
            element.set_stack_data(ElementStackData::new(offset));
        }
        self.final_stack_offset = Some(offset);
        offset
    }

    ///
    /// Normalizes the stack data offsets.
    ///
    pub fn normalize_stack(&mut self, added_offset: isize) {
        if let Some(ref mut initial_stack_offset) = self.initial_stack_offset {
            *initial_stack_offset += added_offset;
        }
        if let Some(ref mut final_stack_offset) = self.final_stack_offset {
            *final_stack_offset += added_offset;
        }

        for element in self.elements.iter_mut() {
            if let Some(ref mut stack_data) = element.stack_data {
                stack_data.current += added_offset;
                if stack_data.current > self.highest_stack_size as isize {
                    self.highest_stack_size = stack_data.current as usize;
                }
            }
        }
    }

    ///
    /// Sets the vertical tags buffer for the specified conditional jump instruction.
    ///
    pub fn set_vertical_tags_buffer(
        &mut self,
        position: usize,
        tags: Vec<usize>,
    ) -> anyhow::Result<()> {
        match self
            .elements
            .get_mut(position)
            .map(|element| &mut element.kind)
        {
            Some(ElementKind::ConditionalJump {
                ref mut vertical_tags_buffer,
                ..
            }) => {
                *vertical_tags_buffer = tags;
                Ok(())
            }
            _ => anyhow::bail!("Conditional jump at index {} not found", position),
        }
    }

    ///
    /// Validates predecessor.
    ///
    /// If the predecessor is present, `false` is returned.
    /// If not, it is inserted, and `true` is returned.
    ///
    pub fn validate_predecessor(&mut self, predecessor: Option<usize>) -> bool {
        if let Some(predecessor) = predecessor {
            if self.has_predecessor(predecessor) {
                return false;
            }

            self.insert_predecessor(predecessor);
        }

        true
    }

    ///
    /// Inserts a predecessor tag.
    ///
    pub fn insert_predecessor(&mut self, tag: usize) {
        self.predecessors.insert(tag);
    }

    ///
    /// Whether the block has the predecessor.
    ///
    pub fn has_predecessor(&self, tag: usize) -> bool {
        self.predecessors.contains(&tag)
    }

    ///
    /// Whether the block is a function return.
    ///
    pub fn is_function_return(&self) -> bool {
        if let Some(Exit::Return) = self.exit {
            return true;
        }

        if let Some(ElementKind::Return) = self.elements.last().map(|element| &element.kind) {
            return true;
        }

        false
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Block
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        if let Some(initial_stack_offset) = self.initial_stack_offset {
            context.evm_mut().stack_offset = initial_stack_offset as usize;
        }

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
