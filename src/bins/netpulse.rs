use std::str::FromStr;

use getopts::Options;
use netpulse::analyze;
use netpulse::records::{Check, CheckType, TARGETS_HTTP};
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
            panic!("{}", f.to_string())
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

fn test_checks() {
    for target in TARGETS_HTTP {
        let check = CheckType::Http.make(
            std::net::IpAddr::from_str(target).expect("a target constant was not an Ip Address"),
        );
        println!("{check}");
    }
}

fn analysis() {
    let store = Store::load().expect("store file not found");
    match analyze::analyze(&store) {
        Err(e) => {
            eprintln!("Error while making the analysis: {e}");
            std::process::exit(1);
        }
        Ok(report) => println!("{report}"),
    }
}
