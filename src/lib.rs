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

//////////////////////////////////////////////////////////////////////////////////////

const INDENT_SYMBOL: char = ' ';
const INDENT_NUMBER: usize = 4;

fn indented(value: &str, indent: usize) -> String {
    let indentation = std::iter::repeat(INDENT_SYMBOL)
        .take(indent * INDENT_NUMBER)
        .collect::<String>();
    indentation + value
}

fn translate_block(block: &Block, indent: usize, return_values: &mut Vec<Identifier>) -> String {
    let mut result = String::from("");
    for stmt in &block.statements {
        result += match stmt {
            Statement::Block(b) => {
                indented("{", indent)
                    + translate_block(&b, indent + 1, return_values).as_str()
                    + "\n"
                    + indented("}", indent).as_str()
            }
            Statement::FunctionDefinition(fundef) => translate_function_definition(&fundef, indent),
            Statement::VariableDeclaration(vardecl) => {
                translate_variable_declaration(&vardecl, indent)
            }
            Statement::Assignment(assign) => translate_assignment(&assign, indent),
            Statement::Expression(expr) => translate_expression(&expr, indent, true),
            Statement::IfStatement(ifstmt) => {
                translate_if_statement(&ifstmt, indent, return_values)
            }
            Statement::SwitchStatement(switch) => {
                translate_switch_statement(&switch, indent, return_values)
            }
            Statement::Break => indented("break", indent),
            Statement::Continue => indented("continue", indent),
            Statement::Leave => return_for(return_values, indent, true),
            _ => unreachable!(),
        }
        .as_str();
    }
    result
}

fn translate_id_with_type(id: &Identifier) -> String {
    match &id.yul_type {
        None => format!("{}: u256", id.name),
        Some(Type::Unknown(s)) => format!("{}: {}", id.name, s),
        _ => unreachable!(),
    }
}

fn translate_id_type(id: &Identifier) -> String {
    match &id.yul_type {
        None => "u256".to_string(),
        Some(Type::Unknown(s)) => s.to_string(),
        _ => unreachable!(),
    }
}

fn default_value_for(yul_type: &Option<Type>) -> String {
    String::from(match yul_type {
        // assume u256 as the default type, maybe we'd need type inference later.
        None => "0",
        Some(Type::Bool) => "false",
        Some(Type::Int(_)) => "0",
        Some(Type::UInt(_)) => "0",
        Some(Type::Unknown(_)) => "0", // TODO: Support UDT?
    })
}

fn define_ret_values(ret_values: &[Identifier], indent: usize) -> String {
    let mut result = String::from("");
    for value in ret_values {
        result += indented("let mut ", indent).as_str();
        result += value.name.as_str();
        result += " = ";
        result += default_value_for(&value.yul_type).as_str();
        result += ";\n";
    }
    result
}

fn return_for(ret_values: &[Identifier], indent: usize, is_return: bool) -> String {
    if ret_values.is_empty() {
        return String::from("");
    }
    let mut result = indented("", indent);
    if is_return {
        result += "return ";
    }
    if ret_values.len() > 1 {
        result += "(";
    }
    for ret in ret_values {
        result += ret.name.as_str();
        result += ", ";
    }
    result.truncate(result.len() - 2);
    if ret_values.len() > 1 {
        result += ")";
    }
    if is_return {
        result += ";"
    }
    result + "\n"
}

fn translate_function_definition(fundef: &FunctionDefinition, indent: usize) -> String {
    let mut result = indented(format!("fn {}(", fundef.name).as_str(), indent);
    for par in &fundef.parameters {
        result += translate_id_with_type(par).as_str();
        result += ", ";
    }
    if !fundef.parameters.is_empty() {
        result.truncate(result.len() - 2);
    }
    result += ")";
    if !result.is_empty() {
        result += " -> ";
        for res in &fundef.result {
            result += translate_id_type(res).as_str();
            result += ", ";
        }
        result.truncate(result.len() - 2);
    }
    result += " {\n";
    result += define_ret_values(&fundef.result, indent + 1).as_str();
    result += translate_block(&fundef.body, indent + 1, &mut fundef.result.clone()).as_str();
    result += return_for(&fundef.result, indent + 1, false).as_str();
    result += indented("}\n", indent).as_str();
    result
}

fn translate_binary_operation(args: &[Expression], operation: &str) -> String {
    assert!(args.len() == 2, "The expression must be binary");
    let mut result = String::from("(");
    result += translate_expression(&args[0], 0, false).as_str();
    result += " ";
    result += operation;
    result += " ";
    result += translate_expression(&args[1], 0, false).as_str();
    result += ")";
    result
}

