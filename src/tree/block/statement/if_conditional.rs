use crate::tree::block::statement::expression::Expression;
use crate::tree::block::Block;

#[derive(Debug, PartialEq)]
pub struct IfConditional {
    pub condition: Expression,
    pub body: Block,
}

impl IfConditional {
    pub fn parse<'a, I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let expression = Expression::parse(iter, None);
        let block_start = iter.next().expect("unexpected eof in if statement");
        if block_start != "{" {
            panic!(
                "unexpected token {} followed after the condition in if statement",
                block_start
            );
        }
        let block = Block::parse(iter);

        Self {
            condition: expression,
            body: block,
        }
    }
}
