//!
//! The string literal lexeme.
//!

use std::fmt;

///
/// The string literal lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub struct String {
    /// The inner string contents.
    pub inner: std::string::String,
    /// Whether the string is hexadecimal.
    pub is_hexadecimal: bool,
}

impl String {
    ///
    /// Creates a string literal value.
    ///
    pub fn new(inner: ::std::string::String, is_hexadecimal: bool) -> Self {
        Self {
            inner,
            is_hexadecimal,
        }
    }

    ///
    /// Parses the value from the source code slice.
    ///
    pub fn parse(input: &str) -> Option<(usize, Self)> {
        let mut length = 0;

        let is_string = input[length..].starts_with('"');
        let is_hex_string = input[length..].starts_with(r#"hex""#);

        if !is_string && !is_hex_string {
            return None;
        }

        if is_string {
            length += 1;
        }
        if is_hex_string {
            length += r#"hex""#.len();
        }

        let mut string = std::string::String::new();
        while !input[length..].starts_with('"') {
            string.push(input.chars().nth(length).expect("Always exists"));
            length += 1;
        }

        length += 1;
        let string = string
            .strip_prefix('"')
            .and_then(|string| string.strip_suffix('"'))
            .unwrap_or(string.as_str())
            .to_owned();

        let literal = Self::new(string, is_hex_string);

        Some((length, literal))
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}
