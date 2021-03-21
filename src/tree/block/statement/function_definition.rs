use crate::tree::block::Block;
use crate::tree::identifier::Identifier;

#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Identifier>,
    pub result: Vec<Identifier>, // TODO: investigate
    pub body: Block,
}

impl FunctionDefinition {
    pub fn parse<'a, I>(iter: &mut I) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let name = iter
            .next()
            .expect("function name must follow 'function' keyword");
        if !Identifier::is_valid(name) {
            panic!("function name must be an identifier, got {}", name);
        }
        let tok = iter
            .next()
            .unwrap_or_else(|| panic!("unexpected end of file in {} function definition", name));
        if tok != "(" {
            panic!("expected '(' in {} function definition, got {}", name, tok);
        }
        let parameters = Identifier::parse_typed_list(iter, ")");
        let tok = iter
            .next()
            .unwrap_or_else(|| panic!("unexpected end of file in {} function definition", name));
        let mut result = Vec::new();
        if tok == "->" {
            result = Identifier::parse_typed_list(iter, "{");
        } else if tok != "{" {
            panic!("unexpected token after parameter list of {} function", name);
        }
        let body = Block::parse(iter);

        Self {
            name: name.clone(),
            parameters,
            result,
            body,
        }
    }
}
