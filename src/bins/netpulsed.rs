use std::fs::{self, File};
use std::time::{self, Duration, UNIX_EPOCH};

use daemonize::Daemonize;
use netpulse::records::{Check, CheckFlag};
use netpulse::store::Store;
use netpulse::{DAEMON_LOG_ERR, DAEMON_LOG_INF};

fn main() {
    let path = Store::path();
    let parent_path = path.parent().expect("store file has no parent directory");
    println!("Parent: {parent_path:?}");

    let logfile = File::create(DAEMON_LOG_INF).expect("could not open info logfile");
    let errfile = File::create(DAEMON_LOG_ERR).expect("could not open error logfile");

    fs::create_dir_all(parent_path).expect("could not create the store directory");

    let daemonize = Daemonize::new()
        .pid_file("/run/netpulse.pid")
        .chown_pid_file(true)
        .working_directory(parent_path)
        .user("netpulse")
        .group("netpulse")
        .stdout(logfile)
        .stderr(errfile)
        .umask(0o027); // rwxr-x---

    println!("daemon setup done");

    match daemonize.start() {
        Ok(_) => daemon(),
        Err(e) => {
            eprintln!("Error starting daemon: {:#}", e);
        }
    }

    eprintln!("end?")
}

// TODO: better error handling, keep going even if everything goes boom
fn daemon() {
    println!("starting daemon...");
    let mut store = Store::load_or_create().expect("boom");
    loop {
        let time = time::SystemTime::now();
        println!("making a check...");
        store.add_check(Check::new(
            time,
            if time.duration_since(UNIX_EPOCH).unwrap().as_secs() % 10 == 0 {
                CheckFlag::Timeout | CheckFlag::TypePing
            } else {
                CheckFlag::Success.into()
            },
            None,
        ));
        println!("saving...");
        store.save().expect("could not save");
        println!("done! sleeping...");
        std::thread::sleep(Duration::from_secs(5));
    }
}
