//!
//! The Ethereal IR block element.
//!

pub mod kind;
pub mod stack_data;

use self::kind::Kind;
use self::stack_data::StackData;

///
/// The Ethereal IR block element.
///
#[derive(Debug, Clone)]
pub struct Element {
    /// The element kind.
    pub kind: Kind,
    /// The stack data.
    pub stack_data: Option<StackData>,
}

impl Element {
    ///
    /// Sets the static stack data.
    ///
    pub fn set_stack_data(&mut self, stack_data: StackData) {
        self.stack_data = Some(stack_data);
    }
}

impl From<Kind> for Element {
    fn from(kind: Kind) -> Self {
        Self {
            kind,
            stack_data: None,
        }
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Element
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.kind.into_llvm(context)?;

        Ok(())
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:80}        {}",
            self.kind.to_string(),
            match self.stack_data {
                Some(ref stack_data) => stack_data.to_string(),
                None => "".to_owned(),
            },
        )
    }
}
