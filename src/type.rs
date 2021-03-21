//!
//! Datatype for a lexeme for further analysis and translation.
//!

///
/// Datatype for a lexeme for further analysis and translation.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Bool,
    Int(u32),
    UInt(u32),
    Unknown(String),
}
