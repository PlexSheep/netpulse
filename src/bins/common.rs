use getopts::Options;
use tracing_subscriber::FmtSubscriber;

#[allow(dead_code)] // idk why it says thet, netpulsed uses it a few times
pub(crate) fn root_guard() {
    if !nix::unistd::getuid().is_root() {
        eprintln!("This needs to be run as root");
        std::process::exit(1)
    }
}

pub(crate) fn print_usage(program: &str, opts: Options) -> ! {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
    std::process::exit(0)
}

pub(crate) fn print_version() -> ! {
    println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
    std::process::exit(0)
}

pub(crate) fn init_logging(level: tracing::Level) {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(level)
        // No need for the time. It's either ran with systemd (which shows the time in journalctl)
        // or it's the reader which doesn't need it.
        .without_time()
        // would show the module where the thing comes from
        .with_target(false)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
