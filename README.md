# Netpulse

![Project badge](https://img.shields.io/badge/language-Rust-blue.svg)
![Crates.io License](https://img.shields.io/crates/l/netpulse)
![GitHub Release](https://img.shields.io/github/v/release/PlexSheep/netpulse)
![GitHub language count](https://img.shields.io/github/languages/count/PlexSheep/netpulse)
[![Rust CI](https://github.com/PlexSheep/netpulse/actions/workflows/cargo.yaml/badge.svg)](https://github.com/PlexSheep/hedu/actions/workflows/cargo.yaml)

Keep track of your internet connection with a daemon

* [GitHub](https://github.com/PlexSheep/netpulse)
* [crates.io](https://crates.io/crates/netpulse)
* [docs.rs](https://docs.rs/crate/netpulse/)

## PING DOES NOT WORK

To use ping/ICMP, the program needs `CAP_NET_RAW` capabilities. This can't be
done with cargo.

Run the following as root to add the capabilities to the executables.

```bash
setcap cap_net_raw+ep $(which netpulsed)
setcap cap_net_raw+ep $(which netpulse)
```
