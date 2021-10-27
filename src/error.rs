//!
//! The Yul compiler error.
//!

use crate::lexer::error::Error as LexerError;
use crate::parser::error::Error as ParserError;

///
/// The Yul compiler error.
///
#[derive(Debug)]
pub enum Error {
    /// The file system error.
    FileSystem(std::io::Error),
    /// The input error.
    Input(serde_json::Error),
    /// The Solidity error.
    Solidity(&'static str),
    /// The contract cannot be found.
    ContractNotFound,
    /// If there is multiple contracts, the main contract name must be specified.
    ContractNotSpecified,
    /// The lexer error.
    Lexer(LexerError),
    /// The parser error.
    Parser(ParserError),
    /// The LLVM error.
    #[allow(clippy::upper_case_acronyms)]
    LLVM(String),
    /// The assembly error.
    Assembly(zkevm_assembly::AssemblyParseError),
}

impl PartialEq<Self> for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Lexer(first), Self::Lexer(second)) => first == second,
            (Self::Parser(first), Self::Parser(second)) => first == second,
            _ => false,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::FileSystem(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Input(error)
    }
}

impl From<LexerError> for Error {
    fn from(error: LexerError) -> Self {
        Self::Lexer(error)
    }
}

impl From<ParserError> for Error {
    fn from(error: ParserError) -> Self {
        Self::Parser(error)
    }
}

impl From<zkevm_assembly::AssemblyParseError> for Error {
    fn from(error: zkevm_assembly::AssemblyParseError) -> Self {
        Self::Assembly(error)
    }
}
