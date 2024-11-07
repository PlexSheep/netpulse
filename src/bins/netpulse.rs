use netpulse::records::Check;
use netpulse::store::Store;

fn main() {
    let store = Store::load().expect("store file not found");
    let checks = store.checks();
    let successes: Vec<&Check> = checks.iter().filter(|c| c.is_ok()).collect();
    println!("store contains {:09} checks.", checks.len());
    println!("store contains {:09} successful checks.", successes.len());
    println!(
        "success ratio: {:02.02}%",
        (successes.len() as f64 / checks.len() as f64) * 100.0
    )
}
