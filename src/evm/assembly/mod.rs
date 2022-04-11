//!
//! The `solc --asm-json` output representation.
//!

pub mod data;
pub mod instruction;

use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::evm::ethereal_ir::entry_link::EntryLink;
use crate::evm::ethereal_ir::EtherealIR;

use self::data::Data;
use self::instruction::name::Name as InstructionName;
use self::instruction::Instruction;

///
/// The JSON assembly representation.
///
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Assembly {
    /// The metadata string.
    #[serde(rename = ".auxdata")]
    pub auxdata: Option<String>,
    /// The constructor code instructions.
    #[serde(rename = ".code")]
    pub code: Option<Vec<Instruction>>,
    /// The selector code representation.
    #[serde(rename = ".data")]
    pub data: Option<BTreeMap<String, Data>>,

    /// The full contract path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_path: Option<String>,
}

impl Assembly {
    ///
    /// Gets the contract `keccak256` hash.
    ///
    pub fn keccak256(&self) -> String {
        let json = serde_json::to_vec(self).expect("Always valid");
        compiler_common::keccak256(json.as_slice())
    }

    ///
    /// Sets the full contract path.
    ///
    pub fn set_full_path(&mut self, full_path: String) {
        self.full_path = Some(full_path);
    }

    ///
    /// Returns the full contract path if it is set, or `<undefined>` otherwise.
    ///
    pub fn full_path(&self) -> String {
        self.full_path
            .to_owned()
            .unwrap_or_else(|| "<undefined>".to_owned())
    }

    ///
    /// Replaces the constructor dependencies with full contract path and returns the list.
    ///
    pub fn constructor_dependencies_pass(
        &mut self,
        full_path: &str,
        hash_data_mapping: &HashMap<String, String>,
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut index_path_mapping = HashMap::with_capacity(hash_data_mapping.len());
        let index = "0".repeat(compiler_common::SIZE_FIELD * 2);
        index_path_mapping.insert(index, full_path.to_owned());

        let dependencies = match self.data.as_mut() {
            Some(dependencies) => dependencies,
            None => return Ok(index_path_mapping),
        };
        for (index, data) in dependencies.iter_mut() {
            if index == "0" {
                continue;
            }

            *data = match data {
                Data::Assembly(assembly) => {
                    let hash = assembly.keccak256();
                    let full_path =
                        hash_data_mapping
                            .get(hash.as_str())
                            .cloned()
                            .ok_or_else(|| {
                                anyhow::anyhow!("Contract path not found for hash `{}`", hash)
                            })?;

                    let mut index_extended =
                        "0".repeat(compiler_common::SIZE_FIELD * 2 - index.len());
                    index_extended.push_str(index.as_str());
                    index_path_mapping.insert(index_extended, full_path.clone());

                    Data::Path(full_path)
                }
                Data::Hash(hash) => {
                    index_path_mapping.insert(index.to_owned(), hash.to_owned());
                    continue;
                }
                _ => continue,
            };
        }

        Ok(index_path_mapping)
    }

    ///
    /// Replaces the selector dependencies with full contract path and returns the list.
    ///
    pub fn selector_dependencies_pass(
        &mut self,
        full_path: &str,
        hash_data_mapping: &HashMap<String, String>,
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut index_path_mapping = HashMap::with_capacity(hash_data_mapping.len());
        let index = "0".repeat(compiler_common::SIZE_FIELD * 2);
        index_path_mapping.insert(index, full_path.to_owned());

        let dependencies = match self
            .data
            .as_mut()
            .and_then(|data| data.get_mut("0"))
            .and_then(|data| data.get_assembly_mut())
            .and_then(|assembly| assembly.data.as_mut())
        {
            Some(dependencies) => dependencies,
            None => return Ok(index_path_mapping),
        };
        for (index, data) in dependencies.iter_mut() {
            *data = match data {
                Data::Assembly(assembly) => {
                    let hash = assembly.keccak256();
                    let full_path =
                        hash_data_mapping
                            .get(hash.as_str())
                            .cloned()
                            .ok_or_else(|| {
                                anyhow::anyhow!("Contract path not found for hash `{}`", hash)
                            })?;

                    let mut index_extended =
                        "0".repeat(compiler_common::SIZE_FIELD * 2 - index.len());
                    index_extended.push_str(index.as_str());
                    index_path_mapping.insert(index_extended, full_path.clone());

                    Data::Path(full_path)
                }
                Data::Hash(hash) => {
                    index_path_mapping.insert(index.to_owned(), hash.to_owned());
                    continue;
                }
                _ => continue,
            };
        }

        Ok(index_path_mapping)
    }
}

