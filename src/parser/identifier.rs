//!
//! The YUL source code identifier.
//!

use regex::Regex;

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
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
    pub fn parse_list<I>(iter: &mut I, first: Lexeme) -> Vec<Self>
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let name = match first {
            Lexeme::Identifier(identifier) => identifier,
            Lexeme::Symbol(Symbol::ParenthesisRight) => return vec![],
            lexeme => panic!("expected identifier, got {}", lexeme),
        };
        let mut result = vec![Self {
            name,
            yul_type: None,
        }];

        while let Some(Lexeme::Symbol(Symbol::Comma)) = iter.next() {
            let name = match iter.next().expect("expected identifier") {
                Lexeme::Identifier(identifier) if Self::is_valid(identifier.as_str()) => identifier,
                lexeme => panic!("expected an identifier in identifier list, got {}", lexeme),
            };
            result.push(Self {
                name,
                yul_type: None,
            });
        }

        result
    }

    // TODO: support declarations w/o initialization
    pub fn parse_typed_list<I>(iter: &mut I, terminator: Lexeme) -> Vec<Self>
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let mut result = Vec::new();
        while let Some(lexeme) = iter.next() {
            if lexeme == terminator {
                break;
            }

            if let Lexeme::Identifier(identifier) = lexeme {
                if Self::is_valid(identifier.as_str()) {
                    result.push(Self {
                        name: identifier,
                        yul_type: None,
                    });
                }
            }

            match iter
                .peek()
                .cloned()
                .expect("unexpected end for typed parameter list")
            {
                Lexeme::Symbol(Symbol::Comma) => {
                    iter.next();
                    continue;
                }
                lexeme if lexeme == terminator => {
                    iter.next();
                    break;
                }
                _lexeme => {}
            }

            // let r#type = match iter.peek().expect("unexpected end for typed parameter list") {
            //     Lexeme::Symbol(Symbol::Colon) => {
            //         iter.next();
            //         match iter.next().expect("expected identifier") {
            //             Lexeme::Identifier(_) => true,
            //             _lexeme => panic!("expected identifier"),
            //         }
            //     },
            //     _ => false,
            // };
        }

        result
    }

    pub fn is_valid(value: &str) -> bool {
        let id_pattern = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9.]*$").expect("invalid regex");
        id_pattern.is_match(value)
    }
}
