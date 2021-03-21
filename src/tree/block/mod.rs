pub mod statement;

use crate::tree::identifier::Identifier;

use self::statement::assignment::Assignment;
use self::statement::expression::Expression;
use self::statement::for_loop::ForLoop;
use self::statement::function_definition::FunctionDefinition;
use self::statement::if_conditional::IfConditional;
use self::statement::switch::Switch;
use self::statement::variable_declaration::VariableDeclaration;
use self::statement::Statement;

#[derive(Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl Block {
    /// Parse a block, panic if a block is ill-formed
    pub fn parse<'a, I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let mut content = Vec::new();
        let mut elem = iter.next();
        while elem != None {
            let value = elem.unwrap();
            if value == "}" {
                break;
            } else if value == "{" {
                content.push(Statement::Block(Block::parse(iter)));
            } else if value == "function" {
                content.push(Statement::FunctionDefinition(FunctionDefinition::parse(
                    iter,
                )));
            } else if value == "let" {
                content.push(Statement::VariableDeclaration(VariableDeclaration::parse(
                    iter,
                )));
            } else if value == "if" {
                content.push(Statement::IfConditional(IfConditional::parse(iter)));
            } else if value == "switch" {
                content.push(Statement::Switch(Switch::parse(iter)));
            } else if value == "for" {
                content.push(Statement::ForLoop(ForLoop::parse(iter)));
            } else if value == "break" {
                content.push(Statement::Break);
            } else if value == "continue" {
                content.push(Statement::Continue);
            } else if value == "leave" {
                content.push(Statement::Leave);
            } else if !Identifier::is_valid(value) || value == "true" || value == "false" {
                content.push(Statement::Expression(Expression::parse(iter, Some(value))));
            } else {
                let lookahead = iter.peek();
                #[allow(clippy::if_same_then_else)]
                if lookahead == None {
                    content.push(Statement::Expression(Expression::parse(iter, Some(value))));
                } else if *lookahead.unwrap() != ":=" && *lookahead.unwrap() != "," {
                    content.push(Statement::Expression(Expression::parse(iter, Some(value))));
                } else if *lookahead.unwrap() == ":=" {
                    iter.next();
                    content.push(Statement::Assignment(Assignment {
                        names: vec![Identifier {
                            name: value.clone(),
                            yul_type: None,
                        }],
                        initializer: Expression::parse(iter, None),
                    }));
                } else {
                    let identifiers = Identifier::parse_list(iter, value);
                    content.push(Statement::Assignment(Assignment {
                        names: identifiers,
                        initializer: Expression::parse(iter, None),
                    }));
                }
            }
            elem = iter.next();
        }
        if elem == None {
            panic!("{}", "Can't find matching '}', Yul input is ill-formed");
        }

        Self {
            statements: content,
        }
    }
}
