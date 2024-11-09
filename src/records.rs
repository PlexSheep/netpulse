//! Core types for representing network connectivity checks and their results.
//!
//! This module defines the fundamental types used throughout netpulse:
//! - [`Check`] - Result of a single connectivity check
//! - [`CheckType`] - Different types of checks (HTTP, ICMP, DNS)
//! - [`CheckFlag`] - Flags indicating check status and metadata
//!
//! # Check Types
//!
//! The following check types are supported:
//! - HTTP(S) - Web connectivity checks
//! - ICMPv4/v6 - Ping checks
//! - DNS - Domain name resolution (planned)
//!
//! # Check Flags
//!
//! Checks use a bitflag system to track:
//! - Success/failure status
//! - Failure reasons (timeout, unreachable)
//! - Protocol used (IPv4/IPv6)
//! - Check type (HTTP, ICMP, DNS)
//!
//! This system may be expanded in future versions
//!
//! # Example
//!
//! ```rust
//! use netpulse::records::{CheckType, Check};
//!
//! // Create new HTTP check
//! let check = CheckType::Http.make("1.1.1.1".parse().unwrap());
//!
//! // Access check results
//! if check.is_success() {
//!     println!("Latency: {}ms", check.latency().unwrap());
//! }
//! ```

use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::time::{self};

use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};

use crate::errors::StoreError;

/// List of target IP addresses used for connectivity checks.
///
/// # Warning
///
/// Only add valid IP addresses to this list. Invalid addresses will cause panics
/// when parsed.
pub const TARGETS: &[&str] = &["1.1.1.1", "2606:4700:4700::1111", "127.0.0.1"];
/// Subset of [TARGETS] used specifically for HTTP checks.
pub const TARGETS_HTTP: &[&str] = &[TARGETS[0], TARGETS[1]];

flags! {
    /// Flags describing the status and type of a check.
    ///
    /// Uses a bitflag system to efficiently store multiple properties:
    /// - Result flags (bits 0-7): Success, failure reasons
    /// - Protocol flags (bits 8-11): IPv4/IPv6
    /// - Type flags (bits 12-15): Check type (HTTP, ICMP, DNS)
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
        /// Check type was ICMP (ping)
        ///
        /// Must be combined with either [IPv4](CheckFlag::IPv4) or [IPv6](CheckFlag::IPv6)
        /// to determine the specific ICMP version used
        TypeIcmp    =   0b0100_0000_0000_0000,
        /// The Check used DNS
        TypeDns     =   0b1000_0000_0000_0000,
    }
}

/// Types of network connectivity checks supported by netpulse.
///
/// This enum represents the different kinds of checks that can be performed.
/// Each variant corresponds to a specific protocol or method of testing connectivity.
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Clone, Copy)]
pub enum CheckType {
    /// DNS resolution check (not yet implemented)
    Dns,
    /// HTTP/HTTPS connectivity check
    Http,
    /// ICMP ping using IPv4
    IcmpV4,
    /// ICMP ping using IPv6
    IcmpV6,
    /// Unknown or invalid check type
    Unknown,
}
impl CheckType {
    /// Creates and performs a new network check of this type.
    ///
    /// # Arguments
    ///
    /// * `remote` - Target IP address to check
    ///
    /// # Returns
    ///
    /// Returns a [Check] instance containing the results.
    ///
    /// # Feature Requirements
    ///
    /// - HTTP checks require the `http` feature
    /// - ICMP checks require the `ping` feature
    ///
    /// # Panics
    ///
    /// - If HTTP check is attempted without `http` feature
    /// - If ICMP check is attempted without `ping` feature
    /// - If check type is `Unknown`
    /// - If check type is `Dns` (not yet implemented)
    pub fn make(&self, remote: IpAddr) -> Check {
        let mut check = Check::new(
            std::time::SystemTime::now(),
            FlagSet::default(),
            None,
            remote,
        );

        match remote {
            IpAddr::V4(_) => check.add_flag(CheckFlag::IPv4),
            IpAddr::V6(_) => check.add_flag(CheckFlag::IPv6),
        }

        match self {
            #[cfg(feature = "http")]
            Self::Http => {
                check.add_flag(CheckFlag::TypeHTTP);
                match crate::checks::check_http(remote) {
                    Err(err) => {
                        eprintln!("unknown error when performing a Http check: {err}")
                    }
                    Ok(lat) => {
                        check.add_flag(CheckFlag::Success);
                        check.latency = Some(lat);
                    }
                }
            }
            #[cfg(not(feature = "http"))]
            Self::Http => {
                panic!("Trying to make a http check, but the http feature is not enabled")
            }

            #[cfg(feature = "ping")]
            Self::IcmpV4 => {
                check.add_flag(CheckFlag::TypeIcmp);
                match crate::checks::just_fucking_ping(remote) {
                    Err(err) => {
                        eprintln!("unknown error when performing a ICMPv4 (ping) check: {err}")
                    }
                    Ok(lat) => {
                        check.add_flag(CheckFlag::Success);
                        check.latency = Some(lat);
                    }
                }
            }
            #[cfg(not(feature = "ping"))]
            Self::IcmpV4 => {
                panic!("Trying to make a ICMPv4 check, but the ping feature is not enabled")
            }
            #[cfg(feature = "ping")]
            Self::IcmpV6 => {
                check.add_flag(CheckFlag::TypeIcmp);
                match crate::checks::just_fucking_ping(remote) {
                    Err(err) => {
                        eprintln!("unknown error when performing a ICMPv6 (ping) check: {err}")
                    }
                    Ok(lat) => {
                        check.add_flag(CheckFlag::Success);
                        check.latency = Some(lat);
                    }
                }
            }
            #[cfg(not(feature = "ping"))]
            Self::IcmpV6 => {
                panic!("Trying to make a ICMPv6 check, but the ping feature is not enabled")
            }
            Self::Unknown => {
                panic!("tried to make an Unknown check");
            }
            Self::Dns => {
                todo!("dns not done yet")
            }
        }

        check
    }

