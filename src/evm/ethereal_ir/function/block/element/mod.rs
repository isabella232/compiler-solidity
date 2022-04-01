//!
//! The Ethereal IR block element.
//!

pub mod stack;

use inkwell::values::BasicValue;

use crate::evm::assembly::instruction::name::Name as InstructionName;
use crate::evm::assembly::instruction::Instruction;

use self::stack::Stack;

///
/// The Ethereal IR block element.
///
#[derive(Debug, Clone)]
pub struct Element {
    /// The instruction.
    pub instruction: Instruction,
    /// The stack data.
    pub stack: Stack,
}

impl Element {
    ///
    /// Pops the specified number of arguments.
    ///
    fn pop_arguments<'ctx, 'dep, D>(
        &mut self,
        context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> Vec<inkwell::values::BasicValueEnum<'ctx>>
    where
        D: compiler_llvm_context::Dependency,
    {
        let input_size = self.instruction.input_size(&context.evm().version);
        let mut arguments = Vec::with_capacity(input_size);
        for index in 0..input_size {
            let pointer = context.evm().stack
                [self.stack.elements.len() - self.instruction.output_size() - index - 1];
            let value = context.build_load(pointer, format!("argument_{}", index).as_str());
            arguments.push(value);
        }
        arguments
    }
}

impl From<Instruction> for Element {
    fn from(instruction: Instruction) -> Self {
        Self {
            instruction,
            stack: Stack::new(),
        }
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Element
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm<'ctx, 'dep>(
        mut self,
        context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> anyhow::Result<()> {
        let input_size = self.instruction.input_size(&context.evm().version);

        let value = match self.instruction.name {
            InstructionName::PUSH => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH_Data => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH_Tag => crate::evm::assembly::instruction::stack::push_tag(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH_Dollar => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH_HashDollar => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSHLIB => {
                let path = self
                    .instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?;

                Ok(Some(
                    context
                        .resolve_library(path.as_str())?
                        .as_basic_value_enum(),
                ))
            }
            InstructionName::PUSHDEPLOYADDRESS => {
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }

            InstructionName::PUSH1 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH2 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH3 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH4 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH5 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH6 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH7 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH8 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH9 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH10 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH11 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH12 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH13 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH14 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH15 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH16 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH17 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH18 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH19 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH20 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH21 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH22 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH23 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH24 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH25 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH26 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH27 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH28 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH29 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH30 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH31 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            InstructionName::PUSH32 => crate::evm::assembly::instruction::stack::push(
                context,
                self.instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),

            InstructionName::DUP1 => {
                crate::evm::assembly::instruction::stack::dup(context, 1, self.stack.elements.len())
            }
            InstructionName::DUP2 => {
                crate::evm::assembly::instruction::stack::dup(context, 2, self.stack.elements.len())
            }
            InstructionName::DUP3 => {
                crate::evm::assembly::instruction::stack::dup(context, 3, self.stack.elements.len())
            }
            InstructionName::DUP4 => {
                crate::evm::assembly::instruction::stack::dup(context, 4, self.stack.elements.len())
            }
            InstructionName::DUP5 => {
                crate::evm::assembly::instruction::stack::dup(context, 5, self.stack.elements.len())
            }
            InstructionName::DUP6 => {
                crate::evm::assembly::instruction::stack::dup(context, 6, self.stack.elements.len())
            }
            InstructionName::DUP7 => {
                crate::evm::assembly::instruction::stack::dup(context, 7, self.stack.elements.len())
            }
            InstructionName::DUP8 => {
                crate::evm::assembly::instruction::stack::dup(context, 8, self.stack.elements.len())
            }
            InstructionName::DUP9 => {
                crate::evm::assembly::instruction::stack::dup(context, 9, self.stack.elements.len())
            }
            InstructionName::DUP10 => crate::evm::assembly::instruction::stack::dup(
                context,
                10,
                self.stack.elements.len(),
            ),
            InstructionName::DUP11 => crate::evm::assembly::instruction::stack::dup(
                context,
                11,
                self.stack.elements.len(),
            ),
            InstructionName::DUP12 => crate::evm::assembly::instruction::stack::dup(
                context,
                12,
                self.stack.elements.len(),
            ),
            InstructionName::DUP13 => crate::evm::assembly::instruction::stack::dup(
                context,
                13,
                self.stack.elements.len(),
            ),
            InstructionName::DUP14 => crate::evm::assembly::instruction::stack::dup(
                context,
                14,
                self.stack.elements.len(),
            ),
            InstructionName::DUP15 => crate::evm::assembly::instruction::stack::dup(
                context,
                15,
                self.stack.elements.len(),
            ),
            InstructionName::DUP16 => crate::evm::assembly::instruction::stack::dup(
                context,
                16,
                self.stack.elements.len(),
            ),

            InstructionName::SWAP1 => crate::evm::assembly::instruction::stack::swap(
                context,
                1,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP2 => crate::evm::assembly::instruction::stack::swap(
                context,
                2,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP3 => crate::evm::assembly::instruction::stack::swap(
                context,
                3,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP4 => crate::evm::assembly::instruction::stack::swap(
                context,
                4,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP5 => crate::evm::assembly::instruction::stack::swap(
                context,
                5,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP6 => crate::evm::assembly::instruction::stack::swap(
                context,
                6,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP7 => crate::evm::assembly::instruction::stack::swap(
                context,
                7,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP8 => crate::evm::assembly::instruction::stack::swap(
                context,
                8,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP9 => crate::evm::assembly::instruction::stack::swap(
                context,
                9,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP10 => crate::evm::assembly::instruction::stack::swap(
                context,
                10,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP11 => crate::evm::assembly::instruction::stack::swap(
                context,
                11,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP12 => crate::evm::assembly::instruction::stack::swap(
                context,
                12,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP13 => crate::evm::assembly::instruction::stack::swap(
                context,
                13,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP14 => crate::evm::assembly::instruction::stack::swap(
                context,
                14,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP15 => crate::evm::assembly::instruction::stack::swap(
                context,
                15,
                self.stack.elements.len(),
            ),
            InstructionName::SWAP16 => crate::evm::assembly::instruction::stack::swap(
                context,
                16,
                self.stack.elements.len(),
            ),

            InstructionName::POP => crate::evm::assembly::instruction::stack::pop(context),

            InstructionName::Tag => {
                let destination: usize = self
                    .instruction
                    .value
                    .expect("Always exists")
                    .parse()
                    .expect("Always valid");

                crate::evm::assembly::instruction::jump::unconditional(
                    context,
                    destination,
                    self.stack.to_string(),
                )
            }
            InstructionName::JUMP => {
                let destination = self.stack.pop_tag()?;

                crate::evm::assembly::instruction::jump::unconditional(
                    context,
                    destination,
                    self.stack.to_string(),
                )
            }
            InstructionName::JUMPI => {
                let destination = self.stack.pop_tag()?;
                self.stack.pop();

                crate::evm::assembly::instruction::jump::conditional(
                    context,
                    destination,
                    self.stack.to_string(),
                    self.stack.elements.len(),
                )
            }
            InstructionName::JUMPDEST => Ok(None),

            InstructionName::ADD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::addition(context, arguments)
            }
            InstructionName::SUB => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::subtraction(context, arguments)
            }
            InstructionName::MUL => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::multiplication(context, arguments)
            }
            InstructionName::DIV => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::division(context, arguments)
            }
            InstructionName::MOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::remainder(context, arguments)
            }
            InstructionName::SDIV => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::division_signed(context, arguments)
            }
            InstructionName::SMOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::remainder_signed(context, arguments)
            }

            InstructionName::LT => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::ULT,
                )
            }
            InstructionName::GT => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::UGT,
                )
            }
            InstructionName::EQ => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::EQ,
                )
            }
            InstructionName::ISZERO => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::comparison::compare(
                    context,
                    [
                        arguments.remove(0),
                        context.field_const(0).as_basic_value_enum(),
                    ],
                    inkwell::IntPredicate::EQ,
                )
            }
            InstructionName::SLT => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::SLT,
                )
            }
            InstructionName::SGT => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::SGT,
                )
            }

            InstructionName::OR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::or(context, arguments)
            }
            InstructionName::XOR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::xor(context, arguments)
            }
            InstructionName::NOT => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::bitwise::xor(
                    context,
                    [
                        arguments.remove(0),
                        context.field_type().const_all_ones().as_basic_value_enum(),
                    ],
                )
            }
            InstructionName::AND => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::and(context, arguments)
            }
            InstructionName::SHL => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::shift_left(context, arguments)
            }
            InstructionName::SHR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::shift_right(context, arguments)
            }
            InstructionName::SAR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::shift_right_arithmetic(context, arguments)
            }
            InstructionName::BYTE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::byte(context, arguments)
            }

            InstructionName::ADDMOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::add_mod(context, arguments)
            }
            InstructionName::MULMOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::mul_mod(context, arguments)
            }
            InstructionName::EXP => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::exponent(context, arguments)
            }
            InstructionName::SIGNEXTEND => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::sign_extend(context, arguments)
            }

            InstructionName::SHA3 => {
                let arguments: [inkwell::values::BasicValueEnum<'ctx>; 2] = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::hash::keccak256(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                )
            }
            InstructionName::KECCAK256 => {
                let arguments: [inkwell::values::BasicValueEnum<'ctx>; 2] = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::hash::keccak256(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                )
            }

            InstructionName::MLOAD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::memory::load(context, arguments)
            }
            InstructionName::MSTORE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::memory::store(context, arguments)
            }
            InstructionName::MSTORE8 => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::memory::store_byte(context, arguments)
            }

            InstructionName::SLOAD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::storage::load(context, arguments)
            }
            InstructionName::SSTORE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::storage::store(context, arguments)
            }
            InstructionName::PUSHIMMUTABLE => {
                let key = self
                    .instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?;
                compiler_llvm_context::immutable::load(context, key)
            }
            InstructionName::ASSIGNIMMUTABLE => {
                let mut arguments = self.pop_arguments(context);
                let key = self
                    .instruction
                    .value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?;
                let value = arguments.pop().expect("Always exists").into_int_value();
                compiler_llvm_context::immutable::store(context, key, value)
            }

            InstructionName::CALLDATALOAD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::calldata::load(context, arguments)
            }
            InstructionName::CALLDATASIZE => compiler_llvm_context::calldata::size(context),
            InstructionName::CALLDATACOPY => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            InstructionName::CODESIZE => compiler_llvm_context::calldata::size(context),
            InstructionName::CODECOPY => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            InstructionName::PUSHSIZE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::EXTCODESIZE => {
                let _arguments = self.pop_arguments(context);
                Ok(Some(context.field_const(0xffff).as_basic_value_enum()))
            }
            InstructionName::RETURNDATASIZE => compiler_llvm_context::return_data::size(context),
            InstructionName::RETURNDATACOPY => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::return_data::copy(context, arguments)
            }

            InstructionName::RETURN => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::r#return::r#return(context, arguments)
            }
            InstructionName::REVERT => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::r#return::revert(context, arguments)
            }
            InstructionName::STOP => compiler_llvm_context::r#return::stop(context),
            InstructionName::INVALID => compiler_llvm_context::r#return::invalid(context),

            InstructionName::LOG0 => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::event::log(
                    context,
                    arguments.remove(0).into_int_value(),
                    arguments.remove(0).into_int_value(),
                    arguments
                        .into_iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            InstructionName::LOG1 => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::event::log(
                    context,
                    arguments.remove(0).into_int_value(),
                    arguments.remove(0).into_int_value(),
                    arguments
                        .into_iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            InstructionName::LOG2 => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::event::log(
                    context,
                    arguments.remove(0).into_int_value(),
                    arguments.remove(0).into_int_value(),
                    arguments
                        .into_iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            InstructionName::LOG3 => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::event::log(
                    context,
                    arguments.remove(0).into_int_value(),
                    arguments.remove(0).into_int_value(),
                    arguments
                        .into_iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            InstructionName::LOG4 => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::event::log(
                    context,
                    arguments.remove(0).into_int_value(),
                    arguments.remove(0).into_int_value(),
                    arguments
                        .into_iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }

            InstructionName::CALL => {
                let mut arguments = self.pop_arguments(context);

                arguments.remove(0);
                let address = arguments.remove(0).into_int_value();
                let value = arguments.remove(0).into_int_value();
                let input_offset = arguments.remove(0).into_int_value();
                let input_size = arguments.remove(0).into_int_value();
                let output_offset = arguments.remove(0).into_int_value();
                let output_size = arguments.remove(0).into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::FarCall,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            InstructionName::CALLCODE => {
                let mut arguments = self.pop_arguments(context);

                arguments.remove(0);
                let address = arguments.remove(0).into_int_value();
                let value = arguments.remove(0).into_int_value();
                let input_offset = arguments.remove(0).into_int_value();
                let input_size = arguments.remove(0).into_int_value();
                let output_offset = arguments.remove(0).into_int_value();
                let output_size = arguments.remove(0).into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::CallCode,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            InstructionName::STATICCALL => {
                let mut arguments = self.pop_arguments(context);

                arguments.remove(0);
                let address = arguments.remove(0).into_int_value();
                let input_offset = arguments.remove(0).into_int_value();
                let input_size = arguments.remove(0).into_int_value();
                let output_offset = arguments.remove(0).into_int_value();
                let output_size = arguments.remove(0).into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::StaticCall,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            InstructionName::DELEGATECALL => {
                let mut arguments = self.pop_arguments(context);

                arguments.remove(0);
                let address = arguments.remove(0).into_int_value();
                let input_offset = arguments.remove(0).into_int_value();
                let input_size = arguments.remove(0).into_int_value();
                let output_offset = arguments.remove(0).into_int_value();
                let output_size = arguments.remove(0).into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    compiler_llvm_context::IntrinsicFunction::DelegateCall,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }

            InstructionName::CREATE => {
                let _arguments = self.pop_arguments(context);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            InstructionName::CREATE2 => {
                let _arguments = self.pop_arguments(context);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }

            InstructionName::ADDRESS => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::Address,
            ),
            InstructionName::CALLER => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::MessageSender,
            ),
            InstructionName::TIMESTAMP => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::BlockTimestamp,
            ),
            InstructionName::NUMBER => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::BlockNumber,
            ),
            InstructionName::GAS => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::GasLeft,
            ),

            InstructionName::GASLIMIT => Ok(Some(
                context.field_const(u32::MAX as u64).as_basic_value_enum(),
            )),
            InstructionName::BASEFEE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::COINBASE => Ok(Some(context.field_const(0).as_basic_value_enum())),

            InstructionName::CALLVALUE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::BALANCE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::SELFBALANCE => Ok(Some(context.field_const(0).as_basic_value_enum())),

            InstructionName::ORIGIN => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::CHAINID => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::BLOCKHASH => Ok(Some(context.field_const(0).as_basic_value_enum())),

            InstructionName::MSIZE => Ok(Some(
                context
                    .field_const(((1 << 16) * compiler_common::SIZE_FIELD) as u64)
                    .as_basic_value_enum(),
            )),
            InstructionName::DIFFICULTY => Ok(Some(context.field_const(0).as_basic_value_enum())),
            InstructionName::PC => Ok(Some(context.field_const(0).as_basic_value_enum())),

            InstructionName::EXTCODECOPY => {
                let _arguments = self.pop_arguments(context);
                Ok(None)
            }
            InstructionName::EXTCODEHASH => {
                let _arguments = self.pop_arguments(context);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            InstructionName::SELFDESTRUCT => {
                let _arguments = self.pop_arguments(context);
                Ok(None)
            }
        }?;

        if let Some(value) = value {
            let pointer = context.evm().stack[self.stack.elements.len() - input_size - 1];
            context.build_store(pointer, value);
        }

        Ok(())
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:88}{}", self.instruction.to_string(), self.stack,)
    }
}
