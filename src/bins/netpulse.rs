//! CLI tool for analyzing netpulse check results.
//!
//! This binary provides commands to:
//! - View analysis reports of collected check data
//! - Run test checks against configured targets
//! - Display version information
//!
//! # Usage
//!
//! Without options, displays analysis of stored check results.
//!
//! Use the `--help` flag for more information about the usage.

use std::str::FromStr;

use getopts::Options;
use netpulse::analyze::{self, display_group};
use netpulse::errors::{RunError, StoreError};
use netpulse::records::{Check, CheckType, TARGETS};
use netpulse::store::Store;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("V", "version", "print the version");
    opts.optflag("t", "test", "test run all checks");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{f}");
            print_usage(program, opts);
            std::process::exit(1)
        }
    };

    if matches.opt_present("help") {
        print_usage(program, opts);
    } else if matches.opt_present("version") {
        println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"))
    } else if matches.opt_present("test") {
        test_checks();
    } else {
        analysis();
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn test_checks() -> Result<(), RunError> {
    let mut checks = Vec::new();
    let mut buf = String::new();
    Store::primitive_make_checks(&mut checks);
    let hack_checks: Vec<&Check> = checks.iter().collect();
    display_group(&hack_checks, &mut buf)?;
    println!("{buf}");
    Ok(())
}

fn analysis() {
    let store = match Store::load() {
        Err(e) => {
            eprintln!("The store could not be loaded: {e}");
            std::process::exit(1)
        }
        Ok(s) => s,
    };
    match analyze::analyze(&store) {
        Err(e) => {
            eprintln!("Error while making the analysis: {e}");
            std::process::exit(1);
        }
        Ok(report) => println!("{report}"),
    }
}
