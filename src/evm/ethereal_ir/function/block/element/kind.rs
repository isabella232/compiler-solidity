//!
//! The Ethereal IR block element kind.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::evm::assembly::instruction::Instruction;

///
/// The Ethereal IR block element kind.
///
#[derive(Debug, Clone)]
pub enum Kind {
    /// The unchanged EVM bytecode instruction.
    Instruction(Instruction),
    /// The JUMP instruction with tags representing the callee and return block key.
    Call {
        /// The callee function tag.
        callee: usize,
        /// The return address tag.
        r#return: usize,
        /// The call trace.
        trace: Vec<usize>,
        /// The function input size.
        input_size: usize,
        /// The function output size.
        output_size: usize,
    },
    /// The JUMPI instruction with tag representing the destination block tag.
    ConditionalJump {
        /// The destination tag.
        destination: usize,
        /// The vertical tags buffer.
        vertical_tags_buffer: Vec<usize>,
    },
    /// The JUMP instruction with tag representing the callee tag.
    UnconditionalJump {
        /// The destination tag.
        destination: usize,
        /// The vertical tags buffer.
        vertical_tags_buffer: Vec<usize>,
    },
    /// The falling through into the adjacent block.
    Fallthrough(usize),
    /// The JUMP instruction representing the function return.
    Return,
}

