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

use crate::error::Error;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;

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
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Lexeme>,
    ) -> Result<(Self, Option<Lexeme>), Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;

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
            lexeme => Err(ParserError::expected_one_of(
                vec![
                    "object", "code", "function", "let", "if", "switch", "for", "break",
                    "continue", "leave",
                ],
                lexeme,
                None,
            )
            .into()),
        }
    }

    ///
    /// Converts the statement into a block.
    ///
    /// # Panics
    /// If there statement is not a block.
    ///
    pub fn into_block(self) -> Block {
        match self {
            Self::Block(block) => block,
            _ => panic!("Expected block"),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_leave() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                x := 42
                if lt(x, 55) {
                    leave
                }
                x := 43
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_continue() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                x := 0
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    if mod(i, 2) {
                        continue
                    }
                    x := add(i, x)
                }
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }

    #[test]
    fn ok_break() {
        let input = r#"object "Test" { code {
            function foo() -> x {
                x:= 0
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    if gt(x, 18) {
                        break
                    }
                    x := add(i, x)
                }
            }
        }}"#;

        assert!(crate::Project::try_from_test_yul(input).is_ok());
    }
}
