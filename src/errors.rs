use thiserror::Error;

/// Could not convert from [CheckFlag](crate::records::CheckFlag) to [CheckType](crate::records::CheckType).
pub struct CheckFlagTypeConversionError;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("The store does not exist")]
    DoesNotExist,
    #[error("IO Error")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Could not Serialize or Deserialize")]
    DeSerialize {
        #[from]
        source: bincode::Error,
    },
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("IO Error")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Ping Error")]
    Ping {
        #[from]
        source: ping::Error,
    },
}

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Something went wrong with the store")]
    StoreError {
        #[from]
        source: StoreError,
    },
    #[error("IO Error")]
    Io {
        #[from]
        source: std::io::Error,
    },
}