fn translate_builtin(call: &FunctionCall) -> Option<String> {
    match call.name.as_str() {
        // Arithmetical
        "add" => Some(translate_binary_operation(&call.arguments, "+")),
        "sub" => Some(translate_binary_operation(&call.arguments, "-")),
        "mul" => Some(translate_binary_operation(&call.arguments, "*")),
        // TODO: YUL semantic is more complex, '/' returns 0 if the divider is 0
        "div" => Some(translate_binary_operation(&call.arguments, "/")),
        "sdiv" => Some(translate_binary_operation(&call.arguments, "/")),
        // TODO: YUL semantic is more complex, '%' returns 0 if the divider is 0
        "mod" => Some(translate_binary_operation(&call.arguments, "%")),
        "smod" => Some(translate_binary_operation(&call.arguments, "%")),
        // Comparison
        "lt" => Some(translate_binary_operation(&call.arguments, "<")),
        "slt" => Some(translate_binary_operation(&call.arguments, "<")),
        "gt" => Some(translate_binary_operation(&call.arguments, ">")),
        "sgt" => Some(translate_binary_operation(&call.arguments, ">")),
        "eq" => Some(translate_binary_operation(&call.arguments, "==")),
        // Bitwise
        "and" => Some(translate_binary_operation(&call.arguments, "&")),
        "or" => Some(translate_binary_operation(&call.arguments, "|")),
        "xor" => Some(translate_binary_operation(&call.arguments, "^")),
        "shl" => Some(translate_binary_operation(&call.arguments, "<<")),
        "shr" => Some(translate_binary_operation(&call.arguments, ">>")),
        "sar" => Some(translate_binary_operation(&call.arguments, ">>")),
        _ => None,
    }
}

fn translate_user_function_call(call: &FunctionCall) -> String {
    let mut result = call.name.clone();
    result += "(";
    for arg in &call.arguments {
        result += translate_expression(&arg, 0, false).as_str();
        result += ", ";
    }
    if !call.arguments.is_empty() {
        result.truncate(result.len() - 2);
    }
    result += ")";
    result
}

fn translate_call(call: &FunctionCall) -> String {
    match translate_builtin(call) {
        Some(s) => s,
        _ => translate_user_function_call(call),
    }
}

fn translate_expression(expression: &Expression, indent: usize, is_standalone: bool) -> String {
    let expr_string = match expression {
        Expression::FunctionCall(call) => translate_call(&call),
        Expression::Identifier(id) => id.name.clone(),
        Expression::Literal(lit) => lit.value.clone(),
    };
    let mut result = indented(expr_string.as_str(), indent);
    if is_standalone {
        result += ";\n";
    }
    result
}

// TODO: implement
fn default_initialize(_variables: &[Identifier]) -> String {
    String::from("")
}

/*
fn default_initialize(variables: &[Identifier]) -> String {
    let mut types = variables.iter().map(|var| match &var.yul_type {
        None => Type::UInt(256),
        Some(t) => t,
    }).collect::<Vec<Type>>();
    let mut result = String::from("");
    for t in types {
        result += default_value_for(&t).as_str();
        result += ", ";
    }
    result.truncate(result.len() - 2);
    result
}
*/

fn translate_variable_declaration_or_assignment(
    variables: &[Identifier],
    expression: &Option<Expression>,
    indent: usize,
    is_declaration: bool,
) -> String {
    assert!(
        !variables.is_empty(),
        "let statement must contain at least on variable to declare"
    );
    let mut result = indented("", indent);
    if is_declaration {
        result += "let mut "
    }
    if variables.len() == 1 {
        result += variables[0].name.as_str();
    } else {
        result += "(";
        for variable in variables {
            result += variable.name.as_str();
            result += ", ";
        }
        result.truncate(result.len() - 2);
        result += ")";
    }
    result += " = ";
    result += match expression {
        Some(e) => translate_expression(e, 0, false),
        // In case we don't have an initializer, initialize with default values;
        None => default_initialize(variables),
    }
    .as_str();
    result += ";\n";
    result
}

// TODO: global scope variable declarations are not yet handled
fn translate_variable_declaration(declaration: &VariableDeclaration, indent: usize) -> String {
    translate_variable_declaration_or_assignment(
        &declaration.names,
        &declaration.initializer,
        indent,
        true, /*is_declaration */
    )
}

fn translate_assignment(assignment: &Assignment, indent: usize) -> String {
    translate_variable_declaration_or_assignment(
        &assignment.names,
        &Some(assignment.initializer.clone()),
        indent,
        false, /*is_declaration */
    )
}

fn translate_if_statement(
    ifstmt: &IfStatement,
    indent: usize,
    return_values: &mut Vec<Identifier>,
) -> String {
    let mut result = indented("if ", indent);
    result += translate_expression(&ifstmt.condition, 0, false).as_str();
    result += " {\n";
    result += translate_block(&ifstmt.body, indent + 1, return_values).as_str();
    result += indented("}\n", indent).as_str();
    result
}

