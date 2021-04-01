//!
//! YUL to LLVM compiler library.
//!

#[cfg(test)]
mod tests;

pub mod generator;
pub mod lexer;
pub mod parser;

pub use self::generator::action::Action;

/// The `bool` type bitlength.
pub const BITLENGTH_BOOLEAN: usize = 1;

/// The `uint` / `uint256` type bitlength.
pub const BITLENGTH_DEFAULT: usize = 256;
