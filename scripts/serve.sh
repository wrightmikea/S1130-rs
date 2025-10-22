#!/bin/bash
set -e

echo "Starting S1130-rs development server on port 1130..."
echo "Access at: http://localhost:1130"
echo ""

# Check if basic-http-server is installed
if ! command -v basic-http-server &> /dev/null; then
    echo "Error: basic-http-server is not installed"
    echo "Install with: cargo install basic-http-server"
    exit 1
fi

# Serve from the UI crate directory
cd crates/s1130-ui
basic-http-server -a 0.0.0.0:1130
