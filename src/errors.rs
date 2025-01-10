//! Error types for the netpulse crate.
//!
//! This module provides specialized error types for different components of netpulse:
//! - [`StoreError`] - Errors related to store operations (loading, saving, versioning)
//! - [`CheckError`] - Errors that occur during network checks (HTTP, ICMP)
//! - [`RunError`] - Errors specific to executable operations
//! - [`AnalysisError`] - Errors that occur during analysis and report generation
//!
//! All error types implement the standard Error trait and provide detailed error information.
//!
//! # Examples
//!
//! ```rust,no_run
//! use netpulse::store::Store;
//! use netpulse::errors::StoreError;
//!
//! fn load_store() -> Result<Store, StoreError> {
//!     match Store::load(false) {
//!         Ok(store) => Ok(store),
//!         Err(StoreError::DoesNotExist) => Store::create(),
//!         Err(e) => Err(e),
//!     }
//! }
//! ```

use flagset::FlagSet;
use thiserror::Error;

use crate::records::CheckFlag;

/// Errors that can occur during store operations.
///
/// These errors handle various failure modes when interacting with the store:
/// - File system operations
/// - Data serialization/deserialization
/// - Version compatibility
#[derive(Error, Debug)]
pub enum StoreError {
    /// The store file does not exist.
    ///
    /// This typically occurs on first run or if the store file was deleted.
    #[error("The store does not exist")]
    DoesNotExist,
    /// An I/O error occurred during store operations.
    ///
    /// This can happen during file reading, writing, or filesystem operations.
    #[error("IO Error: {source}")]
    Io {
        /// Underlying error
        #[from]
        source: std::io::Error,
    },
    /// Failed to load store data from file.
    ///
    /// This typically indicates corruption or an incompatible / outdated store format.
    #[error("Could not deserialize the store from the loaded data: {source}")]
    Load {
        /// Underlying error
        #[from]
        source: bincode::Error,
    },
    /// Failed to convert data to UTF-8.
    ///
    /// This can occur when reading store metadata like file hashes.
    #[error("Could not convert data to Utf8")]
    Str {
        /// Underlying error
        #[from]
        source: std::str::Utf8Error,
    },
    /// A subprocess (like sha256sum) exited with non-zero status.
    #[error("A subprocess ended non successfully")]
    ProcessEndedWithoutSuccess,
    /// Attempted to load a store with an unsupported version number.
    ///
    /// This occurs when the store file version is newer or older than what this version
    /// of netpulse supports.
    #[error("Tried to load a store with an unsupported version")]
    UnsupportedVersion,
    /// A [Check](crate::records::Check) has flags that are exclusive to each other.
    ///
    /// This variant contains a [FlagSet] with only the conflicting [CheckFlags](CheckFlag) set.
    #[error("Check has ambiguous flags: {0:?}")]
    AmbiguousFlags(FlagSet<CheckFlag>),
    /// A [Check](crate::records::Check) that does not have the required flags to be valid.
    ///
    /// This variant contains a [FlagSet] with only the flags [CheckFlags](CheckFlag) set that
    /// would make it a valid state. Exactly one of these flags must be set.
    #[error("Check is missing at least one of these flags: {0:?}")]
    MissingFlag(FlagSet<CheckFlag>),
    /// Occurs when trying to convert an arbitrary [u8] to a [Version](crate::store::Version) that
    /// is not defined. Only known [Versions][crate::store::Version] are valid.
    #[error("Tried to load a store version that does not exist: {0}")]
    BadStoreVersion(u8),
    /// A store can be loaded as readonly if it's corrupted or there is a version mismatch
    #[error("Tried to save a readonly store")]
    IsReadonly,
}

/// Errors that can occur during network checks.
///
/// These errors handle failures during the actual network connectivity tests,
/// whether HTTP, ICMP, or other protocols.
#[derive(Error, Debug)]
pub enum CheckError {
    /// An I/O error occurred during the check.
    ///
    /// This typically indicates network-level failures.
    #[error("IO Error {source}")]
    Io {
        /// Underlying error
        #[from]
        source: std::io::Error,
    },
    /// An error occurred during ICMP ping.
    ///
    /// This variant is only available when the `ping` feature is enabled.
    #[cfg(feature = "ping")]
    #[error("Ping Error: {source}")]
    Ping {
        /// Underlying error
        #[from]
        source: ping::Error,
    },
    /// An error occurred during HTTP check.
    ///
    /// This variant is only available when the `http` feature is enabled.
    #[cfg(feature = "http")]
    #[error("Http Error: {source}")]
    Http {
        /// Underlying error
        #[from]
        source: curl::Error,
    },
}

/// Errors that can occur during daemon operations.
///
/// These errors handle failures in the daemon process, including store
/// operations and process management.
#[derive(Error, Debug)]
pub enum RunError {
    /// An error occurred while operating on the store.
    #[error("Something went wrong with the store: {source}")]
    StoreError {
        /// Underlying error
        #[from]
        source: StoreError,
    },
    /// An I/O error occurred during daemon operations.
    #[error("IO Error: {source}")]
    Io {
        /// Underlying error
        #[from]
        source: std::io::Error,
    },
    /// Failed to format analysis output.
    #[error("Text Formatting error: {source}")]
    Fmt {
        /// Underlying error
        #[from]
        source: std::fmt::Error,
    },
}

/// Errors that can occur during analysis and report generation.
///
/// These errors handle failures when analyzing check results and
/// generating human-readable reports.
#[derive(Error, Debug)]
pub enum AnalysisError {
    /// An error occurred while accessing the store.
    #[error("Something went wrong with the store: {source}")]
    StoreError {
        /// Underlying error
        #[from]
        source: StoreError,
    },
    /// Failed to format analysis output.
    #[error("Text Formatting error: {source}")]
    Fmt {
        /// Underlying error
        #[from]
        source: std::fmt::Error,
    },
    /// An I/O error occurred during analysis operations.
    #[error("IO Error: {source}")]
    Io {
        /// Underlying error
        #[from]
        source: std::io::Error,
    },
    #[cfg(feature = "graph")]
    #[cfg_attr(feature = "graph", error("error while drawing the graph"))]
    GraphDraw {
        reason: String, // plotters error type use generics, and that's just a pain
    },
    #[error("analysis was requested, but an empty list of checks was given")]
    NoChecksToAnalyze,
}