    /// Returns a slice containing all possible check types.
    ///
    /// Used for iterating over available check types, e.g., during analysis.
    pub const fn all() -> &'static [Self] {
        &[Self::Dns, Self::Http, Self::IcmpV4, Self::IcmpV6]
    }

    /// Returns a slice of check types enabled by default.
    ///
    /// Currently only includes HTTP checks because ICMP requires special
    /// privileges (CAP_NET_RAW) which are lost when the daemon drops privileges, and DNS is not
    /// implemented.
    pub const fn default_enabled() -> &'static [Self] {
        &[Self::Http, Self::IcmpV4, Self::IcmpV6]
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

/// Result of a single network connectivity check.
///
/// Contains all information about a check attempt including:
/// - When it was performed
/// - What type of check it was
/// - Whether it succeeded
/// - Measured latency (if successful)
/// - Target address
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Clone, Copy)]
pub struct Check {
    /// Unix timestamp when check was performed (seconds since UNIX_EPOCH)
    timestamp: u64,
    /// Flags describing check type and result
    ///
    /// Stored as a bitset where each bit represents a [CheckFlag]
    flags: FlagSet<CheckFlag>,
    /// Round-trip latency in milliseconds if check succeeded
    ///
    /// Only present if check succeeded and must be less than
    /// [TIMEOUT_MS](crate::TIMEOUT_MS)
    latency: Option<u16>,
    /// Target IP address that was checked
    target: IpAddr,
}

impl Check {
    /// Generates a hash of the in-memory [Check] data.
    ///
    /// Uses [DefaultHasher](std::hash::DefaultHasher) to create a 16-character hexadecimal hash
    /// of the [Check] that can be used to identify this [Check]. Useful for detecting changes.
    pub fn get_hash(&self) -> String {
        let mut hasher = std::hash::DefaultHasher::default();
        self.hash(&mut hasher);
        format!("{:016X}", hasher.finish())
    }

    /// Creates a new check result with the specified properties.
    ///
    /// This does not execute a check and then store the information about that check in this
    /// datastructure, it simply allows the creation of arbitrary check results
    ///
    /// # Arguments
    ///
    /// * `time` - When the check was performed
    /// * `flags` - Initial status flags
    /// * `latency` - Measured latency (if successful)
    /// * `target` - Target IP address
    ///
    /// # Panics
    ///
    /// Panics if timestamp is before UNIX_EPOCH.
    pub fn new(
        time: time::SystemTime,
        flags: impl Into<FlagSet<CheckFlag>>,
        latency: Option<u16>,
        target: IpAddr,
    ) -> Self {
        Check {
            timestamp: time
                .duration_since(time::UNIX_EPOCH)
                .expect("timestamp of check was before UNIX_EPOCH (1970-01-01 00:00:00 UTC)")
                .as_secs(),
            flags: flags.into(),
            latency,
            target,
        }
    }

