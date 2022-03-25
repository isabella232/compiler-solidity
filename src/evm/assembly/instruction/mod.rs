//!
//! The EVM instruction.
//!

pub mod name;
pub mod stack;
pub mod storage;

use std::convert::TryInto;

use inkwell::values::BasicValue;
use serde::Deserialize;
use serde::Serialize;

use self::name::Name;

///
/// The EVM instruction.
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Instruction {
    /// The opcode or tag identifier.
    pub name: Name,
    /// The optional value argument.
    pub value: Option<String>,
}

impl Instruction {
    ///
    /// Returns the number of input stack arguments.
    ///
    pub const fn input_size(&self) -> usize {
        match self.name {
            Name::POP => 1,

            Name::ADD => 2,
            Name::SUB => 2,
            Name::MUL => 2,
            Name::DIV => 2,
            Name::MOD => 2,
            Name::SDIV => 2,
            Name::SMOD => 2,

            Name::LT => 2,
            Name::GT => 2,
            Name::EQ => 2,
            Name::ISZERO => 1,
            Name::SLT => 2,
            Name::SGT => 2,

            Name::OR => 2,
            Name::XOR => 2,
            Name::NOT => 1,
            Name::AND => 2,
            Name::SHL => 2,
            Name::SHR => 2,
            Name::SAR => 2,
            Name::BYTE => 2,

            Name::ADDMOD => 3,
            Name::MULMOD => 3,
            Name::EXP => 2,
            Name::SIGNEXTEND => 2,
            Name::SHA3 => 2,
            Name::KECCAK256 => 2,

            Name::MLOAD => 1,
            Name::MSTORE => 2,
            Name::MSTORE8 => 2,

            Name::SLOAD => 1,
            Name::SSTORE => 2,
            Name::PUSHIMMUTABLE => 0,
            Name::ASSIGNIMMUTABLE => 2,

            Name::CALLDATALOAD => 1,
            Name::CALLDATACOPY => 3,
            Name::CODECOPY => 3,
            Name::EXTCODESIZE => 1,
            Name::RETURNDATACOPY => 3,

            Name::RETURN => 2,
            Name::REVERT => 2,
            Name::SELFDESTRUCT => 1,

            Name::LOG0 => 2,
            Name::LOG1 => 3,
            Name::LOG2 => 4,
            Name::LOG3 => 5,
            Name::LOG4 => 6,

            Name::CALL => 7,
            Name::CALLCODE => 7,
            Name::STATICCALL => 6,
            Name::DELEGATECALL => 6,

            Name::CREATE => 3,
            Name::CREATE2 => 4,

            Name::EXTCODECOPY => 4,
            Name::EXTCODEHASH => 1,

            _ => 0,
        }
    }