impl Kind {
    ///
    /// A shortcut constructor.
    ///
    pub fn call(
        callee: usize,
        r#return: usize,
        trace: Vec<usize>,
        input_size: usize,
        output_size: usize,
    ) -> Self {
        Self::Call {
            callee,
            r#return,
            trace,
            input_size,
            output_size,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn conditional_jump(destination: usize, vertical_tags_buffer: Vec<usize>) -> Self {
        Self::ConditionalJump {
            destination,
            vertical_tags_buffer,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn unconditional_jump(destination: usize, vertical_tags_buffer: Vec<usize>) -> Self {
        Self::UnconditionalJump {
            destination,
            vertical_tags_buffer,
        }
    }

    ///
    /// Returns the number of input stack arguments.
    ///
    pub fn input_size(&self) -> usize {
        match self {
            Self::Instruction(inner) => inner.input_size(),
            Self::Call { input_size, .. } => *input_size,
            Self::ConditionalJump { .. } => 1,
            Self::UnconditionalJump { .. } => 1,
            Self::Fallthrough(_) => 0,
            Self::Return => 1,
        }
    }

    ///
    /// Returns the number of output stack arguments.
    ///
    pub fn output_size(&self) -> Option<usize> {
        match self {
            Self::Instruction(inner) => Some(inner.output_size()),
            Self::Call { output_size, .. } => Some(*output_size),
            Self::ConditionalJump { .. } => Some(0),
            Self::UnconditionalJump { .. } => Some(0),
            Self::Fallthrough(_) => Some(0),
            Self::Return => Some(0),
        }
    }

    ///
    /// Returns the stack depth, where the instruction can reach.
    ///
    pub fn stack_depth(&self) -> usize {
        match self {
            Self::Instruction(inner) => inner.stack_depth(),
            _ => self.input_size(),
        }
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Kind
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        match self {
            Self::Instruction(inner) => {
                inner.into_llvm(context)?;
            }
            Self::Call {
                callee,
                r#return,
                trace,
                ..
            } => {
                let name = format!(
                    "function_{}_{}",
                    context.code_type.expect("Always exists"),
                    callee,
                );
                let function = context
                    .functions
                    .get(name.as_str())
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Undeclared function {}", callee))?;

                let mut arguments = Vec::with_capacity(function.evm().input_size + 1);
                if function.evm().output_size > 1 {
                    let return_type =
                        context.structure_type(vec![
                            context.field_type().as_basic_type_enum();
                            function.evm().output_size
                        ]);
                    let return_pointer = context.build_alloca(return_type, "return_pointer");
                    arguments.push(return_pointer.as_basic_value_enum());
                }
                for index in 0..function.evm().input_size {
                    let pointer = context
                        .evm()
                        .stack_pointer(function.evm().input_size - index);
                    let argument =
                        context.build_load(pointer, format!("argument_{}", index + 1).as_str());
                    arguments.push(argument);
                }
                context
                    .evm_mut()
                    .decrease_stack_pointer(function.evm().input_size);

                let return_value = context.build_invoke(
                    function.value,
                    arguments.as_slice(),
                    format!("call_{}", function.name).as_str(),
                );
                match function.evm().output_size {
                    0 => {}
                    1 => {
                        let stack_pointer = context.evm().stack_pointer(0);
                        context.evm_mut().increase_stack_pointer(1);
                        let return_value = return_value.expect("Always exists").into_int_value();
                        context.build_store(stack_pointer, return_value);
                    }
                    output_size => {
                        let return_pointer =
                            return_value.expect("Always exists").into_pointer_value();
                        for index in 0..output_size {
                            let return_value_pointer = unsafe {
                                context.builder().build_gep(
                                    return_pointer,
                                    &[
                                        context.field_const(0),
                                        context
                                            .integer_type(compiler_common::BITLENGTH_X32)
                                            .const_int(index as u64, false),
                                    ],
                                    format!("return_value_pointer_{}", index).as_str(),
                                )
                            };
                            let stack_pointer = context.evm().stack_pointer(0);
                            context.evm_mut().increase_stack_pointer(1);
                            let return_value = context.build_load(
                                return_value_pointer,
                                format!("return_value_{}", index + 1).as_str(),
                            );
                            context.build_store(stack_pointer, return_value);
                        }
                    }
                }

                let block = context
                    .function()
                    .evm()
                    .block_by_call_trace(r#return, trace.as_slice())?;
                context.build_unconditional_branch(block.inner);
            }
            Self::ConditionalJump {
                destination,
                vertical_tags_buffer,
            } => {
                let condition_pointer = context.evm().stack_pointer(1);
                context.evm_mut().decrease_stack_pointer(1);
                let condition = context.build_load(
                    condition_pointer,
                    format!("conditional_{}_condition", destination).as_str(),
                );
                let condition = context.builder().build_int_compare(
                    inkwell::IntPredicate::NE,
                    condition.into_int_value(),
                    context.field_const(0),
                    format!("conditional_{}_condition_compared", destination).as_str(),
                );

                let then_block = context
                    .function()
                    .evm()
                    .block_by_vertical_tags_buffer(destination, vertical_tags_buffer.as_slice())?;
                let join_block = context
                    .append_basic_block(format!("conditional_{}_join_block", destination).as_str());

                context.build_conditional_branch(condition, then_block.inner, join_block);

                context.set_basic_block(join_block);
            }
            Self::UnconditionalJump {
                destination,
                vertical_tags_buffer,
            } => {
                context.evm_mut().decrease_stack_pointer(1);
                let block = context
                    .function()
                    .evm()
                    .block_by_vertical_tags_buffer(destination, vertical_tags_buffer.as_slice())?;
                context.build_unconditional_branch(block.inner);
            }
            Self::Fallthrough(tag) => {
                let block = context.function().evm().first_block(tag)?;
                context.build_unconditional_branch(block);
            }
            Self::Return => {
                context.build_unconditional_branch(context.function().return_block);
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instruction(inner) => write!(f, "{}", inner),
            Self::Call {
                callee,
                r#return,
                input_size,
                output_size,
                ..
            } => write!(
                f,
                "JUMP: CALL(tag={}, return={}, in={}, out={})",
                callee, r#return, input_size, output_size
            ),
            Self::ConditionalJump {
                destination,
                vertical_tags_buffer,
            } => {
                write!(
                    f,
                    "JUMP: CONDITIONAL({}, {:?})",
                    destination, vertical_tags_buffer
                )
            }
            Self::UnconditionalJump {
                destination,
                vertical_tags_buffer,
            } => write!(
                f,
                "JUMP: UNCONDITIONAL({}, {:?})",
                destination, vertical_tags_buffer
            ),
            Self::Fallthrough(tag) => write!(f, "JUMP: FALLTHROUGH({})", tag),
            Self::Return => write!(f, "JUMP: RETURN"),
        }
    }
}
