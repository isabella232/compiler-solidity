//!
//! The Ethereal IR function.
//!

pub mod block;
pub mod queue_element;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::evm::assembly::instruction::name::Name as InstructionName;
use crate::evm::assembly::instruction::Instruction;
use crate::evm::ethereal_ir::function::block::element::stack::element::Element;
use crate::evm::ethereal_ir::function::block::element::stack::Stack;

use self::block::element::stack::element::Element as StackElement;
use self::block::Block;
use self::queue_element::QueueElement;

///
/// The Ethereal IR function.
///
#[derive(Debug, Clone)]
pub struct Function {
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
    pub fn try_from_blocks(
        code_type: compiler_llvm_context::CodeType,
        blocks: &HashMap<usize, Block>,
        visited: &mut HashSet<QueueElement>,
        functions: &mut BTreeMap<usize, Self>,
    ) -> anyhow::Result<Self> {
        let mut function = Self {
            code_type,
            blocks: BTreeMap::new(),
            stack_size: 0,
        };
        function.consume_block(
            blocks,
            visited,
            functions,
            QueueElement::new(0, None, Stack::new()),
        )?;
        Ok(function)
    }

    ///
    /// Finalizes the function data.
    ///
    pub fn finalize(mut self) -> Self {
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
    /// Consumes the entry or a conditional block attached to another one.
    ///
    pub fn consume_block(
        &mut self,
        blocks: &HashMap<usize, Block>,
        visited: &mut HashSet<QueueElement>,
        functions: &mut BTreeMap<usize, Self>,
        mut queue_element: QueueElement,
    ) -> anyhow::Result<()> {
        let mut queue = vec![];

        loop {
            if visited.contains(&queue_element) {
                break;
            }
            visited.insert(queue_element.clone());

            let block = blocks.get(&queue_element.tag).cloned().ok_or_else(|| {
                anyhow::anyhow!("Undeclared destination block {}", queue_element.tag)
            })?;
            let block = self.insert_block(block);
            block.initial_stack = queue_element.stack.clone();
            block.stack = block.initial_stack.clone();
            if !block.validate_predecessor(queue_element.predecessor.take()) {
                break;
            }

            let mut is_end = false;
            for block_element in block.elements.iter_mut() {
                match block_element.instruction {
                    Instruction {
                        name: InstructionName::PUSH_Tag,
                        value: Some(ref tag),
                    } => {
                        let tag: usize = tag.parse().expect("Always valid");
                        block.stack.push(Element::Tag(tag));
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::JUMP,
                        ..
                    } => {
                        queue_element.predecessor = Some(queue_element.tag);

                        block_element.stack = block.stack.clone();
                        let destination = block.stack.pop_tag()?;
                        log::trace!("Queued  {:4} [unconditional jump]", destination);
                        queue.push(QueueElement::new(
                            destination,
                            queue_element.predecessor,
                            block.stack.to_owned(),
                        ));

                        is_end = true;
                    }
                    Instruction {
                        name: InstructionName::JUMPI,
                        ..
                    } => {
                        queue_element.predecessor = Some(queue_element.tag);

                        block_element.stack = block.stack.clone();
                        let destination = block.stack.pop_tag()?;
                        block.stack.pop();
                        log::trace!("Queued  {:4} [conditional jump]", destination);
                        queue.push(QueueElement::new(
                            destination,
                            queue_element.predecessor,
                            block.stack.to_owned(),
                        ));
                    }
                    Instruction {
                        name: InstructionName::Tag,
                        value: Some(ref destination),
                    } => {
                        block_element.stack = block.stack.clone();

                        let destination: usize = destination.parse().expect("Always valid");
                        queue_element.predecessor = Some(queue_element.tag);
                        queue_element.tag = destination;
                        log::trace!("Queued  {:4} [fallthrough]", queue_element.tag);
                        queue.push(QueueElement::new(
                            destination,
                            queue_element.predecessor,
                            block.stack.to_owned(),
                        ));

                        is_end = true;
                    }
                    Instruction {
                        name: InstructionName::SWAP1,
                        ..
                    } => {
                        block.stack.swap(1);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP2,
                        ..
                    } => {
                        block.stack.swap(2);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP3,
                        ..
                    } => {
                        block.stack.swap(3);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP4,
                        ..
                    } => {
                        block.stack.swap(4);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP5,
                        ..
                    } => {
                        block.stack.swap(5);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP6,
                        ..
                    } => {
                        block.stack.swap(6);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP7,
                        ..
                    } => {
                        block.stack.swap(7);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP8,
                        ..
                    } => {
                        block.stack.swap(8);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP9,
                        ..
                    } => {
                        block.stack.swap(9);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP10,
                        ..
                    } => {
                        block.stack.swap(10);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP11,
                        ..
                    } => {
                        block.stack.swap(11);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP12,
                        ..
                    } => {
                        block.stack.swap(12);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP13,
                        ..
                    } => {
                        block.stack.swap(13);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP14,
                        ..
                    } => {
                        block.stack.swap(14);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP15,
                        ..
                    } => {
                        block.stack.swap(15);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::SWAP16,
                        ..
                    } => {
                        block.stack.swap(16);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP1,
                        ..
                    } => {
                        block.stack.dup(1);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP2,
                        ..
                    } => {
                        block.stack.dup(2);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP3,
                        ..
                    } => {
                        block.stack.dup(3);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP4,
                        ..
                    } => {
                        block.stack.dup(4);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP5,
                        ..
                    } => {
                        block.stack.dup(5);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP6,
                        ..
                    } => {
                        block.stack.dup(6);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP7,
                        ..
                    } => {
                        block.stack.dup(7);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP8,
                        ..
                    } => {
                        block.stack.dup(8);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP9,
                        ..
                    } => {
                        block.stack.dup(9);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP10,
                        ..
                    } => {
                        block.stack.dup(10);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP11,
                        ..
                    } => {
                        block.stack.dup(11);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP12,
                        ..
                    } => {
                        block.stack.dup(12);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP13,
                        ..
                    } => {
                        block.stack.dup(13);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP14,
                        ..
                    } => {
                        block.stack.dup(14);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP15,
                        ..
                    } => {
                        block.stack.dup(15);
                        block_element.stack = block.stack.clone();
                    }
                    Instruction {
                        name: InstructionName::DUP16,
                        ..
                    } => {
                        block.stack.dup(16);
                        block_element.stack = block.stack.clone();
                    }
                    ref instruction => {
                        block
                            .stack
                            .extend(vec![StackElement::Value; instruction.output_size()]);
                        block_element.stack = block.stack.clone();
                        for _ in 0..instruction.input_size() {
                            block.stack.pop();
                        }

                        if let Instruction {
                            name: InstructionName::RETURN,
                            ..
                        }
                        | Instruction {
                            name: InstructionName::REVERT,
                            ..
                        } = instruction
                        {
                            is_end = true;
                        }
                    }
                }
            }

            if is_end {
                break;
            }
        }

        for element in queue.into_iter() {
            self.consume_block(blocks, visited, functions, element)?;
        }

        Ok(())
    }

    ///
    /// Pushes a block into the function.
    ///
    pub fn insert_block(&mut self, block: Block) -> &mut Block {
        let tag = block.tag;

        if let Some(entry) = self.blocks.get_mut(&tag) {
            if entry
                .iter()
                .all(|existing_block| existing_block.initial_stack != block.initial_stack)
            {
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
    /// Returns the function name.
    ///
    pub fn name(&self) -> String {
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
                    compiler_llvm_context::FunctionBlockEVMData::new(
                        block.initial_stack.to_string(),
                    ),
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
            stack_variables.push(pointer);
        }
        context.evm_mut().stack = stack_variables;
        let entry_block = context.function().evm().block_by_stack_pattern(0, "")?;
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

        context.build_catch_block(false);
        context.build_throw_block(false);

        context.set_basic_block(context.function().return_block);
        context.build_return(None);

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "function {} (stack size: {}) {{",
            self.code_type, self.stack_size,
        )?;
        for (tag, blocks) in self.blocks.iter() {
            for (index, block) in blocks.iter().enumerate() {
                writeln!(
                    f,
                    "block_{}/{}: {}",
                    *tag,
                    index,
                    if block.predecessors.is_empty() {
                        "".to_owned()
                    } else {
                        format!("(predecessors: {:?})", block.predecessors)
                    }
                )?;
                write!(f, "{}", block)?;
            }
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}
