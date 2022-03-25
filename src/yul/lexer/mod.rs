//!
//! The compiler lexer.
//!

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
            if let Some(length) = Comment::parse(&self.input[self.index..]) {
                self.index += length;
                continue;
            }

            if let Some((length, literal)) = StringLiteral::parse(&self.input[self.index..]) {
                self.index += length;
                let lexeme = Lexeme::Literal(Literal::String(literal));
                return Ok(lexeme);
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
                        if let Some(literal) = IntegerLiteral::parse(string.as_str()) {
                            Lexeme::Literal(Literal::Integer(literal))
                        } else if Lexeme::is_identifier(string.as_str()) {
                            Lexeme::Identifier(string)
                        } else {
                            dbg!(&string);
                            return Err(Error::invalid_lexeme(string));
                        }
                    }
                };
                self.index += r#match.start();
                lexeme
            } else if !r#match.as_str().trim().is_empty() {
                let lexeme = match Symbol::try_from(r#match.as_str()) {
                    Ok(symbol) => Lexeme::Symbol(symbol),
                    Err(string) => {
                        return Err(Error::invalid_lexeme(string));
                    }
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
}
