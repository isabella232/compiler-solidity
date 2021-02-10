use rand::Rng;
use regex::Regex;

fn remove_comments(src: &mut String) {
    let mut comment = src.find("//");
    while comment != None {
        let pos = comment.unwrap();
        let eol = src[pos..].find('\n').unwrap_or(src.len() - pos - 1);
        src.replace_range(pos..eol, "");
        comment = src.find("//");
    }
}

// TODO: move to lexer.rs
/// Provide vector of tokens for a given source
pub fn get_lexemes(src: &mut String) -> Vec<String> {
    let mut result = Vec::new();
    let mut index = 0;
    // TODO: Check if we can rely on regexp to guarantee that in case of ':=' it will always be
    // ':=' rather than [':', '='].
    let delimeters = Regex::new(r"(:=)|(\s+)|[{}(),:]|(/\*)|(\*/)").expect("invalid regex");
    let mut matched = delimeters.find(&src[index..]);
    while matched != None {
        let the_match = matched.unwrap();
        if the_match.start() != 0 {
            result.push(String::from(&src[index..index + the_match.start()]));
        }
        result.push(String::from(the_match.as_str()));
        index += the_match.end();
        matched = delimeters.find(&src[index..]);
    }
    if index < src.len() {
        result.push(String::from(&src[index..]));
    }
    result
        .into_iter()
        .filter(|x| !Regex::new(r"^\s+$").unwrap().is_match(x))
        .collect()
}

/// File type for input and output files
#[derive(Debug)]
pub enum FileType {
    Solidity,
    Yul,
    Zinc,
    Unknown,
}

/// Provide FileType for a given file based on its extension
pub fn file_type(file: &str) -> FileType {
    let extension = std::path::Path::new(file)
        .extension()
        .and_then(std::ffi::OsStr::to_str);
    match extension {
        None => FileType::Unknown,
        Some("sol") => FileType::Solidity,
        Some("yul") => FileType::Yul,
        Some("zinc") => FileType::Zinc,
        Some(_) => FileType::Unknown,
    }
}

/// Abstract compilation step
#[derive(Debug)]
pub enum Action<'a> {
    SolidityCompiler(&'a str, String),
    CodeGenerator(String),
}

/// Generate temporary output directory for a given solidity input
/// Precondition: input must exist
fn tmp_yul(input: &str) -> String {
    let mut path = std::env::temp_dir();
    let suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let file_stem = std::path::Path::new(input).file_stem().unwrap();
    let tmp_dir_name = String::from(file_stem.to_str().unwrap()) + "-" + suffix.as_str();
    path.push(tmp_dir_name);
    String::from(path.to_str().unwrap())
}

/// Produce sequence of actions required to compile file with specified options
pub fn generate_actions<'a>(file: &'a str, options: &'a str) -> Vec<Action<'a>> {
    match file_type(file) {
        FileType::Yul => vec![Action::CodeGenerator(String::from(file))],
        FileType::Solidity => {
            let tmp_file = tmp_yul(file);
            let options_string = String::from(options) + " --ir -o " + tmp_file.as_str();
            let options_string = String::from(options_string.trim());
            vec![
                Action::SolidityCompiler(file, options_string),
                Action::CodeGenerator(tmp_file),
            ]
        }
        _ => vec![],
    }
}

/// Wrap Solidity compiler invocation
pub fn invoke_solidity(input: &str, options: &str) {
    std::process::Command::new("solc")
        .arg(input)
        .args(options.split(' ').collect::<Vec<&str>>())
        .spawn()
        .expect("Unable to run solidity. Ensure it's in PATH");
}

/// Wrap Zinc generator invocation
pub fn invoke_codegen(input: &str) {
    let lexemes = get_lexemes(&mut std::fs::read_to_string(input).unwrap());
    let fragments = parse(lexemes.iter());
    println!("{:?}", fragments);
}

/// Execute an action by calling corresponding handler
pub fn execute_action<'a>(action: &Action<'a>) {
    match action {
        Action::SolidityCompiler(input, options) => invoke_solidity(input, options.as_str()),
        Action::CodeGenerator(input) => invoke_codegen(input.as_str()),
    }
}

