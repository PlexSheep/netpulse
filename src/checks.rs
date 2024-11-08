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
//! let addr: IpAddr = "1.1.1.1".parse()?;
//!
//! // Perform HTTP check
//! if let Ok(latency) = checks::check_http(addr) {
//!     println!("HTTP latency: {}ms", latency);
//! }
//! ```
use std::net::IpAddr;

use crate::errors::CheckError;
use crate::TIMEOUT;

#[cfg(feature = "ping")]
pub fn just_fucking_ping(remote: IpAddr) -> Result<u16, CheckError> {
    let now = std::time::Instant::now();
    match ping::rawsock::ping(remote, Some(TIMEOUT), None, None, None, None) {
        Ok(_) => Ok(now.elapsed().as_millis() as u16),
        Err(e) => {
            eprintln!("Error while makeing the ping check: {e}");
            Err(e.into())
        }
    }
}

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