    ///
    /// Returns the number of output stack arguments.
    ///
    pub const fn output_size(&self) -> usize {
        match self.name {
            Name::PUSH => 1,
            Name::PUSH_Data => 1,
            Name::PUSH_Tag => 1,
            Name::PUSH_Dollar => 1,
            Name::PUSH_HashDollar => 1,

            Name::PUSH1 => 1,
            Name::PUSH2 => 1,
            Name::PUSH3 => 1,
            Name::PUSH4 => 1,
            Name::PUSH5 => 1,
            Name::PUSH6 => 1,
            Name::PUSH7 => 1,
            Name::PUSH8 => 1,
            Name::PUSH9 => 1,
            Name::PUSH10 => 1,
            Name::PUSH11 => 1,
            Name::PUSH12 => 1,
            Name::PUSH13 => 1,
            Name::PUSH14 => 1,
            Name::PUSH15 => 1,
            Name::PUSH16 => 1,
            Name::PUSH17 => 1,
            Name::PUSH18 => 1,
            Name::PUSH19 => 1,
            Name::PUSH20 => 1,
            Name::PUSH21 => 1,
            Name::PUSH22 => 1,
            Name::PUSH23 => 1,
            Name::PUSH24 => 1,
            Name::PUSH25 => 1,
            Name::PUSH26 => 1,
            Name::PUSH27 => 1,
            Name::PUSH28 => 1,
            Name::PUSH29 => 1,
            Name::PUSH30 => 1,
            Name::PUSH31 => 1,
            Name::PUSH32 => 1,

            Name::DUP1 => 1,
            Name::DUP2 => 1,
            Name::DUP3 => 1,
            Name::DUP4 => 1,
            Name::DUP5 => 1,
            Name::DUP6 => 1,
            Name::DUP7 => 1,
            Name::DUP8 => 1,
            Name::DUP9 => 1,
            Name::DUP10 => 1,
            Name::DUP11 => 1,
            Name::DUP12 => 1,
            Name::DUP13 => 1,
            Name::DUP14 => 1,
            Name::DUP15 => 1,
            Name::DUP16 => 1,

            Name::ADD => 1,
            Name::SUB => 1,
            Name::MUL => 1,
            Name::DIV => 1,
            Name::MOD => 1,
            Name::SDIV => 1,
            Name::SMOD => 1,

            Name::LT => 1,
            Name::GT => 1,
            Name::EQ => 1,
            Name::ISZERO => 1,
            Name::SLT => 1,
            Name::SGT => 1,

            Name::OR => 1,
            Name::XOR => 1,
            Name::NOT => 1,
            Name::AND => 1,
            Name::SHL => 1,
            Name::SHR => 1,
            Name::SAR => 1,
            Name::BYTE => 1,

            Name::ADDMOD => 1,
            Name::MULMOD => 1,
            Name::EXP => 1,
            Name::SIGNEXTEND => 1,
            Name::SHA3 => 1,
            Name::KECCAK256 => 1,

            Name::MLOAD => 1,

            Name::SLOAD => 1,
            Name::PUSHIMMUTABLE => 1,

            Name::CALLDATALOAD => 1,
            Name::CALLDATASIZE => 1,
            Name::CODESIZE => 1,
            Name::PUSHSIZE => 1,
            Name::EXTCODESIZE => 1,
            Name::RETURNDATASIZE => 1,

            Name::ADDRESS => 1,
            Name::CALLER => 1,
            Name::TIMESTAMP => 1,
            Name::NUMBER => 1,
            Name::GAS => 1,

            Name::CALL => 1,
            Name::CALLCODE => 1,
            Name::STATICCALL => 1,
            Name::DELEGATECALL => 1,

            Name::CREATE => 1,
            Name::CREATE2 => 1,

            Name::PC => 1,
            Name::CALLVALUE => 1,
            Name::MSIZE => 1,
            Name::BALANCE => 1,
            Name::SELFBALANCE => 1,
            Name::CHAINID => 1,
            Name::ORIGIN => 1,
            Name::BLOCKHASH => 1,
            Name::COINBASE => 1,
            Name::DIFFICULTY => 1,
            Name::GASLIMIT => 1,
            Name::BASEFEE => 1,
            Name::EXTCODEHASH => 1,

            _ => 0,
        }
    }

