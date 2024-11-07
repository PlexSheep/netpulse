/// How long to wait until considering a connection as timed out, in milliseconds
pub const TIMEOUT_MS: u16 = 30_000;

pub mod errors;
pub mod records;
pub mod store;
