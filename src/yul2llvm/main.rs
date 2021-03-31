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

    let path = args.input;

    println!("Input {}", path.to_str().unwrap());
    if !path.exists() {
        panic!("{} does not exist", path.to_str().unwrap());
    }

    let actions = yul2llvm::Action::new_list(path, args.xsol, args.entry);
    for action in actions.into_iter() {
        action.execute();
    }
}
