pub mod arguments;
mod lib;

use self::arguments::Arguments;
use crate::lib::*;

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

    let actions = generate_actions(&file_name, opts, &run);

    for a in actions.iter() {
        execute_action(a);
    }
}
