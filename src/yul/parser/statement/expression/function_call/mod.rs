//!
//! The function call subexpression.
//!

pub mod name;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::yul::lexer::lexeme::symbol::Symbol;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::statement::expression::Expression;

use self::name::Name;

///
/// The function call subexpression.
///
#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    /// The function name.
    pub name: Name,
    /// The function arguments expression list.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> anyhow::Result<Self> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        let name = match lexeme {
            Lexeme::Identifier(identifier) => Name::from(identifier.as_str()),
            lexeme => {
                anyhow::bail!("Expected one of {:?}, found `{}`", ["{identifier}"], lexeme);
            }
        };

        let mut arguments = Vec::new();
        loop {
            let argument = match lexer.next()? {
                Lexeme::Symbol(Symbol::ParenthesisRight) => break,
                lexeme => Expression::parse(lexer, Some(lexeme))?,
            };

            arguments.push(argument);

            match lexer.peek()? {
                Lexeme::Symbol(Symbol::Comma) => {
                    lexer.next()?;
                    continue;
                }
                Lexeme::Symbol(Symbol::ParenthesisRight) => {
                    lexer.next()?;
                    break;
                }
                _ => break,
            }
        }

        Ok(Self { name, arguments })
    }

    ///
    /// Converts the function call into an LLVM value.
    ///
    pub fn into_llvm<'ctx, D>(
        mut self,
        context: &mut compiler_llvm_context::Context<'ctx, D>,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>>
    where
        D: compiler_llvm_context::Dependency,
    {
        match self.name {
            Name::UserDefined(name)
                if name
                    .starts_with(compiler_llvm_context::Function::ZKSYNC_NEAR_CALL_ABI_PREFIX) =>
            {
                let mut values = Vec::with_capacity(self.arguments.len());
                for argument in self.arguments.into_iter() {
                    let value = argument.into_llvm(context)?.expect("Always exists").value;
                    values.push(value);
                }
                let function = context
                    .functions
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| panic!("Undeclared function {}", name));

                if let Some(compiler_llvm_context::FunctionReturn::Compound { size, .. }) =
                    function.r#return
                {
                    let r#type =
                        context
                            .structure_type(vec![context.field_type().as_basic_type_enum(); size]);
                    let pointer = context.build_alloca(
                        r#type,
                        format!("{}_near_call_return_pointer_argument", name).as_str(),
                    );
                    context.build_store(pointer, r#type.const_zero());
                    values.insert(1, pointer.as_basic_value_enum());
                }

                let function_pointer = context.builder().build_bitcast(
                    function.value,
                    context
                        .field_type()
                        .ptr_type(compiler_llvm_context::AddressSpace::Stack.into()),
                    format!("{}_near_call_function_pointer", name).as_str(),
                );
                values.insert(
                    0,
                    function_pointer.into_pointer_value().as_basic_value_enum(),
                );

                let return_value = context.build_invoke_near_call_abi(
                    function.value,
                    values,
                    format!("{}_near_call", name).as_str(),
                );

                if let Some(compiler_llvm_context::FunctionReturn::Compound { .. }) =
                    function.r#return
                {
                    let return_pointer = return_value.expect("Always exists").into_pointer_value();
                    let return_value = context.build_load(
                        return_pointer,
                        format!("{}_near_call_return_value", name).as_str(),
                    );
                    Ok(Some(return_value))
                } else {
                    Ok(return_value)
                }
            }
            Name::UserDefined(name) => {
                let mut values = Vec::with_capacity(self.arguments.len());
                for argument in self.arguments.into_iter() {
                    let value = argument.into_llvm(context)?.expect("Always exists").value;
                    values.push(value);
                }
                let function = context
                    .functions
                    .get(name.as_str())
                    .cloned()
                    .unwrap_or_else(|| panic!("Undeclared function {}", name));

                if let Some(compiler_llvm_context::FunctionReturn::Compound { size, .. }) =
                    function.r#return
                {
                    let r#type =
                        context
                            .structure_type(vec![context.field_type().as_basic_type_enum(); size]);
                    let pointer = context
                        .build_alloca(r#type, format!("{}_return_pointer_argument", name).as_str());
                    context.build_store(pointer, r#type.const_zero());
                    values.insert(0, pointer.as_basic_value_enum());
                }

                let return_value = context.build_invoke(
                    function.value,
                    values.as_slice(),
                    format!("{}_call", name).as_str(),
                );

                if let Some(compiler_llvm_context::FunctionReturn::Compound { .. }) =
                    function.r#return
                {
                    let return_pointer = return_value.expect("Always exists").into_pointer_value();
                    let return_value = context
                        .build_load(return_pointer, format!("{}_return_value", name).as_str());
                    Ok(Some(return_value))
                } else {
                    Ok(return_value)
                }
            }

            Name::Add => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::addition(context, arguments)
            }
            Name::Sub => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::subtraction(context, arguments)
            }
            Name::Mul => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::multiplication(context, arguments)
            }
            Name::Div => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::division(context, arguments)
            }
            Name::Mod => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::remainder(context, arguments)
            }
            Name::Sdiv => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::division_signed(context, arguments)
            }
            Name::Smod => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::arithmetic::remainder_signed(context, arguments)
            }

            Name::Lt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::ULT,
                )
            }
            Name::Gt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::UGT,
                )
            }
            Name::Eq => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::EQ,
                )
            }
            Name::IsZero => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    [arguments[0], context.field_const(0).as_basic_value_enum()],
                    inkwell::IntPredicate::EQ,
                )
            }
            Name::Slt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::SLT,
                )
            }
            Name::Sgt => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::comparison::compare(
                    context,
                    arguments,
                    inkwell::IntPredicate::SGT,
                )
            }

            Name::Or => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::or(context, arguments)
            }
            Name::Xor => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::xor(context, arguments)
            }
            Name::Not => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::bitwise::xor(
                    context,
                    [
                        arguments[0],
                        context.field_type().const_all_ones().as_basic_value_enum(),
                    ],
                )
            }
            Name::And => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::and(context, arguments)
            }
            Name::Shl => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::shift_left(context, arguments)
            }
            Name::Shr => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::shift_right(context, arguments)
            }
            Name::Sar => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::shift_right_arithmetic(context, arguments)
            }
            Name::Byte => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::bitwise::byte(context, arguments)
            }
            Name::Pop => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(None)
            }

            Name::AddMod => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::math::add_mod(context, arguments)
            }
            Name::MulMod => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::math::mul_mod(context, arguments)
            }
            Name::Exp => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::math::exponent(context, arguments)
            }
            Name::SignExtend => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::math::sign_extend(context, arguments)
            }

            Name::Keccak256 => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::hash::keccak256(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                )
            }

            Name::MLoad => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::memory::load(context, arguments)
            }
            Name::MStore => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::memory::store(context, arguments)
            }
            Name::MStore8 => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::memory::store_byte(context, arguments)
            }

            Name::SLoad => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::storage::load(context, arguments)
            }
            Name::SStore => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::storage::store(context, arguments)
            }
            Name::LoadImmutable => {
                let mut arguments = self.pop_arguments::<D, 1>(context)?;
                let key = arguments[0]
                    .original
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("`load_immutable` literal is missing"))?;
                compiler_llvm_context::immutable::load(context, key)
            }
            Name::SetImmutable => {
                let mut arguments = self.pop_arguments::<D, 3>(context)?;
                let key = arguments[1]
                    .original
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("`load_immutable` literal is missing"))?;
                let value = arguments[2].value.into_int_value();
                compiler_llvm_context::immutable::store(context, key, value)
            }

            Name::CallDataLoad => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                compiler_llvm_context::calldata::load(context, arguments)
            }
            Name::CallDataSize => compiler_llvm_context::calldata::size(context),
            Name::CallDataCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            Name::CodeSize => compiler_llvm_context::calldata::size(context),
            Name::CodeCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::calldata::copy(context, arguments)
            }
            Name::ExtCodeSize => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(Some(context.field_const(0xffff).as_basic_value_enum()))
            }
            Name::ReturnDataSize => compiler_llvm_context::return_data::size(context),
            Name::ReturnDataCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::return_data::copy(context, arguments)
            }

            Name::Return => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::r#return::r#return(context, arguments)
            }
            Name::Revert => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::r#return::revert(context, arguments)
            }
            Name::Stop => compiler_llvm_context::r#return::stop(context),
            Name::Invalid => compiler_llvm_context::r#return::invalid(context),

            Name::Log0 => {
                let arguments = self.pop_arguments_llvm::<D, 2>(context)?;
                compiler_llvm_context::event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    vec![],
                )
            }
            Name::Log1 => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                compiler_llvm_context::event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            Name::Log2 => {
                let arguments = self.pop_arguments_llvm::<D, 4>(context)?;
                compiler_llvm_context::event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            Name::Log3 => {
                let arguments = self.pop_arguments_llvm::<D, 5>(context)?;
                compiler_llvm_context::event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }
            Name::Log4 => {
                let arguments = self.pop_arguments_llvm::<D, 6>(context)?;
                compiler_llvm_context::event::log(
                    context,
                    arguments[0].into_int_value(),
                    arguments[1].into_int_value(),
                    arguments[2..]
                        .iter()
                        .map(|argument| argument.into_int_value())
                        .collect(),
                )
            }

            Name::Call => {
                let arguments = self.pop_arguments_llvm::<D, 7>(context)?;

                let gas = arguments[0].into_int_value();
                let address = arguments[1].into_int_value();
                let value = arguments[2].into_int_value();
                let input_offset = arguments[3].into_int_value();
                let input_size = arguments[4].into_int_value();
                let output_offset = arguments[5].into_int_value();
                let output_size = arguments[6].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    context.runtime.far_call,
                    gas,
                    address,
                    Some(value),
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::CallCode => {
                let _arguments = self.pop_arguments_llvm::<D, 7>(context)?;
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            Name::StaticCall => {
                let arguments = self.pop_arguments_llvm::<D, 6>(context)?;

                let gas = arguments[0].into_int_value();
                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    context.runtime.static_call,
                    gas,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }
            Name::DelegateCall => {
                let arguments = self.pop_arguments_llvm::<D, 6>(context)?;

                let gas = arguments[0].into_int_value();
                let address = arguments[1].into_int_value();
                let input_offset = arguments[2].into_int_value();
                let input_size = arguments[3].into_int_value();
                let output_offset = arguments[4].into_int_value();
                let output_size = arguments[5].into_int_value();

                compiler_llvm_context::contract::call(
                    context,
                    context.runtime.delegate_call,
                    gas,
                    address,
                    None,
                    input_offset,
                    input_size,
                    output_offset,
                    output_size,
                )
            }

            Name::Create => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;

                let value = arguments[0].into_int_value();
                let input_offset = arguments[1].into_int_value();
                let input_size = arguments[2].into_int_value();

                compiler_llvm_context::create::create(context, value, input_offset, input_size)
            }
            Name::Create2 => {
                let arguments = self.pop_arguments_llvm::<D, 4>(context)?;

                let value = arguments[0].into_int_value();
                let input_offset = arguments[1].into_int_value();
                let input_size = arguments[2].into_int_value();
                let salt = arguments[3].into_int_value();

                compiler_llvm_context::create::create2(
                    context,
                    value,
                    input_offset,
                    input_size,
                    Some(salt),
                )
            }
            Name::DataOffset => {
                let mut arguments = self.pop_arguments::<D, 1>(context)?;
                let identifier = arguments[0]
                    .original
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("`dataoffset` object identifier is missing"))?;
                compiler_llvm_context::create::contract_hash(context, identifier)
            }
            Name::DataSize => {
                let mut arguments = self.pop_arguments::<D, 1>(context)?;
                let identifier = arguments[0]
                    .original
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("`dataoffset` object identifier is missing"))?;
                compiler_llvm_context::create::contract_hash_size(context, identifier)
            }
            Name::DataCopy => {
                let arguments = self.pop_arguments_llvm::<D, 3>(context)?;
                let offset = context.builder().build_int_add(
                    arguments[0].into_int_value(),
                    context.field_const(
                        (compiler_common::SIZE_X32 + compiler_common::SIZE_FIELD) as u64,
                    ),
                    "datacopy_contract_hash_offset",
                );
                let value = arguments[1];
                compiler_llvm_context::memory::store(context, [offset.as_basic_value_enum(), value])
            }

            Name::LinkerSymbol => {
                let mut arguments = self.pop_arguments::<D, 1>(context)?;
                let path = arguments[0]
                    .original
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("Linker symbol literal is missing"))?;

                Ok(Some(
                    context
                        .resolve_library(path.as_str())?
                        .as_basic_value_enum(),
                ))
            }
            Name::MemoryGuard => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(Some(arguments[0]))
            }

            Name::Address => Ok(context.build_call(
                context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::Address),
                &[],
                "address",
            )),
            Name::Caller => Ok(context.build_call(
                context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::Caller),
                &[],
                "caller",
            )),
            Name::Timestamp => {
                let meta_packed = context
                    .build_call(
                        context
                            .get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::Meta),
                        &[],
                        "meta",
                    )
                    .expect("Always exists");
                let meta_shifted = context.builder().build_right_shift(
                    meta_packed.into_int_value(),
                    context.field_const(compiler_common::BITLENGTH_X64 as u64),
                    false,
                    "meta_shifted",
                );
                let block_timestamp = context.builder().build_and(
                    meta_shifted,
                    context.field_const(u64::MAX),
                    "block_number",
                );
                Ok(Some(block_timestamp.as_basic_value_enum()))
            }
            Name::Number => {
                let meta_packed = context
                    .build_call(
                        context
                            .get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::Meta),
                        &[],
                        "meta",
                    )
                    .expect("Always exists");
                let block_number = context.builder().build_and(
                    meta_packed.into_int_value(),
                    context.field_const(u64::MAX),
                    "block_number",
                );
                Ok(Some(block_number.as_basic_value_enum()))
            }
            Name::Origin => Ok(context.build_call(
                context.get_intrinsic_function(compiler_llvm_context::IntrinsicFunction::TxOrigin),
                &[],
                "tx_origin",
            )),
            Name::Gas => compiler_llvm_context::ether_gas::gas(context),
            Name::CallValue => compiler_llvm_context::ether_gas::value(context),

            Name::Balance => {
                let arguments = self.pop_arguments_llvm::<D, 1>(context)?;

                let address = arguments[0].into_int_value();
                compiler_llvm_context::ether_gas::balance(context, address)
            }
            Name::SelfBalance => {
                let address = context
                    .build_call(
                        context.get_intrinsic_function(
                            compiler_llvm_context::IntrinsicFunction::Address,
                        ),
                        &[],
                        "self_balance_address",
                    )
                    .expect("Always exists")
                    .into_int_value();

                compiler_llvm_context::ether_gas::balance(context, address)
            }

            Name::GasLimit => Ok(Some(
                context.field_const(u32::MAX as u64).as_basic_value_enum(),
            )),
            Name::MSize => Ok(Some(
                context
                    .field_const(((1 << 16) * compiler_common::SIZE_FIELD) as u64)
                    .as_basic_value_enum(),
            )),

            Name::GasPrice => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::ChainId => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::BlockHash => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::Difficulty => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::Pc => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::CoinBase => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::BaseFee => Ok(Some(context.field_const(0).as_basic_value_enum())),
            Name::ExtCodeCopy => {
                let _arguments = self.pop_arguments_llvm::<D, 4>(context)?;
                Ok(None)
            }
            Name::ExtCodeHash => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(Some(context.field_const(0).as_basic_value_enum()))
            }
            Name::SelfDestruct => {
                let _arguments = self.pop_arguments_llvm::<D, 1>(context)?;
                Ok(None)
            }
        }
    }

    ///
    /// Pops the specified number of arguments, converted into their LLVM values.
    ///
    fn pop_arguments_llvm<'ctx, D, const N: usize>(
        &mut self,
        context: &mut compiler_llvm_context::Context<'ctx, D>,
    ) -> anyhow::Result<[inkwell::values::BasicValueEnum<'ctx>; N]>
    where
        D: compiler_llvm_context::Dependency,
    {
        let mut arguments = Vec::with_capacity(N);
        for expression in self.arguments.drain(0..N) {
            arguments.push(expression.into_llvm(context)?.expect("Always exists").value);
        }

        Ok(arguments.try_into().expect("Always successful"))
    }

    ///
    /// Pops the specified number of arguments.
    ///
    fn pop_arguments<'ctx, D, const N: usize>(
        &mut self,
        context: &mut compiler_llvm_context::Context<'ctx, D>,
    ) -> anyhow::Result<[compiler_llvm_context::Argument<'ctx>; N]>
    where
        D: compiler_llvm_context::Dependency,
    {
        let mut arguments = Vec::with_capacity(N);
        for expression in self.arguments.drain(0..N) {
            arguments.push(expression.into_llvm(context)?.expect("Always exists"));
        }

        Ok(arguments.try_into().expect("Always successful"))
    }
}
