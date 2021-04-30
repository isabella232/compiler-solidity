//!
//! The LLVM intrinsic function.
//!

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
    /// The contract local function call.
    LocalCall,
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
}

impl From<Intrinsic> for &'static str {
    fn from(value: Intrinsic) -> Self {
        match value {
            Intrinsic::StorageLoad => "llvm.syncvm.sload",
            Intrinsic::StorageStore => "llvm.syncvm.sstore",
            Intrinsic::SetStorage => "llvm.syncvm.setstorage",
            Intrinsic::Event => "llvm.syncvm.event",

            Intrinsic::SwitchContext => "llvm.syncvm.switchcontext",
            Intrinsic::CyclesRemain => "llvm.syncvm.cyclesremain",
            Intrinsic::LocalCall => "llvm.syncvm.localcall",
            Intrinsic::FarCall => "llvm.syncvm.farcall",
            Intrinsic::Throw => "llvm.syncvm.throw",

            Intrinsic::HashAbsorb => "llvm.syncvm.habs",
            Intrinsic::HashAbsorbReset => "llvm.syncvm.habsr",
            Intrinsic::HashOutput => "llvm.syncvm.hout",
        }
    }
}
