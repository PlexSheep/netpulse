# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/PlexSheep/netpulse/releases/tag/v0.1.0) - 2024-11-08

### Added

- *(store)* store version checks
- *(analyze)* add hash to store metadata
- *(analyze)* check if any outages exist before analyzing for them
- *(reader)* analysis of HTTP, analyzis module, outages
- *(reader)* show target in display
- *(checks)* add ping feature nodefault and add http check
- improved cli, less panics
- *(checks)* do the pinging
- *(reader)* add check tester
- *(daemon)* actually make pings
- *(ctl)* end now kills after a timeout and removes the pid file if it remains
- *(ctl)* stop the daemon
- *(ctl)* make netpulsed into netpulsectl, a program that starts and stops the daemon
- *(reader)* very basic reader
- *(daemon)* first daemon app with made up checks
- *(store)* [**breaking**] use zstd compression
- *(store)* add_check function
- *(store)* store load, create and save logic
- *(records)* add record datastructures for checks

### Fixed

- *(checks)* http check did not use timeout
- *(http)* url formatting for ipv6
- remove old capabilities code
- *(checks)* use the latency for icmp
- *(checks)* add icmpv6 to the all checks
- *(ctl)* info was only checking for pidfile, not process
- *(daemon)* daemon did not exit unless the cleanup had an error
- *(daemon)* daemon high cpu usage because of incorrect flow control

### Other

- use Self::new for version from u8
- *(api)* dont generate docs for the bins, that conflicts with the lib
- *(api)* fix all doc warnings
- *(api)* analyze module and small fixes
- *(api)* store module documentations
- *(store)* do not save on loading an older version
- *(api)* document the error module
- *(store)* make create function public
- *(api)* checks with examples and extensive docs
- *(api)* fix examples
- *(api)* tons of api docs with llm help
- add targets note to readme
- readme and repo adjustment
- *(analyze)* check if checks are totally empty
- *(analyze)* refactor and generalize analyze outputs
- *(analyze)* own less things
- set the period_seconds to the production value
- *(analyze)* clean up code in analyze
- automatic Rust CI changes
- use specific targets per check type
- feature fixes and targets are now always ips
- less curl features
- rename ping module to checks
- remove icmp from default_enabled check types, because of CAP_NET_RAW missing from the daemon
- add a build script that adds the caps
- don't automatically use all check types, just the enabled ones
- addres CAP_NET_RAW
- script for cap_net_raw
- use different error types
- Merge branch 'devel' of https://github.com/PlexSheep/netpulsed into devel
- automatic Rust CI changes
- *(daemon)* mock daemon has failing checks sometimes now
- *(records)* remove noflags variant and add more trait derives
- remove unused dependencies
- add deps and rename from base template
- Initial commit