    ///
    /// Returns the stack depth, where the instruction can reach.
    ///
    pub const fn stack_depth(&self) -> usize {
        match self.name {
            Name::DUP1 => 1,
            Name::DUP2 => 2,
            Name::DUP3 => 3,
            Name::DUP4 => 4,
            Name::DUP5 => 5,
            Name::DUP6 => 6,
            Name::DUP7 => 7,
            Name::DUP8 => 8,
            Name::DUP9 => 9,
            Name::DUP10 => 10,
            Name::DUP11 => 11,
            Name::DUP12 => 12,
            Name::DUP13 => 13,
            Name::DUP14 => 14,
            Name::DUP15 => 15,
            Name::DUP16 => 16,

            Name::SWAP1 => 2,
            Name::SWAP2 => 3,
            Name::SWAP3 => 4,
            Name::SWAP4 => 5,
            Name::SWAP5 => 6,
            Name::SWAP6 => 7,
            Name::SWAP7 => 8,
            Name::SWAP8 => 9,
            Name::SWAP9 => 10,
            Name::SWAP10 => 11,
            Name::SWAP11 => 12,
            Name::SWAP12 => 13,
            Name::SWAP13 => 14,
            Name::SWAP14 => 15,
            Name::SWAP15 => 16,
            Name::SWAP16 => 17,

            _ => self.input_size(),
        }
    }

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
        let input_size = self.input_size();
        let mut arguments = Vec::with_capacity(input_size);
        for index in 0..input_size {
            let pointer = context.evm().stack_pointer(index + 1);
            let value = context.build_load(pointer, format!("argument_{}", index).as_str());
            arguments.push(value);
        }
        context.evm_mut().decrease_stack_pointer(input_size);
        arguments
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Instruction
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm<'ctx, 'dep>(
        mut self,
        context: &mut compiler_llvm_context::Context<'ctx, 'dep, D>,
    ) -> anyhow::Result<()> {
        let value = match self.name {
            Name::PUSH => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH_Data => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH_Tag => stack::push_tag(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH_Dollar => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH_HashDollar => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),

            Name::PUSH1 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH2 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH3 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH4 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH5 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH6 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH7 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH8 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH9 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH10 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH11 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH12 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH13 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH14 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH15 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH16 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH17 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH18 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH19 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH20 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH21 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH22 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH23 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH24 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH25 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH26 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH27 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH28 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH29 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH30 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH31 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::PUSH32 => stack::push(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),

            Name::DUP1 => stack::dup(context, 1),
            Name::DUP2 => stack::dup(context, 2),
            Name::DUP3 => stack::dup(context, 3),
            Name::DUP4 => stack::dup(context, 4),
            Name::DUP5 => stack::dup(context, 5),
            Name::DUP6 => stack::dup(context, 6),
            Name::DUP7 => stack::dup(context, 7),
            Name::DUP8 => stack::dup(context, 8),
            Name::DUP9 => stack::dup(context, 9),
            Name::DUP10 => stack::dup(context, 10),
            Name::DUP11 => stack::dup(context, 11),
            Name::DUP12 => stack::dup(context, 12),
            Name::DUP13 => stack::dup(context, 13),
            Name::DUP14 => stack::dup(context, 14),
            Name::DUP15 => stack::dup(context, 15),
            Name::DUP16 => stack::dup(context, 16),

            Name::SWAP1 => stack::swap(context, 1),
            Name::SWAP2 => stack::swap(context, 2),
            Name::SWAP3 => stack::swap(context, 3),
            Name::SWAP4 => stack::swap(context, 4),
            Name::SWAP5 => stack::swap(context, 5),
            Name::SWAP6 => stack::swap(context, 6),
            Name::SWAP7 => stack::swap(context, 7),
            Name::SWAP8 => stack::swap(context, 8),
            Name::SWAP9 => stack::swap(context, 9),
            Name::SWAP10 => stack::swap(context, 10),
            Name::SWAP11 => stack::swap(context, 11),
            Name::SWAP12 => stack::swap(context, 12),
            Name::SWAP13 => stack::swap(context, 13),
            Name::SWAP14 => stack::swap(context, 14),
            Name::SWAP15 => stack::swap(context, 15),
            Name::SWAP16 => stack::swap(context, 16),

            Name::POP => stack::pop(context),

            Name::Tag => panic!("Cannot remain in the IR"),
            Name::JUMP => panic!("Cannot remain in the IR"),
            Name::JUMPI => panic!("Cannot remain in the IR"),
            Name::JUMPDEST => panic!("Cannot remain in the IR"),

            Name::ADD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::addition(context, arguments)
            }
            Name::SUB => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::subtraction(context, arguments)
            }
            Name::MUL => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::multiplication(context, arguments)
            }
            Name::DIV => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::division(context, arguments)
            }
            Name::MOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::remainder(context, arguments)
            }
            Name::SDIV => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::division_signed(context, arguments)
            }
            Name::SMOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::arithmetic::remainder_signed(context, arguments)
            }

            Name::LT => {
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
            Name::GT => {
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
            Name::EQ => {
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
            Name::ISZERO => {
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
            Name::SLT => {
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
            Name::SGT => {
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

            Name::OR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::or(context, arguments)
            }
            Name::XOR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::xor(context, arguments)
            }
            Name::NOT => {
                let mut arguments = self.pop_arguments(context);
                compiler_llvm_context::bitwise::xor(
                    context,
                    [
                        arguments.remove(0),
                        context.field_type().const_all_ones().as_basic_value_enum(),
                    ],
                )
            }
            Name::AND => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::and(context, arguments)
            }
            Name::SHL => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::shift_left(context, arguments)
            }
            Name::SHR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::shift_right(context, arguments)
            }
            Name::SAR => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::shift_right_arithmetic(context, arguments)
            }
            Name::BYTE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::bitwise::byte(context, arguments)
            }

            Name::ADDMOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::add_mod(context, arguments)
            }
            Name::MULMOD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::mul_mod(context, arguments)
            }
            Name::EXP => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::exponent(context, arguments)
            }
            Name::SIGNEXTEND => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::math::sign_extend(context, arguments)
            }

            Name::SHA3 => {
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
            Name::KECCAK256 => {
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

            Name::MLOAD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::memory::load(context, arguments)
            }
            Name::MSTORE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::memory::store(context, arguments)
            }
            Name::MSTORE8 => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::memory::store_byte(context, arguments)
            }

            Name::SLOAD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::storage::load(context, arguments)
            }
            Name::SSTORE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::storage::store(context, arguments)
            }
            Name::PUSHIMMUTABLE => storage::push_immutable(
                context,
                self.value
                    .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
            ),
            Name::ASSIGNIMMUTABLE => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                storage::assign_immutable(
                    context,
                    arguments,
                    self.value
                        .ok_or_else(|| anyhow::anyhow!("Instruction value missing"))?,
                )
            }

            Name::CALLDATALOAD => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::calldata::load(context, arguments)
            }
            Name::CALLDATASIZE => compiler_llvm_context::calldata::size(context),
            Name::CALLDATACOPY => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            Name::CODESIZE => compiler_llvm_context::calldata::size(context),
            Name::CODECOPY => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            Name::PUSHSIZE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::EXTCODESIZE => {
                let _arguments = self.pop_arguments(context);
                Ok(Some(context.field_const(0xffff).as_basic_value_enum()))
            }
            Name::RETURNDATASIZE => compiler_llvm_context::return_data::size(context),
            Name::RETURNDATACOPY => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::return_data::copy(context, arguments)
            }

            Name::RETURN => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::r#return::r#return(context, arguments)
            }
            Name::REVERT => {
                let arguments = self
                    .pop_arguments(context)
                    .try_into()
                    .expect("Always valid");
                compiler_llvm_context::r#return::revert(context, arguments)
            }
            Name::STOP => compiler_llvm_context::r#return::stop(context),
            Name::INVALID => compiler_llvm_context::r#return::invalid(context),

            Name::LOG0 => {
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
            Name::LOG1 => {
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
            Name::LOG2 => {
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
            Name::LOG3 => {
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
            Name::LOG4 => {
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

            Name::CALL => {
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
            Name::CALLCODE => {
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
            Name::STATICCALL => {
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
            Name::DELEGATECALL => {
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

            Name::CREATE => {
                let _arguments = self.pop_arguments(context);
                Ok(None)
            }
            Name::CREATE2 => {
                let _arguments = self.pop_arguments(context);
                Ok(None)
            }

            Name::ADDRESS => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::Address,
            ),
            Name::CALLER => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::MessageSender,
            ),
            Name::TIMESTAMP => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::BlockTimestamp,
            ),
            Name::NUMBER => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::BlockNumber,
            ),
            Name::GAS => compiler_llvm_context::contract_context::get(
                context,
                compiler_common::ContextValue::GasLeft,
            ),

            Name::GASLIMIT => Ok(Some(
                context.field_const(u32::MAX as u64).as_basic_value_enum(),
            )),
            Name::BASEFEE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::COINBASE => Ok(Some(context.field_const(0).as_basic_value_enum())),

            Name::CALLVALUE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::BALANCE => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::SELFBALANCE => Ok(Some(context.field_const(0).as_basic_value_enum())),

            Name::ORIGIN => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::CHAINID => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::BLOCKHASH => Ok(Some(context.field_const(0).as_basic_value_enum())),

            Name::MSIZE => Ok(Some(
                context
                    .field_const(((1 << 16) * compiler_common::SIZE_FIELD) as u64)
                    .as_basic_value_enum(),
            )),
            Name::DIFFICULTY => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::PC => Ok(Some(context.field_const(0).as_basic_value_enum())),

            Name::EXTCODECOPY => {
                let _arguments = self.pop_arguments(context);
                Ok(None)
            }
            Name::EXTCODEHASH => {
                let _arguments = self.pop_arguments(context);
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            Name::SELFDESTRUCT => {
                let _arguments = self.pop_arguments(context);
                Ok(None)
            }
        }?;

        if let Some(value) = value {
            let pointer = context.evm().stack_pointer(0);
            context.build_store(pointer, value);
            context.evm_mut().increase_stack_pointer(1);
        }

        Ok(())
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:16}{:64}",
            self.name,
            match self.value {
                Some(ref value) => value.as_str(),
                None => "",
            }
        )
    }
}
