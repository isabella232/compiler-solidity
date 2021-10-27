//!
//! The symbol lexeme.
//!

use std::fmt;

///
/// The symbol lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    /// The `:=` symbol.
    Assignment,
    /// The `->` symbol.
    Arrow,
    /// The `{` symbol.
    BracketCurlyLeft,
    /// The `}` symbol.
    BracketCurlyRight,
    /// The `(` symbol.
    ParenthesisLeft,
    /// The `)` symbol.
    ParenthesisRight,
    /// The `,` symbol.
    Comma,
    /// The `:` symbol.
    Colon,
    /// The `/*` symbol.
    CommentStart,
    /// The `*/` symbol.
    CommentEnd,
}

impl Symbol {
    ///
    /// Returns the regexp used for matching the symbol.
    ///
    pub fn regexp() -> regex::Regex {
        regex::Regex::new(r"(\s+)|(:=)|(\->)|[{}(),:]|(/\*)|(\*/)").expect("Regexp is valid")
    }
}

impl TryFrom<&str> for Symbol {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Ok(match input {
            ":=" => Self::Assignment,
            "->" => Self::Arrow,
            "{" => Self::BracketCurlyLeft,
            "}" => Self::BracketCurlyRight,
            "(" => Self::ParenthesisLeft,
            ")" => Self::ParenthesisRight,
            "," => Self::Comma,
            ":" => Self::Colon,
            "/*" => Self::CommentStart,
            "*/" => Self::CommentEnd,

            _ => return Err(input.to_owned()),
        })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assignment => write!(f, ":="),
            Self::Arrow => write!(f, "->"),
            Self::BracketCurlyLeft => write!(f, "{{"),
            Self::BracketCurlyRight => write!(f, "}}"),
            Self::ParenthesisLeft => write!(f, "("),
            Self::ParenthesisRight => write!(f, ")"),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::CommentStart => write!(f, "/*"),
            Self::CommentEnd => write!(f, "*/"),
        }
    }
}
