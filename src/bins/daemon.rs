//! Core daemon process that runs network checks at regular intervals.
//!
//! The daemon:
//! - Loads or creates a [Store]
//! - Runs checks every [period_seconds](netpulse::store::Store::period_seconds)
//! - Handles graceful shutdown on SIGTERM
//! - Maintains PID file at [DAEMON_PID_FILE]
//!
//! # Signal Handling
//!
//! The daemon handles the following signals:
//! - SIGTERM: Graceful shutdown, saves state and removes PID file
//!
//! # Cleanup
//!
//! On shutdown, the daemon:
//! 1. Saves the current store state
//! 2. Removes its PID file
//! 3. Logs any cleanup errors

use std::sync::atomic::AtomicBool;
use std::time::{self, Duration, UNIX_EPOCH};

use netpulse::analyze::display_group;
use netpulse::errors::DaemonError;
use netpulse::DAEMON_PID_FILE;
use nix::sys::signal::{self, SigHandler, Signal};

use netpulse::store::Store;

use crate::USES_DAEMON_SYSTEM;

static TERMINATE: AtomicBool = AtomicBool::new(false);

/// Main daemon process function.
///
/// This function:
/// 1. Sets up signal handlers
/// 2. Loads/creates the store
/// 3. Enters main check loop
/// 4. Handles graceful shutdown
// TODO: better error handling, keep going even if everything goes boom
pub(crate) fn daemon() {
    signal_hook();
    println!("starting daemon...");
    let mut store = match Store::load_or_create() {
        Err(e) => {
            eprintln!("{e}");
            if let Err(e) = cleanup_without_store() {
                eprintln!("error while trying to cleanup: {e}");
            }
            std::process::exit(1)
        }
        Ok(s) => s,
    };
    println!("store loaded, entering main loop");
    loop {
        if TERMINATE.load(std::sync::atomic::Ordering::Relaxed) {
            println!("terminating the daemon");
            if let Err(e) = cleanup(&store) {
                eprintln!("could not clean up before terminating: {e:#?}");
            }
            std::process::exit(1);
        }
        let time = time::SystemTime::now();
        if time
            .duration_since(UNIX_EPOCH)
            .expect("time is before the UNIX_EPOCH")
            .as_secs()
            % store.period_seconds()
            == 0
        {
            if let Err(err) = wakeup(&mut store) {
                eprintln!("error in the wakeup turn: {err}");
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// Run a check iteration and update store.
///
/// Called periodically by the daemon main loop to:
/// - Run configured checks
/// - Save results to store
/// - Handle any check errors
///
/// # Errors
///
/// Returns [DaemonError] if store operations fail.
fn wakeup(store: &mut Store) -> Result<(), DaemonError> {
    println!("waking up!");

    let mut buf = String::new();
    display_group(&store.make_checks(), &mut buf)?;
    println!("{buf}");

    if let Err(err) = store.save() {
        eprintln!("error while saving to file: {err:}");
    }

    println!("done!");
    Ok(())
}

fn signal_hook() {
    unsafe {
        signal::signal(Signal::SIGTERM, SigHandler::Handler(handle_sigterm))
            .expect("failed to set up signal handler");
    }
}

/// Clean up daemon resources on shutdown.
///
/// Performs:
/// - Final store save
/// - PID file removal
///
/// # Errors
///
/// Returns [DaemonError] if cleanup operations fail.
fn cleanup(store: &Store) -> Result<(), DaemonError> {
    if let Err(err) = store.save() {
        eprintln!("error while saving to file: {err:#?}");
        return Err(err.into());
    }

    cleanup_without_store()?;

    Ok(())
}

fn cleanup_without_store() -> Result<(), DaemonError> {
    // stuff we only need to do if it's a manual daemon
    if USES_DAEMON_SYSTEM.load(std::sync::atomic::Ordering::Relaxed) {
        if let Err(err) = std::fs::remove_file(DAEMON_PID_FILE) {
            if matches!(err.kind(), std::io::ErrorKind::NotFound) {
                // yeah, idk, ignore?
            } else {
                eprintln!("Failed to remove PID file: {}", err);
                return Err(err.into());
            }
        }
    }

    Ok(())
}

/// Signal handler for things like SIGTERM
extern "C" fn handle_sigterm(_: i32) {
    TERMINATE.store(true, std::sync::atomic::Ordering::Relaxed);
}
