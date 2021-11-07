//!
//! The Solidity compiler.
//!

pub mod input;
pub mod output;

use std::io::Write;
use std::str::FromStr;

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

impl Default for Compiler {
    fn default() -> Self {
        Self {
            executable: "solc".to_owned(),
            version: semver::Version::from_str(env!("CARGO_PKG_VERSION")).expect("Always valid"),
        }
    }
}

impl Compiler {
    ///
    /// Compiles the Solidity `--standard-json` input into Yul IR.
    ///
    pub fn standard_json(
        &self,
        input: Input,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
    ) -> Result<Output, String> {
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
            .map_err(|_| "Solc spawning error".to_owned())?;
        solc_process
            .stdin
            .as_ref()
            .ok_or_else(|| "Solc stdin getting error".to_owned())?
            .write_all(input_json.as_slice())
            .map_err(|_| "Solc stdin writing error".to_owned())?;

        let solc_output = solc_process
            .wait_with_output()
            .map_err(|_| "Solc process error".to_owned())?;
        if !solc_output.status.success() {
            return Err(String::from_utf8_lossy(solc_output.stderr.as_slice()).to_string());
        }

        let output = serde_json::from_slice(solc_output.stdout.as_slice()).expect("Always valid");

        Ok(output)
    }
}
