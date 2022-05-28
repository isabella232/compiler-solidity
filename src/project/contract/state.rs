//!
//! The project contract state.
//!

use crate::build::contract::Contract as ContractBuild;
use crate::project::contract::Contract;

///
/// The project contract state.
///
#[derive(Debug)]
pub enum State {
    /// The contract is waiting for be built.
    Source(Contract),
    /// The contract is built.
    Build(ContractBuild),
}
