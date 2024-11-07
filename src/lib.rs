/// How long to wait until considering a connection as timed out, in milliseconds
pub const TIMEOUT_MS: u16 = 30_000;
/// How long to wait until considering a connection as timed out
pub const TIMEOUT: std::time::Duration = std::time::Duration::new(TIMEOUT_MS as u64 / 1000, 0);

pub const DAEMON_PID_FILE: &str = "/run/netpulse/netpulse.pid";
pub const DAEMON_LOG_ERR: &str = "/var/log/netpulse.err";
pub const DAEMON_LOG_INF: &str = "/var/log/netpulse.log";
pub const DAEMON_USER: &str = "netpulse";

pub mod errors;
pub mod records;
pub mod store;
