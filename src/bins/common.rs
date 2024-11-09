use getopts::Options;
use netpulse::DAEMON_PID_FILE;
use tracing::{error, info, trace};
use tracing_subscriber::FmtSubscriber;

pub const ENV_LOG_LEVEL: &str = "NETPULSE_LOG_LEVEL";

#[allow(dead_code)] // idk why it says thet, netpulsed uses it a few times
pub(crate) fn root_guard() {
    if !nix::unistd::getuid().is_root() {
        eprintln!("This needs to be run as root");
        std::process::exit(1)
    }
}

pub(crate) fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
    std::process::exit(0)
}

pub(crate) fn print_version() -> ! {
    println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
    std::process::exit(0)
}

pub(crate) fn init_logging(level: tracing::Level) {
    let level: tracing::Level = match std::env::var(ENV_LOG_LEVEL) {
        Err(_) => level,
        Ok(raw) => match tracing::Level::from_str(&raw) {
            Err(e) => {
                eprintln!("Bad log level was given with the environment variable '{ENV_LOG_LEVEL}': '{raw}', must be one of 'TRACE', 'DEBUG', 'INFO', 'WARN', 'ERROR'");
                eprintln!("{e}");
                std::process::exit(1)
            }
            Ok(ll) => ll,
        },
    };

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(level)
        // No need for the time. It's either ran with systemd (which shows the time in journalctl)
        // or it's the reader which doesn't need it.
        .without_time()
        // would show the module where the thing comes from
        .with_target(false)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    trace!("logging initialized with level {level}");
}

use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;

/// Prompts the user for confirmation with a custom message.
/// Returns true if the user confirms, false otherwise.
///
/// Accepts various forms of input:
/// - Yes: "y", "Y", "yes", "Yes", "YES"
/// - No: "n", "N", "no", "No", "NO", "" (empty input), literally anything else than yes
///
/// # Arguments
/// * `message` - The message to display before " y/N: "
///
/// # Examples
/// ```
/// if confirm("Delete all files") {
///     println!("Deleting...");
/// } else {
///     println!("Operation cancelled");
/// }
/// ```
#[allow(dead_code)] // idk why it says thet, netpulsed uses it a few times
pub(crate) fn confirm(message: &str) -> bool {
    // Print prompt and flush to ensure it's displayed before reading input
    print!("{} y/N: ", message);
    io::stdout().flush().unwrap();

    // Read user input
    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        error!("could not read from stdin: {e}");
        return false;
    }

    // Trim whitespace and convert to lowercase for flexible matching
    let input = input.trim().to_lowercase();

    // Check for various forms of "yes"
    matches!(input.as_str(), "y" | "yes")
}

#[allow(dead_code)] // idk why it says thet, netpulsed uses it a few times
pub(crate) fn exec_cmd_for_user(cmd: &mut Command) {
    info!("running cmd: {cmd:?}");
    let out = match cmd.output() {
        Err(e) => {
            error!("{e}");
            std::process::exit(1)
        }
        Ok(o) => o,
    };
    if !out.status.success() {
        let info = String::from_utf8_lossy(&out.stdout);
        let err = String::from_utf8_lossy(&out.stderr);
        error!("command failed: {cmd:?}\nSTDERR:\n{err}\nSTDIN:\n{info}");
        std::process::exit(1)
    }
}

/// Return the PID of netpulsed if it runs
#[allow(dead_code)] // idk why it says thet, netpulsed uses it a few times
pub(crate) fn netpulsed_is_running() -> Option<i32> {
    getpid().filter(|p| pid_runs(*p))
}

/// Check if a process with `pid` exists
pub(crate) fn pid_runs(pid: i32) -> bool {
    std::fs::exists(format!("/proc/{pid}")).expect("could not check if the process exists")
}

/// Ger the netpulsed pid from the pidfile
#[allow(dead_code)] // idk why it says thet, netpulsed uses it a few times
pub(crate) fn getpid() -> Option<i32> {
    if !std::fs::exists(DAEMON_PID_FILE).expect("couldn't check if the pid file exists") {
        None
    } else {
        let pid_raw = std::fs::read_to_string(DAEMON_PID_FILE)
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
