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
//! # #[cfg(feature = "http")] {// only works with that feature
//! use netpulse::records::{CheckType, Check};
//!
//! // Create new HTTP check
//! let check = CheckType::Http.make("1.1.1.1".parse().unwrap());
//!
//! // Access check results
//! if check.is_success() {
//!     println!("Latency: {}ms", check.latency().unwrap());
//! }
//! # }
//! ```

use std::fmt::{Display, Write};
use std::hash::Hash;
use std::net::IpAddr;

use chrono::{DateTime, Local, TimeZone, Utc};
use deepsize::DeepSizeOf;
use flagset::{flags, FlagSet};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::analyze::fmt_timestamp;
use crate::errors::StoreError;
use crate::store::Version;

/// Type of [IpAddr]
///
/// This enum can be used to work with just abstract IP versions, not whole [Ip Addresses](IpAddr).
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Clone, Copy, DeepSizeOf)]
pub enum IpType {
    /// Type is IPv4
    V4,
    /// Type is IPv6
    V6,
}

/// List of target IP addresses used for connectivity checks.
///
/// # Warning
///
/// Only add valid IP addresses to this list. Invalid addresses will cause panics
/// when parsed.
pub const TARGETS: &[&str] = &["1.1.1.1", "2606:4700:4700::1111"];

flags! {
    /// Flags describing the status and type of a check.
    ///
    /// Uses a bitflag system to efficiently store multiple properties:
    /// - Result flags (bits 0-7): Success, failure reasons
    /// - Type flags (bits 8-15): Check type (HTTP, ICMP, DNS)
    #[derive(Hash, Deserialize, Serialize)]
    pub enum CheckFlag: u16 {
        /// If this is not set, the check will be considered failed
        Success     =   0b0000_0000_0000_0001,
        /// Failure because of a timeout
        Timeout     =   0b0000_0000_0000_0010,
        /// Failure because the destination is unreachable
        Unreachable =   0b0000_0000_0000_0100,

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
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Clone, Copy, DeepSizeOf)]
pub enum CheckType {
    /// DNS resolution check (not yet implemented)
    Dns,
    /// HTTP/HTTPS connectivity check
    Http,
    /// ICMP ping (Echo)
    Icmp,
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
        let mut check = Check::new(Utc::now(), FlagSet::default(), None, remote);

