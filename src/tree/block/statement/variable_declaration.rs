use crate::tree::block::statement::expression::Expression;
use crate::tree::identifier::Identifier;

#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    pub names: Vec<Identifier>,
    pub initializer: Option<Expression>,
}

impl VariableDeclaration {
    pub fn parse<'a, I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let decl = Identifier::parse_typed_list(iter, ":=");
        let init = Expression::parse(iter, None);

        Self {
            names: decl,
            initializer: Some(init),
        }
    }
}
