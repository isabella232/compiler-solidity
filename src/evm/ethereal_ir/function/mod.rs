//!
//! The Ethereal IR function.
//!

pub mod block;
pub mod queue_element;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use self::block::element::kind::Kind as BlockElementKind;
use self::block::exit::Exit as BlockExit;
use self::block::Block;
use self::queue_element::QueueElement;

///
/// The Ethereal IR function.
///
#[derive(Debug, Clone)]
pub struct Function {
    /// The function unique identifier, represented by a tag in the bytecode.
    /// Is 0 in the entry function.
    pub tag: usize,
    /// The contract part, which the function belongs to.
    pub code_type: compiler_llvm_context::CodeType,
    /// The separately labelled blocks.
    pub blocks: BTreeMap<usize, Vec<Block>>,
    /// The function callees.
    pub callees: HashSet<usize>,
    /// The function callers.
    pub callers: HashSet<usize>,
    /// The function input size.
    pub input_size: usize,
    /// The function output size.
    pub output_size: usize,
    /// The function stack size.
    pub stack_size: usize,
}

impl Function {
    /// The callees vector initial capacity.
    pub const CALLEES_VECTOR_DEFAULT_CAPACITY: usize = 4;
    /// The callers vector initial capacity.
    pub const CALLERS_VECTOR_DEFAULT_CAPACITY: usize = 4;

    ///
    /// A shortcut constructor.
    ///
    pub fn try_from_tag(
        tag: usize,
        code_type: compiler_llvm_context::CodeType,
        blocks: &HashMap<usize, Block>,
        entries: &BTreeSet<usize>,
        visited: &mut HashSet<usize>,
        functions: &mut BTreeMap<usize, Self>,
    ) -> anyhow::Result<Self> {
        let mut function = Self {
            tag,
            code_type,
            blocks: BTreeMap::new(),
            callees: HashSet::with_capacity(Self::CALLEES_VECTOR_DEFAULT_CAPACITY),
            callers: HashSet::with_capacity(Self::CALLERS_VECTOR_DEFAULT_CAPACITY),
            input_size: 0,
            output_size: 0,
            stack_size: 0,
        };
        function.consume_block(
            blocks,
            entries,
            visited,
            functions,
            QueueElement::new(tag, None, vec![], 0),
        )?;
        Ok(function.finalize())
    }

