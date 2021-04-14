//!
//! The switch statement.
//!

pub mod case;

use crate::error::Error;
use crate::generator::llvm::Context as LLVMContext;
use crate::generator::ILLVMWritable;
use crate::lexer::lexeme::keyword::Keyword;
use crate::lexer::lexeme::Lexeme;
use crate::lexer::Lexer;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::block::Block;
use crate::parser::statement::expression::Expression;

use self::case::Case;

///
/// The switch statement.
///
#[derive(Debug, PartialEq, Clone)]
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
    ///
    /// The element parser, which acts like a constructor.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Lexeme>) -> Result<Self, Error> {
        let lexeme = crate::parser::take_or_next(initial, lexer)?;
        let mut state = State::CaseOrDefaultKeyword;

        let expression = Expression::parse(lexer, Some(lexeme.clone()))?;
        let mut cases = Vec::new();
        let mut default = None;

        loop {
            match state {
                State::CaseOrDefaultKeyword => match lexer.peek()? {
                    Lexeme::Keyword(Keyword::Case) => state = State::CaseBlock,
                    Lexeme::Keyword(Keyword::Default) => state = State::DefaultBlock,
                    _ => break,
                },
                State::CaseBlock => {
                    lexer.next()?;
                    cases.push(Case::parse(lexer, None)?);
                    state = State::CaseOrDefaultKeyword;
                }
                State::DefaultBlock => {
                    lexer.next()?;
                    default = Some(Block::parse(lexer, None)?);
                    break;
                }
            }
        }

        if cases.is_empty() && default.is_none() {
            return Err(ParserError::expected_one_of(vec!["case", "default"], lexeme, None).into());
        }

        Ok(Self {
            expression,
            cases,
            default,
        })
    }
}

impl ILLVMWritable for Switch {
    fn into_llvm<'ctx>(mut self, context: &mut LLVMContext<'ctx>) {
        let default = context
            .llvm
            .append_basic_block(context.function(), "switch.default");
        let join = context
            .llvm
            .append_basic_block(context.function(), "switch.join");
        let mut cases: Vec<(
            inkwell::values::IntValue<'ctx>,
            inkwell::basic_block::BasicBlock<'ctx>,
        )> = Vec::with_capacity(self.cases.len());
        for case in self.cases.iter() {
            let value = case.literal.to_owned().into_llvm(context).into_int_value();
            let basic_block = context
                .llvm
                .append_basic_block(context.function(), "switch.case");
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
    fn ok_single_case() {
        let input = r#"object "Test" { code {
            switch expr
                case "a" {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_single_case_default() {
        let input = r#"object "Test" { code {
            switch expr
                case "a" {}
                default {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_multiple_cases() {
        let input = r#"object "Test" { code {
            switch expr
                case "a" {}
                case "b" {}
                case "c" {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_multiple_cases_default() {
        let input = r#"object "Test" { code {
            switch expr
                case "a" {}
                case "b" {}
                case "c" {}
                default {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_default() {
        let input = r#"object "Test" { code {
            switch expr
                default {}
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn ok_side_effects() {
        let input = r#"object "Test" { code {
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
        }}"#;

        assert!(crate::parse(input).is_ok());
    }

    #[test]
    fn error_expected_case() {
        let input = r#"object "Test" { code {
            switch {}
        }}"#;

        assert!(crate::parse(input).is_err());
    }

    #[test]
    fn error_case_after_default() {
        let input = r#"object "Test" { code {
            switch expr
                default {}
                case 3 {}
        }}"#;

        assert!(crate::parse(input).is_err());
    }
}
