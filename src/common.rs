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
use std::fmt::{Display, Write};
use std::io::{self, Write as _};
use std::process::Command;
use std::str::FromStr;

use getopts::Options;
use sysinfo::{Pid, System};
use tracing::{debug, error, trace, warn};
use tracing_subscriber::FmtSubscriber;

/// Environment variable name for configuring log level
pub const ENV_LOG_LEVEL: &str = "NETPULSE_LOG_LEVEL";

/// a hacky way to make [Command] [Display]
trait CommandExt {
    fn parts(&self) -> Vec<&str>;
    fn to_string(&self) -> String {
        trace!("using custom to_string function");
        let mut buf: String = String::new();
        let parts = self.parts();
        let len = parts.len();
        for (idx, p) in parts.iter().enumerate() {
            write!(buf, "{p}").expect("could not append to buffer");
            if idx < len - 1 {
                write!(buf, " ").expect("could not append to buffer");
            }
        }
        buf
    }
}

impl CommandExt for Command {
    fn parts(&self) -> Vec<&str> {
        let mut v: Vec<&str> = vec![self
            .get_program()
            .to_str()
            .expect("program was not a proper string?")];
        v.extend(
            self.get_args()
                .map(|a| a.to_str().expect("arg was not a proper string?")),
        );

        v
    }
}

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
    let brief = format!("Usage: {program} [options]");
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
pub fn confirm(message: impl Display) -> bool {
    // Print prompt and flush to ensure it's displayed before reading input
    print!("{message} y/N: ");
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
/// Will ask the user to confirm if a command should be ran.
///
/// # Arguments
///
/// * `cmd` - Command to execute
/// * `skip_checks` - Do not confirm with the user if true
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
/// exec_cmd_for_user(Command::new("systemctl").arg("daemon-reload"), false);
/// ```
pub fn exec_cmd_for_user(cmd: &mut Command, skip_checks: bool) {
    if !skip_checks && !confirm(format!("running cmd: {}", cmd.to_string())) {
        trace!("returning early from exec_cmd_for_user because not confirmed");
        return;
    }
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

/// Get the pid of the running netpulsed daemon
pub fn getpid_running() -> Option<Pid> {
    let pid_of_current_process = std::process::id();
    let s = System::new_all();
    let mut processes: Vec<&sysinfo::Process> = s
        .processes_by_exact_name("netpulsed".as_ref())
        .filter(
            |p| p.thread_kind().is_none(), /* only real processes, not threads */
        )
        .filter(|p| p.pid().as_u32() != pid_of_current_process) // ignore the currently running
        // process
        .collect();

    if processes.is_empty() {
        None
    } else if processes.len() == 1 {
        Some(processes[0].pid())
    } else {
        warn!("netpulsed is running multiple times ({})", processes.len());
        processes.sort_by_key(|a| a.pid());
        debug!(
            "listing netpulsed processes: {:?}",
            processes.iter().map(|p| p.pid()).collect::<Vec<_>>()
        );
        Some(processes[0].pid())
    }
}

/// Sets up a custom panic handler for user-friendly error reporting.
///
/// Should be called early in the program startup, ideally before any other operations.
/// In debug builds, uses the default panic handler for detailed debugging output.
/// In release builds, provides a user-friendly error message with reporting instructions.
///
/// # Example Output
///
/// ```text
/// Well, this is embarrassing.
///
/// netpulse had a problem and crashed. This is a bug and should be reported!
///
/// Technical details:
/// Version:     0.1.0
/// OS:          linux x86_64
/// Command:     netpulse --check
/// Error:       called `Option::unwrap()` on a `None` value
/// Location:    src/store.rs:142
///
/// Please create a new issue at https://github.com/PlexSheep/netpulse/issues
/// with the above technical details and what you were doing when this happened.
/// ```
pub fn setup_panic_handler() {
    if !cfg!(debug_assertions) {
        // Only override in release builds
        std::panic::set_hook(Box::new(|panic_info| {
            let mut message = String::new();
            message.push_str("\nWell, this is embarrassing.\n\n");
            message.push_str(&format!(
                "{} had a problem and crashed. This is a bug and should be reported!\n\n",
                env!("CARGO_PKG_NAME")
            ));

            message.push_str("Technical details:\n");
            message.push_str(&format!("Version:     {}\n", env!("CARGO_PKG_VERSION")));

            // Get OS info
            #[cfg(target_os = "linux")]
            let os = "linux";
            #[cfg(target_os = "macos")]
            let os = "macos";
            #[cfg(target_os = "windows")]
            let os = "windows";

            message.push_str(&format!("OS:          {} {}\n", os, std::env::consts::ARCH));

            // Get command line
            let args: Vec<_> = std::env::args().collect();
            message.push_str(&format!("Command:     {}\n", args.join(" ")));

            // Extract error message and location
            if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
                message.push_str(&format!("Error:       {msg}\n"));
            } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
                message.push_str(&format!("Error:       {msg}\n"));
            }

            if let Some(location) = panic_info.location() {
                message.push_str(&format!(
                    "Location:    {}:{}\n",
                    location.file(),
                    location.line()
                ));
            }

            message.push_str(
                "\nPlease create a new issue at https://github.com/PlexSheep/netpulse/issues\n",
            );
            message.push_str(
                "with the above technical details and what you were doing when this happened.\n",
            );

            eprintln!("{message}");
            std::process::exit(1);
        }));
    }
}
