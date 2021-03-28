//!
//! The function definition statement.
//!

use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::block::Block;
use crate::parser::identifier::Identifier;

///
/// The function definition statement.
///
#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Identifier>,
    pub result: Vec<Identifier>, // TODO: investigate
    pub body: Block,
}

impl FunctionDefinition {
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let name = match iter.next() {
            Some(Lexeme::Identifier(name)) if Identifier::is_valid(name.as_str()) => name,
            lexeme => panic!(
                "function name must follow 'function' keyword, got {:?}",
                lexeme
            ),
        };

        match iter.next() {
            Some(Lexeme::Symbol(Symbol::ParenthesisLeft)) => {}
            lexeme => panic!(
                "expected '(' in {} function definition, got {:?}",
                name, lexeme
            ),
        }

        let parameters =
            Identifier::parse_typed_list(iter, Lexeme::Symbol(Symbol::ParenthesisRight));

        let result = match iter.peek().expect("unexpected eof") {
            Lexeme::Symbol(Symbol::Arrow) => {
                iter.next();
                Identifier::parse_typed_list(iter, Lexeme::Symbol(Symbol::BracketCurlyLeft))
            }
            Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                iter.next();
                vec![]
            }
            lexeme => panic!("expected -> or {{, got {}", lexeme),
        };

        let body = Block::parse(iter);

        Self {
            name,
            parameters,
            result,
            body,
        }
    }
}
