//!
//! The LLVM intrinsic function.
//!

use crate::generator::llvm::Context as LLVMContext;
use inkwell::types::BasicType;

///
/// The LLVM intrinsic function.
///
#[derive(Debug, Clone)]
pub enum Intrinsic {
    /// The contract storage load.
    StorageLoad,
    /// The contract storage store.
    StorageStore,
    /// The contract storage set.
    SetStorage,
    /// The event emitting.
    Event,

    /// The contract context switch.
    SwitchContext,
    /// The contract execution remaining cycles.
    CyclesRemain,
    /// The another contract function call.
    FarCall,
    /// The error throwing.
    Throw,

    /// The hash absorbing.
    HashAbsorb,
    /// The hash absorbing with reset.
    HashAbsorbReset,
    /// The hash output.
    HashOutput,

    /// The memory copy.
    MemoryCopy,
    /// The memory copy to parent.
    MemoryCopyToParent,
    /// The memory copy from child.
    MemoryCopyFromChild,
    /// The memory move.
    MemoryMove,
    /// The memory set.
    MemorySet,
}

impl Intrinsic {
    ///
    /// Returns the inner LLVM intrinsic function identifier.
    ///
    pub fn name(&self) -> &'static str {
        match self {
            Intrinsic::StorageLoad => "llvm.syncvm.sload",
            Intrinsic::StorageStore => "llvm.syncvm.sstore",
            Intrinsic::SetStorage => "llvm.syncvm.setstorage",
            Intrinsic::Event => "llvm.syncvm.event",

            Intrinsic::SwitchContext => "llvm.syncvm.switchcontext",
            Intrinsic::CyclesRemain => "llvm.syncvm.cyclesremain",
            Intrinsic::FarCall => "llvm.syncvm.farcall",
            Intrinsic::Throw => "llvm.syncvm.throw",

            Intrinsic::HashAbsorb => "llvm.syncvm.habs",
            Intrinsic::HashAbsorbReset => "llvm.syncvm.habsr",
            Intrinsic::HashOutput => "llvm.syncvm.hout",

            Intrinsic::MemoryCopy => "llvm.memcpy",
            Intrinsic::MemoryCopyToParent => "llvm.memcpy",
            Intrinsic::MemoryCopyFromChild => "llvm.memcpy",
            Intrinsic::MemoryMove => "llvm.memmov",
            Intrinsic::MemorySet => "llvm.memset",
        }
    }

    ///
    /// Returns the LLVM types for selecting via the signature.
    ///
    pub fn argument_types<'ctx>(
        &self,
        context: &LLVMContext<'ctx>,
    ) -> Vec<inkwell::types::BasicTypeEnum<'ctx>> {
        match self {
            Self::StorageLoad => vec![],
            Self::StorageStore => vec![],
            Self::SetStorage => vec![],
            Self::Event => vec![],

            Self::SwitchContext => vec![],
            Self::CyclesRemain => vec![],
            Self::FarCall => vec![],
            Self::Throw => vec![],

            Self::HashAbsorb => vec![],
            Self::HashAbsorbReset => vec![],
            Self::HashOutput => vec![],

            Self::MemoryCopy => vec![
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .as_basic_type_enum(),
            ],
            Self::MemoryCopyToParent => vec![
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Global)
                    .as_basic_type_enum(), // TODO: change the address space '2'
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .as_basic_type_enum(),
            ],
            Self::MemoryCopyFromChild => vec![
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Shared)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .as_basic_type_enum(),
            ],
            Self::MemoryMove => vec![
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .as_basic_type_enum(),
            ],
            Self::MemorySet => vec![
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .ptr_type(inkwell::AddressSpace::Generic)
                    .as_basic_type_enum(),
                context
                    .integer_type(compiler_const::bitlength::FIELD)
                    .as_basic_type_enum(),
            ],
        }
    }
}