        match self {
            #[cfg(feature = "http")]
            Self::Http => {
                check.add_flag(CheckFlag::TypeHTTP);
                match crate::checks::check_http(remote) {
                    Err(err) => {
                        error!("error while performing an Http check: {err}")
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
            Self::Icmp => {
                check.add_flag(CheckFlag::TypeIcmp);
                match crate::checks::just_fucking_ping(remote) {
                    Err(err) => {
                        error!("error while performing an ICMPv4 check: {err}")
                    }
                    Ok(lat) => {
                        check.add_flag(CheckFlag::Success);
                        check.latency = Some(lat);
                    }
                }
            }
            #[cfg(not(feature = "ping"))]
            Self::Icmp => {
                panic!("Trying to make a ICMPv4 check, but the ping feature is not enabled")
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
        &[Self::Dns, Self::Http, Self::Icmp]
    }

    /// Returns a slice of check types enabled by default.
    ///
    /// Currently only includes HTTP checks because ICMP requires special
    /// privileges (CAP_NET_RAW) which are lost when the daemon drops privileges, and DNS is not
    /// implemented.
    pub const fn default_enabled() -> &'static [Self] {
        &[Self::Http, Self::Icmp]
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
                Self::Icmp => "ICMP",
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
    timestamp: i64,
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

impl DeepSizeOf for Check {
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.latency.deep_size_of_children(context)
    }
}

impl Check {
    /// Generates a cryptographic hash of the [Check] data.
    ///
    /// Uses [blake3] for consistent hashing across Rust versions and platforms.
    /// The hash remains stable as long as the check's contents don't change,
    /// making it suitable for persistent identification of checks.
    ///
    /// # Implementation Details
    ///
    /// - Uses [bincode] for serialization of check data
    /// - Uses [blake3] for cryptographic hashing
    /// - Produces a 32-byte (256-bit) hash
    ///
    /// # Panics
    ///
    /// May panic if serialization fails, which can happen in extreme cases:
    /// - System is out of memory
    /// - System is in a severely degraded state
    ///
    /// Normal [Check] data will always serialize successfully.
    pub fn get_hash(&self) -> blake3::Hash {
        blake3::hash(&bincode::serialize(&self).expect("serialization of a check failed"))
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
        time: impl Into<DateTime<Utc>>,
        flags: impl Into<FlagSet<CheckFlag>>,
        latency: Option<u16>,
        target: IpAddr,
    ) -> Self {
        let t: DateTime<Utc> = time.into();
        Check {
            timestamp: t.timestamp(),
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
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timestamp of this [`Check`] as [SystemTime](std::time::SystemTime).
    ///
    /// The [`Check`] structure stores just seconds since UNIX_EPOCH, which is agnostic of
    /// timezones. The seconds since the UNIX_EPOCH (1970-01-01 00:00) are converted to a timestamp
    /// in UTC, and just for the formatting the timestamp is converted to the timezone of the user.
    pub fn timestamp_parsed(&self) -> chrono::DateTime<Local> {
        let t: DateTime<Local> = Local.timestamp_opt(self.timestamp(), 0).unwrap();
        t
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
    pub fn calc_type(&self) -> Result<CheckType, StoreError> {
        Ok(if self.flags.contains(CheckFlag::TypeHTTP) {
            CheckType::Http
        } else if self.flags.contains(CheckFlag::TypeDns) {
            CheckType::Dns
        } else if self.flags.contains(CheckFlag::TypeIcmp) {
            CheckType::Icmp
        } else {
            CheckType::Unknown
        })
    }

    /// Updates the target IP address of this check.
    pub fn set_target(&mut self, target: IpAddr) {
        self.target = target;
    }

    /// Determines whether the check used IPv4 or IPv6.
    ///
    /// Examines the [check's](Check) [target](Check::target()) to determine which IP version was used.
    ///
    /// # Returns
    ///
    /// The [IpType] that was used
    pub fn ip_type(&self) -> IpType {
        IpType::from(self.target)
    }

    /// Migrate a [Check] to the next [Version] that follows `current`
    pub fn migrate(&mut self, current: Version) -> Result<(), StoreError> {
        match current {
            Version::V0 => (),
            Version::V1 => self.timestamp = i64::from_ne_bytes(self.timestamp.to_ne_bytes()), // was originally u64
            _ => unimplemented!("migrating from Version {current} is not yet imlpemented"),
        }
        Ok(())
    }

    /// Returns the target of this [`Check`].
    pub fn target(&self) -> IpAddr {
        self.target
    }
}

impl Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Time: {}\nType: {}\nOk: {}\nTarget: {}\nLatency: {}\nHash: {}",
            fmt_timestamp(self.timestamp_parsed()),
            self.calc_type().unwrap_or(CheckType::Unknown),
            self.is_success(),
            self.target,
            match self.latency() {
                Some(l) => format!("{l} ms"),
                None => "(Error)".to_string(),
            },
            self.get_hash()
        )
    }
}

impl From<IpAddr> for IpType {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(_) => Self::V4,
            IpAddr::V6(_) => Self::V6,
        }
    }
}

/// Display a formatted list of checks.
///
/// Each check is formatted with:
/// - Index number
/// - Indented check details
/// - Nested line breaks preserved
///
/// # Arguments
///
/// * `group` - Slice of check references to format
/// * `f` - String buffer to write formatted output
///
/// # Errors
///
/// Returns [`std::fmt::Error`] if string formatting fails.
pub fn display_group(group: &[&Check], f: &mut String) -> Result<(), std::fmt::Error> {
    if group.is_empty() {
        writeln!(f, "\t<Empty>")?;
        return Ok(());
    }
    for (cidx, check) in group.iter().enumerate() {
        writeln!(f, "{cidx}:")?;
        writeln!(f, "\t{}", check.to_string().replace("\n", "\n\t"))?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::TIMEOUT_MS;
    use std::time; // no need to change it, since the api can work with both std and chrono now

    use super::*;

    #[test]
    fn test_creating_check() {
        let _c = Check::new(
            time::SystemTime::now(),
            CheckFlag::Success,
            Some(TIMEOUT_MS),
            "127.0.0.1".parse().unwrap(),
        );
        // if it can be created, that's good enough for me, I'm just worried that I'll change the
        // timeout ms some day and this will break
    }

    #[test]
    fn test_check_size_of_check() {
        let c = Check::new(
            time::SystemTime::now(),
            CheckFlag::Success,
            Some(TIMEOUT_MS),
            "127.0.0.1".parse().unwrap(),
        );
        assert_eq!(
            c.deep_size_of(),
            std::mem::size_of::<IpAddr>() // self.target
            + std::mem::size_of::<i64>() // self.timestamp
            + std::mem::size_of::<u16>() // self.flags
            +3 /* latency */ + 2 // latency padding?
        );
        let c1 = Check::new(
            time::SystemTime::now(),
            CheckFlag::Timeout,
            None,
            "127.0.0.1".parse().unwrap(),
        );
        assert_eq!(
            c1.deep_size_of(),
            std::mem::size_of::<IpAddr>() // self.target
            + std::mem::size_of::<i64>() // self.timestamp
            + std::mem::size_of::<u16>() // self.flags
            +3 /* latency */ + 2 // latency padding?
        );
        let c2 = Check::new(
            time::SystemTime::now(),
            CheckFlag::Timeout,
            None,
            "::1".parse().unwrap(),
        );
        assert_eq!(
            c2.deep_size_of(),
            std::mem::size_of::<IpAddr>() // self.target
            + std::mem::size_of::<i64>() // self.timestamp
            + std::mem::size_of::<u16>() // self.flags
            +3 /* latency */ + 2 // latency padding?
        )
    }
}
