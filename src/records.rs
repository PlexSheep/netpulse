use std::fmt::Display;
use std::net::IpAddr;
use std::str::FromStr;
use std::time;

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};

pub const TARGETS: &[&str] = &["1.1.1.1", "2606:4700:4700::1111", "127.0.0.1"];

flags! {
    #[derive(Hash, Deserialize, Serialize)]
    pub enum CheckFlag: u16 {
        /// If this is not set, the check will be considered failed
        Success     =   0b0000_0000_0000_0001,
        /// Failure because of a timeout
        Timeout     =   0b0000_0000_0000_0010,
        /// Failure because the destination is unreachable
        Unreachable =   0b0000_0000_0000_0100,

        /// The Check used IPv4
        IPv4        =   0b0000_0001_0000_0000,
        /// The Check used IPv6
        IPv6        =   0b0000_0010_0000_0000,

        /// The Check used HTTP/HTTPS
        TypeHTTP    =   0b0001_0000_0000_0000,
        /// The Check used Ping/ICMP v4/v6
        ///
        /// Depends of the IPv6/IPv4 flags to determine if it's ICMPv4 or ICMPv6
        TypeIcmp    =   0b0100_0000_0000_0000,
        /// The Check used DNS
        TypeDns     =   0b1000_0000_0000_0000,
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Clone, Copy)]
pub enum CheckType {
    Dns,
    Http,
    IcmpV4,
    IcmpV6,
    Unknown,
}
impl CheckType {
    /// Make a new [Check] of this type.
    ///
    /// This is the actual thing that carries out the checking
    pub fn make(&self) -> Check {
        let mut check = Check::new(std::time::SystemTime::now(), FlagSet::default(), None, 0);

        match self {
            Self::IcmpV4 => {
                const TARGET_IDX: usize = 0;
                check.add_flag(CheckFlag::IPv4);
                check.add_flag(CheckFlag::TypeIcmp);
                check.set_target(TARGET_IDX);
                match crate::ping::just_fucking_ping(IpAddr::from_str(TARGETS[TARGET_IDX]).unwrap())
                {
                    Err(err) => {
                        eprintln!("unknown error when performing a ICMPv4 (ping) check: {err}")
                    }
                    Ok(lat) => {
                        check.add_flag(CheckFlag::Success);
                        check.latency = Some(lat);
                    }
                }
            }
            Self::IcmpV6 => {
                const TARGET_IDX: usize = 1;
                check.add_flag(CheckFlag::IPv6);
                check.add_flag(CheckFlag::TypeIcmp);
                check.set_target(TARGET_IDX);
                match crate::ping::just_fucking_ping(IpAddr::from_str(TARGETS[TARGET_IDX]).unwrap())
                {
                    Err(err) => {
                        eprintln!("unknown error when performing a ICMPv6 (ping) check: {err}")
                    }
                    Ok(lat) => {
                        check.add_flag(CheckFlag::Success);
                        check.latency = Some(lat);
                    }
                }
            }
            _ => {
                eprintln!("skipping unimplemented check")
            }
        }

        check
    }

    /// Get all variants of this enum.
    pub const fn all() -> &'static [Self] {
        &[Self::Dns, Self::Http, Self::IcmpV4, Self::IcmpV6]
    }

    /// Get all default enabled variants of this enum.
    ///
    /// You may want to use more check types, but these are the ones commonly used. The ICMP types
    /// are removed from this, because they require CAP_NET_ADD, which the daemon does not
    /// keep when dropping to the user priviledges.
    pub const fn default_enabled() -> &'static [Self] {
        &[Self::Http]
    }
}

impl Display for CheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Dns => "DNS",
                Self::Http => "HTTP(S)",
                Self::IcmpV4 => "ICMPv4",
                Self::IcmpV6 => "ICMPv6",
                Self::Unknown => "Unknown",
            }
        )
    }
}

/// Information about connectivity
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Clone, Copy)]
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
    /// Index of the remote, based on [TARGETS]
    target: usize,
}

impl Check {
    /// Create a new [Check], and fill it with arbitrary data
    pub fn new(
        time: time::SystemTime,
        flags: impl Into<FlagSet<CheckFlag>>,
        latency: Option<u16>,
        target_idx: usize,
    ) -> Self {
        Check {
            timestamp: time
                .duration_since(time::UNIX_EPOCH)
                .expect("timestamp of check was before UNIX_EPOCH (1970-01-01 00:00:00 UTC)")
                .as_secs(),
            flags: flags.into(),
            latency,
            target: target_idx,
        }
    }

    /// Checks if this [`Check`] is ok.
    ///
    /// Ok means, it's a [Success](CheckFlag::Success), and has no weird anomalies (that this
    /// checks for).
    pub fn is_success(&self) -> bool {
        self.flags.contains(CheckFlag::Success)
    }

    /// Returns the latency of this [`Check`].
    pub fn latency(&self) -> Option<u16> {
        if !self.is_success() {
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

    /// Returns a mutable reference to the flags of this [`Check`].
    pub fn flags_mut(&mut self) -> &mut FlagSet<CheckFlag> {
        &mut self.flags
    }

    /// Add the given flag to the flags of this [Check]
    pub fn add_flag(&mut self, flag: CheckFlag) {
        self.flags |= flag
    }

    /// Check the flags and infer the [CheckType]
    pub fn calc_type(&self) -> CheckType {
        if self.flags.contains(CheckFlag::TypeHTTP) {
            CheckType::Http
        } else if self.flags.contains(CheckFlag::TypeDns) {
            CheckType::Dns
        } else if self.flags.contains(CheckFlag::TypeIcmp) {
            if self.flags.contains(CheckFlag::IPv4) {
                CheckType::IcmpV4
            } else if self.flags.contains(CheckFlag::IPv6) {
                CheckType::IcmpV6
            } else {
                eprintln!("flag for ICMP is set, but not if ipv4 or ipv6 was used");
                CheckType::Unknown
            }
        } else {
            CheckType::Unknown
        }
    }

    pub fn set_target(&mut self, target: usize) {
        self.target = target;
    }
}

impl Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Type: {}\nOk: {}", self.calc_type(), self.is_success())?;
        writeln!(f, "Latency: {}", {
            match self.latency() {
                Some(l) => format!("{l} ms"),
                None => "(Error)".to_string(),
            }
        })
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
            CheckFlag::Success,
            Some(TIMEOUT_MS),
            0,
        );
        // if it can be created, that's good enough for me, I'm just worried that I'll change the
        // timeout ms some day and this will break
    }
}
