use std::time;

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};

use crate::errors::CheckFlagTypeConversionError;

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Default)]
pub struct Store {
    checks: Vec<Check>,
}

flags! {
    #[derive(Hash, Deserialize, Serialize)]
    pub enum CheckFlag: u16 {
        NoFlags     =   0b0000_0000_0000_0000,
        /// If this is not set, the check will be considered failed
        Success     =   0b0000_0000_0000_0001,
        /// Failure because of a timeout
        Timeout     =   0b0000_0000_0000_0010,
        /// Failure because the destination is unreachable
        Unreachable =   0b0000_0000_0000_0100,

        /// The Check used HTTP/HTTPS
        TypeHTTP    =   0b0010_0000_0000_0000,
        /// The Check used Ping/ICMP
        TypePing    =   0b0100_0000_0000_0000,
        /// The Check used DNS
        TypeDns     =   0b1000_0000_0000_0000,
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum CheckType {
    Dns,
    Http,
    Ping,
}

impl From<CheckType> for CheckFlag {
    fn from(value: CheckType) -> Self {
        match value {
            CheckType::Dns => CheckFlag::TypeDns,
            CheckType::Http => CheckFlag::TypeHTTP,
            CheckType::Ping => CheckFlag::TypePing,
        }
    }
}

impl TryFrom<CheckFlag> for CheckType {
    type Error = CheckFlagTypeConversionError;

    fn try_from(value: CheckFlag) -> Result<Self, Self::Error> {
        Ok(match value {
            CheckFlag::TypeDns => Self::Dns,
            CheckFlag::TypeHTTP => Self::Http,
            CheckFlag::TypePing => Self::Ping,
            _ => return Err(CheckFlagTypeConversionError),
        })
    }
}

/// Information about connectivity
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Check {
    /// Unix timestamp
    timestamp: u64,
    /// Describes how the [Check] went.
    ///
    /// This will be encoded as a [u16], where each bit signifies if a [CheckFlag](CheckFlags) applies to the [Check].
    flags: FlagSet<CheckFlag>,
    /// If [CheckFlags::Success], this will be the latency of the connection that was made.
    ///
    /// This needs to be big enough, that the latency will always be less. Because of that,
    /// netpulse will only wait for [TIMEOUT_MS](crate::TIMEOUT_MS) milliseconds until deciding
    /// that a connection has timed out.
    latency: Option<u16>,
}

impl Check {
    /// Create a new [Check], and fill it with arbitrary data
    pub fn new(
        time: time::SystemTime,
        flags: impl Into<FlagSet<CheckFlag>>,
        latency: Option<u16>,
    ) -> Self {
        Check {
            timestamp: time
                .duration_since(time::UNIX_EPOCH)
                .expect("timestamp of check was before UNIX_EPOCH (1970-01-01 00:00:00 UTC)")
                .as_secs(),
            flags: flags.into(),
            latency,
        }
    }

    /// Checks if this [`Check`] is ok.
    ///
    /// Ok means, it's a [Success](CheckFlag::Success), and has no weird anomalies (that this
    /// checks for).
    pub fn is_ok(&self) -> bool {
        self.flags.contains(CheckFlag::Success)
    }

    /// Returns the latency of this [`Check`].
    pub fn latency(&self) -> Option<u16> {
        if !self.is_ok() {
            None
        } else {
            self.latency
        }
    }

    /// Returns the flags of this [`Check`].
    pub fn flags(&self) -> FlagSet<CheckFlag> {
        self.flags
    }

    /// Returns the timestamp of this [`Check`].
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

#[cfg(test)]
mod test {
    use crate::TIMEOUT_MS;

    use super::*;

    #[test]
    fn test_max_time_fits_in_latency_field() {
        let _c = Check::new(
            time::SystemTime::now(),
            CheckFlag::NoFlags,
            Some(TIMEOUT_MS),
        );
        // if it can be created, that's good enough for me, I'm just worried that I'll change the
        // timeout ms some day and this will break
    }
}
