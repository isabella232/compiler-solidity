use crate::tree::block::statement::expression::Expression;
use crate::tree::block::Block;

#[derive(Debug, PartialEq)]
pub struct ForLoop {
    pub initializer: Block,
    pub condition: Expression,
    pub finalizer: Block,
    pub body: Block,
}

impl ForLoop {
    pub fn parse<'a, I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        if iter.next().expect("unexpected eof") != "{" {
            panic!("expected block in for loop initializer");
        }
        let pre = Block::parse(iter);
        let cond = Expression::parse(iter, None);
        if iter.next().expect("unexpected eof") != "{" {
            panic!("expected block in for loop body");
        }
        let post = Block::parse(iter);
        if iter.next().expect("unexpected eof") != "{" {
            panic!("expected block in for loop finalizer");
        }
        let body = Block::parse(iter);

        Self {
            initializer: pre,
            condition: cond,
            finalizer: post,
            body,
        }
    }
}
