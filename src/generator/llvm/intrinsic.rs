//!
//! The LLVM intrinsic function.
//!

use inkwell::types::BasicType;

use crate::generator::llvm::Context as LLVMContext;

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
    /// The contract context getter.
    GetFromContext,
    /// The external contract call.
    FarCall,
    /// The external contract code call.
    CallCode,
    /// The external contract delegate call.
    DelegateCall,
    /// The external contract static call.
    StaticCall,

    /// The hash absorbing.
    HashAbsorb,
    /// The hash absorbing with reset.
    HashAbsorbReset,
    /// The hash output.
    HashOutput,

    /// The memory copy.
    MemoryCopy,
    /// The memory copy from parent.
    MemoryCopyFromParent,
    /// The memory copy to parent.
    MemoryCopyToParent,
    /// The memory copy from child.
    MemoryCopyFromChild,
    /// The memory copy to child.
    MemoryCopyToChild,
    /// The memory copy from child to parent.
    MemoryCopyFromChildToParent,
    /// The memory move.
    MemoryMove,
    /// The memory set.
    MemorySet,

    /// The `eq` flag getter.
    EqualsFlag,
    /// The `gt` flag getter.
    GreaterFlag,
    /// The `lt`/overflow flag getter.
    LesserFlag,
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
            Intrinsic::GetFromContext => "llvm.syncvm.getfromcontext",
            Intrinsic::FarCall => "llvm.syncvm.farcall",
            Intrinsic::CallCode => "llvm.syncvm.callcode",
            Intrinsic::DelegateCall => "llvm.syncvm.delegatecall",
            Intrinsic::StaticCall => "llvm.syncvm.staticcall",

            Intrinsic::HashAbsorb => "llvm.syncvm.habs",
            Intrinsic::HashAbsorbReset => "llvm.syncvm.habsr",
            Intrinsic::HashOutput => "llvm.syncvm.hout",

            Intrinsic::MemoryCopy => "llvm.memcpy",
            Intrinsic::MemoryCopyFromParent => "llvm.memcpy",
            Intrinsic::MemoryCopyToParent => "llvm.memcpy",
            Intrinsic::MemoryCopyFromChild => "llvm.memcpy",
            Intrinsic::MemoryCopyToChild => "llvm.memcpy",
            Intrinsic::MemoryCopyFromChildToParent => "llvm.memcpy",
            Intrinsic::MemoryMove => "llvm.memmov",
            Intrinsic::MemorySet => "llvm.memset",

            Intrinsic::EqualsFlag => "llvm.syncvm.eqflag",
            Intrinsic::LesserFlag => "llvm.syncvm.ltflag",
            Intrinsic::GreaterFlag => "llvm.syncvm.gtflag",
        }
    }

    ///
    /// Returns the LLVM types for selecting via the signature.
    ///
    pub fn argument_types<'ctx, 'src>(
        &self,
        context: &LLVMContext<'ctx, 'src>,
    ) -> Vec<inkwell::types::BasicTypeEnum<'ctx>> {
        match self {
            Self::StorageLoad => vec![],
            Self::StorageStore => vec![],
            Self::SetStorage => vec![],
            Self::Event => vec![],

            Self::SwitchContext => vec![],
            Self::GetFromContext => vec![],
            Self::FarCall => vec![],
            Self::CallCode => vec![],
            Self::DelegateCall => vec![],
            Self::StaticCall => vec![],

            Self::HashAbsorb => vec![],
            Self::HashAbsorbReset => vec![],
            Self::HashOutput => vec![],

            Self::MemoryCopy => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemoryCopyFromParent => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Parent.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemoryCopyToParent => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Parent.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemoryCopyFromChild => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Child.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemoryCopyToChild => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Child.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemoryCopyFromChildToParent => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Parent.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Child.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemoryMove => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .as_basic_type_enum(),
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            Self::MemorySet => vec![
                context
                    .field_type()
                    .ptr_type(compiler_common::AddressSpace::Stack.into())
                    .as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],

            Self::EqualsFlag => vec![],
            Self::GreaterFlag => vec![],
            Self::LesserFlag => vec![],
        }
    }
}
