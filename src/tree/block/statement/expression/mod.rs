pub mod function_call;

use crate::tree::identifier::Identifier;
use crate::tree::literal::Literal;

use self::function_call::FunctionCall;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    Literal(Literal),
}

impl Expression {
    pub fn parse<'a, I>(iter: &mut I, first_tok: Option<&'a str>) -> Self
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let tok =
            first_tok.unwrap_or_else(|| iter.next().expect("expected an expression, eof found"));
        if tok == "true" || tok == "false" || !Identifier::is_valid(tok) {
            return Expression::Literal(Literal {
                value: tok.to_string(),
            });
        } else if tok == "hex" {
            // TODO: Check the hex
            return Expression::Literal(Literal {
                value: iter.next().expect("missing value after 'hex'").clone(),
            });
        }
        let tok_ahead = iter.peek();
        if tok_ahead == None || *tok_ahead.unwrap() != "(" {
            return Expression::Identifier(Identifier {
                name: tok.to_string(),
                yul_type: None,
            });
        }
        // function call
        let mut arguments = Vec::new();
        let error_msg = format!("unexpected end of file in {} call", tok);
        iter.next().expect(&error_msg);
        let mut tok_ahead = *iter.peek().expect(&error_msg);
        while *tok_ahead != ")" {
            arguments.push(Expression::parse(iter, None));
            tok_ahead = *iter.peek().expect(&error_msg);
            if tok_ahead == "," {
                iter.next().expect(&error_msg);
                tok_ahead = *iter.peek().expect(&error_msg);
                continue;
            } else if tok_ahead == ")" {
                break;
            }
            panic!(
                "unexpected token {} in function {} argument list",
                *tok_ahead, tok
            );
        }
        iter.next();

        Self::FunctionCall(FunctionCall {
            name: tok.to_string(),
            arguments,
        })
    }
}
