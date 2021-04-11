//!
//! YUL to LLVM compiler action.
//!

use std::path::Path;

///
/// The compilation action.
///
#[derive(Debug)]
pub struct Action {}

impl Action {
    ///
    /// Executes the Solidity compiler.
    ///
    pub fn solc(input: &Path, options: String) {
        let child = std::process::Command::new("solc")
            .arg(&input)
            .args(options.split(' ').collect::<Vec<&str>>())
            .spawn()
            .expect("The `solc` spawning error. Ensure it's in PATH");
        let output = child.wait_with_output().expect("The `solc` waiting error");
        if !output.status.success() {
            let mut message = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
            message.push_str(String::from_utf8_lossy(output.stderr.as_slice()).as_ref());
            panic!("The `solc` error: {}", message);
        }
    }
}
