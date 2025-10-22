#!/bin/bash
set -e

echo "Building S1130-rs WebAssembly..."

# Build the WASM package
cd crates/s1130-wasm
wasm-pack build --target web --out-dir ../../pkg

echo "Build complete! Output in ./pkg/"
