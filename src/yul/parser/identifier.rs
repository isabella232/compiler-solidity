//!
//! The YUL source code identifier.
//!

use crate::yul::lexer::lexeme::symbol::Symbol;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;
use crate::yul::parser::r#type::Type;

///
/// The YUL source code identifier.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    /// The identifier string.
    pub name: String,
    /// The type, if it has been explicitly specified.
    pub yul_type: Option<Type>,
}

impl Identifier {
    ///
    /// Parses the identifier list where the types cannot be specified.
    ///
    pub fn parse_list(
        lexer: &mut Lexer,
        mut initial: Option<Lexeme>,
    ) -> anyhow::Result<(Vec<String>, Option<Lexeme>)> {
        let mut result = Vec::new();

        let mut expected_comma = false;
        loop {
            let lexeme = crate::yul::parser::take_or_next(initial.take(), lexer)?;

            match lexeme {
                Lexeme::Identifier(identifier) if !expected_comma => {
                    result.push(identifier);
                    expected_comma = true;
                }
                Lexeme::Symbol(Symbol::Comma) if expected_comma => {
                    expected_comma = false;
                }
                lexeme => return Ok((result, Some(lexeme))),
            }
        }
    }

    ///
    /// Parses the identifier list where the types may be optionally specified.
    ///
    pub fn parse_typed_list(
        lexer: &mut Lexer,
        mut initial: Option<Lexeme>,
    ) -> anyhow::Result<(Vec<Self>, Option<Lexeme>)> {
        let mut result = Vec::new();

        let mut expected_comma = false;
        loop {
            let lexeme = crate::yul::parser::take_or_next(initial.take(), lexer)?;

            match lexeme {
                Lexeme::Identifier(identifier) if !expected_comma => {
                    let yul_type = match lexer.peek()? {
                        Lexeme::Symbol(Symbol::Colon) => {
                            lexer.next()?;
                            Some(Type::parse(lexer, None)?)
                        }
                        _ => None,
                    };
                    result.push(Self {
                        name: identifier,
                        yul_type,
                    });
                    expected_comma = true;
                }
                Lexeme::Symbol(Symbol::Comma) if expected_comma => {
                    expected_comma = false;
                }
                lexeme => return Ok((result, Some(lexeme))),
            }
        }
    }
}
