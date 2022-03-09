//!
//! The single-line comment lexeme.
//!

///
/// The single-line comment lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {}

impl Comment {
    /// The start symbol.
    pub const START: &'static str = "//";
    /// The end symbol.
    pub const END: &'static str = "\n";

    ///
    /// Returns the comment's length, including the trimmed whitespace around it.
    ///
    pub fn parse(input: &str) -> usize {
        let end_position = input.find(Self::END).unwrap_or(input.len());
        end_position + Self::END.len()
    }
}
