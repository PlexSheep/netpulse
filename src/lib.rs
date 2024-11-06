/// The filename of the database, in [DB_PATH]
pub const DB_NAME: &str = "netpulse.store";
/// Path to the database of netpulse (combine with [DB_NAME])
pub const DB_PATH: &str = "/var/lib/netpulse/";
/// How long to wait until considering a connection as timed out, in milliseconds
pub const TIMEOUT_MS: u16 = 30_000;

pub mod errors;
pub mod records;
