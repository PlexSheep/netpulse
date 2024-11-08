//! Netpulse is a network monitoring tool that performs connectivity checks and stores results for analysis.
//!
//! # Architecture
//!
//! The crate is organized into several key modules:
//! - [`store`] - Handles persistence of check results
//! - [`records`] - Defines core types for representing checks and their results
//! - [`checks`] - Implements the actual connectivity checks
//! - [`analyze`] - Provides analysis of check results
//! - [`errors`] - Error types
//! - [`analyze`] - Analysis functionalities for extrapolating the data in the [Store](store)
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use netpulse::store::Store;
//!
//! // Load or create store
//! let mut store = Store::load_or_create().unwrap();
//!
//! // Add checks for configured targets
//! store.make_checks();
//!
//! // Save results
//! store.save().unwrap();
//! ```

#![warn(missing_docs)]

/// How long to wait until considering a connection as timed out, in milliseconds
pub const TIMEOUT_MS: u16 = 10_000;
/// How long to wait until considering a connection as timed out
pub const TIMEOUT: std::time::Duration = std::time::Duration::new(TIMEOUT_MS as u64 / 1000, 0);

/// Lockfile of the daemon containing it#s pid
pub const DAEMON_PID_FILE: &str = "/run/netpulse/netpulse.pid";
/// Redirect the stderr of the daemon here
pub const DAEMON_LOG_ERR: &str = "/var/log/netpulse.err";
/// Redirect the stdout of the daemon here
pub const DAEMON_LOG_INF: &str = "/var/log/netpulse.log";
/// username of the user the daemon should drop to after being started
pub const DAEMON_USER: &str = "netpulse";

pub mod analyze;
pub mod checks;
pub mod errors;
pub mod records;
pub mod store;