// TODO: move to grammar.rs
// Block = '{' Statement* '}'
// Statement =
//     Block |
//     FunctionDefinition |
//     VariableDeclaration |
//     Assignment |
//     If |
//     Expression |
//     Switch |
//     ForLoop |
//     BreakContinue |
//     Leave
// FunctionDefinition =
//     'function' Identifier '(' TypedIdentifierList? ')'
//     ( '->' TypedIdentifierList )? Block
// VariableDeclaration =
//     'let' TypedIdentifierList ( ':=' Expression )?
// Assignment =
//     IdentifierList ':=' Expression
// Expression =
//     FunctionCall | Identifier | Literal
// If =
//     'if' Expression Block
// Switch =
//     'switch' Expression ( Case+ Default? | Default )
// Case =
//     'case' Literal Block
// Default =
//     'default' Block
// ForLoop =
//     'for' Block Expression Block Block
// BreakContinue =
//     'break' | 'continue'
// Leave = 'leave'
// FunctionCall =
//     Identifier '(' ( Expression ( ',' Expression )* )? ')'
// Identifier = [a-zA-Z_$] [a-zA-Z_$0-9.]*
// IdentifierList = Identifier ( ',' Identifier)*
// TypeName = Identifier
// TypedIdentifierList = Identifier ( ':' TypeName )? ( ',' Identifier ( ':' TypeName )? )*
// Literal =
//     (NumberLiteral | StringLiteral | HexLiteral | TrueLiteral | FalseLiteral) ( ':' TypeName )?
// NumberLiteral = HexNumber | DecimalNumber
// HexLiteral = 'hex' ('"' ([0-9a-fA-F]{2})* '"' | '\'' ([0-9a-fA-F]{2})* '\'')
// StringLiteral = '"' ([^"\r\n\\] | '\\' .)* '"'
// TrueLiteral = 'true'
// FalseLiteral = 'false'
// HexNumber = '0x' [0-9a-fA-F]+
// DecimalNumber = [0-9]+

/// Datatype for a lexeme for further analysis and translation
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Bool,
    Int(u32),
    UInt(u32),
    Unknown(String),
}

/// Represent a literal in yul without differentiating its type
#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub value: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub name: String,
    pub yul_type: Option<Type>,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Identifier>,
    pub result: Vec<Identifier>, // TODO: investigate
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    pub names: Vec<Identifier>,
    pub initializer: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub names: Vec<Identifier>,
    pub initializer: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    Literal(Literal),
}

#[derive(Debug, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct SwitchCase {
    pub label: Literal,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct SwitchStatement {
    pub expression: Expression,
    pub cases: Vec<SwitchCase>,
    pub default: Option<Block>,
}

#[derive(Debug, PartialEq)]
pub struct ForLoop {
    pub initializer: Block,
    pub condition: Expression,
    pub finalizer: Block,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Block(Block),
    FunctionDefinition(FunctionDefinition),
    VariableDeclaration(VariableDeclaration),
    Assignment(Assignment),
    IfStatement(IfStatement),
    Expression(Expression),
    SwitchStatement(SwitchStatement),
    ForLoop(ForLoop),
    Break,
    Continue,
    Leave,
}

/// A Yul fragent which is either a piece of Yul or a comment
#[derive(Debug, PartialEq)]
pub enum Fragment {
    Statement(Statement),
    Unparsed(String), // wip temporary accumulator
}

fn is_identifier(value: &str) -> bool {
    let id_pattern = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9.]*$").expect("invalid regex");
    id_pattern.is_match(value)
}

pub trait PeekableIterator: std::iter::Iterator {
    fn peek(&mut self) -> Option<&Self::Item>;
}

impl<I: std::iter::Iterator> PeekableIterator for std::iter::Peekable<I> {
    fn peek(&mut self) -> Option<&Self::Item> {
        std::iter::Peekable::peek(self)
    }
}

/// Skip all lexemes until '*/' is found
fn parse_comment<'a, I>(iter: &mut I)
where
    I: PeekableIterator<Item = &'a String>,
{
    let mut elem = iter.next();
    while elem != None {
        if elem.unwrap() == "*/" {
            break;
        }
        elem = iter.next();
    }
    if elem == None {
        panic!("Can't find matching '*/', Yul input is ill-formed");
    }
}

