#!/bin/bash
# Simplified server that bypasses database requirement
export DAINTY_TEST_HTTP_PORT=3001
export RUST_LOG=info

# Mock the postgres connection for testing
cargo build --release 2>&1 | tail -5
echo "Note: This would normally require postgres. For the livereload test, we'll use the minimal example instead."
