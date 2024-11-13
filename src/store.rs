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
use std::fs::{self};
use std::hash::Hash;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use deepsize::DeepSizeOf;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::fmt::writer::MutexGuardWriter;

use crate::errors::StoreError;
use crate::records::{Check, CheckType, TARGETS};
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

/// How long to wait between running workloads for the daemon
pub const DEFAULT_PERIOD: i64 = 60;
/// Environment variable name for the time period after which the daemon wakes up.
///
/// If set, its value will be used instead of [DEFAULT_PERIOD].
/// Primarily intended for development and testing.
pub const ENV_PERIOD: &str = "NETPULSE_PERIOD";

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
#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    Copy,
    Clone,
    DeepSizeOf,
    PartialOrd,
    Ord,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[allow(missing_docs)] // It's just versions man
#[repr(u8)]
pub enum Version {
    V0 = 0,
    V1 = 1,
    V2 = 2,
}

/// Main storage type for netpulse check results.
///
/// The Store handles persistence of check results and provides methods for
/// loading, saving, and managing the data. It includes versioning support
/// for future format changes.
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize, DeepSizeOf)]
pub struct Store {
    /// Store format version
    version: Version,
    /// Collection of all recorded checks
    checks: Vec<Check>,
    // if true, this store will never be saved
    #[serde(skip)]
    readonly: bool,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw())
    }
}

impl TryFrom<u8> for Version {
    type Error = StoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::V0,
            1 => Self::V1,
            2 => Self::V2,
            _ => return Err(StoreError::BadStoreVersion(value)),
        })
    }
}

impl From<Version> for u8 {
    fn from(value: Version) -> Self {
        value.raw()
    }
}

impl Version {
    /// Current version of the store format
    pub const CURRENT: Self = Self::V2;

    /// List of supported store format versions
    ///
    /// Used for compatibility checking when loading stores.
    pub const SUPPROTED: &[Self] = &[Self::V0, Self::V1, Self::V2];

    /// Gets the raw [Version] as [u8]
    pub const fn raw(&self) -> u8 {
        *self as u8
    }

