//!
//! The contract dependency in different compilation states.
//!

use std::convert::TryFrom;

use crate::parser::statement::object::Object;
use crate::source_data::SourceData;

///
/// The contract dependency at different compilation states.
///
#[derive(Debug, Clone)]
pub enum Dependency {
    /// The parsed Yul object.
    Parsed(Object),
    /// The compiled zkEVM bytecode.
    Compiled(Vec<u8>),
}

impl Dependency {
    ///
    /// A shortcut constructor.
    ///
    pub fn new_object(object: Object) -> Self {
        Self::Parsed(object)
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_bytecode(bytecode: Vec<u8>) -> Self {
        Self::Compiled(bytecode)
    }

    ///
    /// Compiles the Yul object, if it has not been compiled yet.
    ///
    pub fn compile(self, optimization_level: inkwell::OptimizationLevel) -> Vec<u8> {
        match self {
            Self::Parsed(object) => {
                let identifier = object.identifier.clone();
                let llvm_ir = SourceData::new(object)
                    .compile(optimization_level, optimization_level, false)
                    .unwrap_or_else(|error| {
                        panic!("Dependency `{}` compiling error: {:?}", identifier, error)
                    });
                let assembly =
                    zkevm_assembly::Assembly::try_from(llvm_ir).unwrap_or_else(|error| {
                        panic!(
                            "Dependency `{}` assembly parsing error: {}",
                            identifier, error
                        )
                    });
                Vec::<u8>::from(&assembly)
            }
            Self::Compiled(bytecode) => bytecode,
        }
    }
}
