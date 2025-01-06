docker run -it -v $PWD:/opt/nps:ro rust bash -c 'cd /opt/nps && CARGO_TARGET_DIR=/opt/netpulse cargo install --path . && NETPULSE_LOG_LEVEL=trace bash'
