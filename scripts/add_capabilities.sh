#!/bin/bash
setcap cap_net_raw=+ep target/debug/netpulse
setcap cap_net_raw=+ep target/debug/netpulsed
setcap cap_net_raw=+ep target/release/netpulse
setcap cap_net_raw=+ep target/release/netpulsed
