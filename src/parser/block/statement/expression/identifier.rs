//!
//! The YUL source code identifier.
//!

use regex::Regex;

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
    ) -> (Vec<Self>, Option<Lexeme>) {
        let mut result = Vec::new();

        loop {
            let lexeme = match initial.take() {
                Some(lexeme) => lexeme,
                None => lexer.next(),
            };

            match lexeme {
                Lexeme::Identifier(identifier) if Self::is_valid(identifier.as_str()) => {
                    result.push(Self {
                        name: identifier,
                        yul_type: None,
                    });
                }
                Lexeme::Symbol(Symbol::Colon) => {
                    lexer.next();
                }
                Lexeme::Symbol(Symbol::Comma) => {}
                lexeme => return (result, Some(lexeme)),
            }
        }
    }

    pub fn is_valid(value: &str) -> bool {
        let id_pattern = Regex::new(r"^[a-zA-Z_\$][a-zA-Z0-9_\$\.]*$").expect("invalid regex");
        id_pattern.is_match(value)
    }
}
