extern crate getopts;
use cw_build::{contract::build_contract, workspace::build_workspace};
use getopts::Options;

use std::{env, path::PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("w", "workspace", "workspace flag");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("workspace") {
        build_workspace();
    } else {
        build_contract(&PathBuf::from("."));
    }
}
