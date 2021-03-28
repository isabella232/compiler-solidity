//!
//! The YUL to LLVM tester binary.
//!

pub mod arguments;

use std::fs;
use std::path::Path;

use colored::Colorize;
use regex::Regex;

use self::arguments::Arguments;

///
/// Arguments and expected result for a single test run.
///
#[derive(Debug)]
struct TestRun<'a> {
    /// Test to compile
    source: &'a Path,
    /// Function to run after the test compilation
    function: String,
    /// Expected result
    result: u64,
}

fn read_test_config(file: &Path) -> Result<TestRun, &'static str> {
    let source = fs::read_to_string(file).unwrap();
    let run_rx = Regex::new(r"RUN:\s*(\w*)").unwrap();
    let result_rx = Regex::new(r"EXPECT:\s*([0-9]*)").unwrap();
    let function = match run_rx.captures(source.as_str()) {
        Some(m) => m.get(1).unwrap().as_str(),
        _ => return Err("The file does not contain RUN line"),
    }
    .to_string();
    let result = match result_rx.captures(source.as_str()) {
        Some(m) => m.get(1).unwrap().as_str(),
        _ => return Err("The file does not contain EXPECT line"),
    };
    let result = result.parse::<u64>().unwrap();
    Ok(TestRun {
        source: file,
        function,
        result,
    })
}

fn run_test(run: &TestRun) -> Result<(), String> {
    let invocation = std::process::Command::new("yul2llvm")
        .arg(run.source.to_str().unwrap())
        .arg("-r")
        .arg(run.function.as_str())
        .output();
    if let Err(msg) = invocation {
        return Err(format!("{}", msg));
    }
    if let Ok(out) = invocation {
        if !out.status.success() {
            return Err("Failed to compile".to_string());
        }
        let out = String::from_utf8(out.stdout).unwrap();
        let result_rx = Regex::new(r"Result:\s*([0-9]+)").unwrap();
        let result = match result_rx.captures(out.as_str()) {
            Some(m) => m.get(1).unwrap().as_str(),
            _ => return Err("The compiler ouput does not contain Result".to_string()),
        }
        .parse::<u64>()
        .unwrap();
        if result != run.result {
            return Err(format!("Expected {}, got {}", run.result, result));
        }
    }
    Ok(())
}

fn handle_test(run: &Result<TestRun, &'static str>) -> Result<(), String> {
    match run {
        Err(msg) => Err(msg.to_string()),
        Ok(run) => run_test(&run),
    }
}

///
/// The application entry point.
///
fn main() {
    let args = Arguments::new();

    let path = args.input;
    if !path.exists() {
        panic!("{} does not exist", path.to_str().unwrap());
    }
    let meta = fs::metadata(path.clone()).unwrap();

    let filenames = if meta.is_file() {
        vec![path]
    } else {
        std::fs::read_dir(path)
            .unwrap()
            .map(|x| x.unwrap().path())
            .collect()
    };

    let runs: Vec<_> = filenames
        .iter()
        .map(|name| read_test_config(name))
        .collect();

    println!(
        "[INTEGRATION] Started with {} worker threads",
        rayon::current_num_threads(),
    );

    for run in runs {
        let result = handle_test(&run);
        match result {
            Ok(_) => println!(
                "[{}] {} ({})",
                "INTEGRATION".green(),
                "PASSED".green(),
                run.unwrap().source.to_str().unwrap()
            ),
            Err(msg) => println!(
                "[{}] {} ({})",
                "INTEGRATION".bright_red(),
                "FAILED".bright_red(),
                msg
            ),
        }
    }
}
