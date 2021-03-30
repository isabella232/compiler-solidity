//!
//! The keyword lexeme.
//!

use std::convert::TryFrom;
use std::fmt;

///
/// The keyword lexeme.
///
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    /// The `function` keyword.
    Function,
    /// The `let` keyword.
    Let,
    /// The `if` keyword.
    If,
    /// The `switch` keyword.
    Switch,
    /// The `case` keyword.
    Case,
    /// The `default` keyword.
    Default,
    /// The `for` keyword.
    For,
    /// The `break` keyword.
    Break,
    /// The `continue` keyword.
    Continue,
    /// The `leave` keyword.
    Leave,
    /// The `true` keyword.
    True,
    /// The `false` keyword.
    False,
    /// The `bool` keyword.
    Bool,
    /// The `int{N}` keyword.
    Int(usize),
    /// The `uint{N}` keyword.
    Uint(usize),
}

impl TryFrom<&str> for Keyword {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        if let Some(input) = input.strip_prefix("int") {
            if let Ok(bitlength) = input.parse::<usize>() {
                return Ok(Self::Int(bitlength));
            }
        }

        if let Some(input) = input.strip_prefix("uint") {
            if let Ok(bitlength) = input.parse::<usize>() {
                return Ok(Self::Uint(bitlength));
            }
        }

        Ok(match input {
            "function" => Self::Function,
            "let" => Self::Let,
            "if" => Self::If,
            "switch" => Self::Switch,
            "case" => Self::Case,
            "default" => Self::Default,
            "for" => Self::For,
            "break" => Self::Break,
            "continue" => Self::Continue,
            "leave" => Self::Leave,
            "true" => Self::True,
            "false" => Self::False,
            "bool" => Self::Bool,

            _ => return Err(input.to_owned()),
        })
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Let => write!(f, "let"),
            Self::If => write!(f, "if"),
            Self::Switch => write!(f, "switch"),
            Self::Case => write!(f, "case"),
            Self::Default => write!(f, "default"),
            Self::For => write!(f, "for"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Leave => write!(f, "leave"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Bool => write!(f, "bool"),
            Self::Int(bitlength) => write!(f, "int{}", bitlength),
            Self::Uint(bitlength) => write!(f, "uint{}", bitlength),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::statement::Statement;
    use crate::parser::block::Block;

    #[test]
    fn ok_break() {
        let input = r#"{
            break
        }"#;

        let result = crate::tests::parse(input);
        assert_eq!(
            result,
            [Statement::Block(Block {
                statements: vec![Statement::Break]
            })]
        );
    }

    #[test]
    fn ok_continue() {
        let input = r#"{
            continue
        }"#;

        let result = crate::tests::parse(input);
        assert_eq!(
            result,
            [Statement::Block(Block {
                statements: vec![Statement::Continue]
            })]
        );
    }

    #[test]
    fn ok_leave() {
        let input = r#"{
            leave
        }"#;

        let result = crate::tests::parse(input);
        assert_eq!(
            result,
            [Statement::Block(Block {
                statements: vec![Statement::Leave]
            })]
        );
    }
}
