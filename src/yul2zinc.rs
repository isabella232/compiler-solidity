use crate::*;

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
        None => unreachable!(),
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
    use crate::yul2zinc::translate;

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
