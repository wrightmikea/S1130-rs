#!/bin/bash
set -e

echo "Starting S1130-rs development server on port 1130..."
echo "Access at: http://localhost:1130"

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "Error: trunk is not installed"
    echo "Install with: cargo install trunk"
    exit 1
fi

# Serve from the UI crate directory using Trunk
cd crates/s1130-ui
trunk serve --port 1130 --address 0.0.0.0
