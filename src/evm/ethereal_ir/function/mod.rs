//!
//! The Ethereal IR function.
//!

pub mod block;
pub mod queue_element;
pub mod visited_element;

use inkwell::values::BasicValue;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::evm::assembly::instruction::name::Name as InstructionName;
use crate::evm::assembly::instruction::Instruction;
use crate::evm::ethereal_ir::function::block::element::stack::element::Element;
use crate::evm::ethereal_ir::function::block::element::stack::Stack;

use self::block::element::stack::element::Element as StackElement;
use self::block::element::Element as BlockElement;
use self::block::Block;
use self::queue_element::QueueElement;
use self::visited_element::VisitedElement;

///
/// The Ethereal IR function.
///
#[derive(Debug, Clone)]
pub struct Function {
    /// The Solidity compiler version.
    pub solc_version: semver::Version,
    /// The contract part where the function belongs.
    pub code_type: compiler_llvm_context::CodeType,
    /// The separately labelled blocks.
    pub blocks: BTreeMap<usize, Vec<Block>>,
    /// The function stack size.
    pub stack_size: usize,
}

impl Function {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        solc_version: semver::Version,
        code_type: compiler_llvm_context::CodeType,
        blocks: &HashMap<usize, Block>,
        visited: &mut HashSet<VisitedElement>,
    ) -> anyhow::Result<Self> {
        let mut function = Self {
            solc_version,
            code_type,
            blocks: BTreeMap::new(),
            stack_size: 0,
        };
        function.consume_block(blocks, visited, QueueElement::new(0, None, Stack::new()))?;
        Ok(function.finalize())
    }

    ///
    /// Consumes the entry or a conditional block attached to another one.
    ///
    fn consume_block(
        &mut self,
        blocks: &HashMap<usize, Block>,
        visited: &mut HashSet<VisitedElement>,
        mut queue_element: QueueElement,
    ) -> anyhow::Result<()> {
        let version = self.solc_version.to_owned();

        let mut queue = vec![];

        let visited_element = VisitedElement::new(queue_element.tag, queue_element.stack.hash());
        if visited.contains(&visited_element) {
            return Ok(());
        }
        visited.insert(visited_element);

        let mut block = blocks
            .get(&queue_element.tag)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Undeclared destination block {}", queue_element.tag))?;
        block.initial_stack = queue_element.stack.clone();
        let block = self.insert_block(block);
        block.stack = block.initial_stack.clone();
        if let Some(predecessor) = queue_element.predecessor.take() {
            block.insert_predecessor(predecessor);
        }

        let mut block_size = 0;
        for block_element in block.elements.iter_mut() {
            block_size += 1;

            if Self::handle_instruction(
                &mut block.stack,
                block_element,
                &version,
                &mut queue,
                &mut queue_element,
            )
            .is_err()
            {
                block_element.stack = block.stack.clone();
                block_element.instruction = Instruction::invalid();
                break;
            }
        }
        block.elements.truncate(block_size);

        for element in queue.into_iter() {
            self.consume_block(blocks, visited, element)?;
        }

        Ok(())
    }

    ///
    /// Processes an instruction, returning an error, if there is an invalid stack state.
    ///
    /// The blocks with an invalid stack state are considered being partially unreachable, and
    /// the invalid part is truncated after terminating with an `INVALID` instruction.
    ///
    fn handle_instruction(
        block_stack: &mut Stack,
        block_element: &mut BlockElement,
        version: &semver::Version,
        queue: &mut Vec<QueueElement>,
        queue_element: &mut QueueElement,
    ) -> anyhow::Result<()> {
        match block_element.instruction {
            Instruction {
                name: InstructionName::PUSH_Tag,
                value: Some(ref mut tag),
            } => {
                let element = match tag.parse() {
                    Ok(tag) => Element::Tag(tag),
                    Err(_) => {
                        *tag = "0".to_owned();
                        Element::Tag(0)
                    }
                };
                block_stack.push(element);

                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::JUMP,
                ..
            } => {
                queue_element.predecessor = Some(queue_element.tag);

                block_element.stack = block_stack.clone();
                let destination = block_stack.pop_tag()?;
                queue.push(QueueElement::new(
                    destination,
                    queue_element.predecessor,
                    block_stack.to_owned(),
                ));
            }
            Instruction {
                name: InstructionName::JUMPI,
                ..
            } => {
                queue_element.predecessor = Some(queue_element.tag);

                block_element.stack = block_stack.clone();
                let destination = block_stack.pop_tag()?;
                block_stack.pop()?;
                queue.push(QueueElement::new(
                    destination,
                    queue_element.predecessor,
                    block_stack.to_owned(),
                ));
            }
            Instruction {
                name: InstructionName::Tag,
                value: Some(ref destination),
            } => {
                block_element.stack = block_stack.clone();

                let destination: usize = destination.parse().expect("Always valid");
                queue_element.predecessor = Some(queue_element.tag);
                queue_element.tag = destination;
                queue.push(QueueElement::new(
                    destination,
                    queue_element.predecessor,
                    block_stack.to_owned(),
                ));
            }

            Instruction {
                name: InstructionName::SWAP1,
                ..
            } => {
                block_stack.swap(1)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP2,
                ..
            } => {
                block_stack.swap(2)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP3,
                ..
            } => {
                block_stack.swap(3)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP4,
                ..
            } => {
                block_stack.swap(4)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP5,
                ..
            } => {
                block_stack.swap(5)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP6,
                ..
            } => {
                block_stack.swap(6)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP7,
                ..
            } => {
                block_stack.swap(7)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP8,
                ..
            } => {
                block_stack.swap(8)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP9,
                ..
            } => {
                block_stack.swap(9)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP10,
                ..
            } => {
                block_stack.swap(10)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP11,
                ..
            } => {
                block_stack.swap(11)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP12,
                ..
            } => {
                block_stack.swap(12)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP13,
                ..
            } => {
                block_stack.swap(13)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP14,
                ..
            } => {
                block_stack.swap(14)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP15,
                ..
            } => {
                block_stack.swap(15)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::SWAP16,
                ..
            } => {
                block_stack.swap(16)?;
                block_element.stack = block_stack.clone();
            }

            Instruction {
                name: InstructionName::DUP1,
                ..
            } => {
                block_stack.dup(1)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP2,
                ..
            } => {
                block_stack.dup(2)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP3,
                ..
            } => {
                block_stack.dup(3)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP4,
                ..
            } => {
                block_stack.dup(4)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP5,
                ..
            } => {
                block_stack.dup(5)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP6,
                ..
            } => {
                block_stack.dup(6)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP7,
                ..
            } => {
                block_stack.dup(7)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP8,
                ..
            } => {
                block_stack.dup(8)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP9,
                ..
            } => {
                block_stack.dup(9)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP10,
                ..
            } => {
                block_stack.dup(10)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP11,
                ..
            } => {
                block_stack.dup(11)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP12,
                ..
            } => {
                block_stack.dup(12)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP13,
                ..
            } => {
                block_stack.dup(13)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP14,
                ..
            } => {
                block_stack.dup(14)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP15,
                ..
            } => {
                block_stack.dup(15)?;
                block_element.stack = block_stack.clone();
            }
            Instruction {
                name: InstructionName::DUP16,
                ..
            } => {
                block_stack.dup(16)?;
                block_element.stack = block_stack.clone();
            }

            Instruction {
                name:
                    InstructionName::PUSH
                    | InstructionName::PUSH_Data
                    | InstructionName::PUSH_ContractHash
                    | InstructionName::PUSH_ContractHashSize
                    | InstructionName::PUSH1
                    | InstructionName::PUSH2
                    | InstructionName::PUSH3
                    | InstructionName::PUSH4
                    | InstructionName::PUSH5
                    | InstructionName::PUSH6
                    | InstructionName::PUSH7
                    | InstructionName::PUSH8
                    | InstructionName::PUSH9
                    | InstructionName::PUSH10
                    | InstructionName::PUSH11
                    | InstructionName::PUSH12
                    | InstructionName::PUSH13
                    | InstructionName::PUSH14
                    | InstructionName::PUSH15
                    | InstructionName::PUSH16
                    | InstructionName::PUSH17
                    | InstructionName::PUSH18
                    | InstructionName::PUSH19
                    | InstructionName::PUSH20
                    | InstructionName::PUSH21
                    | InstructionName::PUSH22
                    | InstructionName::PUSH23
                    | InstructionName::PUSH24
                    | InstructionName::PUSH25
                    | InstructionName::PUSH26
                    | InstructionName::PUSH27
                    | InstructionName::PUSH28
                    | InstructionName::PUSH29
                    | InstructionName::PUSH30
                    | InstructionName::PUSH31
                    | InstructionName::PUSH32
                    | InstructionName::PUSHLIB
                    | InstructionName::PUSHDEPLOYADDRESS,
                value: Some(ref constant),
            } => {
                block_stack.push(StackElement::Constant(constant.to_owned()));
                block_element.stack = block_stack.clone();
            }

            ref instruction @ Instruction {
                name: InstructionName::SHL | InstructionName::SHR,
                ..
            } => {
                block_stack.push(
                    match block_stack.elements.get(block_stack.elements.len() - 2) {
                        Some(StackElement::Tag(tag)) => StackElement::Tag(*tag),
                        _ => StackElement::Value,
                    },
                );
                block_element.stack = block_stack.clone();
                let output = block_stack.pop()?;
                for _ in 0..instruction.input_size(version) {
                    block_stack.pop()?;
                }
                block_stack.push(output);
            }
            ref instruction @ Instruction {
                name: InstructionName::OR | InstructionName::XOR | InstructionName::AND,
                ..
            } => {
                let input_size = instruction.input_size(version);
                block_stack.push(
                    match block_stack
                        .elements
                        .iter()
                        .rev()
                        .take(input_size)
                        .find(|element| matches!(element, StackElement::Tag(tag) if *tag != 0))
                    {
                        Some(StackElement::Tag(tag)) => StackElement::Tag(*tag),
                        _ => StackElement::Value,
                    },
                );
                block_element.stack = block_stack.clone();
                let output = block_stack.pop()?;
                for _ in 0..instruction.input_size(version) {
                    block_stack.pop()?;
                }
                block_stack.push(output);
            }

            ref instruction if instruction.output_size() == 1 => {
                block_stack.push(StackElement::Value);
                block_element.stack = block_stack.clone();
                let output = block_stack.pop()?;
                for _ in 0..instruction.input_size(version) {
                    block_stack.pop()?;
                }
                block_stack.push(output);
            }

            ref instruction => {
                block_element.stack = block_stack.clone();
                for _ in 0..instruction.input_size(version) {
                    block_stack.pop()?;
                }
            }
        }

        Ok(())
    }

    ///
    /// Pushes a block into the function.
    ///
    fn insert_block(&mut self, block: Block) -> &mut Block {
        let tag = block.tag;

        if let Some(entry) = self.blocks.get_mut(&tag) {
            if entry.iter().all(|existing_block| {
                existing_block.initial_stack.hash() != block.initial_stack.hash()
            }) {
                entry.push(block);
            }
        } else {
            self.blocks.insert(tag, vec![block]);
        }

        self.blocks
            .get_mut(&tag)
            .expect("Always exists")
            .last_mut()
            .expect("Always exists")
    }

    ///
    /// Finalizes the function data.
    ///
    fn finalize(mut self) -> Self {
        for (_tag, blocks) in self.blocks.iter() {
            for block in blocks.iter() {
                for block_element in block.elements.iter() {
                    if block_element.stack.elements.len() > self.stack_size {
                        self.stack_size = block_element.stack.elements.len();
                    }
                }
            }
        }

        self
    }

    ///
    /// Returns the function name.
    ///
    fn name(&self) -> String {
        format!("function_{}", self.code_type)
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Function
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        context.add_function_evm(
            self.name().as_str(),
            context.void_type().fn_type(&[], false),
            Some(inkwell::module::Linkage::Private),
            compiler_llvm_context::FunctionEVMData::new(self.stack_size),
        );

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let name = self.name();
        let function = context
            .functions
            .get(name.as_str())
            .cloned()
            .expect("Always exists");
        context.set_function(function);

        for (tag, blocks) in self.blocks.iter() {
            for (index, block) in blocks.iter().enumerate() {
                let inner = context.append_basic_block(format!("block_{}/{}", tag, index).as_str());
                let block = compiler_llvm_context::FunctionBlock::new_evm(
                    inner,
                    compiler_llvm_context::FunctionBlockEVMData::new(block.initial_stack.hash()),
                );
                context.function_mut().evm_mut().insert_block(*tag, block);
            }
        }

        context.set_basic_block(context.function().entry_block);
        let mut stack_variables = Vec::with_capacity(self.stack_size);
        for stack_index in 0..self.stack_size {
            let pointer = context.build_alloca(
                context.field_type(),
                format!("stack_var_{:03}", stack_index).as_str(),
            );
            stack_variables.push(compiler_llvm_context::Argument::new(
                pointer.as_basic_value_enum(),
            ));
        }
        context.evm_mut().stack = stack_variables;
        let entry_block = context
            .function()
            .evm()
            .find_block(0, &Stack::default().hash())?;
        context.build_unconditional_branch(entry_block.inner);

        for (tag, blocks) in self.blocks.into_iter() {
            for (llvm_block, ir_block) in context
                .function()
                .evm()
                .blocks
                .get(&tag)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undeclared function block {}", tag))?
                .into_iter()
                .map(|block| block.inner)
                .zip(blocks)
            {
                context.set_basic_block(llvm_block);
                ir_block.into_llvm(context)?;
            }
        }

        context.build_catch_block();
        context.build_throw_block();

        context.set_basic_block(context.function().return_block);
        context.build_return(None);

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "function {} (max_sp = {}) {{",
            self.code_type, self.stack_size,
        )?;
        for (tag, blocks) in self.blocks.iter() {
            for (index, block) in blocks.iter().enumerate() {
                writeln!(
                    f,
                    "{:92}{}",
                    format!(
                        "block_{}/{}: {}",
                        *tag,
                        index,
                        if block.predecessors.is_empty() {
                            "".to_owned()
                        } else {
                            format!("(predecessors: {:?})", block.predecessors)
                        }
                    ),
                    block.initial_stack,
                )?;
                write!(f, "{}", block)?;
            }
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}
