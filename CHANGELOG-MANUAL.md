# Changelog

## [0.7.0] - 2025-01-08

### 🚀 Features

- _(daemon)_ Create user in setup #29
- _(common)_ Impl to_string for Command
- _(analyze)_ Short outages in general view, add -o option to reader
- _(records)_ Impl Ord for Check
- _(analyze)_ Sort outage groups
- _(records)_ Cut off seconds and nanoseconds
- _(reader)_ Dump outages
- _(outage)_ Add serevity struct and creation #44
- _(outage)_ Impl PartialOrd
- _(outage)_ Add severity to display #44
- _(outage)_ Display sorted by outage severity #40

### 🚜 Refactor

- _(daemon)_ Confirm takes any display argument
- _(daemon)_ Ask confirmation and combine setups a bit more
- _(daemon)_ Remove daemonizing with daemonize crate #26
- _(analyze)_ Fail_groups now much simpler
- _(analyze)_ Fail_groups simplistic again
- _(analyze)_ Outages simplified
- _(analyze)_ Move outage things to a new module
- Use error where eprintln was

### 📚 Documentation

- _(common)_ Doctest now specifys skip_checks for exec_cmd_for_user example
- Mention that user creation is part of setup in readme
- Update readme with tons of small things
- Update the readme with outage info and storage info
- Use which for root in readme
- Remove redundant updating section
- _(records)_ Adjust TypeIcmp flag
- _(readme)_ Adjust example reader output
- _(readme)_ Adjust readme example

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes
- Add a small docker test script
- Automatic Rust CI changes
- Add debug info for outages
- Bump to v0.7.0-rc0
- Automatic Rust CI changes
- Bump version to v0.7.0-rc1
- Automatic Rust CI changes
- Automatic Rust CI changes
- Automatic Rust CI changes
- Release v0.7.0-rc1.1
- Bump version to v0.7.0

### 🐛 Bug Fixes

- _(daemon)_ General setup was skipped, loglevel is now info
- _(analyze)_ Outages were not displayed #34
- _(analyze)_ Only one outage group was found
- _(analyze)_ Show last 10 outages
- _(analyze)_ Now more outage duplicates #35
- _(analyze)_ Only take actual failed checks into outages
- _(analyze)_ Split check groups for outage at the right point
- _(analyze)_ Display off by one
- _(analyze)_ Fail_groups finally delivers correct results

### ⚡ Performance

- _(analyze)_ Fail_groups is now faster

### 🧪 Testing

- _(analyze)_ Fail_groups fails so much, I need to test it properly
- _(analyze)_ Test group_by_time
- _(analyze)_ Refactor tests

## [0.6.1] - 2024-11-13

### 🚜 Refactor

- _(store)_ Make checks in separate threads
- _(records)_ Default enable depends on enabled features

### 📚 Documentation

- _(api)_ Fixes and documenting primitive_make_checks with mutlithreading

### ⚙️ Miscellaneous Tasks

- Clean up imports in store
- Automatic Rust CI changes
- Release v0.6.1

## [0.6.0] - 2024-11-12

### 🚀 Features

- Use chrono instead of humantime, make times more readable
- [**breaking**] Use blake3 for hashes #16
- _(store)_ Set period for daemon with env var
- _(reader)_ Rewrite store option
- _(store)_ Add readonly mode to store
- _(store)_ Peek version from fs
- Add a panic handler to netpulse and netpulsed #13

### 🐛 Bug Fixes

- _(store)_ Serialization and deserialization of Version enum did not work

### 🚜 Refactor

- _(records)_ Remove ip type flag and infer ip type from the stored target
- _(store)_ Adjust log levels for some messages
- Use chrono instead of std
- _(store)_ [**breaking**] Use i64 for timestamp instead of u64
- _(store)_ Version is now an enum
- _(store)_ [**breaking**] Version is now an enum
- Move time formatting utils to analyze
- _(reader)_ Load store as readonly

### 📚 Documentation

