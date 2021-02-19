#[macro_use]
extern crate clap;

mod lib;

use crate::lib::*;
use std::path::Path;

fn main() {
    let matches = clap_app! (sol2tvm =>
        (version: "0.1")
        (author: "MatterLabs")
        (about: "Solidity to Zinc translator")
        (@arg INPUT: +takes_value +required "Input file")
        (@arg RUN: -r +takes_value "function to run")
        (@arg XSOL: +takes_value "Command line options to pass to Solidity compiler")
    )
    .get_matches();

    let file_name = matches.value_of("INPUT").unwrap();

    if !Path::new(file_name).exists() {
        panic!("{} does not exist", file_name);
    }

    let file_type = file_type(file_name);
    println!("{:?}", file_type);

    let opts = matches.value_of("XSOL").unwrap_or("");

    let run = matches.value_of("RUN");
    let actions = generate_actions(file_name, opts, &run);
    println!("{:?}", actions);

    for a in actions.iter() {
        execute_action(a);
    }

    println!("Input: {}", file_name);
}
