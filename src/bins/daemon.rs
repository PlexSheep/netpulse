use std::sync::atomic::AtomicBool;
use std::time::{self, Duration, UNIX_EPOCH};

use netpulse::errors::StoreError;
use netpulse::DAEMON_PID_FILE;
use nix::sys::signal::{self, SigHandler, Signal};

use netpulse::store::Store;

static TERMINATE: AtomicBool = AtomicBool::new(false);

// TODO: better error handling, keep going even if everything goes boom
pub(crate) fn daemon() {
    signal_hook();
    println!("starting daemon...");
    let mut store = Store::load_or_create().expect("boom");
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
        println!("done! sleeping...");
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn wakeup(store: &mut Store) -> Result<(), StoreError> {
    println!("waking up!");

    store.make_checks();

    if let Err(err) = store.save() {
        eprintln!("error while saving to file: {err:}");
    }
    Ok(())
}

fn signal_hook() {
    unsafe {
        signal::signal(Signal::SIGTERM, SigHandler::Handler(handle_sigterm))
            .expect("failed to set up signal handler");
    }
}

fn cleanup(store: &Store) -> Result<(), StoreError> {
    if let Err(err) = store.save() {
        eprintln!("error while saving to file: {err:#?}");
        return Err(err);
    }

    // FIXME: does what I think it should do, but also errors with errno 2 not found
    if let Err(err) = std::fs::remove_file(DAEMON_PID_FILE) {
        if matches!(err.kind(), std::io::ErrorKind::NotFound) {
            // yeah, idk, ignore?
        } else {
            eprintln!("Failed to remove PID file: {}", err);
            return Err(err.into());
        }
    }

    Ok(())
}

/// Signal handler for things like SIGTERM
extern "C" fn handle_sigterm(_: i32) {
    TERMINATE.store(true, std::sync::atomic::Ordering::Relaxed);
}
