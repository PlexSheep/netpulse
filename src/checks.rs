//! Implementation of network connectivity checks.
//!
//! This module contains the actual check implementations for different protocols:
//! - HTTP checks via HEAD requests
//! - ICMP checks via ping
//! - DNS checks (planned)
//!
//! All check functions follow the pattern:
//! - Take a target IP address
//! - Perform the check with timeout
//! - Return latency on success or error on failure
//!
//! # Feature Flags
//!
//! Check types can be enabled/disabled via feature flags:
//! - `http` - Enable HTTP checks
//! - `ping` - Enable ICMP checks
//!
//! # Example
//!
//! ```rust
//! use netpulse::checks;
//! use std::net::IpAddr;
//!
//! let addr: IpAddr = "1.1.1.1".parse().unwrap();
//!
//! // Perform HTTP check
//! if let Ok(latency) = checks::check_http(addr) {
//!     println!("HTTP latency: {}ms", latency);
//! }
//! ```
use std::net::IpAddr;

use crate::errors::CheckError;
use crate::TIMEOUT;

/// Performs an ICMP ping check to the specified IP address.
///
/// Uses raw sockets to send ICMP echo request and measure round-trip time.
/// This function requires the `ping` feature to be enabled.
///
/// # Required Capabilities
///
/// This function requires the `CAP_NET_RAW` capability to create and use raw sockets for ICMP.
/// Without this capability, the function will fail with a permission error.
///
/// **Note**: When running as a daemon, this capability is typically lost when dropping privileges
/// from root to the daemon user. As a result, ICMP checks may not work in daemon mode.
///
/// # Arguments
///
/// * `remote` - Target IP address to ping (IPv4 or IPv6)
///
/// # Returns
///
/// * `Ok(u16)` - Round-trip time in milliseconds if ping succeeds
/// * `Err(CheckError)` - If ping fails (timeout, network error, etc)
///
/// # Errors
///
/// Returns `CheckError` if:
/// - Raw socket creation fails (typically due to missing CAP_NET_RAW)
/// - Ping times out ([`TIMEOUT`])
/// - Network is unreachable
/// - Permission denied
///
/// # Examples
///
/// ```rust,no_run
/// use std::net::IpAddr;
/// use netpulse::checks::just_fucking_ping;
///
/// let addr: IpAddr = "1.1.1.1".parse().unwrap();
/// match just_fucking_ping(addr) {
///     Ok(latency) => println!("Ping latency: {}ms", latency),
///     Err(e) => eprintln!("Ping failed: {}", e),
/// }
/// ```
#[cfg(feature = "ping")]
pub fn just_fucking_ping(remote: IpAddr) -> Result<u16, CheckError> {
    let now = std::time::Instant::now();
    match ping::rawsock::ping(remote, Some(TIMEOUT), None, None, None, None) {
        Ok(_) => Ok(now.elapsed().as_millis() as u16),
        Err(e) => Err(e.into()),
    }
}

/// Performs an HTTP HEAD request to check connectivity to the specified IP address.
///
/// Makes an HTTP/HTTPS HEAD request to measure response time. Uses curl under the hood
/// and requires the `http` feature to be enabled.
///
/// # Arguments
///
/// * `remote` - Target IP address for HTTP check (IPv4 or IPv6)
///
/// # Returns
///
/// * `Ok(u16)` - Round-trip time in milliseconds if request succeeds
/// * `Err(CheckError)` - If request fails (timeout, connection refused, etc)
///
/// # Errors
///
/// Returns `CheckError` if:
/// - DNS resolution fails
/// - Connection fails or is refused
/// - Request times out ([`TIMEOUT`])
/// - HTTP response indicates error
/// - URL construction fails
///
/// # IPv6 Handling
///
/// When checking IPv6 addresses, the address is wrapped in square brackets
/// to form a valid URL (e.g. `http://[2606:4700:4700::1111]`).
///
/// # Examples
///
/// ```rust
/// use std::net::IpAddr;
/// use netpulse::checks::check_http;
///
/// let addr: IpAddr = "1.1.1.1".parse().unwrap();
/// match check_http(addr) {
///     Ok(latency) => println!("HTTP latency: {}ms", latency),
///     Err(e) => eprintln!("HTTP check failed: {}", e),
/// }
/// ```
#[cfg(feature = "http")]
pub fn check_http(remote: IpAddr) -> Result<u16, CheckError> {
    let start = std::time::Instant::now();
    let mut easy = curl::easy::Easy::new();

    easy.url(&match remote {
        IpAddr::V4(_) => remote.to_string(),
        IpAddr::V6(_) => format!("[{remote}]"),
    })?;
    easy.nobody(true)?; // HEAD request only
    easy.timeout(TIMEOUT)?;
    easy.perform()?;

    Ok(start.elapsed().as_millis() as u16)
}
