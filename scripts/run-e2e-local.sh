#!/usr/bin/env bash
set -euo pipefail
PORT=${1:-12222}
ADDR=127.0.0.1:$PORT

# Run mock in background
cargo run -p moto-hses-mock -- $ADDR &
MOCK_PID=$!
trap "kill $MOCK_PID" EXIT

# Give it a moment
sleep 0.5

# Run client example
cargo run -p moto-hses-client --example read_status -- $ADDR