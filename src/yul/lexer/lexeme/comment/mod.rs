//!
//! The comment lexeme.
//!

pub mod multi_line;
pub mod single_line;

use self::multi_line::Comment as MultiLineComment;
use self::single_line::Comment as SingleLineComment;

///
/// The comment lexeme.
///
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Comment {
    /// The single-line comment.
    SingleLine(SingleLineComment),
    /// The multi-line comment.
    MultiLine(MultiLineComment),
}

impl Comment {
    ///
    /// Returns the comment's length, including the trimmed whitespace around it.
    ///
    pub fn parse(input: &str) -> Option<usize> {
        let mut length = 0;
        let trimmed_start = input.trim_start();
        length += input.len() - trimmed_start.len();

        if trimmed_start.starts_with(SingleLineComment::START) {
            return Some(length + SingleLineComment::parse(trimmed_start));
        }

        if trimmed_start.starts_with(MultiLineComment::START) {
            return Some(length + MultiLineComment::parse(trimmed_start));
        }

        None
    }
}
