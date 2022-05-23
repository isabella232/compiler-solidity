//!
//! The block statement.
//!

pub mod assignment;
pub mod block;
pub mod code;
pub mod expression;
pub mod for_loop;
pub mod function_definition;
pub mod if_conditional;
pub mod object;
pub mod switch;
pub mod variable_declaration;

use crate::yul::lexer::lexeme::keyword::Keyword;
use crate::yul::lexer::lexeme::Lexeme;
use crate::yul::lexer::Lexer;

use self::assignment::Assignment;
use self::block::Block;
use self::code::Code;
use self::expression::Expression;
use self::for_loop::ForLoop;
use self::function_definition::FunctionDefinition;
use self::if_conditional::IfConditional;
use self::object::Object;
use self::switch::Switch;
use self::variable_declaration::VariableDeclaration;

///
/// The block statement.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    /// The object element.
    Object(Object),
    /// The code element.
    Code(Code),
    /// The code block.
    Block(Block),
    /// The expression.
    Expression(Expression),
    /// The `function` statement.
    FunctionDefinition(FunctionDefinition),
    /// The `let` statement.
    VariableDeclaration(VariableDeclaration),
    /// The `:=` existing variables reassignment statement.
    Assignment(Assignment),
    /// The `if` statement.
    IfConditional(IfConditional),
    /// The `switch` statement.
    Switch(Switch),
    /// The `for` statement.
    ForLoop(ForLoop),
    /// The `continue` statement.
    Continue,
    /// The `break` statement.
    Break,
    /// The `leave` statement.
    Leave,
}

impl Statement {
    ///
    /// The element parser.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Lexeme>,
    ) -> anyhow::Result<(Self, Option<Lexeme>)> {
        let lexeme = crate::yul::parser::take_or_next(initial, lexer)?;

        match lexeme {
            lexeme @ Lexeme::Keyword(Keyword::Object) => {
                Ok((Statement::Object(Object::parse(lexer, Some(lexeme))?), None))
            }
            Lexeme::Keyword(Keyword::Code) => {
                Ok((Statement::Code(Code::parse(lexer, None)?), None))
            }
            Lexeme::Keyword(Keyword::Function) => Ok((
                Statement::FunctionDefinition(FunctionDefinition::parse(lexer, None)?),
                None,
            )),
            Lexeme::Keyword(Keyword::Let) => {
                let (statement, next) = VariableDeclaration::parse(lexer, None)?;
                Ok((Statement::VariableDeclaration(statement), next))
            }
            Lexeme::Keyword(Keyword::If) => Ok((
                Statement::IfConditional(IfConditional::parse(lexer, None)?),
                None,
            )),
            Lexeme::Keyword(Keyword::Switch) => {
                Ok((Statement::Switch(Switch::parse(lexer, None)?), None))
            }
            Lexeme::Keyword(Keyword::For) => {
                Ok((Statement::ForLoop(ForLoop::parse(lexer, None)?), None))
            }
            Lexeme::Keyword(Keyword::Continue) => Ok((Statement::Continue, None)),
            Lexeme::Keyword(Keyword::Break) => Ok((Statement::Break, None)),
            Lexeme::Keyword(Keyword::Leave) => Ok((Statement::Leave, None)),
            lexeme => {
                anyhow::bail!(
                    "Expected one of {:?}, found `{}`",
                    [
                        "object", "code", "function", "let", "if", "switch", "for", "continue",
                        "break", "leave",
                    ],
                    lexeme
                );
            }
        }
    }
}
