//!
//! The project contract state.
//!

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

use crate::build::contract::Contract as ContractBuild;
use crate::project::contract::Contract;

///
/// The project contract state.
///
#[derive(Debug)]
pub enum State {
    /// The contract is waiting for be built.
    Source(Contract),
    /// The contract is being built.
    Waiter(Arc<(Mutex<()>, Condvar)>),
    /// The contract is built.
    Build(ContractBuild),
}

impl State {
    ///
    /// A shortcut waiter constructor.
    ///
    pub fn waiter() -> Arc<(Mutex<()>, Condvar)> {
        Arc::new((Mutex::new(()), Condvar::new()))
    }
}
