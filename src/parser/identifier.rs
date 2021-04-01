//!
//! The YUL source code identifier.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::r#type::Type;

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
    pub fn parse_list(
        lexer: &mut Lexer,
        mut initial: Option<Lexeme>,
    ) -> (Vec<String>, Option<Lexeme>) {
        let mut result = Vec::new();

        loop {
            let lexeme = initial.take().unwrap_or_else(|| lexer.next());

            match lexeme {
                Lexeme::Identifier(identifier) => {
                    result.push(identifier);
                }
                Lexeme::Symbol(Symbol::Comma) => {}
                lexeme => return (result, Some(lexeme)),
            }
        }
    }

    pub fn parse_typed_list(
        lexer: &mut Lexer,
        mut initial: Option<Lexeme>,
    ) -> (Vec<Self>, Option<Lexeme>) {
        let mut result = Vec::new();

        loop {
            let lexeme = initial.take().unwrap_or_else(|| lexer.next());

            match lexeme {
                Lexeme::Identifier(identifier) => {
                    let yul_type = match lexer.peek() {
                        Lexeme::Symbol(Symbol::Colon) => {
                            lexer.next();
                            Some(Type::parse(lexer, None))
                        }
                        _ => None,
                    };
                    result.push(Self {
                        name: identifier,
                        yul_type,
                    });
                }
                Lexeme::Symbol(Symbol::Comma) => {}
                lexeme => return (result, Some(lexeme)),
            }
        }
    }
}