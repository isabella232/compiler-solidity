//!
//! The contract data representation.
//!

pub mod source;

use std::collections::HashMap;

use compiler_llvm_context::WriteLLVM;

use crate::dump_flag::DumpFlag;
use crate::project::Project;

use self::source::Source;

///
/// The contract data representation.
///
#[derive(Debug, Clone)]
pub struct Contract {
    /// The absolute file path.
    pub path: String,
    /// The contract type name.
    pub name: String,
    /// The source code data.
    pub source: Source,
    /// The factory dependencies.
    pub factory_dependencies: HashMap<String, String>,
    /// The zkEVM build.
    pub build: Option<compiler_llvm_context::Build>,
}

impl Contract {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(path: String, name: String, source: Source) -> Self {
        Self {
            path,
            name,
            source,
            factory_dependencies: HashMap::new(),
            build: None,
        }
    }

    ///
    /// Compiles the specified contract, setting its build artifacts.
    ///
    pub fn compile(
        mut self,
        project: &mut Project,
        contract_path: &str,
        optimization_level_middle: inkwell::OptimizationLevel,
        optimization_level_back: inkwell::OptimizationLevel,
        run_inliner: bool,
        dump_flags: Vec<DumpFlag>,
    ) -> anyhow::Result<compiler_llvm_context::Build> {
        let llvm = inkwell::context::Context::create();
        let optimizer = compiler_llvm_context::Optimizer::new(
            optimization_level_middle,
            optimization_level_back,
            run_inliner,
        )?;
        let dump_flags = compiler_llvm_context::DumpFlag::initialize(
            dump_flags.contains(&DumpFlag::Yul),
            dump_flags.contains(&DumpFlag::EthIR),
            dump_flags.contains(&DumpFlag::EVM),
            false,
            dump_flags.contains(&DumpFlag::LLVM),
            dump_flags.contains(&DumpFlag::Assembly),
        );
        let module_name = match self.source {
            Source::Yul(ref yul) => yul.object.identifier.to_owned(),
            Source::EVM(ref evm) => evm.full_path.to_owned(),
        };
        let mut context = match self.source {
            Source::Yul(_) => compiler_llvm_context::Context::new(
                &llvm,
                module_name.as_str(),
                optimizer,
                Some(project),
                dump_flags,
            ),
            Source::EVM(_) => {
                let version = project.version.to_owned();
                compiler_llvm_context::Context::new_evm(
                    &llvm,
                    module_name.as_str(),
                    optimizer,
                    Some(project),
                    dump_flags,
                    compiler_llvm_context::ContextEVMData::new(version),
                )
            }
        };

        self.source.declare(&mut context).map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` LLVM IR generator declaration pass error: {}",
                contract_path,
                error
            )
        })?;
        self.source.into_llvm(&mut context).map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` LLVM IR generator definition pass error: {}",
                contract_path,
                error
            )
        })?;
        let build = context.build(contract_path)?;

        Ok(build)
    }

    ///
    /// Inserts a factory dependency.
    ///
    pub fn insert_factory_dependency(&mut self, hash: String, path: String) {
        self.factory_dependencies.insert(hash, path);
    }
}

impl<D> WriteLLVM<D> for Contract
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.source.declare(context)
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        self.source.into_llvm(context)
    }
}
