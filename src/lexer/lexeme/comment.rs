//!
//! The comment lexeme.
//!

///
/// The comment lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Comment {
    /// The `// ... \n` comment.
    SingleLine,
    /// The `/* ... */` comment.
    MultiLine,
}

impl Comment {
    ///
    /// Removes comments from the given source code.
    ///
    pub fn remove_all(src: &mut String) {
        loop {
            let next_multiline = src.find("/*");
            let next_oneline = src.find("//");

            let (position, r#type) = match (next_multiline, next_oneline) {
                (Some(next_multiline), Some(next_oneline)) if next_oneline < next_multiline => {
                    (next_oneline, Comment::SingleLine)
                }
                (Some(next_multiline), Some(_next_oneline)) => (next_multiline, Comment::MultiLine),
                (Some(next_multiline), None) => (next_multiline, Comment::MultiLine),
                (None, Some(next_oneline)) => (next_oneline, Comment::SingleLine),
                (None, None) => break,
            };

            match r#type {
                Comment::SingleLine => {
                    let end_of_line =
                        src[position..].find('\n').unwrap_or(src.len() - position) + position;
                    src.replace_range(position..end_of_line, "");
                }
                Comment::MultiLine => {
                    let end_of_comment =
                        src[position..].find("*/").unwrap_or(src.len() - position) + position;
                    src.replace_range(position..end_of_comment + 2, "");
                }
            }
        }
    }
}