    ///
    /// Consumes the entry or a conditional block attached to another one.
    ///
    pub fn consume_block(
        &mut self,
        blocks: &HashMap<usize, Block>,
        entries: &BTreeSet<usize>,
        visited: &mut HashSet<usize>,
        functions: &mut BTreeMap<usize, Self>,
        mut element: QueueElement,
    ) -> anyhow::Result<()> {
        let code_type = self.code_type;

        let mut queue = vec![];
        let mut trace = vec![element.tag];
        let mut callees = HashSet::with_capacity(Self::CALLEES_VECTOR_DEFAULT_CAPACITY);

        let mut is_end = false;
        loop {
            visited.insert(element.tag);

            let block = blocks
                .get(&element.tag)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undeclared destination block {}", element.tag))?;
            let predecessor_block = element.predecessor.and_then(|tag| blocks.get(&tag));
            let block = self.insert_block(
                block,
                predecessor_block,
                trace.as_slice(),
                element.vertical_tags_buffer.as_slice(),
            );
            if !block.validate_predecessor(element.predecessor.take()) {
                break;
            }

            let conditional_tags_buffer = element.vertical_tags_buffer.clone();
            element.vertical_tags_buffer.append(&mut block.tags);

            match block.exit.take() {
                Some(BlockExit::Call { callee }) => {
                    let r#return = match element.vertical_tags_buffer.pop() {
                        Some(destination) => destination,
                        None => {
                            anyhow::bail!("Function call return address is missing");
                        }
                    };

                    let (input_size, output_size) = match functions.get(&callee).cloned() {
                        Some(function) => (function.input_size, function.output_size),
                        None => {
                            let function = Self::try_from_tag(
                                callee, code_type, blocks, entries, visited, functions,
                            )?;
                            let (input_size, output_size) =
                                (function.input_size, function.output_size);
                            functions.insert(callee, function);
                            (input_size, output_size)
                        }
                    };

                    callees.insert(callee);
                    trace.push(r#return);
                    block.elements.push(
                        BlockElementKind::call(
                            callee,
                            r#return,
                            trace.to_owned(),
                            input_size,
                            output_size,
                        )
                        .into(),
                    );
                    element.predecessor = Some(element.tag);
                    element.tag = r#return;
                    log::trace!("Queued  {:4} [return address]", element.tag);
                }
                Some(BlockExit::Fallthrough {
                    destination: callee,
                }) if entries.contains(&callee) => {
                    let r#return = match element.vertical_tags_buffer.pop() {
                        Some(destination) => destination,
                        None => {
                            anyhow::bail!("Function fallthrough call return address is missing")
                        }
                    };

                    let (input_size, output_size) = match functions.get(&callee).cloned() {
                        Some(function) => (function.input_size, function.output_size),
                        None => {
                            let function = Self::try_from_tag(
                                callee, code_type, blocks, entries, visited, functions,
                            )?;
                            let (input_size, output_size) =
                                (function.input_size, function.output_size);
                            functions.insert(callee, function);
                            (input_size, output_size)
                        }
                    };

                    callees.insert(callee);
                    trace.push(r#return);
                    block.elements.push(
                        BlockElementKind::call(
                            callee,
                            r#return,
                            trace.to_owned(),
                            input_size,
                            output_size,
                        )
                        .into(),
                    );
                    element.predecessor = Some(element.tag);
                    element.tag = r#return;
                    log::trace!("Queued  {:4} [return address]", element.tag);
                }
                Some(BlockExit::Unconditional) => {
                    let destination = match element.vertical_tags_buffer.pop() {
                        Some(destination) => destination,
                        None => anyhow::bail!("Unconditional jump address is missing"),
                    };
                    block.elements.push(
                        BlockElementKind::unconditional_jump(
                            destination,
                            element.vertical_tags_buffer.clone(),
                        )
                        .into(),
                    );
                    element.predecessor = Some(element.tag);
                    element.tag = destination;
                    trace.clear();
                    log::trace!("Queued  {:4} [unconditional jump]", element.tag);
                }
                Some(BlockExit::Fallthrough { destination }) => {
                    block
                        .elements
                        .push(BlockElementKind::Fallthrough(destination).into());
                    element.predecessor = Some(element.tag);
                    element.tag = destination;
                    trace.clear();
                    log::trace!("Queued  {:4} [fallthrough]", element.tag);
                }
                Some(BlockExit::Return) if !element.vertical_tags_buffer.is_empty() => {
                    let destination = match element.vertical_tags_buffer.pop() {
                        Some(destination) => destination,
                        None => unreachable!(),
                    };
                    block.elements.push(
                        BlockElementKind::unconditional_jump(
                            destination,
                            element.vertical_tags_buffer.clone(),
                        )
                        .into(),
                    );
                    element.predecessor = Some(element.tag);
                    element.tag = destination;
                    trace.clear();
                    log::trace!("Queued  {:4} [return fallthrough]", element.tag);
                }
                Some(BlockExit::Return) => {
                    block.elements.push(BlockElementKind::Return.into());
                    element.predecessor = Some(element.tag);
                    is_end = true;
                }
                None => {
                    element.predecessor = Some(element.tag);
                    is_end = true;
                }
            }

            element.stack_offset = block.set_stack_data(element.stack_offset);

            #[allow(clippy::unnecessary_to_owned)]
            for jump in block.jumps.to_owned().into_iter().rev() {
                log::trace!("Queued  {:4} [conditional jump]", jump.destination);
                let mut vertical_tags_buffer = conditional_tags_buffer.clone();
                vertical_tags_buffer.extend_from_slice(jump.tags.as_slice());
                block.set_vertical_tags_buffer(jump.position, vertical_tags_buffer.clone())?;
                let stack_offset = block.elements[jump.position]
                    .stack_data
                    .as_ref()
                    .expect("Always exists")
                    .current;
                queue.push(QueueElement::new(
                    jump.destination,
                    element.predecessor,
                    vertical_tags_buffer,
                    stack_offset,
                ));
            }

            if is_end {
                break;
            }
        }

        self.callees.extend(callees);
        for element in queue.into_iter() {
            self.consume_block(blocks, entries, visited, functions, element)?;
        }

        Ok(())
    }

