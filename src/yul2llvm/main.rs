//!
//! YUL to LLVM compiler binary.
//!

pub mod arguments;

use self::arguments::Arguments;

fn main() {
    let args = Arguments::new();

    let file_name = args.input;

    println!("Input {}", file_name.to_str().unwrap());
    if !file_name.exists() {
        panic!("{} does not exist", file_name.to_str().unwrap());
    }

    let opts = args.xsol.as_str();

    let run = match args.run.as_str() {
        "" => None,
        v => Some(v),
    };

    let actions = yul2llvm::generate_actions(&file_name, opts, &run);

    for a in actions.iter() {
        yul2llvm::execute_action(a);
    }
}
