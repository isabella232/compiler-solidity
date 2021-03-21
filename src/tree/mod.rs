pub mod block;
pub mod comment;
pub mod identifier;
pub mod literal;

use self::block::statement::Statement;
use self::block::Block;
use self::comment::Comment;

///
/// A Yul fragent which is either a piece of Yul or a comment
///
#[derive(Debug, PartialEq)]
pub enum Fragment {
    Statement(Statement),
    Unparsed(String), // wip temporary accumulator
}

///
/// The parser entry point.
///
pub fn parse<'a, I>(iter: I) -> Vec<Fragment>
where
    I: Iterator<Item = &'a String>,
{
    let mut result = Vec::new();
    let peekable = &mut iter.peekable();
    let mut elem = peekable.next();
    while elem != None {
        let value = elem.unwrap();
        if value == "/*" {
            Comment::parse(peekable);
        } else if value == "{" {
            result.push(Fragment::Statement(Statement::Block(Block::parse(
                peekable,
            ))));
        } else {
            result.push(Fragment::Unparsed(value.clone()));
        }
        elem = peekable.next();
    }
    result
}
