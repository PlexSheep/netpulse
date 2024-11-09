use getopts::Options;

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
