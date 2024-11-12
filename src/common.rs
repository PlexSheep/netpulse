//! Common functionality shared between netpulse binaries.
//!
//! This module provides shared utilities used by both the netpulse reader
//! and netpulsed daemon binaries, including:
//! - Privilege checks
//! - Logging setup
//! - PID file management
//! - Process management
//! - User interaction
//!
//! # Exits
//!
//! Some functions in this module exit when a condition is not met, printing an error then.
//!
//! # Logging
//!
//! Logging can be configured via the `NETPULSE_LOG_LEVEL` environment variable.
//! Valid levels are: TRACE, DEBUG, INFO, WARN, ERROR
//!
//! # Examples
//!
//! ```rust,no_run
//! use netpulse::common;
//!
//! // Check for root privileges
//! common::root_guard();
//!
//! // Initialize logging
//! common::init_logging(tracing::Level::INFO);
//!
//! // Check if daemon is running
//! if let Some(pid) = common::getpid_running() {
//!     println!("Daemon running with PID: {}", pid);
//! }
//! ```
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;

use crate::DAEMON_PID_FILE;

use chrono::{DateTime, Local};
use getopts::Options;
use tracing::{error, info, trace};
use tracing_subscriber::FmtSubscriber;

/// Environment variable name for configuring log level
pub const ENV_LOG_LEVEL: &str = "NETPULSE_LOG_LEVEL";
/// Formatting rules for timestamps that are easily readable by humans.
///
/// ```rust
/// use chrono::{DateTime, Local};
/// # use netpulse::common::TIME_FORMAT_HUMANS;
/// let datetime: DateTime<Local> = Local::now();
/// println!("it is now: {}", datetime.format(TIME_FORMAT_HUMANS));
/// ```
pub const TIME_FORMAT_HUMANS: &str = "%Y-%m-%d %H:%M:%S %Z";

/// Ensures the program is running with root privileges.
///
/// # Exits
///
/// Exits the program with status code 1 if not running as root.
pub fn root_guard() {
    if !nix::unistd::getuid().is_root() {
        eprintln!("This needs to be run as root");
        std::process::exit(1)
    }
}

/// Displays program usage information and exits.
///
/// Formats and prints the usage information using the provided program name
/// and options configuration.
///
/// # Arguments
///
/// * `program` - Name of the program to show in usage
/// * `opts` - Configured program options
///
/// # Exits
///
/// Always exits with status code 0 after displaying usage.
pub fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
    std::process::exit(0)
}

/// Initializes the logging system with the specified level.
///
/// The log level can be overridden by setting the [ENV_LOG_LEVEL] environment variable.
/// Logging is configured without timestamps (relies on systemd/journald for timing)
/// and without module targets for cleaner output.
///
/// # Arguments
///
/// * `level` - Default log level if not overridden by environment
///
/// # Exits
///
/// Exits with status code 1 if:
/// - Invalid log level specified in environment variable
/// - Failed to set up logging system
pub fn init_logging(level: tracing::Level) {
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
/// use netpulse::common::confirm;
/// if confirm("Delete all files") {
///     println!("Deleting...");
/// } else {
///     println!("Operation cancelled");
/// }
/// ```
pub fn confirm(message: &str) -> bool {
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

/// Executes a command and handles errors and output.
///
/// # Arguments
///
/// * `cmd` - Command to execute
///
/// # Exits
///
/// Exits with status code 1 if:
/// - Command fails to execute
/// - Command returns non-zero status
///
/// # Logging
///
/// - Logs command execution at INFO level
/// - Logs errors at ERROR level including stdout/stderr
///
/// # Examples
///
/// ```rust,no_run
/// use std::process::Command;
/// use netpulse::common::exec_cmd_for_user;
/// exec_cmd_for_user(Command::new("systemctl").arg("daemon-reload"));
/// ```
pub fn exec_cmd_for_user(cmd: &mut Command) {
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

/// Checks if the netpulse daemon is currently running.
///
/// # Returns
///
/// * `Some(pid)` - Daemon is running with the given PID
/// * `None` - Daemon is not running
///
/// Checks both PID file existence and process existence.
pub fn getpid_running() -> Option<i32> {
    getpid().filter(|p| pid_runs(*p))
}

/// Checks if a process with the given PID exists.
///
/// # Arguments
///
/// * `pid` - Process ID to check
///
/// # Returns
///
/// * `true` if process exists
/// * `false` if process does not exist
///
/// # Panics
///
/// Panics if unable to check process existence (e.g., permission denied)
pub fn pid_runs(pid: i32) -> bool {
    std::fs::exists(format!("/proc/{pid}")).expect("could not check if the process exists")
}

/// Reads the daemon's PID from its PID file.
///
/// # Returns
///
/// * `Some(pid)` - Successfully read PID from file
/// * `None` - If PID file doesn't exist or contains invalid data
///
/// # Panics
///
/// Panics if:
/// - Unable to check PID file existence
/// - PID file exists but can't be read
pub fn getpid() -> Option<i32> {
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

/// Formats a [SystemTime](std::time::SystemTime) as an easily readable timestamp for humans.
///
/// Works with [`std::time::SystemTime`] and [`chrono::DateTime<Local>`].
///
/// # Examples
///
/// ```rust
/// # use netpulse::common::fmt_timestamp;
/// use std::time::SystemTime;
/// use chrono;
/// let datetime: SystemTime = SystemTime::now();
/// println!("it is now: {}", fmt_timestamp(datetime));
/// let datetime: chrono::DateTime<chrono::Local> = chrono::Local::now();
/// println!("it is now: {}", fmt_timestamp(datetime));
/// let datetime: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
/// println!("it is now: {}", fmt_timestamp(datetime));
/// ```
pub fn fmt_timestamp(timestamp: impl Into<DateTime<Local>>) -> String {
    let a: chrono::DateTime<chrono::Local> = timestamp.into();
    format!("{}", a.format(TIME_FORMAT_HUMANS))
}
