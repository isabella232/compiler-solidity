//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() {
    let args = Arguments::new();

    let file_name = args.input;

    println!("Input {}", file_name.to_str().unwrap());
    if !file_name.exists() {
        panic!("{} does not exist", file_name.to_str().unwrap());
    }

    let actions = yul2llvm::generate_actions(&file_name, args.xsol.as_str(), args.entry);
    for action in actions.into_iter() {
        yul2llvm::execute_action(action);
    }
}
