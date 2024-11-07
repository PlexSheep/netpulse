use std::time::{self, Duration, UNIX_EPOCH};

use netpulse::records::{Check, CheckFlag};
use netpulse::store::Store;

// TODO: better error handling, keep going even if everything goes boom
pub(crate) fn daemon() {
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
