//!
//! The compiler lexer.
//!

pub mod lexeme;

use std::convert::TryFrom;

use self::lexeme::keyword::Keyword;
use self::lexeme::symbol::Symbol;
use self::lexeme::Lexeme;

///
/// The compiler lexer.
///
pub struct Lexer {
    input: String,
}

impl Lexer {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(mut input: String) -> Self {
        Self::remove_comments(&mut input);
        Self { input }
    }

    ///
    /// Provides vector of tokens for a given source.
    ///
    pub fn get_lexemes(&mut self) -> Vec<Lexeme> {
        let mut lexemes = Vec::new();
        let mut index = 0;
        self.input.push('\n');

        let regex = Symbol::regexp();
        while let Some(r#match) = regex.find(&self.input[index..]) {
            if r#match.start() != 0 {
                let lexeme = match Keyword::try_from(&self.input[index..index + r#match.start()]) {
                    Ok(keyword) => Lexeme::Keyword(keyword),
                    Err(string) => Lexeme::Identifier(string),
                };
                lexemes.push(lexeme);
            }
            if !r#match.as_str().trim().is_empty() {
                let lexeme = Symbol::try_from(r#match.as_str())
                    .map(Lexeme::Symbol)
                    .unwrap();
                lexemes.push(lexeme);
            }
            index += r#match.end();
        }

        lexemes
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
