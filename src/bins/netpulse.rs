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

use getopts::Options;
use netpulse::analyze;
use netpulse::common::{init_logging, print_usage, setup_panic_handler};
use netpulse::errors::RunError;
use netpulse::records::{display_group, Check};
use netpulse::store::Store;
use tracing::error;

fn main() {
    setup_panic_handler();
    init_logging(tracing::Level::INFO);
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    let mut failed_only = false;
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("V", "version", "print the version");
    opts.optflag("t", "test", "test run all checks");
    opts.optflag("d", "dump", "print out all checks");
    opts.optflag(
        "r",
        "rewrite",
        "load store and immediately save to rewrite the file",
    );
    opts.optflag("f", "failed", "only consider failed checks for dumping");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{f}");
            print_usage(program, opts);
        }
    };

    if matches.opt_present("help") {
        print_usage(program, opts);
    }
    if matches.opt_present("failed") {
        failed_only = true;
    }
    if matches.opt_present("version") {
        print_version()
    }
    if matches.opt_present("dump") {
        if let Err(e) = dump(failed_only) {
            error!("{e}");
            std::process::exit(1)
        }
    } else if matches.opt_present("test") {
        if let Err(e) = test_checks() {
            error!("{e}");
            std::process::exit(1)
        }
    } else if matches.opt_present("rewrite") {
        if let Err(e) = rewrite() {
            error!("{e}");
            std::process::exit(1)
        }
    } else {
        if let Err(e) = analysis() {
            error!("{e}");
            std::process::exit(1)
        }
    }
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

fn dump(failed_only: bool) -> Result<(), RunError> {
    let store = Store::load(true)?;
    let mut buf = String::new();
    let ref_checks: Vec<&Check> = if failed_only {
        store.checks().iter().filter(|c| !c.is_success()).collect()
    } else {
        store.checks().iter().collect()
    };
    if let Err(e) = display_group(&ref_checks, &mut buf) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    println!("{buf}");
    Ok(())
}

fn rewrite() -> Result<(), RunError> {
    let s = Store::load(true)?;
    s.save()?;
    Ok(())
}

fn analysis() -> Result<(), RunError> {
    let store = Store::load(true)?;
    match analyze::analyze(&store) {
        Err(e) => {
            eprintln!("Error while making the analysis: {e}");
            std::process::exit(1);
        }
        Ok(report) => println!("{report}"),
    }
    Ok(())
}

fn print_version() -> ! {
    println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
    std::process::exit(0)
}
