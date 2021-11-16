//!
//! The Solidity compiler.
//!

pub mod hashes;
pub mod input;
pub mod output;

use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::error::Error;

use self::hashes::Hashes;
use self::input::Input;
use self::output::Output;

///
/// The Solidity compiler.
///
pub struct Compiler {
    /// The binary executable name.
    pub executable: String,
    /// The compiler version.
    pub version: semver::Version,
}

impl Compiler {
    ///
    /// A shortcut constructor.
    ///
    /// Different tools may use different `executable` names. For example, the integration tester
    /// uses `solc-<version>` format.
    ///
    pub fn new(executable: String, version: semver::Version) -> Self {
        Self {
            executable,
            version,
        }
    }

    ///
    /// Compiles the Solidity `--standard-json` input into Yul IR.
    ///
    pub fn standard_json(
        &self,
        input: Input,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
    ) -> Result<Output, Error> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.stdin(std::process::Stdio::piped());
        solc_command.stdout(std::process::Stdio::piped());
        solc_command.arg("--standard-json");

        if let Some(base_path) = base_path {
            solc_command.arg("--base-path");
            solc_command.arg(base_path);
        }
        for include_path in include_paths.into_iter() {
            solc_command.arg("--include-path");
            solc_command.arg(include_path);
        }
        if let Some(allow_paths) = allow_paths {
            solc_command.arg("--allow-paths");
            solc_command.arg(allow_paths);
        }

        let input_json = serde_json::to_vec(&input)?;

        let solc_process = solc_command.spawn()?;
        solc_process
            .stdin
            .as_ref()
            .unwrap_or_else(|| panic!("Solc stdin getting error"))
            .write_all(input_json.as_slice())?;

        let solc_output = solc_process.wait_with_output()?;
        if !solc_output.status.success() {
            return Err(Error::Solc(
                String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string(),
            ));
        }

        let output = serde_json::from_slice(solc_output.stdout.as_slice()).expect("Always valid");

        Ok(output)
    }

    ///
    /// Returns the entry hashes for the contract at `path`.
    ///
    pub fn hashes(&self, path: &Path) -> Result<Hashes, Error> {
        let solc_pipeline = std::process::Command::new(self.executable.as_str())
            .arg(&path)
            .arg("--combined-json")
            .arg("hashes")
            .output()?;
        if !solc_pipeline.status.success() {
            return Err(Error::Solc(
                String::from_utf8_lossy(solc_pipeline.stderr.as_slice()).to_string(),
            ));
        }

        Ok(serde_json::from_slice(solc_pipeline.stdout.as_slice())?)
    }

    ///
    /// The `solc --abi --hashes ...` mirror.
    ///
    pub fn extra_output(
        &self,
        paths: &[PathBuf],
        output_abi: bool,
        output_hashes: bool,
    ) -> Result<String, Error> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.args(paths);
        if output_abi {
            solc_command.arg("--abi");
        }
        if output_hashes {
            solc_command.arg("--hashes");
        }
        let solc_pipeline = solc_command.output()?;
        if !solc_pipeline.status.success() {
            return Err(Error::Solc(
                String::from_utf8_lossy(solc_pipeline.stderr.as_slice()).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(solc_pipeline.stdout.as_slice()).to_string())
    }

    ///
    /// The `solc --combined-json abi,hashes...` mirror.
    ///
    pub fn combined_json(
        &self,
        paths: &[PathBuf],
        combined_json_argument: String,
    ) -> Result<String, Error> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.args(paths);
        solc_command.arg("--combined-json");
        solc_command.arg(combined_json_argument);
        let solc_pipeline = solc_command.output()?;
        if !solc_pipeline.status.success() {
            return Err(Error::Solc(
                String::from_utf8_lossy(solc_pipeline.stderr.as_slice()).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(solc_pipeline.stdout.as_slice()).to_string())
    }
}
