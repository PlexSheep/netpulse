# Netpulse

![Project badge](https://img.shields.io/badge/language-Rust-blue.svg)
![Crates.io License](https://img.shields.io/crates/l/netpulse)
![GitHub Release](https://img.shields.io/github/v/release/PlexSheep/netpulse)
![GitHub language count](https://img.shields.io/github/languages/count/PlexSheep/netpulse)
[![Rust CI](https://github.com/PlexSheep/netpulse/actions/workflows/cargo.yaml/badge.svg)](https://github.com/PlexSheep/hedu/actions/workflows/cargo.yaml)

Keep track of your internet connection with a daemon. Licensed under MIT.

- [GitHub](https://github.com/PlexSheep/netpulse)
- [crates.io](https://crates.io/crates/netpulse)
- [docs.rs](https://docs.rs/crate/netpulse/)

## Why?

My ISP has trouble pretty much every year some month delivering constant uptime,
Netpulse helps keep track of when your internet connectivity goes down.

## Platform Support

- Primary support: GNU/Linux x86_64
- Other architectures: May work but untested
- Windows: Not supported
- macOS: Unknown/untested

I have it running on my homeserver and laptop with Debian based modern Operating
Systems.

## How it Works

Netpulse performs comprehensive connectivity checks using multiple methods:

1. HTTP Checks: Makes HTTP requests to test application-layer connectivity
2. ICMP Checks: Sends ping requests to test basic network reachability
3. Dual-Stack: Each check is performed over both IPv4 and IPv6 to monitor both network stacks

The daemon performs these checks every 60 seconds against reliable targets
(currently Cloudflare's DNS servers). This multi-protocol approach helps
distinguish between different types of connectivity issues.

## Installation

### Via Cargo

The simplest way to install Netpulse is through Cargo:

```bash
cargo install netpulse
```

This will install both the `netpulse` and `netpulsed` executables.

### System Setup

After installing the binaries, you'll need to set up the daemon environment:

```bash
sudo $(which netpulsed) --setup
```

This will:

- Create the netpulse user and group
- Copy the `netpulsed` executable to `/usr/local/bin/`
- Create necessary directories and set permissions
- Install a systemd unit file
- Configure logging

Note: `cargo` usually installs the binary for your local user, not for the whole
program. If executing as root, you will need to specify the full path. That's
what the `$(which netpulsed)` is for, it just returns the absolute path.

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

The `netpulsed` daemon can be run either through systemd (recommended) or as a standalone process.

#### Using Systemd (Recommended)

After running the setup (`sudo $(which netpulsed) --setup`), you can manage the daemon using standard systemd commands:

```bash
sudo systemctl start netpulsed.service   # Start the daemon
sudo systemctl stop netpulsed.service    # Stop the daemon
sudo systemctl status netpulsed.service  # Check daemon status
```

#### Running Standalone

You can also run `netpulsed` directly as a regular program. Note that root privileges are required for setup, but the daemon will drop privileges to the `netpulse` user and group during operation.

#### Logging

The daemon's log level can be controlled using the `NETPULSE_LOG_LEVEL` environment variable. Valid values are:

- `error`
- `warn`
- `info` (default)
- `debug`
- `trace`

For example:

```bash
NETPULSE_LOG_LEVEL=debug netpulsed --start
```

### The Reader

You can use `netpulse --test` to run the checks the daemon would run and see the
status. Just using `netpulse` without arguments will result in it trying to load
and analyze the store.

#### Example Output

The processed output of `netpulse` currently looks somewhat like this:

```txt
========== General =======================================
checks                  : 00306208
checks ok               : 00305466
checks bad              : 00000742
success ratio           : 99.76%
first check at          : 2024-11-09 00:38:00 +01:00
last check at           : 2025-01-07 16:01:00 +01:00

========== HTTP ==========================================
checks                  : 00151750
checks ok               : 00151447
checks bad              : 00000303
success ratio           : 99.80%
first check at          : 2024-11-09 00:38:00 +01:00
last check at           : 2025-01-07 16:01:00 +01:00

========== ICMP ==========================================
checks                  : 00154458
checks ok               : 00154019
checks bad              : 00000439
success ratio           : 99.72%
first check at          : 2024-11-09 03:19:00 +01:00
last check at           : 2025-01-07 16:01:00 +01:00

========== IPv4 ==========================================
checks                  : 00153104
checks ok               : 00152930
checks bad              : 00000174
success ratio           : 99.89%
first check at          : 2024-11-09 00:38:00 +01:00
last check at           : 2025-01-07 16:01:00 +01:00

========== IPv6 ==========================================
checks                  : 00153104
checks ok               : 00152536
checks bad              : 00000568
success ratio           : 99.63%
first check at          : 2024-11-09 00:38:00 +01:00
last check at           : 2025-01-07 16:01:00 +01:00

========== Outages =======================================
Latest
0:	From 2025-01-07 15:06:00 +01:00 To 2025-01-07 15:06:00 +01:00, Total      4, Partial (25.00 %)
1:	From 2025-01-07 13:44:00 +01:00 To 2025-01-07 13:44:00 +01:00, Total      4, Partial (50.00 %)
2:	From 2025-01-07 13:40:00 +01:00 To 2025-01-07 13:41:00 +01:00, Total      8, Partial (37.50 %)
3:	From 2025-01-07 13:37:00 +01:00 To 2025-01-07 13:38:00 +01:00, Total      8, Partial (50.00 %)
4:	From 2025-01-07 11:50:00 +01:00 To 2025-01-07 11:50:00 +01:00, Total      4, Partial (25.00 %)
5:	From 2025-01-06 14:35:00 +01:00 To 2025-01-06 14:35:00 +01:00, Total      4, Partial (50.00 %)
6:	From 2025-01-06 11:14:00 +01:00 To 2025-01-06 11:14:00 +01:00, Total      4, Partial (50.00 %)
7:	From 2025-01-05 09:45:00 +01:00 To 2025-01-05 09:45:00 +01:00, Total      4, Partial (50.00 %)
8:	From 2025-01-05 09:42:00 +01:00 To 2025-01-05 09:42:00 +01:00, Total      4, Partial (50.00 %)
9:	From 2025-01-05 01:13:00 +01:00 To 2025-01-05 01:13:00 +01:00, Total      4, Partial (50.00 %)

showing only the 10 latest outages...

Most severe
0:	From 2024-12-05 10:53:00 +01:00 To 2024-12-05 11:25:00 +01:00, Total    132, Complete
1:	From 2024-12-20 09:40:00 +01:00 To 2024-12-20 09:41:00 +01:00, Total      8, Complete
2:	From 2024-12-20 09:05:00 +01:00 To 2024-12-20 09:06:00 +01:00, Total      8, Complete
3:	From 2024-12-12 10:40:00 +01:00 To 2024-12-12 10:41:00 +01:00, Total      8, Complete
4:	From 2024-12-22 05:31:00 +01:00 To 2024-12-22 05:31:00 +01:00, Total      4, Complete
5:	From 2024-11-29 16:39:00 +01:00 To 2024-11-29 16:39:00 +01:00, Total      4, Complete
6:	From 2024-11-29 08:09:00 +01:00 To 2024-11-29 08:09:00 +01:00, Total      4, Complete
7:	From 2024-12-20 09:25:00 +01:00 To 2024-12-20 09:30:00 +01:00, Total     24, Partial (91.67 %)
8:	From 2024-12-20 09:14:00 +01:00 To 2024-12-20 09:18:00 +01:00, Total     20, Partial (85.00 %)
9:	From 2024-12-09 04:08:00 +01:00 To 2024-12-09 04:11:00 +01:00, Total     16, Partial (62.50 %)

showing only the 10 most severe outages...

========== Store Metadata ================================
Hash mem blake3         : 9f942007cb174234c4bd0274f5394e41e6c1dffcf44056a50d947c3b99c7e959
Hash file sha256        : c0b6958069c88924549b743ad6d024295c44501564f7b2865d17e8c31e67cd0f
Store Version (mem)     : 2
Store Version (file)    : 2
Store Size (mem)        : 16777248
Store Size (file)       : 1123673
File to Mem Ratio       : 0.06697600226211116
```

### Files and Directories

`netpulsed` will try to create a few directories / files:

- `/run/netpulse/netpulse.pid` – lockfile with the PID of the daemon to make sure it doesn't run multiple times
- `/var/lib/netpulse/netpuse.store` – the database where your checks are stored
- `/var/log/netpulse.log` – contains the stdout of the daemon
- `/var/log/netpulse.err` – contains the stderr of the daemon

**Storage Requirement of the Store**

Netpulse has been running for almost three months on my homeserver now. The
server shuts down over night usually for about 6 hours. Other than that, it
performs the six default checks every 60 seconds. My store file has in that time
grown to 1.1 MB. ZSTD compression and encoding does a lot here.

### Targets

The target IPs with which checks are made are defined in the constant `TARGETS` [here](./src/records.rs).

Currently, it boils down to `1.1.1.1` (cloudflare's DNS server), and the
respective IPv6 adress of that.
