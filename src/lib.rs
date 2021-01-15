use rand::Rng;
use regex::Regex;

// TODO: move to lexer.rs
/// Provide vector of tokens for a given source
pub fn get_lexemes(src: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut index = 0;
    let delimeters = Regex::new(r"(\s+)|[{}()]|(/\*)|(\\\*)").expect("invalid regex");
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
pub fn generate_actions<'a>(file: &'a str, options: &'a str) -> std::vec::Vec<Action<'a>> {
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
    let lexemes = get_lexemes(std::fs::read_to_string(input).unwrap().as_str());
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
#[derive(Debug)]
pub enum Type {
    Bool,
    Int(u32),
    UInt(u32),
    Unknown(String)
}

/// Represent a literal in yul without differentiating its type
#[derive(Debug)]
pub struct Literal {
    pub value: String
}

#[derive(Debug)]
pub struct Identifier {
    pub name: String,
    pub yul_type: Option<Type>
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub parameters: Vec<Identifier>,
    pub result: Vec<Identifier>, // TODO: investigate
    pub body: Block
}

#[derive(Debug)]
pub struct FunctionCall {
    pub name: Identifier,
    pub arguments: Vec<Expression>
}

#[derive(Debug)]
pub struct VariableDeclaration {
    pub names: Vec<Identifier>,
    pub initializer: Option<Expression>
}

#[derive(Debug)]
pub struct Assignment {
    pub names: Vec<Identifier>,
    pub initializer: Option<Expression>
}

#[derive(Debug)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    Literal(Literal)
}

#[derive(Debug)]
pub struct IfStatement {
    pub condition: Expression,
    pub body: Block
}

#[derive(Debug)]
pub struct SwitchCase {
    pub label: Literal,
    pub body: Block
}

#[derive(Debug)]
pub struct SwitchStatement {
    pub expression: Expression,
    pub cases: Vec<SwitchCase>,
    pub default: Block
}

#[derive(Debug)]
pub struct ForLoop {
    pub initializer: Block,
    pub condition: Expression,
    pub finalizer: Block,
    pub body: Block,
}

#[derive(Debug)]
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
    Leave
}

/// A comment (any text between /* and */) wich is not defined in yul grammar, but yul
/// implementation supports them
#[derive(Debug)]
pub struct Comment {
    pub text: String
}

/// A Yul fragent which is either a piece of Yul or a comment
#[derive(Debug)]
pub enum Fragment {
    Statement(Statement),
    Comment(Comment),
    Unparsed(String) // wip temporary accumulator
}

pub fn parse<'a, I>(mut iter: I) -> Vec<Fragment>
where
    I: Iterator<Item = &'a String>
{
    let mut result = Vec::new(); //<Fragment>;
    while (iter.nth(0) != None) {
        let elem = iter.nth(0).unwrap();
        if (elem == "/*") {
            result.push(Fragment::Comment(Comment{text: elem.clone()}));
        } else {
            result.push(Fragment::Unparsed(elem.clone()));
        }
        iter.next();
    }
    result
}
