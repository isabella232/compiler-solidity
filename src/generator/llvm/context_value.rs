//!
//! The contract context value.
//!

///
/// The contract context value.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextValue {
    /// The `msg.sender` value.
    MessageSender,
    /// The `block.number` value.
    BlockNumber,
    /// The `block.timestamp` value.
    BlockTimestamp,
    /// The `gas()` value.
    GasLeft,
    /// The `msg.sig` value.
    MessageSignature,
    /// The remaining execution cycles value.
    RemainingCycles,
}

impl From<ContextValue> for u64 {
    fn from(value: ContextValue) -> Self {
        match value {
            ContextValue::MessageSender => 0,
            ContextValue::BlockNumber => 1,
            ContextValue::BlockTimestamp => 2,
            ContextValue::GasLeft => 3,
            ContextValue::MessageSignature => 4,
            ContextValue::RemainingCycles => 5,
        }
    }
}
