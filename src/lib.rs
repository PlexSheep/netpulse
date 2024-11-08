/// How long to wait until considering a connection as timed out, in milliseconds
pub const TIMEOUT_MS: u16 = 10_000;
/// How long to wait until considering a connection as timed out
pub const TIMEOUT: std::time::Duration = std::time::Duration::new(TIMEOUT_MS as u64 / 1000, 0);

pub const DAEMON_PID_FILE: &str = "/run/netpulse/netpulse.pid";
pub const DAEMON_LOG_ERR: &str = "/var/log/netpulse.err";
pub const DAEMON_LOG_INF: &str = "/var/log/netpulse.log";
pub const DAEMON_USER: &str = "netpulse";

/// where the actual checks are made
pub mod checks;
/// error types
pub mod errors;
/// check records that are put in the store, and working with them
pub mod records;
/// the store contains all info, is written and loaded to and from the disk
pub mod store;
