use core::panic;
use std::fs::{self, File};
use std::path::PathBuf;

use daemonize::Daemonize;
use getopts::Options;
use netpulse::store::Store;
use netpulse::{DAEMON_LOG_ERR, DAEMON_LOG_INF, DAEMON_PID_FILE, DAEMON_USER};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

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
        print_usage(program, opts);
    } else if matches.opt_present("start") {
        startd();
    } else if matches.opt_present("info") {
        infod();
    } else if matches.opt_present("end") {
        endd();
    } else {
        print_usage(program, opts);
    }
}

fn getpid() -> Option<i32> {
    if !fs::exists(DAEMON_PID_FILE).expect("couldn't check if the pid file exists") {
        None
    } else {
        let pid_raw = fs::read_to_string(DAEMON_PID_FILE)
            .expect("pid file does not exist")
            .trim()
            .to_string();
        let pid = match pid_raw.parse() {
            Ok(pid) => pid,
            Err(err) => {
                eprintln!("Error while parsing the pid from file ('{pid_raw}'): {err}");
                return None;
            }
        };
        Some(pid)
    }
}

fn infod() {
    match getpid() {
        Some(pid) => {
            println!("netpulsed was started with pid {pid}");
        }
        None => println!("netpulsed is not running"),
    }
}

fn endd() {
    match getpid() {
        Some(pid) => {
            match signal::kill(Pid::from_raw(pid), Signal::SIGTERM) {
                Ok(()) => {
                    println!("Sent termination signal to netpulsed (pid: {pid})");
                    // Optionally: wait to confirm process ended
                    // Could check /proc/{pid} or try kill(0)
                }
                Err(e) => {
                    eprintln!("Failed to terminate netpulsed: {e}");
                }
            }
        }
        None => {
            println!("netpulsed is not running");
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn startd() {
    let path = Store::path();
    let parent_path = path.parent().expect("store file has no parent directory");
    println!("Parent: {parent_path:?}");

    let pid_path = PathBuf::from(DAEMON_PID_FILE);
    let pid_parent_path = pid_path.parent().expect("pid file has no parent directory");
    println!("Pid Parent: {pid_parent_path:?}");

    let logfile = File::create(DAEMON_LOG_INF).expect("could not open info logfile");
    let errfile = File::create(DAEMON_LOG_ERR).expect("could not open error logfile");

    let user = nix::unistd::User::from_name(DAEMON_USER)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        .expect("could not get user for netpulse")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "netpulse user not found"))
        .expect("could not get user for netpulse");

    fs::create_dir_all(parent_path).expect("could not create the store directory");
    fs::create_dir_all(pid_parent_path).expect("could not create the pid directory");
    std::os::unix::fs::chown(
        pid_parent_path,
        Some(user.uid.into()),
        Some(user.gid.into()),
    )
    .expect("could not set permissions for the netpulse run directory");

    let daemonize = Daemonize::new()
        .pid_file(pid_path)
        .chown_pid_file(true)
        .working_directory(parent_path)
        .user("netpulse")
        .group("netpulse")
        .stdout(logfile)
        .stderr(errfile)
        .umask(0o022); // rw-r--r--

    println!("daemon setup done");

    let outcome = daemonize.execute();
    match outcome {
        daemonize::Outcome::Parent(result) => {
            if result.is_ok() {
                println!("netpulsed was started",);
            } else {
                eprintln!("error while starting netpulsed: {}", result.unwrap_err());
            }
        }
        daemonize::Outcome::Child(result) => {
            if result.is_ok() {
                daemon();
            } else {
                panic!("error while starting the daemon: {}", result.unwrap_err())
            }
        }
    }
}
