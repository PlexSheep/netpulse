//! Daemon control binary for starting/stopping the netpulse daemon.
//!
//! This binary manages the netpulse daemon process:
//! - Starting the daemon with proper privileges
//! - Stopping running daemon instances
//! - Checking daemon status
//!
//! # Usage
//!
//! Use the `--help` flag for more information about the usage.
//!
//! # Privileges
//!
//! The daemon requires root to start but drops privileges to run as the netpulse user.
//! Note that ICMP checks require `CAP_NET_RAW` capability which is lost on privilege drop.
//!
//! # Files
//!
//! - PID file: `/var/run/netpulse/netpulsed.pid`
//! - Info log: `/var/log/netpulse/info.log`
//! - Error log: `/var/log/netpulse/error.log`

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use getopts::Options;
use netpulse::common::{
    confirm, exec_cmd_for_user, getpid_running, init_logging, print_usage, root_guard,
    setup_panic_handler,
};
use netpulse::errors::RunError;
use netpulse::store::Store;
use netpulse::{DAEMON_PID_FILE, DAEMON_USER};
use nix::errno::Errno;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use sysinfo::System;
use tracing::{debug, error, info, trace};

mod daemon;
use daemon::daemon;

const SERVICE_FILE: &str = include_str!("../../data/netpulsed.service");
const SYSTEMD_SERVICE_PATH: &str = "/etc/systemd/system/netpulsed.service";

/// Whether the executable is being executed as a daemon by a framework like systemd
///
/// `true` => yes, something like systemd is taking care of things like stdout and pidfile
/// `false` => no, we're doing it all manually
static USES_DAEMON_SYSTEM: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), RunError> {
    setup_panic_handler();
    init_logging(tracing::Level::INFO);
    let args: Vec<String> = std::env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("V", "version", "print the version");
    opts.optflag(
        "u",
        "setup",
        "setup the directories and so on needed for netpulse, including a systemd service (netpulsed.service)",
    );
    opts.optflag(
        "d",
        "daemon",
        "run directly as the daemon, do not setup a pidfile or drop privileges, for use when using a daemonizing system like systemd",
    );
    opts.optflag("i", "info", "info about the running netpulse daemon");
    opts.optflag("e", "end", "stop the running netpulse daemon");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{f}");
            print_usage(program, opts);
        }
    };

    if matches.opt_present("help") {
        print_usage(program, opts);
    } else if matches.opt_present("version") {
        print_version()
    } else if matches.opt_present("info") {
        infod();
    } else if matches.opt_present("setup") {
        root_guard();
        if let Err(e) = setup_systemd(false) {
            error!("While making the systemd setup: {e}");
            std::process::exit(1)
        }
        if let Err(e) = Store::setup() {
            error!("While making the store setup: {e}");
            std::process::exit(1)
        }
    } else if matches.opt_present("end") {
        endd();
    } else if matches.opt_present("daemon") {
        USES_DAEMON_SYSTEM.store(true, std::sync::atomic::Ordering::Release);
        daemon();
    } else {
        print_usage(program, opts);
    }
    Ok(())
}

fn setup_general(skip_checks: bool) -> Result<(), RunError> {
    debug!("starting general setup");
    if !skip_checks && !confirm("Perform general daemon setup?") {
        debug!("general setup skipped");
        return Ok(());
    }

    // create netpulse user if it does not exist
    if !nix::unistd::User::from_name(DAEMON_USER).is_ok_and(|o| o.is_some()) {
        if skip_checks || confirm("create netpulse user?") {
            trace!("trying to create a new user with useradd");
            exec_cmd_for_user(
                Command::new("useradd")
                    .arg("--system")
                    .arg("--shell")
                    .arg("/sbin/nologin")
                    .arg(DAEMON_USER),
                skip_checks,
            );
        } else {
            info!("user {DAEMON_USER} exists")
        }
    }

    // copying netpulsed to /usr/local/bin/
    let current_exe = std::env::current_exe()?;
    let target_path = format!("/usr/local/bin/{}", env!("CARGO_BIN_NAME"));
    info!(
        "copying the netpulsed executable from '{:?}' to '{target_path}'",
        current_exe
    );
    fs::copy(current_exe, target_path)?;

    Ok(())
}

