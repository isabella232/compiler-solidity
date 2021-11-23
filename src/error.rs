//!
//! The Yul compiler error.
//!

use crate::lexer::error::Error as LexerError;
use crate::parser::error::Error as ParserError;

///
/// The Yul compiler error.
///
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
    /// The file system error.
    FileSystem(std::io::Error),
    /// The JSON conversion error.
    Json(serde_json::Error),
    /// The Solidity compiler error.
    Solc(String),
    /// The library input is invalid.
    LibraryInput(String),
    /// The input contains no contracts.
    NoContractsFound,
    /// The specified contract cannot be found.
    ContractNotFound(String),

    /// The lexer error.
    Lexer(LexerError),
    /// The parser error.
    Parser(ParserError),
    /// The LLVM error.
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
        Self::Json(error)
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