impl<D> compiler_llvm_context::WriteLLVM<D> for Assembly
where
    D: compiler_llvm_context::Dependency,
{
    fn declare(&mut self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let mut entry = compiler_llvm_context::EntryFunction::default();
        entry.declare(context)?;

        compiler_llvm_context::ConstructorFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;
        compiler_llvm_context::SelectorFunction::new(
            compiler_llvm_context::DummyLLVMWritable::default(),
        )
        .declare(context)?;

        entry.into_llvm(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut compiler_llvm_context::Context<D>) -> anyhow::Result<()> {
        let full_path = self.full_path();

        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EVM) {
            println!("Contract `{}` constructor EVM:\n\n{}", full_path, self);
        }
        let mut constructor_ethereal_ir = EtherealIR::new(
            context.evm().version.to_owned(),
            self.code
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Constructor instructions not found"))?,
            compiler_llvm_context::CodeType::Deploy,
        )
        .map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` constructor Ethereal IR generator error: {}",
                full_path,
                error
            )
        })?;
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EthIR) {
            println!(
                "Contract `{}` constructor Ethereal IR:\n\n{}",
                full_path, constructor_ethereal_ir
            );
        }
        let constructor = compiler_llvm_context::ConstructorFunction::new(EntryLink::new(
            compiler_llvm_context::CodeType::Deploy,
        ));
        constructor_ethereal_ir.declare(context)?;
        constructor.into_llvm(context)?;
        constructor_ethereal_ir.into_llvm(context)?;

        let data = self
            .data
            .ok_or_else(|| anyhow::anyhow!("Selector data not found"))?
            .remove("0")
            .expect("Always exists");
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EVM) {
            println!("Contract `{}` selector EVM:\n\n{}", full_path, data);
        };
        let selector_instructions = match data {
            Data::Assembly(assembly) => assembly
                .code
                .ok_or_else(|| anyhow::anyhow!("Selector instructions not found"))?,
            Data::Hash(hash) => {
                anyhow::bail!("Expected selector instructions, found hash `{}`", hash)
            }
            Data::Path(path) => {
                anyhow::bail!("Expected selector instructions, found path `{}`", path)
            }
        };
        let mut selector_ethereal_ir = EtherealIR::new(
            context.evm().version.to_owned(),
            selector_instructions.as_slice(),
            compiler_llvm_context::CodeType::Runtime,
        )
        .map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` selector Ethereal IR generator error: {}",
                full_path,
                error
            )
        })?;
        if context.has_dump_flag(compiler_llvm_context::DumpFlag::EthIR) {
            println!(
                "Contract `{}` selector Ethereal IR:\n\n{}",
                full_path, selector_ethereal_ir
            );
        }
        let selector = compiler_llvm_context::SelectorFunction::new(EntryLink::new(
            compiler_llvm_context::CodeType::Runtime,
        ));
        selector_ethereal_ir.declare(context)?;
        selector.into_llvm(context)?;
        selector_ethereal_ir.into_llvm(context)?;

        Ok(())
    }
}

impl std::fmt::Display for Assembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(instructions) = self.code.as_ref() {
            for (index, instruction) in instructions.iter().enumerate() {
                match instruction.name {
                    InstructionName::Tag => writeln!(f, "{:03} {}", index, instruction)?,
                    _ => writeln!(f, "{:03}     {}", index, instruction)?,
                }
            }
        }

        Ok(())
    }
}
