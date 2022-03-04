//!
//! The compiler lexer.
//!

#[cfg(test)]
mod tests;

pub mod error;
pub mod lexeme;

use self::error::Error;
use self::lexeme::comment::Comment;
use self::lexeme::keyword::Keyword;
use self::lexeme::literal::boolean::Boolean as BooleanLiteral;
use self::lexeme::literal::integer::Integer as IntegerLiteral;
use self::lexeme::literal::string::String as StringLiteral;
use self::lexeme::literal::Literal;
use self::lexeme::symbol::Symbol;
use self::lexeme::Lexeme;

///
/// The compiler lexer.
///
pub struct Lexer {
    /// The input source code.
    input: String,
    /// The tokenization regular expression.
    regexp: regex::Regex,
    /// The position in the source code.
    index: usize,
    /// The peeked lexeme, waiting to be fetched.
    peeked: Option<Lexeme>,
}

impl Lexer {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(mut input: String) -> Self {
        Self::remove_comments(&mut input);
        input.push('\n');

        Self {
            input,
            regexp: Symbol::regexp(),
            index: 0,
            peeked: None,
        }
    }

    ///
    /// Advances the lexer, returning the next lexeme.
    ///
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Lexeme, Error> {
        if let Some(peeked) = self.peeked.take() {
            return Ok(peeked);
        }

        loop {
            let is_string = self.input[self.index..].starts_with('"');
            let is_hex_string = self.input[self.index..].starts_with(r#"hex""#);
            if is_string || is_hex_string {
                if is_string {
                    self.index += 1;
                }
                if is_hex_string {
                    self.index += r#"hex""#.len();
                }
                let mut string = String::new();
                while !self.input[self.index..].starts_with('"') {
                    string.push(self.input.chars().nth(self.index).expect("Always exists"));
                    self.index += 1;
                }
                self.index += 1;
                let string = string
                    .strip_prefix('"')
                    .and_then(|string| string.strip_suffix('"'))
                    .unwrap_or_else(|| string.as_str())
                    .to_owned();
                return Ok(Lexeme::Literal(Literal::String(StringLiteral::new(
                    string,
                    is_hex_string,
                ))));
            }

            let r#match = match self.regexp.find(&self.input[self.index..]) {
                Some(r#match) => r#match,
                None => return Ok(Lexeme::EndOfFile),
            };

            let lexeme = if r#match.start() != 0 {
                let lexeme = match Keyword::try_from(
                    &self.input[self.index..self.index + r#match.start()],
                ) {
                    Ok(keyword) => match BooleanLiteral::try_from(keyword) {
                        Ok(literal) => Lexeme::Literal(Literal::Boolean(literal)),
                        Err(keyword) => Lexeme::Keyword(keyword),
                    },
                    Err(string) => {
                        let decimal = regex::Regex::new("^[0-9]+$").expect("Regexp is valid");
                        let hexadecimal =
                            regex::Regex::new(r#"^0x[0-9a-fA-F]+$"#).expect("Regexp is valid");

                        if decimal.is_match(string.as_str()) {
                            Lexeme::Literal(Literal::Integer(IntegerLiteral::new_decimal(string)))
                        } else if hexadecimal.is_match(string.as_str()) {
                            Lexeme::Literal(Literal::Integer(IntegerLiteral::new_hexadecimal(
                                string,
                            )))
                        } else if Lexeme::is_identifier(string.as_str()) {
                            Lexeme::Identifier(string)
                        } else {
                            return Err(Error::invalid_lexeme(string));
                        }
                    }
                };
                self.index += r#match.start();
                lexeme
            } else if !r#match.as_str().trim().is_empty() {
                let lexeme = match Symbol::try_from(r#match.as_str()) {
                    Ok(symbol) => Lexeme::Symbol(symbol),
                    Err(string) => return Err(Error::invalid_lexeme(string)),
                };
                self.index += r#match.as_str().len();
                lexeme
            } else {
                self.index += r#match.as_str().len();
                continue;
            };

            return Ok(lexeme);
        }
    }

    ///
    /// Peeks the next lexeme without advancing the iterator.
    ///
    pub fn peek(&mut self) -> Result<Lexeme, Error> {
        match self.peeked {
            Some(ref peeked) => Ok(peeked.clone()),
            None => {
                let peeked = self.next()?;
                self.peeked = Some(peeked.clone());
                Ok(peeked)
            }
        }
    }

    ///
    /// Tokenizes the entire input source code string.
    ///
    /// Only for testing purposes.
    ///
    pub fn tokenize(&mut self) -> Result<Vec<Lexeme>, Error> {
        let mut lexemes = Vec::new();
        loop {
            match self.next()? {
                Lexeme::EndOfFile => return Ok(lexemes),
                lexeme => lexemes.push(lexeme),
            }
        }
    }

    ///
    /// Returns the unprocessed slice of the input.
    ///
    pub fn remaining(&self) -> &str {
        &self.input[self.index..]
    }

    ///
    /// Removes comments from the given source code.
    ///
    fn remove_comments(src: &mut String) {
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
