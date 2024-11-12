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

use netpulse::errors::RunError;
use netpulse::records::display_group;
use netpulse::DAEMON_PID_FILE;
use nix::sys::signal::{self, SigHandler, Signal};

use netpulse::store::Store;
use tracing::{error, info};

use crate::USES_DAEMON_SYSTEM;

static TERMINATE: AtomicBool = AtomicBool::new(false);
static RESTART: AtomicBool = AtomicBool::new(false);

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
    info!("starting daemon...");
    let mut store = load_store();
    info!("store loaded, entering main loop");
    loop {
        if TERMINATE.load(std::sync::atomic::Ordering::Relaxed) {
            info!("terminating the daemon");
            if let Err(e) = cleanup(&store) {
                error!("could not clean up before terminating: {e:#?}");
            }
            std::process::exit(1);
        }
        if RESTART.load(std::sync::atomic::Ordering::Relaxed) {
            info!("restarting the daemon");
            store = load_store();
        }
        if chrono::Utc::now().timestamp() % store.period_seconds() == 0 {
            if let Err(err) = wakeup(&mut store) {
                error!("error in the wakeup turn: {err}");
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn load_store() -> Store {
    match Store::load_or_create() {
        Err(e) => {
            error!("{e}");
            if let Err(e) = cleanup_without_store() {
                error!("error while trying to cleanup: {e}");
            }
            std::process::exit(1)
        }
        Ok(s) => s,
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
/// Returns [RunError] if store operations fail.
fn wakeup(store: &mut Store) -> Result<(), RunError> {
    info!("waking up!");

    let mut buf = String::new();
    display_group(&store.make_checks(), &mut buf)?;
    info!("Made checks\n{buf}");

    if let Err(err) = store.save() {
        error!("error while saving to file: {err:}");
    }

    info!("done!");
    Ok(())
}

fn signal_hook() {
    unsafe {
        signal::signal(Signal::SIGTERM, SigHandler::Handler(handle_signal))
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
/// Returns [RunError] if cleanup operations fail.
fn cleanup(store: &Store) -> Result<(), RunError> {
    if let Err(err) = store.save() {
        error!("error while saving to file: {err:#?}");
        return Err(err.into());
    }

    cleanup_without_store()?;

    Ok(())
}

fn cleanup_without_store() -> Result<(), RunError> {
    // stuff we only need to do if it's a manual daemon
    if USES_DAEMON_SYSTEM.load(std::sync::atomic::Ordering::Relaxed) {
        if let Err(err) = std::fs::remove_file(DAEMON_PID_FILE) {
            if matches!(err.kind(), std::io::ErrorKind::NotFound) {
                // yeah, idk, ignore?
            } else {
                error!("Failed to remove PID file: {}", err);
                return Err(err.into());
            }
        }
    }

    Ok(())
}

/// Signal handler for things like SIGTERM and SIGHUP that should terminate, restart or otherwise influence the program
///
/// Default behavior is terminating the program in a controlled manner
extern "C" fn handle_signal(signal: i32) {
    let signal: nix::sys::signal::Signal =
        nix::sys::signal::Signal::try_from(signal).expect("got an undefined SIGNAL");
    match signal {
        Signal::SIGTERM => {
            TERMINATE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        Signal::SIGHUP => {
            RESTART.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        _ => {
            // the default behavior is terminating
            TERMINATE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
