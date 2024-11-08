//! The store module handles persistence and versioning of check results.
//!
//! The store is saved to disk at a configurable location (default `/var/lib/netpulse/netpulse.store`).
//! The store format is versioned to allow for future changes while maintaining backwards compatibility
//! with older store files.
//!
//! # Store Location
//!
//! The store location can be configured via:
//! - Environment variable: `NETPULSE_STORE_PATH` (for debugging)
//! - Default path: `/var/lib/netpulse/netpulse.store`
//!
//! # Versioning
//!
//! The store uses a simple version number to track format changes. [Version::CURRENT] is the current version.
//! When loading a store, the version is checked and migration is performed if needed.

use std::fmt::Display;
use std::fs::{self, Permissions};
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::StoreError;
use crate::records::{Check, CheckType, TARGETS_HTTP};
use crate::DAEMON_USER;

#[cfg(feature = "compression")]
use zstd;

/// The filename of the netpulse store database
///
/// Used in combination with [DB_PATH] to form the complete store path.
/// Default value: "netpulse.store"
pub const DB_NAME: &str = "netpulse.store";

/// Base directory for the netpulse store
///
/// Used in combination with [DB_NAME] to form the complete store path.
/// Default value: "/var/lib/netpulse"
pub const DB_PATH: &str = "/var/lib/netpulse";

/// Compression level used when the "compression" feature is enabled
///
/// Higher values provide better compression but slower performance.
/// Default value: 4 (balanced between compression and speed)
#[cfg(feature = "compression")]
pub const ZSTD_COMPRESSION_LEVEL: i32 = 4;

/// Environment variable name for overriding the store path
///
/// If set, its value will be used instead of [DB_PATH] to locate the store.
/// Primarily intended for development and testing.
pub const ENV_PATH: &str = "NETPULSE_STORE_PATH";

/// Version information for the store format.
///
/// The [Store] definition might change over time as netpulse is developed. To work with older or
/// newer [Stores](Store), we need to be able to easily distinguish between versions. The store
/// version is just stored as a [u8].
///
/// See [Version::CURRENT] for the current version and [Version::SUPPROTED] for all store versions
/// supported by this version of Netpulse
///
/// This only describes the version of the [Store], not of [Netpulse](crate) itself.
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Version {
    /// Raw version number as u8
    inner: u8,
}

