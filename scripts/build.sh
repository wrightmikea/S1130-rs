#!/bin/bash
set -e

echo "Building S1130-rs Yew application..."

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "Error: trunk is not installed"
    echo "Install with: cargo install trunk"
    exit 1
fi

# Build the Yew UI with Trunk
cd crates/s1130-ui
trunk build --release

echo "Build complete! Output in ./crates/s1130-ui/dist/"
