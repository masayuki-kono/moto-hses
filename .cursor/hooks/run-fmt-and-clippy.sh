#!/bin/bash

# Read JSON input from stdin
# { "status": "completed" | "aborted" | "error" }
json_input=$(cat)

# Check status
status=$(echo $json_input | jq -r '.status')

# Execute only when status is completed
if [ "$status" == "completed" ]; then
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
fi
