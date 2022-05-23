//!
//! The Solidity project build.
//!

pub mod contract;

use std::collections::HashMap;
use std::path::Path;

use crate::solc::combined_json::CombinedJson;
use crate::solc::standard_json::output::contract::evm::EVM as StandardJsonOutputContractEVM;
use crate::solc::standard_json::output::Output as StandardJsonOutput;

use self::contract::Contract;

///
/// The Solidity project build.
///
#[derive(Debug, Default, Clone)]
pub struct Build {
    /// The contract data,
    pub contracts: HashMap<String, Contract>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            contracts: HashMap::with_capacity(capacity),
        }
    }

    ///
    /// Writes all contracts to the specified directory.
    ///
    pub fn write_to_directory(
        self,
        output_directory: &Path,
        output_assembly: bool,
        output_binary: bool,
        overwrite: bool,
    ) -> anyhow::Result<()> {
        for (_path, contract) in self.contracts.into_iter() {
            contract.write_to_directory(
                output_directory,
                output_assembly,
                output_binary,
                overwrite,
            )?;
        }

        Ok(())
    }

    ///
    /// Writes all contracts assembly and bytecode to the combined JSON.
    ///
    pub fn write_to_combined_json(self, combined_json: &mut CombinedJson) -> anyhow::Result<()> {
        for (path, contract) in self.contracts.into_iter() {
            let combined_json_contract = combined_json
                .contracts
                .iter_mut()
                .find_map(|(json_path, contract)| {
                    if path.ends_with(json_path) {
                        Some(contract)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("Contract `{}` not found in the project", path))?;

            contract.write_to_combined_json(combined_json_contract)?;
        }

        Ok(())
    }

    ///
    /// Writes all contracts assembly and bytecode to the standard JSON.
    ///
    pub fn write_to_standard_json(
        mut self,
        standard_json: &mut StandardJsonOutput,
    ) -> anyhow::Result<()> {
        let contracts = match standard_json.contracts.as_mut() {
            Some(contracts) => contracts,
            None => return Ok(()),
        };

        for (path, contracts) in contracts.iter_mut() {
            for (name, contract) in contracts.iter_mut() {
                let full_name = format!("{}:{}", path, name);

                if let Some(contract_data) = self.contracts.remove(full_name.as_str()) {
                    let bytecode = hex::encode(contract_data.build.bytecode.as_slice());

                    contract.ir_optimized = None;
                    contract.evm =
                        Some(StandardJsonOutputContractEVM::new_zkevm_bytecode(bytecode));
                    contract.factory_dependencies = Some(contract_data.build.factory_dependencies);
                    contract.hash = Some(contract_data.build.hash);
                }
            }
        }

        Ok(())
    }
}