fn translate_switch_statement(
    switch: &SwitchStatement,
    indent: usize,
    return_values: &mut Vec<Identifier>,
) -> String {
    let mut result = indented("match ", indent);
    result += translate_expression(&switch.expression, 0, false).as_str();
    result += " {\n";
    for case in &switch.cases {
        result += indented(case.label.value.as_str(), indent + 1).as_str();
        result += " => {\n";
        result += translate_block(&case.body, indent + 2, return_values).as_str();
        result += indented("},\n", indent + 1).as_str();
    }
    if let Some(default) = &switch.default {
        result += indented("_", indent + 1).as_str();
        result += " => {\n";
        result += translate_block(&default, indent + 2, return_values).as_str();
        result += indented("},\n", indent + 1).as_str();
    }
    result += indented("};\n", indent).as_str();
    result
}

pub fn translate(statement: &Statement) -> String {
    let mut result = "".to_string();
    match statement {
        Statement::Block(block) => {
            result += translate_block(block, 0, &mut Vec::new()).as_str();
        }
        _ => unreachable!(),
    }
    result
}

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
        translate(program)
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

    #[test]
    fn nested_blocks_should_compile() {
        assert_eq!(compile("{{}}"), "{\n}");
    }

    #[test]
    fn function_definition_should_compile() {
        assert_eq!(
            compile("{function foo(a : A) -> x: T, z: Y {}}"),
            "fn foo(a: A) -> T, Y {\n    let mut x = 0;\n    let mut z = 0;\n    (x, z)\n}\n"
        );
        assert_eq!(
            compile("{function foo(a: A, b) -> x: T, z: Y {}}"),
            "fn foo(a: A, b: u256) -> T, Y {\n    let mut x = 0;\n    let mut z = 0;\n    (x, z)\n}\n"
        );
    }

    #[test]
    fn variable_declaration_should_compile() {
        assert_eq!(
            compile("{let x := an_identifier}"),
            "let mut x = an_identifier;\n"
        );
        assert_eq!(compile("{let x := 0}"), "let mut x = 0;\n");
        assert_eq!(compile("{let x := \"abc\"}"), "let mut x = \"abc\";\n");
        assert_eq!(
            compile("{let x, y, z := foo()}"),
            "let mut (x, y, z) = foo();\n"
        );
    }

    #[test]
    fn assignment_should_compile() {
        assert_eq!(compile("{x := 42}"), "x = 42;\n");
        assert_eq!(compile("{x,y := bar()}"), "(x, y) = bar();\n");
    }

    #[test]
    fn expression_should_compile() {
        assert_eq!(compile("{42}"), "42;\n");
        assert_eq!(compile("{\"abc\"}"), "\"abc\";\n");
        assert_eq!(compile("{foo(bar())}"), "foo(bar());\n");
    }

    #[test]
    fn if_statement_should_compile() {
        assert_eq!(compile("{if expr {}}"), "if expr {\n}\n");
    }

    #[test]
    fn switch_statement_should_compile() {
        assert_eq!(
            compile("{switch expr case \"a\" {} case \"b\" {}}"),
            "match expr {\n    \"a\" => {\n    },\n    \"b\" => {\n    },\n};\n"
        );
        assert_eq!(
            compile("{switch expr case \"a\" {} default {}}"),
            "match expr {\n    \"a\" => {\n    },\n    _ => {\n    },\n};\n"
        );
        assert_eq!(
            compile("{switch expr default {}}"),
            "match expr {\n    _ => {\n    },\n};\n"
        );
    }

    #[test]
    fn builtin_call_should_be_recognized() {
        // Arithmetical
        assert_eq!(compile("{add(a, b)}"), "(a + b);\n");
        assert_eq!(compile("{sub(a, b)}"), "(a - b);\n");
        assert_eq!(compile("{mul(a, b)}"), "(a * b);\n");
        assert_eq!(compile("{div(a, b)}"), "(a / b);\n");

        // Comparison
        assert_eq!(compile("{gt(a, b)}"), "(a > b);\n");
        assert_eq!(compile("{sgt(a, b)}"), "(a > b);\n");
        assert_eq!(compile("{lt(a, b)}"), "(a < b);\n");
        assert_eq!(compile("{slt(a, b)}"), "(a < b);\n");
        assert_eq!(compile("{eq(a, b)}"), "(a == b);\n");

        // Bitwise
        assert_eq!(compile("{and(a, b)}"), "(a & b);\n");
        assert_eq!(compile("{or(a, b)}"), "(a | b);\n");
    }

    #[test]
    fn return_should_be_compiled_correctly() {
        assert_eq!(
            compile("{function zero() -> ret {ret :=0 }}"),
            "fn zero() -> u256 {\n    let mut ret = 0;\n    ret = 0;\n    ret\n}\n"
        );
    }

    #[test]
    fn leave_should_be_compiled_correctly() {
        assert_eq!(
            compile("{function foo(y) -> x {if y {x:=y leave} x:=1}}"),
            "fn foo(y: u256) -> u256 {\n    let mut x = 0;\n    if y {\n        x = y;\n        return x;\n    }\n    x = 1;\n    x\n}\n"
        );
    }
}