/// Parse a block, panic if a block is ill-formed
fn parse_block<'a, I>(iter: &mut I) -> Block
where
    I: PeekableIterator<Item = &'a String>,
{
    let mut content = Vec::new();
    let mut elem = iter.next();
    while elem != None {
        let value = elem.unwrap();
        if value == "}" {
            break;
        } else if value == "{" {
            content.push(Statement::Block(parse_block(iter)));
        } else if value == "function" {
            content.push(Statement::FunctionDefinition(parse_function_definition(
                iter,
            )));
        } else if value == "let" {
            content.push(Statement::VariableDeclaration(parse_variable_declaration(
                iter,
            )));
        } else if value == "if" {
            content.push(Statement::IfStatement(parse_if(iter)));
        } else if value == "switch" {
            content.push(Statement::SwitchStatement(parse_switch(iter)));
        } else if value == "for" {
            content.push(Statement::ForLoop(parse_for_loop(iter)));
        } else if value == "break" {
            content.push(Statement::Break);
        } else if value == "continue" {
            content.push(Statement::Continue);
        } else if value == "leave" {
            content.push(Statement::Leave);
        } else if !is_identifier(value) || value == "true" || value == "false" {
            content.push(Statement::Expression(parse_expression(iter, Some(value))));
        } else {
            let lookahead = iter.peek();
            #[allow(clippy::if_same_then_else)]
            if lookahead == None {
                content.push(Statement::Expression(parse_expression(iter, Some(value))));
            } else if *lookahead.unwrap() != ":=" && *lookahead.unwrap() != "," {
                content.push(Statement::Expression(parse_expression(iter, Some(value))));
            } else if *lookahead.unwrap() == ":=" {
                iter.next();
                content.push(Statement::Assignment(Assignment {
                    names: vec![Identifier {
                        name: value.clone(),
                        yul_type: None,
                    }],
                    initializer: parse_expression(iter, None),
                }));
            } else {
                let identifiers = parse_identifier_list(iter, value);
                content.push(Statement::Assignment(Assignment {
                    names: identifiers,
                    initializer: parse_expression(iter, None),
                }));
            }
        }
        elem = iter.next();
    }
    if elem == None {
        panic!("Can't find matching '}', Yul input is ill-formed");
    }
    Block {
        statements: content,
    }
}

