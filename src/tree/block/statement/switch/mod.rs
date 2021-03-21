pub mod case;

use crate::tree::block::statement::expression::Expression;
use crate::tree::block::Block;
use crate::tree::literal::Literal;

use self::case::Case;

#[derive(Debug, PartialEq)]
pub struct Switch {
    pub expression: Expression,
    pub cases: Vec<Case>,
    pub default: Option<Block>,
}

impl Switch {
    pub fn parse<'a, I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let expression = Expression::parse(iter, None);
        let mut keyword = iter.next().expect("unexpected eof in switch statement");
        let mut cases = Vec::new();
        while keyword == "case" {
            // TODO: Check literal
            let literal = iter.next().expect("unexpected eof in switch statement");
            if iter.next().expect("unexpected eof in switch statement") != "{" {
                panic!("expected block in switch case");
            }
            cases.push(Case {
                label: Literal {
                    value: literal.clone(),
                },
                body: Block::parse(iter),
            });
            if iter.peek() != None
                && (*iter.peek().unwrap() == "case" || *iter.peek().unwrap() == "default")
            {
                keyword = iter.next().unwrap();
            } else {
                break;
            }
        }
        if keyword == "default" {
            if iter.next().expect("unexpected eof in switch statement") != "{" {
                panic!("expected block in switch case");
            }
            return Self {
                expression,
                cases,
                default: Some(Block::parse(iter)),
            };
        }
        if cases.is_empty() {
            panic!("expected either 'defaut' or at least one 'case' in switch statemet");
        }

        Self {
            expression,
            cases,
            default: None,
        }
    }
}
