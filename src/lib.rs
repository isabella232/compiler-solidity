//!
//! YUL to LLVM compiler library.
//!

#[cfg(test)]
mod tests;

pub mod generator;
pub mod lexer;
pub mod parser;

pub use self::generator::action::Action;

pub const BITLENGTH_BOOLEAN: usize = 1;
pub const BITLENGTH_DEFAULT: usize = 256;