    /// Returns the next sequential [Version], if one exists.
    ///
    /// Used for version migration logic to determine the next version to upgrade to.
    ///
    /// # Returns
    ///
    /// * `Some(Version)` - The next version in sequence:
    ///   - V0 → V1
    ///   - V1 → V2
    ///   - ...
    /// * `None` - If current version is the latest version
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use netpulse::store::Version;
    /// assert_eq!(Version::V0.next(), Some(Version::V1));
    /// assert_eq!(Version::V1.next(), Some(Version::V2));
    /// assert_eq!(Version::CURRENT.next(), None);  // No version after latest
    /// ```
    pub fn next(&self) -> Option<Self> {
        Some(match *self {
            Self::V0 => Self::V1,
            Self::V1 => Self::V2,
            Self::V2 => return None,
        })
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
            readonly: false,
        }
    }

    /// Sets up the store directory with proper permissions.
    ///
    /// This function must be called with root privileges before starting the daemon. It:
    /// 1. Creates the store directory if it doesn't exist
    /// 2. Sets ownership of the directory to the netpulse daemon user
    ///
    /// # Privilege Requirements
    ///
    /// This function requires root privileges because it:
    /// - Creates directories in system locations (`/var/lib/netpulse`)
    /// - Changes ownership of directories to the daemon user
    ///
    /// # Workflow
    ///
    /// The typical usage flow is:
    /// 1. Call `Store::setup()` as root during daemon initialization
    /// 2. Drop privileges to other user user
    /// 3. Use [`Store::load_or_create`], [`Store::create()`] or [`Store::load()`] as lower priviledged user
    ///
    /// # Errors
    ///
    /// Returns [StoreError] if:
    /// - Directory creation fails
    /// - Ownership change fails
    /// - Netpulse user doesn't exist in the system
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Store path has no parent directory
    /// - Unable to query system for netpulse user
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netpulse::store::Store;
    ///
    /// // Must run as root
    /// Store::setup().unwrap();
    ///
    /// // Now can drop privileges to netpulse user
    /// // and continue with normal store operations
    /// let store = Store::load_or_create().unwrap();
    /// ```
    pub fn setup() -> Result<(), StoreError> {
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
        std::os::unix::fs::chown(parent_path, Some(user.uid.into()), Some(user.gid.into()))
            .inspect_err(|e| {
                error!("could not set owner of store directory to the daemon user: {e}")
            })?;
        Ok(())
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
        let file = match fs::File::options()
            .read(false)
            .write(true)
            .append(false)
            .create_new(true)
            .mode(0o644)
            .open(Self::path())
        {
            Ok(file) => file,
            Err(err) => {
                error!("opening the store file for writing failed: {err}");
                return Err(err.into());
            }
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
        match Self::load(false) {
            Ok(store) => Ok(store),
            Err(err) => match &err {
                StoreError::DoesNotExist => Self::create(),
                StoreError::Load { source } => {
                    dbg!(source);
                    error!("{err}");

                    #[allow(clippy::single_match)] // more will certainly come later
                    match &(**source) {
                        bincode::ErrorKind::Io(io_err) => match io_err.kind() {
                            ErrorKind::UnexpectedEof => {
                                error!("The file ends too early, might be an old format, cut off, or empty. Not doing anything in case you need to keep old data");
                            }
                            _ => (),
                        },
                        _ => (),
                    }

                    Err(err)
                }
                _ => {
                    error!("Error while trying to load the store: {err:#}");
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
    pub fn load(readonly: bool) -> Result<Self, StoreError> {
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
                    ErrorKind::PermissionDenied => error!("Not allowed to access store"),
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

        if store.version != Version::CURRENT {
            warn!("The store that was loaded is not of the current version: store has {} but the current version is {}", store.version, Version::CURRENT);
            if Version::SUPPROTED.contains(&store.version) {
                warn!("The different store version is still supported, migrating to newer version");
                warn!("Temp migration in memory, can be made permanent by saving");

                if store.version > Version::CURRENT {
                    warn!("The store version is newer than this version of netpulse can normally handle! Trying to ignore potential differences and loading as READONLY!");
                    store.readonly = true;
                }

                while store.version < Version::CURRENT {
                    for check in store.checks_mut().iter_mut() {
                        if let Err(e) = check.migrate(Version::V0) {
                            panic!("Error while migrating check '{}': {e}", check.get_hash());
                        }
                    }
                    store.version = store
                        .version
                        .next()
                        .expect("Somehow migrated to a version that does not exist");
                }

                assert_eq!(store.version, Version::CURRENT);
            } else {
                error!("The store version is not supported");
                return Err(StoreError::UnsupportedVersion);
            }
        }

        if readonly {
            store.set_readonly();
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
    /// - Trying to save a readonly [Store]
    pub fn save(&self) -> Result<(), StoreError> {
        info!("Saving the store");
        if self.readonly {
            return Err(StoreError::IsReadonly);
        }
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
    /// Default is [DEFAULT_PERIOD], but this value can be overridden by setting [ENV_PERIOD] as
    /// environment variable.
    pub fn period_seconds(&self) -> i64 {
        if let Ok(v) = std::env::var(ENV_PERIOD) {
            v.parse().unwrap_or(DEFAULT_PERIOD)
        } else {
            DEFAULT_PERIOD
        }
    }

    /// Generates a cryptographic hash of the entire [Store].
    ///
    /// Uses [blake3] for consistent hashing across Rust versions and platforms.
    /// The hash changes when any check (or other field) in the store is modified,
    /// added, or removed.
    ///
    /// # Implementation Details
    ///
    /// - Uses [bincode] for serialization of store data
    /// - Uses [blake3] for cryptographic hashing
    /// - Produces a 32-byte (256-bit) hash
    /// - Performance scales linearly with store size
    ///
    /// # Memory Usage
    ///
    /// For a netpulsed running continuously:
    /// - ~34 bytes per check
    /// - ~50MB per year at 1 check/minute
    /// - Serialization and hashing remain efficient
    ///
    /// # Panics
    ///
    /// May panic if serialization fails, which can happen in extreme cases:
    /// - System is out of memory
    /// - System is in a severely degraded state
    ///
    /// Normal [Store] data (checks, version info) will always serialize successfully.
    pub fn get_hash(&self) -> blake3::Hash {
        blake3::hash(&bincode::serialize(&self).expect("serialization of the store failed"))
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
    pub fn get_hash_of_file(&self) -> Result<String, StoreError> {
        let out = Command::new("sha256sum").arg(Self::path()).output()?;

        if !out.status.success() {
            error!(
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
    /// Iterates through [CheckType::default_enabled] and [TARGETS] and creates a [Checks](Check).
    ///
    /// Only HTTP checks are done for now, as ICMP needs `CAP_NET_RAW` and DNS is not yet
    /// implemented.
    pub fn make_checks(&mut self) -> Vec<&Check> {
        let last_old = self
            .checks
            .iter()
            .enumerate()
            .last()
            .map(|a| a.0)
            .unwrap_or(0);

        Self::primitive_make_checks(&mut self.checks);

        let mut made_checks = Vec::new();
        for new_check in self.checks.iter().skip(last_old) {
            made_checks.push(new_check);
        }

        made_checks
    }

    /// Creates and adds checks for all configured targets.
    ///
    /// Iterates through [CheckType::default_enabled] and [TARGETS] and creates a [Checks](Check).
    pub fn primitive_make_checks(buf: &mut Vec<Check>) {
        let arcbuf = Arc::new(Mutex::new(Vec::new()));
        let mut threads = Vec::new();
        for check_type in CheckType::default_enabled() {
            trace!("check type: {check_type}");
            if *check_type == CheckType::Icmp && !has_cap_net_raw() {
                warn!("Does not have CAP_NET_RAW, can't use {check_type}, skipping");
                continue;
            }
            for target in TARGETS {
                let thread_ab = arcbuf.clone();
                threads.push(std::thread::spawn(move || {
                    trace!("start thread for {target} with {check_type}");
                    let check = check_type.make(
                        std::net::IpAddr::from_str(target)
                            .expect("a target constant was not an Ip Address"),
                    );
                    thread_ab.lock().expect("lock is poisoned").push(check);
                    trace!("end thread for {target} with {check_type}");
                }));
            }
        }
        for th in threads {
            th.join().expect("could not join thread");
        }
        let abuf = arcbuf.lock().unwrap();
        for check in abuf.iter() {
            buf.push(*check);
        }
    }

    /// Returns the version of this [`Store`].
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns a mutable reference to the checks of this [`Store`].
    pub fn checks_mut(&mut self) -> &mut Vec<Check> {
        &mut self.checks
    }

    /// Reads only the [Version] from a store file without loading the entire [Store].
    ///
    /// This function efficiently checks the store version by:
    /// 1. Opening the store file (decompressing it if enabled)
    /// 2. Deserializing only the version field
    /// 3. Skipping the rest of the data
    ///
    /// This is more efficient than loading the full store when only version
    /// information is needed, such as during version compatibility checks. It may also keep
    /// working if the format/version of the store is incompatible with what this version of
    /// netpulse uses.
    ///
    /// # Feature Flags
    ///
    /// If the "compression" feature is enabled, this function will decompress
    /// the store file using [zstd] before reading the version.
    ///
    /// # Errors
    ///
    /// Returns [StoreError] if:
    /// - Store file doesn't exist ([`StoreError::DoesNotExist`])
    /// - Store file is corrupt or truncated ([`StoreError::Load`])
    /// - File permissions prevent reading ([`StoreError::Io`])
    /// - Decompression fails (with "compression" feature) ([`StoreError::Io`])
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netpulse::store::Store;
    /// use netpulse::errors::StoreError;
    ///
    /// match Store::peek_file_version() {
    ///     Ok(version) => println!("Store version in file: {}", version),
    ///     Err(StoreError::DoesNotExist) => println!("No store file found"),
    ///     Err(e) => eprintln!("Error reading store version: {}", e),
    /// }
    /// ```
    pub fn peek_file_version() -> Result<Version, StoreError> {
        #[derive(Deserialize)]
        struct VersionOnly {
            version: Version,
            #[serde(skip)]
            _rest: serde::de::IgnoredAny,
        }

        let file = std::fs::File::open(Self::path())?;
        #[cfg(feature = "compression")]
        let reader = zstd::Decoder::new(file)?;
        #[cfg(not(feature = "compression"))]
        let reader = file;

        let version_only: VersionOnly = bincode::deserialize_from(reader)?;
        Ok(version_only.version)
    }

    /// True if this [Store] is read only
    pub fn readonly(&self) -> bool {
        self.readonly
    }

    /// Make this [Store] read only
    pub fn set_readonly(&mut self) {
        self.readonly = true;
    }
}

fn has_cap_net_raw() -> bool {
    // First check if we're root (which implies all capabilities)
    if nix::unistd::getuid().is_root() {
        return true;
    }

    // Check current process capabilities
    if let Ok(caps) = caps::read(None, caps::CapSet::Effective) {
        caps.contains(&caps::Capability::CAP_NET_RAW)
    } else {
        warn!("Could not read capabilities");
        false
    }
}