    ///
    /// Pushes a block into the function.
    ///
    /// The block is cloned if:
    /// - it is a function return address block
    /// - it is not a conditional jump destination
    /// - it has not been visited by the passed predecessor
    ///
    pub fn insert_block(
        &mut self,
        mut block: Block,
        predecessor_block: Option<&Block>,
        trace: &[usize],
        vertical: &[usize],
    ) -> &mut Block {
        let tag = block.tag;

        let is_return_block_clone = matches!(
            (
                block.exit.as_ref(),
                predecessor_block.and_then(|block| block.exit.as_ref()),
            ),
            (Some(BlockExit::Call { .. }), Some(BlockExit::Call { .. }))
        );

        let is_conditional_destination = match predecessor_block {
            Some(predecessor) => predecessor
                .jumps
                .iter()
                .any(|jump| jump.destination == block.tag),
            None => false,
        };

        let is_call_trace_known = self
            .blocks
            .get(&tag)
            .map(|blocks| {
                blocks
                    .iter()
                    .any(|block| block.call_trace.as_slice() == trace)
            })
            .unwrap_or_default();

        let is_vertical_tags_buffer_known = self
            .blocks
            .get(&tag)
            .map(|blocks| {
                blocks
                    .iter()
                    .any(|block| block.vertical_tags_buffer.as_slice() == vertical)
            })
            .unwrap_or_default();

        block.call_trace = trace.to_owned();
        block.vertical_tags_buffer = vertical.to_owned();
        if let Some(entry) = self.blocks.get_mut(&tag) {
            if (!is_call_trace_known && !is_conditional_destination && is_return_block_clone)
                || !is_vertical_tags_buffer_known
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
    /// Inserts a caller tag.
    ///
    pub fn insert_caller(&mut self, tag: usize) {
        self.callers.insert(tag);
    }

    ///
    /// Returns the function name.
    ///
    pub fn name(&self) -> String {
        format!("function_{}_{}", self.code_type, self.tag)
    }

    ///
    /// Updates the function stack and I/O data.
    ///
    pub fn finalize(mut self) -> Self {
        let mut deepest_stack_offset = 0;
        for (_tag, blocks) in self.blocks.iter_mut() {
            for block in blocks.iter_mut() {
                if block.deepest_stack_offset < deepest_stack_offset {
                    deepest_stack_offset = block.deepest_stack_offset;
                }
            }
        }
        self.input_size = (-deepest_stack_offset) as usize;
        if self.tag != 0 && self.input_size == 0 {
            self.input_size = 1;
        }
        self.stack_size = self.input_size;

        for (_tag, blocks) in self.blocks.iter_mut() {
            for block in blocks.iter_mut() {
                block.normalize_stack(self.input_size as isize);
                if block.highest_stack_size > self.stack_size {
                    self.stack_size = block.highest_stack_size;
                }
            }
        }

        'outer: for (_tag, blocks) in self.blocks.iter() {
            for block in blocks.iter() {
                if block.is_function_return() {
                    self.output_size = block.final_stack_offset.unwrap_or_default() as usize;
                    break 'outer;
                }
            }
        }

        self
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Function
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let function_type = match self.output_size {
            0 => context.void_type().fn_type(
                vec![context.field_type().as_basic_type_enum(); self.input_size].as_slice(),
                false,
            ),
            1 => context.field_type().fn_type(
                vec![context.field_type().as_basic_type_enum(); self.input_size].as_slice(),
                false,
            ),
            output_size => {
                let output_type = context
                    .structure_type(vec![context.field_type().as_basic_type_enum(); output_size])
                    .ptr_type(compiler_llvm_context::AddressSpace::Stack.into());
                let mut argument_types = Vec::with_capacity(self.input_size + 1);
                argument_types.push(output_type.as_basic_type_enum());
                argument_types.extend(vec![
                    context.field_type().as_basic_type_enum();
                    self.input_size
                ]);
                output_type.fn_type(argument_types.as_slice(), false)
            }
        };
        context.add_function_evm(
            self.name().as_str(),
            function_type,
            Some(inkwell::module::Linkage::Private),
            compiler_llvm_context::FunctionEVMData::new(
                self.input_size,
                self.output_size,
                self.stack_size,
            ),
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
                        block.call_trace.to_owned(),
                        block.vertical_tags_buffer.to_owned(),
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
            if stack_index < self.input_size {
                let value = context
                    .function()
                    .value
                    .get_nth_param((stack_index + if self.output_size > 1 { 1 } else { 0 }) as u32)
                    .expect("Always exists");
                context.build_store(pointer, value);
            }
            stack_variables.push(pointer);
        }
        context.evm_mut().stack = stack_variables;
        let entry_block = context.function().evm().first_block(self.tag)?;
        context.build_unconditional_branch(entry_block);

        for (tag, blocks) in self.blocks.into_iter() {
            let llvm_blocks = context.function().evm().all_blocks(tag)?;
            for (llvm_block, ir_block) in llvm_blocks.into_iter().zip(blocks) {
                context.set_basic_block(llvm_block);
                ir_block.into_llvm(context)?;
            }
        }

        context.build_catch_block(false);
        context.build_throw_block(false);

        context.set_basic_block(context.function().return_block);
        match self.output_size {
            0 => {
                context.build_return(None);
            }
            1 => {
                let return_pointer = context.evm().stack[0];
                let return_value = context.build_load(return_pointer, "return_value");
                context.build_return(Some(&return_value));
            }
            output_size => {
                let return_pointer = context
                    .function()
                    .value
                    .get_first_param()
                    .expect("Always exists")
                    .into_pointer_value();
                for index in 0..output_size {
                    let source_pointer = context.evm().stack[index];
                    let destination_pointer = unsafe {
                        context.builder().build_gep(
                            return_pointer,
                            &[
                                context.field_const(0),
                                context
                                    .integer_type(compiler_common::BITLENGTH_X32)
                                    .const_int(index as u64, false),
                            ],
                            format!("destination_pointer_{}", index).as_str(),
                        )
                    };
                    let return_value = context.build_load(
                        source_pointer,
                        format!("return_value_{}", index + 1).as_str(),
                    );
                    context.build_store(destination_pointer, return_value);
                }
                context.build_return(Some(&return_pointer.as_basic_value_enum()));
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "function {}({}) -> {} (callers: {:?}, stack size: {}) {{",
            self.tag, self.input_size, self.output_size, self.callers, self.stack_size,
        )?;
        for (tag, blocks) in self.blocks.iter() {
            for (index, block) in blocks.iter().enumerate() {
                writeln!(
                    f,
                    "block_{}/{}: {}",
                    *tag,
                    index,
                    if block.predecessors.is_empty()
                        && block.call_trace.is_empty()
                        && block.vertical_tags_buffer.is_empty()
                    {
                        "".to_owned()
                    } else {
                        format!(
                            "(predecessors: {:?}, call trace: {:?}, tag buffer: {:?})",
                            block.predecessors, block.call_trace, block.vertical_tags_buffer
                        )
                    }
                )?;
                write!(f, "{}", block)?;
            }
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}
