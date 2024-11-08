use std::net::IpAddr;

use crate::errors::CheckError;
use crate::TIMEOUT;

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
