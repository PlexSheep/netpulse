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
    #[error("Could not load the store from file: {source}")]
    Load {
        #[from]
        source: bincode::Error,
    },
    #[error("Could not convert data to Utf8")]
    Str {
        #[from]
        source: std::str::Utf8Error,
    },
    #[error("A subprocess ended non successfully")]
    ProcessEndedWithoutSuccess,
    #[error("Tried to load a store with an unsupported version")]
    UnsupportedVersion,
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("IO Error")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[cfg(feature = "ping")]
    #[error("Ping Error")]
    Ping {
        #[from]
        source: ping::Error,
    },
    #[cfg(feature = "http")]
    #[error("Http Error")]
    Http {
        #[from]
        source: curl::Error,
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

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Something went wrong with the store")]
    StoreError {
        #[from]
        source: StoreError,
    },
    #[error("Text Formatting error")]
    Fmt {
        #[from]
        source: std::fmt::Error,
    },
}
