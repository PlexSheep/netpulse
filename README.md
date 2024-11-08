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

## Why?

My ISP has trouble pretty much every year some month delivering constant uptime,
Netpulse helps keep track of when your internet connectivity goes down.

## Usage

Netpulse has two parts:

1. `netpulsed` – A daemon that is supposed to run all the time on your server
   / machine that should keep track of your internet connection
2. `netpulse` – A Tool that can read and analyze the store, which contains the
   checks made by `netpulsed`

To use Netpulse, you need to let the daemon `netpulsed` run for a while, and
then you can read out the data with `netpulse`.

Basically, `netpulsed` will try to make HTTP requests to a few targets every 60
seconds.

### The Daemon

The daemon of Netpulse can be started, ended and so on with the `netpulsed`
executable.

A simple `sudo netpulsed --start` will let the daemon run until you stop it or
your system shuts down. Root privileges are required for starting and setup,
but privileges will be dropped to the user `netpulse` with the group
`netpulse`.

Therefore, you need to create a user `netpulse` on your system to use the
daemon:

```bash
useradd -r -s /usr/sbin/nologin netpulse
```

### The Reader

You can use `netpulse --test` to run the checks the daemon would run and see the
status. Just using `netpulse` without arguments will result in it trying to load
and analyze the store.

### Files and Directories

`netpulsed` will try to create a few directories / files:

* `/run/netpulse/netpulse.pid` – lockfile with the PID of the daemon to make sure it doesn't run multiple times
* `/var/lib/netpulse/netpuse.store` – the database where your checks are stored
* `/var/log/netpulse.log` – contains the stdout of the daemon
* `/var/log/netpulse.err` – contains the stderr of the daemon