fn setup_systemd(skip_checks: bool) -> Result<(), RunError> {
    if let Some(pid) = getpid_running() {
        let s = System::new_all();
        info!("daemon runs with pid {pid}");
        let process = s
            .process(pid)
            .expect("process for the pid of the daemon not found");
        if !skip_checks || !confirm("terminate the daemon now?") {
            println!("stopping setup");
            std::process::exit(0);
        }
        process
            .kill_with(sysinfo::Signal::Term)
            .expect("SIGTERM does not exist on this platform");
        process.wait(); // wait until the daemon has stopped
    }

    setup_general(skip_checks)?;

    // Create service file path
    let service_path = Path::new(SYSTEMD_SERVICE_PATH);

    // Create parent directories if they don't exist
    if let Some(parent) = service_path.parent() {
        info!("creating parent dir of systemd service {parent:?}");
        fs::create_dir_all(parent)?;
    }

    // Write service file
    info!("creating the systemd service");
    let mut file = fs::File::create(service_path)?;
    file.write_all(SERVICE_FILE.as_bytes())?;

    // Set permissions to 644 (rw-r--r--)
    info!("setting permissions for the systemd service");
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(service_path, perms)?;

    info!("Created the netpulsed.service in '{SYSTEMD_SERVICE_PATH}'.");
    println!("To update the reload the daemon definitions, run the following as root:");
    println!("  systemctl daemon-reload");
    println!("To enable and start the service, run the following as root:");
    println!("  systemctl enable netpulsed.service --now");
    println!("To just start the service once, run the following as root:");
    println!("  systemctl start netpulsed.service --now");
    println!();
    if !confirm("Reload, enable and start netpulsed.service now?") {
        return Ok(());
    }

    exec_cmd_for_user(Command::new("systemctl").arg("daemon-reload"), true);
    exec_cmd_for_user(
        Command::new("systemctl")
            .arg("enable")
            .arg("netpulsed.service"),
        true,
    );
    exec_cmd_for_user(
        Command::new("systemctl")
            .arg("restart")
            .arg("netpulsed.service"),
        true,
    );

    Ok(())
}

fn infod() {
    match getpid_running() {
        Some(pid) => {
            println!("netpulsed is running with pid {pid}")
        }
        None => println!("netpulsed is not running"),
    }
}

fn pid_runs(pid: i32) -> bool {
    fs::exists(format!("/proc/{pid}")).expect("could not check if the process exists")
}

fn endd() {
    root_guard();
    let mut terminated = false;
    let pid: Pid = match getpid_running() {
        None => {
            println!("netpulsed is not running");
            return;
        }
        Some(raw) => Pid::from_raw(raw.as_u32() as i32), // this is weird, sysinfo has another raw type
                                                         // for pids than nix
    };

    match signal::kill(pid, Signal::SIGTERM) {
        Ok(()) => {
            println!("Sent termination signal to netpulsed (pid: {pid})");
            // Optionally: wait to confirm process ended
            // Could check /proc/{pid} or try kill(0)
        }
        Err(e) => {
            match e {
                // no such process
                Errno::ESRCH => {
                    terminated = true;
                }
                _ => {
                    eprintln!("Failed to terminate netpulsed: {e}");
                    std::process::exit(1)
                }
            }
        }
    }

    let sent_sig = std::time::Instant::now();
    while !terminated && sent_sig.elapsed().as_secs() < 5 {
        if pid_runs(pid.as_raw()) {
            std::thread::sleep(std::time::Duration::from_millis(20));
        } else {
            terminated = true
        }
    }
    if !terminated {
        println!("netpulsed (pid {pid}) is taking too long to terminate, killing it",);
        match signal::kill(pid, Signal::SIGKILL) {
            Ok(()) => {
                println!("Sent kill signal to netpulsed (pid: {pid})");
            }
            Err(e) => {
                eprintln!("Failed to kill netpulsed: {e}");
            }
        }
    }
    if fs::exists(DAEMON_PID_FILE).expect("could not check if the pid file exists") {
        eprintln!("The pid file ({DAEMON_PID_FILE}) still exists even though the daemon is not running, removing it");
        if let Err(err) = fs::remove_file(DAEMON_PID_FILE) {
            eprintln!("Could not remove the pid file: {err}")
        }
    }
}

fn print_version() -> ! {
    println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
    std::process::exit(0)
}
