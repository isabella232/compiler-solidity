pub mod utils {

    use rand::Rng;

    /// File type for input and output files
    #[derive(Debug)]
    pub enum FileType {
        Solidity,
        Yul,
        Zinc,
        Unknown,
    }

    /// Provide FileType for a given file based on its extension
    pub fn file_type(file: &str) -> FileType {
        let extension = std::path::Path::new(file)
            .extension()
            .and_then(std::ffi::OsStr::to_str);
        match extension {
            None => FileType::Unknown,
            Some("sol") => FileType::Solidity,
            Some("yul") => FileType::Yul,
            Some("zinc") => FileType::Zinc,
            Some(_) => FileType::Unknown,
        }
    }

    /// Abstract compilation step
    #[derive(Debug)]
    pub enum Action<'a> {
        SolidityCompiler(&'a str, String),
        CodeGenerator(String),
    }

    /// Generate temporary output directory for a given solidity input
    /// Precondition: input must exist
    fn tmp_yul(input: &str) -> String {
        let mut path = std::env::temp_dir();
        let suffix: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let file_stem = std::path::Path::new(input).file_stem().unwrap();
        let tmp_dir_name = String::from(file_stem.to_str().unwrap()) + "-" + suffix.as_str();
        path.push(tmp_dir_name);
        String::from(path.to_str().unwrap())
    }

    /// Produce sequence of actions required to compile file with specified options
    pub fn generate_actions<'a>(file: &'a str, options: &'a str) -> std::vec::Vec<Action<'a>> {
        match file_type(file) {
            FileType::Yul => vec![Action::CodeGenerator(String::from(file))],
            FileType::Solidity => {
                let tmp_file = tmp_yul(file);
                let options_string = String::from(options) + " --ir -o " + tmp_file.as_str();
                let options_string = String::from(options_string.trim());
                vec![
                    Action::SolidityCompiler(file, options_string),
                    Action::CodeGenerator(tmp_file),
                ]
            }
            _ => vec![],
        }
    }

    /// Wrap Solidity compiler invocation
    pub fn invoke_solidity(input: &str, options: &str) {
        std::process::Command::new("solc")
            .arg(input)
            .args(options.split(' ').collect::<Vec<&str>>())
            .spawn()
            .expect("Unable to run solidity. Ensure it's in PATH");
    }

    /// Wrap Zinc generator invocation
    pub fn invoke_codegen(_input: &str) {
        // TODO: implement
        std::unreachable!()
    }

    /// Execute an action by calling corresponding handler
    pub fn execute_action<'a>(action: &Action<'a>) {
        match action {
            Action::SolidityCompiler(input, options) => invoke_solidity(input, options.as_str()),
            Action::CodeGenerator(input) => invoke_codegen(input.as_str()),
        }
    }
}
