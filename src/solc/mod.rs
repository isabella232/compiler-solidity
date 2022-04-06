//!
//! The Solidity compiler.
//!

pub mod combined_json;
pub mod pipeline;
pub mod standard_json;

use std::io::Write;
use std::path::PathBuf;

use self::combined_json::CombinedJson;
use self::standard_json::input::Input as StandardJsonInput;
use self::standard_json::output::Output as StandardJsonOutput;

///
/// The Solidity compiler.
///
pub struct Compiler {
    /// The binary executable name.
    pub executable: String,
}

impl Compiler {
    /// The default executable name.
    pub const DEFAULT_EXECUTABLE_NAME: &'static str = "solc";

    ///
    /// A shortcut constructor.
    ///
    /// Different tools may use different `executable` names. For example, the integration tester
    /// uses `solc-<version>` format.
    ///
    pub fn new(executable: String) -> Self {
        Self { executable }
    }

    ///
    /// Compiles the Solidity `--standard-json` input into Yul IR.
    ///
    pub fn standard_json(
        &self,
        input: StandardJsonInput,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
    ) -> anyhow::Result<StandardJsonOutput> {
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

        let input_json = serde_json::to_vec(&input).expect("Always valid");

        let solc_process = solc_command.spawn().map_err(|error| {
            anyhow::anyhow!("{} subprocess spawning error: {:?}", self.executable, error)
        })?;
        solc_process
            .stdin
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("{} stdin getting error", self.executable))?
            .write_all(input_json.as_slice())
            .map_err(|error| {
                anyhow::anyhow!("{} stdin writing error: {:?}", self.executable, error)
            })?;

        let solc_output = solc_process.wait_with_output().map_err(|error| {
            anyhow::anyhow!("{} subprocess output error: {:?}", self.executable, error)
        })?;
        if !solc_output.status.success() {
            anyhow::bail!(
                "{} error: {}",
                self.executable,
                String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string()
            );
        }

        let output = serde_json::from_slice(solc_output.stdout.as_slice()).expect("Always valid");

        Ok(output)
    }

    ///
    /// The `solc --combined-json abi,hashes...` mirror.
    ///
    pub fn combined_json(
        &self,
        paths: &[PathBuf],
        combined_json_argument: &str,
    ) -> anyhow::Result<CombinedJson> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.args(paths);
        solc_command.arg("--combined-json");
        solc_command.arg(combined_json_argument);
        let solc_output = solc_command.output().map_err(|error| {
            anyhow::anyhow!("{} subprocess error: {:?}", self.executable, error)
        })?;
        if !solc_output.status.success() {
            anyhow::bail!(
                "{} error: {}",
                self.executable,
                String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string()
            );
        }

        let combined_json =
            serde_json::from_slice(solc_output.stdout.as_slice()).expect("Always valid");

        Ok(combined_json)
    }

    ///
    /// The `solc --abi --hashes ...` mirror.
    ///
    pub fn extra_output(
        &self,
        paths: &[PathBuf],
        output_abi: bool,
        output_hashes: bool,
    ) -> anyhow::Result<String> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.args(paths);
        if output_abi {
            solc_command.arg("--abi");
        }
        if output_hashes {
            solc_command.arg("--hashes");
        }
        let solc_output = solc_command.output().map_err(|error| {
            anyhow::anyhow!("{} subprocess error: {:?}", self.executable, error)
        })?;
        if !solc_output.status.success() {
            anyhow::bail!(
                "{} error: {}",
                self.executable,
                String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string()
            );
        }

        Ok(String::from_utf8_lossy(solc_output.stdout.as_slice()).to_string())
    }

    ///
    /// The `solc --version` mini-parser.
    ///
    pub fn version(&self) -> anyhow::Result<semver::Version> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.arg("--version");
        let solc_output = solc_command.output().map_err(|error| {
            anyhow::anyhow!("{} subprocess error: {:?}", self.executable, error)
        })?;
        if !solc_output.status.success() {
            anyhow::bail!(
                "{} error: {}",
                self.executable,
                String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string()
            );
        }

        let stdout = String::from_utf8_lossy(solc_output.stdout.as_slice());
        let version: semver::Version = stdout
            .lines()
            .nth(1)
            .ok_or_else(|| {
                anyhow::anyhow!("{} version parsing: not enough lines", self.executable)
            })?
            .split(' ')
            .nth(1)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{} version parsing: not enough words in the 2nd line",
                    self.executable
                )
            })?
            .split('+')
            .next()
            .ok_or_else(|| {
                anyhow::anyhow!("{} version parsing: metadata dropping", self.executable)
            })?
            .parse()
            .map_err(|error| anyhow::anyhow!("{} version parsing: {}", self.executable, error))?;

        Ok(version)
    }
}
