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

use std::error::Error;

use getopts::{Matches, Options};
use netpulse::analyze::{
    self, get_checks, outages_detailed, CheckAccessConstraints, IpAddrConstraint,
};
use netpulse::common::{init_logging, print_usage, setup_panic_handler};
use netpulse::errors::RunError;
use netpulse::records::{display_group, Check};
use netpulse::store::Store;
use tracing::error;

fn main() {
    setup_panic_handler();
    #[cfg(not(debug_assertions))]
    init_logging(tracing::Level::INFO);
    #[cfg(debug_assertions)]
    init_logging(tracing::Level::TRACE);
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    let mut constraints = CheckAccessConstraints::default();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("V", "version", "print the version");
    opts.optflag("t", "test", "test run all checks");
    opts.optflag(
        "o",
        "outages",
        "print out all outages, use --dump to show all contained",
    );
    opts.optflag("d", "dump", "print out all checks");
    opts.optflag("4", "ipv4", "only consider ipv4");
    opts.optflag("6", "ipv6", "only consider ipv6");
    opts.optflag("6", "ipv6", "only consider ipv6");
    opts.optflag("c", "complete", "only consider complete outages");
    opts.optopt(
        "s",
        "since",
        "only consider checks after DATETIME (Format: 2025-06-11T12:00:00Z)",
        "DATETIME",
    );
    opts.optopt(
        "l",
        "latest",
        "only consider the N latest checks or outages",
        "N",
    );
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
    if matches.opt_present("version") {
        print_version()
    }
    if matches.opt_present("failed") {
        constraints.failed_only = true;
    }
    if matches.opt_present("complete") {
        constraints.failed_only = true;
        constraints.only_complete = true;
    }
    if matches.opt_present("ipv4") {
        constraints.ip = IpAddrConstraint::V4;
    } else if matches.opt_present("ipv6") {
        constraints.ip = IpAddrConstraint::V6;
    }
    match matches.opt_get("since") {
        Ok(since) => constraints.since_date = since,
        Err(e) => err_handler(e),
    }

    if let Err(e) = analyze(constraints, matches) {
        err_handler(e)
    }
}

fn err_handler(e: impl Error) -> ! {
    error!("{e}");
    std::process::exit(1)
}

fn analyze(constraints: CheckAccessConstraints, matches: Matches) -> Result<(), RunError> {
    let store = Store::load(true)?;

    let latest: Option<usize> = match matches.opt_get("latest") {
        Ok(l) => l,
        Err(e) => err_handler(e),
    };

    macro_rules! incheck {
        () => {{
            get_checks(&store, constraints).map_err(|e| RunError::from(e))?
        }};
    }
    macro_rules! checks {
        () => {{
            let mut _checks = incheck!();
            tracing::debug!("Get checks final checks: {}", _checks.len());
            _checks
        }};
        ($latest:expr) => {{
            let mut _checks = incheck!();
            if let Some(latest) = $latest {
                // we want to cut the last ones, not the first ones
                _checks.sort_by(|a, b| a.cmp(b).reverse());
                _checks.truncate(latest);
                _checks.sort();
            }
            tracing::debug!("Get checks final checks: {}", _checks.len());
            _checks
        }};
    }

    if matches.opt_present("outages") {
        print_outages(&checks!(), latest, matches.opt_present("dump"))?;
    } else if matches.opt_present("dump") {
        dump(&checks!(latest))?;
    } else if matches.opt_present("test") {
        test_checks()?;
    } else if matches.opt_present("rewrite") {
        rewrite()?;
    } else {
        analysis(&store, &checks!(latest))?;
    }
    Ok(())
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

fn print_outages(checks: &[&Check], latest: Option<usize>, dump: bool) -> Result<(), RunError> {
    let mut buf = String::new();
    if let Err(e) = outages_detailed(checks, latest, &mut buf, dump) {
        eprintln!("{e}");
        std::process::exit(1);
    }
    println!("{buf}");
    Ok(())
}

fn dump(checks: &[&Check]) -> Result<(), RunError> {
    let mut buf = String::new();
    if let Err(e) = display_group(checks, &mut buf) {
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

fn analysis(store: &Store, relevant_checks: &[&Check]) -> Result<(), RunError> {
    match analyze::analyze(store, relevant_checks) {
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
