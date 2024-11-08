use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::errors::StoreError;
use crate::records::{Check, CheckType, TARGETS_HTTP};

#[cfg(feature = "compression")]
use zstd;

/// The filename of the database, in [DB_PATH]
pub const DB_NAME: &str = "netpulse.store";
/// Path to the database of netpulse (combine with [DB_NAME])
pub const DB_PATH: &str = "/var/lib/netpulse";
#[cfg(feature = "compression")]
pub const ZSTD_COMPRESSION_LEVEL: i32 = 4;
pub const ENV_PATH: &str = "NETPULSE_STORE_PATH";

/// A version of the [Store].
///
/// The [Store] definition might change over time as netpulse is developed. To work with older or
/// newer [Stores](Store), we need to be able to easily distinguish between versions. The store
/// version is just stored as a [u8].
#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Version {
    inner: u8,
}

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Store {
    version: Version,
    checks: Vec<Check>,
}

impl From<u8> for Version {
    fn from(value: u8) -> Self {
        Self { inner: value }
    }
}

impl From<Version> for u8 {
    fn from(value: Version) -> Self {
        value.inner
    }
}

impl Version {
    pub const CURRENT: Self = Version::new(0);
    pub const SUPPROTED: &[Self] = &[Version::new(0)];

    pub(crate) const fn new(raw: u8) -> Self {
        Self { inner: raw }
    }
}

impl Store {
    pub fn path() -> PathBuf {
        if let Some(var) = std::env::var_os(ENV_PATH) {
            let mut p = PathBuf::from(var);
            p.push(DB_NAME);
            p
        } else {
            PathBuf::from(format!("{DB_PATH}/{DB_NAME}"))
        }
    }

    fn new() -> Self {
        Self {
            version: Version::CURRENT,
            checks: Vec::new(),
        }
    }

    fn create() -> Result<Self, StoreError> {
        fs::create_dir_all(
            Self::path()
                .parent()
                .expect("the store path has no parent directory"),
        )?;

        let file = match fs::File::options()
            .read(false)
            .write(true)
            .append(false)
            .create_new(true)
            .mode(0o644)
            .open(Self::path())
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

    pub fn load_or_create() -> Result<Self, StoreError> {
        match Self::load() {
            Ok(store) => Ok(store),
            Err(err) => {
                if matches!(err, StoreError::DoesNotExist) {
                    Self::create()
                } else {
                    eprintln!("Error while trying to load the store: {err:#}");
                    Err(err)
                }
            }
        }
    }

    pub fn load() -> Result<Self, StoreError> {
        let file = match fs::File::options()
            .read(true)
            .write(false)
            .create_new(false)
            .open(Self::path())
        {
            Ok(file) => file,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return Err(StoreError::DoesNotExist),
                _ => return Err(err.into()),
            },
        };

        #[cfg(feature = "compression")]
        let reader = zstd::Decoder::new(file)?;
        #[cfg(not(feature = "compression"))]
        let mut reader = file;

        Ok(bincode::deserialize_from(reader)?)
    }

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

    pub fn add_check(&mut self, check: impl Into<Check>) {
        self.checks.push(check.into());
    }

    pub fn checks(&self) -> &[Check] {
        &self.checks
    }

    /// Check every _ seconds
    pub const fn period_seconds(&self) -> u64 {
        60
    }

    /// Hash this database (in memory)
    pub fn display_hash(&self) -> String {
        let mut hasher = std::hash::DefaultHasher::default();
        self.hash(&mut hasher);
        format!("{:016X}", hasher.finish())
    }

    /// Hash this database (the store file in the real filesystem)
    ///
    /// Uses `sha256sum`
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