- _(api)_ Much simplify the documentation for Check::ip_type
- _(api)_ Fix doc links and old info
- _(api)_ Fix doc links and old info
- _(api)_ Fix peek_version adjacent stuff

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes
- Release v0.6.0

## [0.5.1] - 2024-11-11

### 🐛 Bug Fixes

- _(analyze)_ Ip analyze was always done with ipv4

### ⚙️ Miscellaneous Tasks

- Release v0.5.1

## [0.5.0] - 2024-11-10

### 🐛 Bug Fixes

- _(checks)_ Compiler error when not using the ping feature
- _(checks)_ Don't throw compiler error for disabled features

### 📚 Documentation

- _(api)_ Fix examples for no default options

### ⚙️ Miscellaneous Tasks

- Adjust headline for icmp analysis
- Test without default features
- Docs.rs should show all features
- Release v0.4.2

## [0.4.1] - 2024-11-09

### 🚀 Features

- Consider an environment variable when initializing the logging

### 🐛 Bug Fixes

- _(daemon)_ Systemctl stop of the service args were in a single arg

### 🚜 Refactor

- _(common)_ Move the common module into the library with the "executable" feature
- _(records)_ [**breaking**] Consolidate ICMPv4 and ICMPv6 into just ICMP check type

### 📚 Documentation

- _(api)_ Fix tests in common module

### ⚙️ Miscellaneous Tasks

- Clean up imports in netpulsed
- Run doctests too in ci
- Run doctests too in ci
- Release v0.4.1

## [0.4.0] - 2024-11-09

### 🚀 Features

