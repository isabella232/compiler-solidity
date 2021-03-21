use regex::Regex;

use crate::r#type::Type;

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub name: String,
    pub yul_type: Option<Type>,
}

impl Identifier {
    pub fn parse_list<'a, I>(iter: &mut I, first: &str) -> Vec<Self>
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let mut result = vec![Self {
            name: first.to_string(),
            yul_type: None,
        }];
        let mut tok = iter.next();
        while tok.expect("unexpected eof in assignment") == "," {
            tok = iter.next();
            let value = tok.expect("unexpected eof after ','");
            if !Self::is_valid(value) {
                panic!("expected an identifier in identifier list, got {}", value);
            }
            result.push(Self {
                name: value.clone(),
                yul_type: None,
            });
            tok = iter.next();
        }
        if tok.expect("unexpected eof in assigment") != ":=" {
            panic!("expected ':=' in assignment");
        }
        result
    }

    // TODO: support declarations w/o initialization
    pub fn parse_typed_list<'a, I>(iter: &mut I, terminator: &'a str) -> Vec<Self>
    where
        I: crate::PeekableIterator<Item = &'a String>,
    {
        let mut result = Vec::new();
        let mut elem = iter.next();
        while elem != None {
            let name = elem.unwrap();
            if name == terminator {
                break;
            } else if !Self::is_valid(name) {
                panic!(
                    "unxepected identifier in typed parameter list, got '{}'",
                    name
                );
            }
            elem = iter.next();
            let value = elem.expect("unexpected end for typed parameter list");
            if value == terminator {
                result.push(Self {
                    name: name.clone(),
                    yul_type: None,
                });
                break;
            } else if value == "," {
                elem = iter.next();
                result.push(Self {
                    name: name.clone(),
                    yul_type: None,
                });
                continue;
            } else if value == ":" {
                elem = iter.next();
                let value = elem.expect("unexpected end for typed parameter list");
                if !Self::is_valid(value) {
                    panic!("bad typename for {} parameter, got {}", name, value);
                }
                // TODO: skip analyzing type for now
                result.push(Self {
                    name: name.clone(),
                    yul_type: Some(Type::Unknown(value.clone())),
                });
                elem = iter.next();
                let value = elem.expect("unexpected end for typed parameter list");
                if Self::is_valid(value) {
                    panic!("missing ',' before {}", value);
                }
                if value == "," {
                    elem = iter.next();
                }
            }
        }
        if elem == None {
            panic!("unexpected end for typed parameter list");
        }
        result
    }

    pub fn is_valid(value: &str) -> bool {
        let id_pattern = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9.]*$").expect("invalid regex");
        id_pattern.is_match(value)
    }
}
