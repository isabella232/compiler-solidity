//!
//! The Solidity compiler.
//!

pub mod combined_json;
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
        input: StandardJsonInput,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
    ) -> Result<StandardJsonOutput, String> {
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

        let solc_process = solc_command
            .spawn()
            .map_err(|error| format!("solc subprocess spawning error: {:?}", error))?;
        solc_process
            .stdin
            .as_ref()
            .ok_or_else(|| "solc stdin getting error".to_owned())?
            .write_all(input_json.as_slice())
            .map_err(|error| format!("solc stdin writing error: {:?}", error))?;

        let solc_output = solc_process
            .wait_with_output()
            .map_err(|error| format!("solc subprocess output error: {:?}", error))?;
        if !solc_output.status.success() {
            return Err(String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string());
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
    ) -> Result<CombinedJson, String> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.args(paths);
        solc_command.arg("--combined-json");
        solc_command.arg(combined_json_argument);
        let solc_pipeline = solc_command
            .output()
            .map_err(|error| format!("solc subprocess error: {:?}", error))?;
        if !solc_pipeline.status.success() {
            return Err(String::from_utf8_lossy(solc_pipeline.stderr.as_slice()).to_string());
        }

        let combined_json =
            serde_json::from_slice(solc_pipeline.stdout.as_slice()).expect("Always valid");

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
    ) -> Result<String, String> {
        let mut solc_command = std::process::Command::new(self.executable.as_str());
        solc_command.args(paths);
        if output_abi {
            solc_command.arg("--abi");
        }
        if output_hashes {
            solc_command.arg("--hashes");
        }
        let solc_pipeline = solc_command
            .output()
            .map_err(|error| format!("solc subprocess error: {:?}", error))?;
        if !solc_pipeline.status.success() {
            return Err(String::from_utf8_lossy(solc_pipeline.stderr.as_slice()).to_string());
        }

        Ok(String::from_utf8_lossy(solc_pipeline.stdout.as_slice()).to_string())
    }
}
