//!
//! The Ethereal IR entry function link.
//!

///
/// The Ethereal IR entry function link.
///
#[derive(Debug, Clone)]
pub struct EntryLink {
    /// The code part type.
    pub code_type: compiler_llvm_context::CodeType,
}

impl EntryLink {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(code_type: compiler_llvm_context::CodeType) -> Self {
        Self { code_type }
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for EntryLink
where
    D: compiler_llvm_context::Dependency,
{
    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let target_name = format!("function_{}", self.code_type);
        let target = context
            .functions
            .get(target_name.as_str())
            .expect("Always exists")
            .value;
        context.build_call(target, &[], format!("call_link_{}", target_name).as_str());

        Ok(())
    }
}
