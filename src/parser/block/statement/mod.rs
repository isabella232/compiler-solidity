//!
//! The block statement.
//!

pub mod assignment;
pub mod expression;
pub mod for_loop;
pub mod function_definition;
pub mod if_conditional;
pub mod switch;
pub mod variable_declaration;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::block::Block;

use self::assignment::Assignment;
use self::expression::Expression;
use self::for_loop::ForLoop;
use self::function_definition::FunctionDefinition;
use self::if_conditional::IfConditional;
use self::switch::Switch;
use self::variable_declaration::VariableDeclaration;

///
/// The block statement.
///
#[derive(Debug, PartialEq)]
pub enum Statement {
    Block(Block),
    Expression(Expression),
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    IfConditional(IfConditional),
    Switch(Switch),
    ForLoop(ForLoop),
    Break,
    Continue,
    Leave,
}

impl Statement {
    pub fn parse(lexer: &mut Lexer, mut initial: Option<Lexeme>) -> Self {
        let lexeme = match initial.take() {
            Some(lexeme) => lexeme,
            None => lexer.next(),
        };

        match lexeme {
            Lexeme::Keyword(Keyword::Function) => Statement::FunctionDefinition(FunctionDefinition::parse(lexer, None)),
            Lexeme::Keyword(Keyword::Let) => Statement::VariableDeclaration(VariableDeclaration::parse(lexer, None)),
            Lexeme::Keyword(Keyword::If) => Statement::IfConditional(IfConditional::parse(lexer, None)),
            Lexeme::Keyword(Keyword::Switch) => Statement::Switch(Switch::parse(lexer, None)),
            Lexeme::Keyword(Keyword::For) => Statement::ForLoop(ForLoop::parse(lexer, None)),
            Lexeme::Keyword(Keyword::Break) => Statement::Break,
            Lexeme::Keyword(Keyword::Continue) => Statement::Continue,
            Lexeme::Keyword(Keyword::Leave) => Statement::Leave,
            lexeme => panic!("expected one of `function`, `let`, `if`, `switch`, `for`, `break`, `continue`, `leave`, got {}", lexeme),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn leave_should_compile() {
        let input = r#"{
            function foo() -> x {
                x := 42
                if lt(x, 55) {
                    leave
                }
                x := 43
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 42);
    }

    #[test]
    fn continue_should_compile() {
        let input = r#"{
            function foo() -> x {
                x := 0
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    if mod(i, 2) {
                        continue
                    }
                    x := add(i, x)
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 20);
    }

    #[test]
    fn break_should_compile() {
        let input = r#"{
            function foo() -> x {
                x:= 0
                for { let i := 0 } lt(i, 10) { i := add(i, 1) } {
                    if gt(x, 18) {
                        break
                    }
                    x := add(i, x)
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 21);
    }
}