fn parse_identifier_list<'a, I>(iter: &mut I, first: &str) -> Vec<Identifier>
where
    I: PeekableIterator<Item = &'a String>,
{
    let mut result = vec![Identifier {
        name: first.to_string(),
        yul_type: None,
    }];
    let mut tok = iter.next();
    while tok.expect("unexpected eof in assignment") == "," {
        tok = iter.next();
        let value = tok.expect("unexpected eof after ','");
        if !is_identifier(value) {
            panic!("expected an identifier in identifier list, got {}", value);
        }
        result.push(Identifier {
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
fn parse_typed_identifier_list<'a, I>(iter: &mut I, terminator: &'a str) -> Vec<Identifier>
where
    I: PeekableIterator<Item = &'a String>,
{
    let mut result = Vec::new();
    let mut elem = iter.next();
    while elem != None {
        let name = elem.unwrap();
        if name == terminator {
            break;
        } else if !is_identifier(name) {
            panic!(
                "unxepected identifier in typed parameter list, got '{}'",
                name
            );
        }
        elem = iter.next();
        let value = elem.expect("unexpected end for typed paramater list");
        if value == terminator {
            result.push(Identifier {
                name: name.clone(),
                yul_type: None,
            });
            break;
        } else if value == "," {
            elem = iter.next();
            result.push(Identifier {
                name: name.clone(),
                yul_type: None,
            });
            continue;
        } else if value == ":" {
            elem = iter.next();
            let value = elem.expect("unexpected end for typed paramater list");
            if !is_identifier(value) {
                panic!("bad typename for {} parameter, got {}", name, value);
            }
            // TODO: skip analyzing type for now
            result.push(Identifier {
                name: name.clone(),
                yul_type: Some(Type::Unknown(value.clone())),
            });
            elem = iter.next();
            let value = elem.expect("unexpected end for typed paramater list");
            if is_identifier(value) {
                panic!("missing ',' before {}", value);
            }
            if value == "," {
                elem = iter.next();
            }
        }
    }
    if elem == None {
        panic!("unexpected end for typed paramater list");
    }
    result
}

fn parse_function_definition<'a, I>(iter: &mut I) -> FunctionDefinition
where
    I: PeekableIterator<Item = &'a String>,
{
    let name = iter
        .next()
        .expect("function name must follow 'function' keyword");
    if !is_identifier(name) {
        panic!("function name must be an identifier, got {}", name);
    }
    let tok = iter
        .next()
        .unwrap_or_else(|| panic!("unexpected end of file in {} function definition", name));
    if tok != "(" {
        panic!("expected '(' in {} function definition, got {}", name, tok);
    }
    let parameters = parse_typed_identifier_list(iter, ")");
    let tok = iter
        .next()
        .unwrap_or_else(|| panic!("unexpected end of file in {} function definition", name));
    let mut result = Vec::new();
    if tok == "->" {
        result = parse_typed_identifier_list(iter, "{");
    } else if tok != "{" {
        panic!("unexpected token after paramater list of {} function", name);
    }
    let body = parse_block(iter);
    FunctionDefinition {
        name: name.clone(),
        parameters,
        result,
        body,
    }
}

fn parse_variable_declaration<'a, I>(iter: &mut I) -> VariableDeclaration
where
    I: PeekableIterator<Item = &'a String>,
{
    let decl = parse_typed_identifier_list(iter, ":=");
    let init = parse_expression(iter, None);
    VariableDeclaration {
        names: decl,
        initializer: Some(init),
    }
}

fn parse_expression<'a, I>(iter: &mut I, first_tok: Option<&'a str>) -> Expression
where
    I: PeekableIterator<Item = &'a String>,
{
    let tok = first_tok.unwrap_or_else(|| iter.next().expect("expected an expression, eof found"));
    if tok == "true" || tok == "false" || !is_identifier(tok) {
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
        arguments.push(parse_expression(iter, None));
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
    Expression::FunctionCall(FunctionCall {
        name: tok.to_string(),
        arguments,
    })
}

pub fn parse_if<'a, I>(iter: &mut I) -> IfStatement
where
    I: PeekableIterator<Item = &'a String>,
{
    let expression = parse_expression(iter, None);
    let block_start = iter.next().expect("unexpected eof in if statement");
    if block_start != "{" {
        panic!(
            "unexpected token {} followed after the condition in if statement",
            block_start
        );
    }
    let block = parse_block(iter);
    IfStatement {
        condition: expression,
        body: block,
    }
}

pub fn parse_switch<'a, I>(iter: &mut I) -> SwitchStatement
where
    I: PeekableIterator<Item = &'a String>,
{
    let expression = parse_expression(iter, None);
    let mut keyword = iter.next().expect("unexpected eof in switch statement");
    let mut cases = Vec::new();
    while keyword == "case" {
        // TODO: Check literal
        let literal = iter.next().expect("unexpected eof in switch statement");
        if iter.next().expect("unexpected eof in switch statement") != "{" {
            panic!("expected block in switch case");
        }
        cases.push(SwitchCase {
            label: Literal {
                value: literal.clone(),
            },
            body: parse_block(iter),
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
        return SwitchStatement {
            expression,
            cases,
            default: Some(parse_block(iter)),
        };
    }
    if cases.is_empty() {
        panic!("expected either 'defaut' or at least one 'case' in switch statemet");
    }
    SwitchStatement {
        expression,
        cases,
        default: None,
    }
}

pub fn parse_for_loop<'a, I>(iter: &mut I) -> ForLoop
where
    I: PeekableIterator<Item = &'a String>,
{
    if iter.next().expect("unexpected eof") != "{" {
        panic!("expected block in for loop initializer");
    }
    let pre = parse_block(iter);
    let cond = parse_expression(iter, None);
    if iter.next().expect("unexpected eof") != "{" {
        panic!("expected block in for loop body");
    }
    let post = parse_block(iter);
    if iter.next().expect("unexpected eof") != "{" {
        panic!("expected block in for loop finalizer");
    }
    let body = parse_block(iter);
    ForLoop {
        initializer: pre,
        condition: cond,
        finalizer: post,
        body,
    }
}

/// Entry point for parsing
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
            parse_comment(peekable);
        } else if value == "{" {
            result.push(Fragment::Statement(Statement::Block(parse_block(peekable))));
        } else {
            result.push(Fragment::Unparsed(value.clone()));
        }
        elem = peekable.next();
    }
    result
}

//------------------------------------ YUL to Zinc -----------------------------------
pub mod yul2zinc;

#[cfg(test)]
mod tests {
    use crate::*;

    static YUL: &'static str = "/*123 comment ***/{}";

    fn get_lexemes_str(input: &str) -> Vec<String> {
        get_lexemes(&mut String::from(input))
    }

    fn lexparse(input: &str) -> Vec<Fragment> {
        parse(get_lexemes_str(input).iter())
    }

    fn compile(input: &str) -> String {
        let parsed = lexparse(input);
        if parsed.len() != 1 {
            panic!("Unparsed parts exist");
        }
        let program = match &parsed[0] {
            Fragment::Statement(s) => s,
            _ => unreachable!(),
        };
        "".to_string()
        //translate(program)
    }

    #[test]
    fn whitespaces_should_be_ignored() {
        assert_eq!(get_lexemes_str("   a    b c\td"), ["a", "b", "c", "d"]);
    }

    fn single_line_comments_should_be_ignored() {
        assert_eq!(
            get_lexemes_str("   a////comment\nb c\td//comment"),
            ["a", "b", "c", "d"]
        );
    }

    #[test]
    fn multi_line_comments_should_be_tokenized() {
        assert_eq!(
            get_lexemes_str(YUL),
            ["/*", "123", "comment", "**", "*/", "{", "}"]
        );
    }

    #[test]
    fn comment_should_not_be_parsed() {
        assert_eq!(
            lexparse(YUL),
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![]
            }))]
        );
    }

    #[test]
    #[should_panic]
    fn ill_formed_comment_should_panic() {
        lexparse("/* xxx yyy");
    }

    #[test]
    fn nested_blocks_should_be_parsed() {
        assert_eq!(
            lexparse("{{}}"),
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![Statement::Block(Block { statements: vec![] })]
            }))]
        );
    }

    #[test]
    #[should_panic]
    fn ill_formed_block_should_panic() {
        lexparse("{{}{}{{}");
    }

    #[test]
    #[should_panic]
    fn badly_named_function_should_panic() {
        lexparse("{ function 42(){}}");
    }

    #[test]
    #[should_panic]
    fn function_with_bad_parameter_list_should_panic() {
        lexparse("{ function 42){}}");
    }

    #[test]
    fn well_formed_void_function_should_be_parsed() {
        lexparse("{function foo(a : A, b){}}");
    }

    #[test]
    fn well_formed_non_void_function_should_be_parsed() {
        lexparse("{function foo(a : A, b) -> x: T, z: Y {}}");
    }

    #[test]
    fn vardecl_true_should_be_parsed() {
        lexparse("{let x := true}");
    }

    #[test]
    fn vardecl_false_should_be_parsed() {
        lexparse("{let x := false}");
    }

    #[test]
    fn vardecl_string_should_be_parsed() {
        lexparse("{let x := \"abc\"}");
    }

    #[test]
    fn vardecl_dec_number_should_be_parsed() {
        lexparse("{let x := 42}");
    }

    #[test]
    fn vardecl_hex_number_should_be_parsed() {
        lexparse("{let x := 0x42}");
    }

    #[test]
    fn vardecl_identifier_should_be_parsed() {
        lexparse("{let x := y}");
    }

    #[test]
    fn vardecl_function_call_should_be_parsed() {
        lexparse("{let x := foo()}");
        lexparse("{let x := foo(x, y)}");
        lexparse("{let x := foo(bar(x, baz()))}");
    }

    #[test]
    fn if_statement_should_be_parsed() {
        lexparse("{if expr {}}");
    }

    #[test]
    fn switch_statement_should_be_parsed() {
        lexparse("{switch expr case \"a\" {} case \"b\" {}}");
        lexparse("{switch expr case \"a\" {} default {}}");
        lexparse("{switch expr default {}}");
    }

    #[test]
    #[should_panic]
    fn ill_formed_switch_statement_should_panic() {
        lexparse("{switch {}}");
        lexparse("{switch expr default {} case 3 {}}");
    }

    #[test]
    fn for_loop_should_be_parsed() {
        lexparse("{for {} expr {}{}}");
    }

    #[test]
    fn keywords_should_not_be_parsed_as_identifiers() {
        let kw_break = lexparse("{break}");
        let kw_continue = lexparse("{continue}");
        let kw_leave = lexparse("{leave}");
        assert_eq!(
            kw_break,
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![Statement::Break]
            }))]
        );
        assert_eq!(
            kw_continue,
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![Statement::Continue]
            }))]
        );
        assert_eq!(
            kw_leave,
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![Statement::Leave]
            }))]
        );
    }

    #[test]
    fn true_false_should_be_parsed_as_literals() {
        let kw_true = lexparse("{true}");
        let kw_false = lexparse("{false}");
        assert_eq!(
            kw_true,
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![Statement::Expression(Expression::Literal(Literal {
                    value: "true".to_string()
                }))]
            }))]
        );
        assert_eq!(
            kw_false,
            [Fragment::Statement(Statement::Block(Block {
                statements: vec![Statement::Expression(Expression::Literal(Literal {
                    value: "false".to_string()
                }))]
            }))]
        );
    }
    #[test]
    fn expressions_should_be_parsed() {
        lexparse("{id 3 foo(x, y)}");
    }

    #[test]
    fn assignments_should_be_parsed() {
        lexparse("{x := foo(x)}");
        lexparse("{x,y := foo(x)}");
    }

    #[test]
    #[should_panic]
    fn ill_formed_assignment_should_panic() {
        lexparse("{x := }");
    }

    #[test]
    #[should_panic]
    fn id_list_wo_assignment_should_panic() {
        lexparse("{x,y}");
    }
}
