//!
//! The source code block.
//!

pub mod statement;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::parser::identifier::Identifier;

use self::statement::assignment::Assignment;
use self::statement::expression::Expression;
use self::statement::for_loop::ForLoop;
use self::statement::function_definition::FunctionDefinition;
use self::statement::if_conditional::IfConditional;
use self::statement::switch::Switch;
use self::statement::variable_declaration::VariableDeclaration;
use self::statement::Statement;

///
/// The source code block.
///
#[derive(Debug, PartialEq)]
pub struct Block {
    /// The block statements.
    pub statements: Vec<Statement>,
}

impl Block {
    ///
    /// Parses a block, panic if a block is ill-formed.
    ///
    pub fn parse<I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = Lexeme>,
    {
        let mut statements = Vec::new();

        loop {
            let lexeme = iter.next().expect("unexpected eof");
            match lexeme {
                Lexeme::Keyword(Keyword::Function) => statements.push(
                    Statement::FunctionDefinition(FunctionDefinition::parse(iter)),
                ),
                Lexeme::Keyword(Keyword::Let) => statements.push(Statement::VariableDeclaration(
                    VariableDeclaration::parse(iter),
                )),
                Lexeme::Keyword(Keyword::If) => {
                    statements.push(Statement::IfConditional(IfConditional::parse(iter)))
                }
                Lexeme::Keyword(Keyword::Switch) => {
                    statements.push(Statement::Switch(Switch::parse(iter)))
                }
                Lexeme::Keyword(Keyword::For) => {
                    statements.push(Statement::ForLoop(ForLoop::parse(iter)))
                }
                Lexeme::Keyword(Keyword::Break) => statements.push(Statement::Break),
                Lexeme::Keyword(Keyword::Continue) => statements.push(Statement::Continue),
                Lexeme::Keyword(Keyword::Leave) => statements.push(Statement::Leave),
                Lexeme::Keyword(Keyword::True) | Lexeme::Keyword(Keyword::False) => {
                    statements.push(Statement::Expression(Expression::parse(iter, Some(lexeme))));
                }
                Lexeme::Identifier(ref identifier)
                    if !Identifier::is_valid(identifier.as_str()) =>
                {
                    statements.push(Statement::Expression(Expression::parse(iter, Some(lexeme))));
                }
                Lexeme::Identifier(_) => {
                    let lookahead = iter.peek().cloned();
                    match lookahead {
                        Some(Lexeme::Symbol(Symbol::Assignment)) => {
                            statements.push(Statement::Assignment(Assignment::parse(iter, lexeme)));
                        }
                        Some(Lexeme::Symbol(Symbol::Comma)) => {
                            statements.push(Statement::Assignment(Assignment::parse(iter, lexeme)));
                        }
                        Some(_) => {
                            statements
                                .push(Statement::Expression(Expression::parse(iter, Some(lexeme))));
                        }
                        None => {
                            panic!("unexpected end of input");
                        }
                    }
                }
                Lexeme::Symbol(Symbol::BracketCurlyLeft) => {
                    statements.push(Statement::Block(Block::parse(iter)))
                }
                Lexeme::Symbol(Symbol::ParenthesisRight) => break,
                Lexeme::Symbol(Symbol::BracketCurlyRight) => break,
                _ => panic!("YUL is malformed"),
            }
        }

        Self { statements }
    }
}
