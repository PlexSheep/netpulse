//! Make the actual checks
//!
//! All of these functions perform a check with an [IpAddr] and return a [Result<u16, CheckError>].
//! The [u16] is the latency in milliseconds.
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
    easy.perform()?;

    Ok(start.elapsed().as_millis() as u16)
}