- _(analyze)_ Print the store version of in memory store
- _(analyze)_ Show size of store in mem and fs + ratio
- _(analyze)_ Dump the entire store (it's checks) #8
- _(reader)_ Dump all and dump only failed
- _(daemon)_ Reload the store on SIGHUP
- Use logging with tracing for everything in the library and set it up for the executables #5
- _(daemon)_ Have setup ask to execute the systemd stuff for the user
- _(setup)_ Stop the netpulsed service at the start of setup
- Consider an environment variable when initializing the logging

### 🐛 Bug Fixes

- _(setup)_ Stop the running service first
- _(daemon)_ Stop_requested was initialized with true
- _(setup)_ Args need to be split
- Logging in common/netpulsed

### 🚜 Refactor

- _(analyze)_ Improve display functions
- _(records)_ Display_groups moved to records, better display
- _(bins)_ Share some code in the new common module
- _(daemon)_ Better error handling in main
- _(setup)_ More debug prints for the systemd setup

### 📚 Documentation

- _(api)_ Fix test

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes
- Automatic Rust CI changes
- Fix pedantic warnings
- Automatic Rust CI changes
- Automatic Rust CI changes
- Release v0.4.0

## [0.3.0] - 2024-11-09

### 🚀 Features

- _(daemon)_ Add setup flag and make the daemon flag official
- _(systemd)_ Add netpulsed.service file
- _(systemd)_ Install service file
- _(systemd)_ Only remove pidfile if it's a manual daemon
- _(systemd)_ Copy the netpulsed to /usr/local/bin/ in the setup
- _(store)_ Return the new checks from make_checks and let the daemon print them
- Default enable icmp again, as CAP_NET_RAW is okay with systemd

### 🐛 Bug Fixes

- _(daemon)_ Setup copy was missing the bin name
- _(analyze)_ Check for ip flag needs to be inverted

### 🚜 Refactor

- _(daemon)_ Consolidate setup and setup-systemd into just setup
- _(analyze)_ Make analyze functions more generic and add more parts
- Rename DaemonError to RunError and print netpulse -t data with group_display
- Refactor logs, primitive_make_checks

### 📚 Documentation

- Add setup and update sections for the daemon to the readme

### ⚙️ Miscellaneous Tasks

- Remove unused import
- Automatic Rust CI changes
- Release v0.3.0

## [0.2.0] - 2024-11-08

### 🚀 Features

- _(checks)_ Ip_type and get_hash
- _(error)_ Add AmbiguousFlags and MissingFlags errors
- _(analyze)_ Add ipv4 and ipv6 section

### 🐛 Bug Fixes

- _(daemon)_ Daemon paniced when loading failed
- Show source for errors
- _(store)_ Chown store directory after creating it
- _(cli)_ Don't panic when a bad option is passed
- _(store)_ Setperms fails, print more info
- _(store)_ Chown instead of chmod
- _(cli)_ Print usage when bad options are given

### 🚜 Refactor

- _(daemon)_ Cleanup when store load does not work
- _(store)_ Print additional err messages in store create
- _(store)_ Add a setup function so that the setup can run as root
- _(cli)_ Remove daemon --fail

### 📚 Documentation

- _(api)_ Fix Store::setup example

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes
- Release v0.1.1

## [0.1.0] - 2024-11-08

### 🚀 Features

- _(records)_ Add record datastructures for checks
- _(store)_ Store load, create and save logic
- _(store)_ Add_check function
- _(store)_ [**breaking**] Use zstd compression
- _(daemon)_ First daemon app with made up checks
- _(reader)_ Very basic reader
- _(ctl)_ Make netpulsed into netpulsectl, a program that starts and stops the daemon
- _(ctl)_ Stop the daemon
- _(ctl)_ End now kills after a timeout and removes the pid file if it remains
- _(daemon)_ Actually make pings
- _(reader)_ Add check tester
- _(checks)_ Do the pinging
- Improved cli, less panics
- _(checks)_ Add ping feature nodefault and add http check
- _(reader)_ Show target in display
- _(reader)_ Analysis of HTTP, analyzis module, outages
- _(analyze)_ Check if any outages exist before analyzing for them
- _(analyze)_ Add hash to store metadata
- _(store)_ Store version checks

### 🐛 Bug Fixes

- _(daemon)_ Daemon high cpu usage because of incorrect flow control
- _(daemon)_ Daemon did not exit unless the cleanup had an error
- _(ctl)_ Info was only checking for pidfile, not process
- _(checks)_ Add icmpv6 to the all checks
- _(checks)_ Use the latency for icmp
- Remove old capabilities code
- _(http)_ Url formatting for ipv6
- _(checks)_ Http check did not use timeout

### 🚜 Refactor

- _(records)_ Remove noflags variant and add more trait derives
- _(daemon)_ Mock daemon has failing checks sometimes now
- Use different error types
- Don't automatically use all check types, just the enabled ones
- Remove icmp from default_enabled check types, because of CAP_NET_RAW missing from the daemon
- Rename ping module to checks
- Feature fixes and targets are now always ips
- Use specific targets per check type
- _(analyze)_ Clean up code in analyze
- _(analyze)_ Own less things
- _(analyze)_ Refactor and generalize analyze outputs
- _(analyze)_ Check if checks are totally empty
- _(store)_ Make create function public
- _(store)_ Do not save on loading an older version
- Use Self::new for version from u8

### 📚 Documentation

- Addres CAP_NET_RAW
- Readme and repo adjustment
- Add targets note to readme
- _(api)_ Tons of api docs with llm help
- _(api)_ Fix examples
- _(api)_ Checks with examples and extensive docs
- _(api)_ Document the error module
- _(api)_ Store module documentations
- _(api)_ Analyze module and small fixes
- _(api)_ Fix all doc warnings
- _(api)_ Dont generate docs for the bins, that conflicts with the lib

### ⚙️ Miscellaneous Tasks

- Remove scripts dir
- Add deps and rename from base template
- Remove unused dependencies
- Automatic Rust CI changes
- Automatic Rust CI changes
- Script for cap_net_raw
- Add a build script that adds the caps
- Less curl features
- Automatic Rust CI changes
- Remove build script
- Automatic Rust CI changes
- Set the period_seconds to the production value
- Changelog
- Release v0.1.0
