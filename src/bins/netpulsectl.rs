use core::panic;
use std::fs::{self, File};

use daemonize::Daemonize;
use getopts::Options;
use netpulse::store::Store;
use netpulse::{DAEMON_LOG_ERR, DAEMON_LOG_INF};

mod daemon;
use daemon::daemon;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("s", "start", "start the netpulse daemon");
    opts.optflag("i", "info", "info about the running netpulse daemon");
    opts.optflag("e", "end", "stop the running netpulse daemon");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("help") {
        print_usage(&program, opts);
    } else if matches.opt_present("start") {
        startd();
    } else if matches.opt_present("info") {
        infod();
    } else if matches.opt_present("end") {
        endd();
    } else {
        print_usage(&program, opts);
    }
}

fn infod() {
    todo!()
}

fn endd() {
    todo!()
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn startd() {
    let path = Store::path();
    let parent_path = path.parent().expect("store file has no parent directory");
    println!("Parent: {parent_path:?}");

    let logfile = File::create(DAEMON_LOG_INF).expect("could not open info logfile");
    let errfile = File::create(DAEMON_LOG_ERR).expect("could not open error logfile");

    fs::create_dir_all(parent_path).expect("could not create the store directory");

    let daemonize = Daemonize::new()
        .pid_file("/run/netpulse.pid")
        .chown_pid_file(true)
        .working_directory(parent_path)
        .user("netpulse")
        .group("netpulse")
        .stdout(logfile)
        .stderr(errfile)
        .umask(0o027); // rwxr-x---

    println!("daemon setup done");

    let outcome = daemonize.execute();
    match outcome {
        daemonize::Outcome::Parent(result) => {
            if result.is_ok() {
                println!("netpulsed was started");
            } else {
                eprintln!("error while starting netpulsed: {result:#?}");
            }
        }
        daemonize::Outcome::Child(result) => {
            if result.is_ok() {
                daemon();
            } else {
                panic!("error while starting the daemon: {result:#?}")
            }
        }
    }
}
