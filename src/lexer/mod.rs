//!
//! The compiler lexer.
//!

#[cfg(test)]
mod tests;

pub mod lexeme;

use std::convert::TryFrom;

use self::lexeme::keyword::Keyword;
use self::lexeme::literal::boolean::Boolean as BooleanLiteral;
use self::lexeme::literal::integer::Integer as IntegerLiteral;
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
    pub fn next(&mut self) -> Lexeme {
        if let Some(peeked) = self.peeked.take() {
            return peeked;
        }

        loop {
            let r#match = match self.regexp.find(&self.input[self.index..]) {
                Some(r#match) => r#match,
                None => return Lexeme::EndOfFile,
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
                            regex::Regex::new("^0x[0-9a-fA-F]+$").expect("Regexp is valid");

                        if decimal.is_match(string.as_str()) {
                            Lexeme::Literal(Literal::Integer(IntegerLiteral::new_decimal(string)))
                        } else if hexadecimal.is_match(string.as_str()) {
                            Lexeme::Literal(Literal::Integer(IntegerLiteral::new_hexadecimal(
                                string,
                            )))
                        } else if string.starts_with('"') {
                            Lexeme::Literal(Literal::String(string.into()))
                        } else {
                            Lexeme::Identifier(string)
                        }
                    }
                };
                self.index += r#match.start();
                lexeme
            } else if !r#match.as_str().trim().is_empty() {
                let lexeme = match Symbol::try_from(r#match.as_str()) {
                    Ok(symbol) => Lexeme::Symbol(symbol),
                    Err(token) => panic!("invalid token `{}`", token),
                };
                self.index += r#match.as_str().len();
                lexeme
            } else {
                self.index += r#match.as_str().len();
                continue;
            };

            return lexeme;
        }
    }

    ///
    /// Peeks the next lexeme without advancing the iterator.
    ///
    pub fn peek(&mut self) -> Lexeme {
        match self.peeked {
            Some(ref peeked) => peeked.clone(),
            None => {
                let peeked = self.next();
                self.peeked = Some(peeked.clone());
                peeked
            }
        }
    }

    ///
    /// Tokenizes the entire input source code string.
    ///
    /// Only for testing purposes.
    ///
    pub fn tokenize(&mut self) -> Vec<Lexeme> {
        let mut lexemes = Vec::new();
        loop {
            match self.next() {
                Lexeme::EndOfFile => return lexemes,
                lexeme => lexemes.push(lexeme),
            }
        }
    }

    ///
    /// Removes comments from the given source code.
    ///
    fn remove_comments(src: &mut String) {
        let mut comment = src.find("//");
        while comment != None {
            let pos = comment.unwrap();
            let eol = src[pos..].find('\n').unwrap_or(src.len() - pos) + pos;
            src.replace_range(pos..eol, "");
            comment = src.find("//");
        }
    }
}
