#!/bin/bash

# Create temporary directory for node data
mkdir -p $BTC_TMP_DIR

echo "Running BTC node..."
$BTC_DAEMON_PATH \
    -datadir=$BTC_TMP_DIR \
    -chain=$BTC_CHAIN \
    -bind=$BTC_ADDRESS \
    -debug=net

