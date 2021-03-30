//!
//! The switch statement.
//!

pub mod case;

use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::symbol::Symbol;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::llvm::Context;
use crate::parser::block::statement::expression::Expression;
use crate::parser::block::Block;

use self::case::Case;

///
/// The switch statement.
///
#[derive(Debug, PartialEq)]
pub struct Switch {
    /// The expression being matched.
    pub expression: Expression,
    /// The non-default cases.
    pub cases: Vec<Case>,
    /// The optional default case, if `cases` do not cover all possible values.
    pub default: Option<Block>,
}

///
/// The parsing state.
///
pub enum State {
    /// After match expression.
    CaseOrDefaultKeyword,
    /// After `case`.
    CaseBlock,
    /// After `default`.
    DefaultBlock,
}

impl Switch {
    pub fn parse(lexer: &mut Lexer, _initial: Option<Lexeme>) -> Self {
        let mut state = State::CaseOrDefaultKeyword;

        let expression = Expression::parse(lexer, None);
        let mut cases = Vec::new();
        let mut default = None;

        loop {
            match state {
                State::CaseOrDefaultKeyword => match lexer.peek() {
                    Lexeme::Keyword(Keyword::Case) => state = State::CaseBlock,
                    Lexeme::Keyword(Keyword::Default) => state = State::DefaultBlock,
                    _ => break,
                },
                State::CaseBlock => {
                    lexer.next();
                    cases.push(Case::parse(lexer, None));
                    state = State::CaseOrDefaultKeyword;
                }
                State::DefaultBlock => {
                    lexer.next();
                    match lexer.next() {
                        Lexeme::Symbol(Symbol::BracketCurlyLeft) => {}
                        lexeme => panic!("expected `{{`, got {}", lexeme),
                    }
                    default = Some(Block::parse(lexer, None));
                    break;
                }
            }
        }

        if cases.is_empty() && default.is_none() {
            panic!(
                "expected either the 'default' block or at least one 'case' in switch statement"
            );
        }

        Self {
            expression,
            cases,
            default,
        }
    }

    pub fn into_llvm<'ctx>(mut self, context: &mut Context<'ctx>) {
        let default = context
            .llvm
            .append_basic_block(context.function.unwrap(), "switch.default");
        let join = context
            .llvm
            .append_basic_block(context.function.unwrap(), "switch.join");
        let mut cases: Vec<(
            inkwell::values::IntValue<'ctx>,
            inkwell::basic_block::BasicBlock<'ctx>,
        )> = Vec::with_capacity(self.cases.len());
        for case in self.cases.iter() {
            let value = case.literal.to_owned().into_llvm(context).into_int_value();
            let basic_block = context
                .llvm
                .append_basic_block(context.function.unwrap(), "switch.case");
            cases.push((value, basic_block));
        }
        context.builder.build_switch(
            self.expression
                .into_llvm(context)
                .expect("Always exists")
                .into_int_value(),
            default,
            &cases,
        );
        for (_value, basic_block) in cases.into_iter() {
            context.builder.position_at_end(basic_block);
            self.cases.remove(0).block.into_llvm_local(context);
            context.builder.build_unconditional_branch(join);
        }
        context.builder.position_at_end(default);
        if let Some(block) = self.default.take() {
            block.into_llvm_local(context);
        }
        context.builder.build_unconditional_branch(join);
        context.builder.position_at_end(join);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ok_parse_single_case() {
        let input = r#"{
            switch expr
                case "a" {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_single_case_default() {
        let input = r#"{
            switch expr
                case "a" {}
                default {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_multiple_cases() {
        let input = r#"{
            switch expr
                case "a" {}
                case "b" {}
                case "c" {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_multiple_cases_default() {
        let input = r#"{
            switch expr
                case "a" {}
                case "b" {}
                case "c" {}
                default {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_parse_default() {
        let input = r#"{
            switch expr
                default {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    fn ok_compile_side_effects() {
        let input = r#"{
            function foo() -> x {
                x := 42
                switch x
                case 1 {
                    x := 22
                }
                default {
                    x := 17
                }
            }
        }"#;

        let result = crate::tests::compile(input, Some("foo"));
        assert_eq!(result, 17);
    }

    #[test]
    #[should_panic]
    fn error_expected_case() {
        let input = r#"{
            switch {}
        }"#;

        crate::tests::parse(input);
    }

    #[test]
    #[should_panic]
    fn error_case_after_default() {
        let input = r#"{
            switch expr
                default {}
                case 3 {}
        }"#;

        crate::tests::parse(input);
    }
}
