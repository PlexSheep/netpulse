use getopts::Options;
use netpulse::records::{Check, CheckType};
use netpulse::store::Store;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("t", "test", "test run all checks");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("help") {
        print_usage(program, opts);
    } else if matches.opt_present("test") {
        test_checks();
    } else {
        analyze();
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn test_checks() {
    for check_type in CheckType::enabled() {
        let check = check_type.make();
        println!("{check}");
    }
}

fn analyze() {
    let store = Store::load().expect("store file not found");
    let checks = store.checks();
    let successes: Vec<&Check> = checks.iter().filter(|c| c.is_success()).collect();
    println!("store contains {:09} checks.", checks.len());
    println!("store contains {:09} successful checks.", successes.len());
    println!(
        "success ratio: {:02.02}%",
        (successes.len() as f64 / checks.len() as f64) * 100.0
    )
}