/// Main storage type for netpulse check results.
///
/// The Store handles persistence of check results and provides methods for
/// loading, saving, and managing the data. It includes versioning support
/// for future format changes.
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Store {
    /// Store format version
    version: Version,
    /// Collection of all recorded checks
    checks: Vec<Check>,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<u8> for Version {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl From<Version> for u8 {
    fn from(value: Version) -> Self {
        value.inner
    }
}

impl Version {
    /// Current version of the store format
    pub const CURRENT: Self = Version::new(0);

    /// List of supported store format versions
    ///
    /// Used for compatibility checking when loading stores.
    pub const SUPPROTED: &[Self] = &[Version::new(0)];

    /// Creates a new Version with the given raw version number
    pub(crate) const fn new(raw: u8) -> Self {
        Self { inner: raw }
    }
}

impl Store {
    /// Returns the full path to the store file.
    ///
    /// The path is determined by:
    /// 1. Checking [ENV_PATH] environment variable
    /// 2. Falling back to [DB_PATH]/[DB_NAME] if not set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netpulse::store::Store;
    ///
    /// let path = Store::path();
    /// println!("Store located at: {}", path.display());
    /// ```
    pub fn path() -> PathBuf {
        if let Some(var) = std::env::var_os(ENV_PATH) {
            let mut p = PathBuf::from(var);
            p.push(DB_NAME);
            p
        } else {
            PathBuf::from(format!("{DB_PATH}/{DB_NAME}"))
        }
    }

    /// Creates a new empty store with current version.
    ///
    /// Used internally by [create](Store::create) when initializing a new store.
    fn new() -> Self {
        Self {
            version: Version::CURRENT,
            checks: Vec::new(),
        }
    }

    /// Creates a new store file on disk.
    ///
    /// # File Creation
    /// - Creates parent directories if needed
    /// - Sets file permissions to 0o644
    /// - Initializes with empty check list
    /// - Optionally compresses data if compression feature is enabled
    ///
    /// # Errors
    ///
    /// Returns [StoreError] if:
    /// - Directory creation fails
    /// - File creation fails
    /// - Serialization fails
    /// - Write fails
    pub fn create() -> Result<Self, StoreError> {
        let path = Self::path();
        let parent_path = path
            .parent()
            .expect("the store path has no parent directory");
        let user = nix::unistd::User::from_name(DAEMON_USER)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .expect("could not get user for netpulse")
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "netpulse user not found")
            })
            .expect("could not get user for netpulse");

        fs::create_dir_all(parent_path)?;
        std::os::unix::fs::chown(parent_path, Some(user.uid.into()), Some(user.gid.into()))?;

        let file = match fs::File::options()
            .read(false)
            .write(true)
            .append(false)
            .create_new(true)
            .mode(0o644)
            .open(path)
        {
            Ok(file) => file,
            Err(err) => return Err(err.into()),
        };

        let store = Store::new();

        #[cfg(feature = "compression")]
        let mut writer = zstd::Encoder::new(file, ZSTD_COMPRESSION_LEVEL)?;
        #[cfg(not(feature = "compression"))]
        let mut writer = file;

        writer.write_all(&bincode::serialize(&store)?)?;
        writer.flush()?;
        Ok(store)
    }

    /// Loads existing store or creates new one if not found.
    ///
    /// This is the recommended way to obtain the [Store] when not just analyzing the contents.
    ///
    /// # Error Handling
    ///
    /// - If store doesn't exist: Creates new one
    /// - If store is corrupt/truncated: Returns error but preserves file
    /// - If version unsupported: Returns error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netpulse::store::Store;
    ///
    /// let mut store = Store::load_or_create().unwrap();
    /// store.make_checks();
    /// store.save().unwrap();
    /// ```
    pub fn load_or_create() -> Result<Self, StoreError> {
        match Self::load() {
            Ok(store) => Ok(store),
            Err(err) => match &err {
                StoreError::DoesNotExist => Self::create(),
                StoreError::Load { source } => {
                    dbg!(source);
                    eprintln!("{err}");

                    #[allow(clippy::single_match)] // more will certainly come later
                    match &(**source) {
                        bincode::ErrorKind::Io(io_err) => match io_err.kind() {
                            ErrorKind::UnexpectedEof => {
                                eprintln!("The file ends too early, might be an old format, cut off, or empty. Not doing anything in case you need to keep old data");
                            }
                            _ => (),
                        },
                        _ => (),
                    }

                    Err(err)
                }
                _ => {
                    eprintln!("Error while trying to load the store: {err:#}");
                    Err(err)
                }
            },
        }
    }

    /// Loads an existing store from disk.
    ///
    /// This is the recommended way to obtain a store instance when the [Store] won't change.
    ///
    /// # Version Handling
    ///
    /// - Checks version compatibility
    /// - Automatically migrates supported old versions in memory
    /// - Returns error for unsupported versions
    ///
    /// # Errors
    ///
    /// Returns [StoreError] if:
    /// - Store file doesn't exist
    /// - Read/parse fails
    /// - Version unsupported
    pub fn load() -> Result<Self, StoreError> {
        let file = match fs::File::options()
            .read(true)
            .write(false)
            .create_new(false)
            .open(Self::path())
        {
            Ok(file) => file,
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => return Err(StoreError::DoesNotExist),
                    ErrorKind::PermissionDenied => eprintln!("Not allowed to access store"),
                    _ => (),
                };

                return Err(err.into());
            }
        };

        #[cfg(feature = "compression")]
        let reader = zstd::Decoder::new(file)?;
        #[cfg(not(feature = "compression"))]
        let mut reader = file;

        let mut store: Store = bincode::deserialize_from(reader)?;

        // TODO: somehow account for old versions that are not compatible with the store struct
        if store.version != Version::CURRENT {
            eprintln!("The store that was loaded is not of the current version:\nstore has {} but the current version is {}", store.version, Version::CURRENT);
            if Version::SUPPROTED.contains(&store.version) {
                eprintln!("The old store version is still supported, migrating to newer version");
                store.version = Version::CURRENT;
            } else {
                eprintln!("The store version is not supported");
                return Err(StoreError::UnsupportedVersion);
            }
        }
        Ok(store)
    }

    /// Saves the store to disk.
    ///
    /// # File Handling
    ///
    /// - Truncates existing file
    /// - Optionally compresses if feature enabled
    /// - Maintains original permissions
    ///
    /// # Errors
    ///
    /// Returns [StoreError] if:
    /// - File doesn't exist
    /// - Write fails
    /// - Serialization fails
    pub fn save(&self) -> Result<(), StoreError> {
        let file = match fs::File::options()
            .read(false)
            .write(true)
            .append(false)
            .create_new(false)
            .truncate(true)
            .create(false)
            .open(Self::path())
        {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return Err(StoreError::DoesNotExist),
                _ => return Err(err.into()),
            },
        };

        #[cfg(feature = "compression")]
        let mut writer = zstd::Encoder::new(file, ZSTD_COMPRESSION_LEVEL)?;
        #[cfg(not(feature = "compression"))]
        let mut writer = file;

        writer.write_all(&bincode::serialize(&self)?)?;
        writer.flush()?;
        Ok(())
    }

    /// Adds a new check to the store.
    pub fn add_check(&mut self, check: impl Into<Check>) {
        self.checks.push(check.into());
    }

    /// Returns a reference to the checks of this [`Store`].
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }

    /// Returns the check interval in seconds.
    ///
    /// This determines how frequently the daemon performs checks.
    /// Currently fixed at 60 seconds.
    pub const fn period_seconds(&self) -> u64 {
        60
    }

    /// Generates a hash of the in-memory store data.
    ///
    /// Uses [DefaultHasher](std::hash::DefaultHasher) to create a 16-character hexadecimal hash
    /// of the entire store contents. Useful for detecting changes.
    pub fn display_hash(&self) -> String {
        let mut hasher = std::hash::DefaultHasher::default();
        self.hash(&mut hasher);
        format!("{:016X}", hasher.finish())
    }

    /// Generates SHA-256 hash of the store file on disk.
    ///
    /// This calls `sha256sum` on the store file.
    ///
    /// # External Dependencies
    ///
    /// Requires `sha256sum` command to be available in PATH.
    ///
    /// # Errors
    ///
    /// Returns [StoreError] if:
    /// - sha256sum command fails
    /// - Output parsing fails
    pub fn display_hash_of_file(&self) -> Result<String, StoreError> {
        let out = Command::new("sha256sum").arg(Self::path()).output()?;

        if !out.status.success() {
            eprintln!(
                "error while making the hash over the store file:\nStdout\n{:?}\n\nStdin\n{:?}",
                out.stdout, out.stderr
            );
            return Err(StoreError::ProcessEndedWithoutSuccess);
        }

        Ok(std::str::from_utf8(&out.stdout)?
            .split(" ")
            .collect::<Vec<&str>>()[0]
            .to_string())
    }

    /// Creates and adds checks for all configured targets.
    ///
    /// Iterates through [TARGETS_HTTP] and creates an HTTP check
    /// for each target IP address.
    ///
    /// Only HTTP checks are done for now, as ICMP needs `CAP_NET_RAW` and DNS is not yet
    /// implemented.
    pub fn make_checks(&mut self) {
        for target in TARGETS_HTTP {
            self.checks.push(
                CheckType::Http.make(
                    std::net::IpAddr::from_str(target)
                        .expect("a target constant was not an Ip Address"),
                ),
            );
        }
    }
}