    /// Returns whether this check was successful.
    ///
    /// A check is considered successful if it has the [Success](CheckFlag::Success) flag
    /// and no unexpected flag combinations.
    pub fn is_success(&self) -> bool {
        self.flags.contains(CheckFlag::Success)
    }

    /// Returns the measured latency if check was successful.
    ///
    /// Returns None if:
    /// - Check failed
    /// - Check succeeded but no latency was recorded
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

    /// Returns the timestamp of this [`Check`] as [SystemTime](std::time::SystemTime).
    pub fn timestamp_parsed(&self) -> time::SystemTime {
        time::UNIX_EPOCH + time::Duration::from_secs(self.timestamp())
    }

    /// Returns a mutable reference to the flags of this [`Check`].
    pub fn flags_mut(&mut self) -> &mut FlagSet<CheckFlag> {
        &mut self.flags
    }

    /// Add the given flag to the flags of this [Check]
    pub fn add_flag(&mut self, flag: CheckFlag) {
        self.flags |= flag
    }

    /// Determines [CheckType] from this checks flags.
    ///
    /// Examines the type and protocol flags to determine the specific
    /// kind of check that was performed.
    ///
    /// Returns [CheckType::Unknown] if flags indicate an invalid combination.
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

    /// Updates the target IP address of this check.
    pub fn set_target(&mut self, target: IpAddr) {
        self.target = target;
    }

    /// Determines whether the check used IPv4 or IPv6.
    ///
    /// Examines the check's flags to determine which IP version was used.
    /// A check should have either IPv4 or IPv6 flag set, but not both.
    ///
    /// # Returns
    ///
    /// * `CheckFlag::IPv4` - Check used IPv4
    /// * `CheckFlag::IPv6` - Check used IPv6
    ///
    /// # Errors
    ///
    /// * Returns [`StoreError::AmbiguousFlags`] if both IPv4 and IPv6 flags are set,
    ///   as this represents an invalid state that should never occur.
    ///
    /// * Returns [`StoreError::MissingFlag`] if neither IPv4 or IPv6 flags are set,
    ///   as this represents an invalid state that should never occur.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netpulse::records::{Check, CheckFlag};
    /// use flagset::FlagSet;
    ///
    /// let mut check = Check::new(std::time::SystemTime::now(), FlagSet::default(), None, "1.1.1.1".parse().unwrap());
    ///
    /// assert!(check.ip_type().is_err()); // we haven't set the IP flags! We need to set either IPv4 or IPv6
    ///
    /// check.add_flag(CheckFlag::IPv4);
    ///
    /// match check.ip_type().unwrap() {
    ///     CheckFlag::IPv4 => println!("IPv4 check"),
    ///     CheckFlag::IPv6 => println!("IPv6 check"),
    ///     _ => unreachable!()
    /// }
    ///
    /// check.add_flag(CheckFlag::IPv6); // But what if we now also add IPv6?
    ///
    /// assert!(check.ip_type().is_err()); // Oh no! Now it's ambiguos
    /// ```
    pub fn ip_type(&self) -> Result<CheckFlag, StoreError> {
        let flags = self.flags();
        if flags.contains(CheckFlag::IPv4) && flags.contains(CheckFlag::IPv6) {
            Err(StoreError::AmbiguousFlags(
                CheckFlag::IPv4 | CheckFlag::IPv6,
            ))
        } else if flags.contains(CheckFlag::IPv4) {
            Ok(CheckFlag::IPv4)
        } else if flags.contains(CheckFlag::IPv6) {
            Ok(CheckFlag::IPv6)
        } else {
            Err(StoreError::MissingFlag(CheckFlag::IPv4 | CheckFlag::IPv6))
        }
    }
}

impl Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Time: {}\nType: {}\nOk: {}\nTarget: {}",
            humantime::format_rfc3339_seconds(self.timestamp_parsed()),
            self.calc_type(),
            self.is_success(),
            self.target
        )?;
        write!(f, "Latency: {}", {
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
            "127.0.0.1".parse().unwrap(),
        );
        // if it can be created, that's good enough for me, I'm just worried that I'll change the
        // timeout ms some day and this will break
    }
}
