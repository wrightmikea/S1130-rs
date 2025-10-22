# S1130-rs: IBM 1130 Emulator in Rust

A pure Rust implementation of the IBM 1130 computer emulator with WebAssembly support.

## Overview

This project is a complete rewrite of the S1130 IBM 1130 emulator, originally implemented in C# and JavaScript, now ported to pure Rust with a Yew-based WebAssembly frontend. The goal is to create a high-performance, cross-platform emulator that can run both natively and in web browsers.

## Project Status

**Currently in development - Phase 4: Device Implementation**

### Completed Phases
- ✅ **Phase 0**: Project scaffolding and workspace setup
- ✅ **Phase 1**: Instruction decoding (28 opcodes)
- ✅ **Phase 2**: Instruction execution (34 instruction tests passing)
- ✅ **Phase 3**: Two-pass assembler (all tests passing)
- 🔄 **Phase 4**: I/O Device implementation (in progress)
  - ✅ Device system foundation and IOCC handling
  - ✅ XIO instruction execution
  - ✅ Console keyboard device (character-mode)
  - ✅ Console printer device (character-mode)
  - 🔄 Integration tests (in progress)

### Planned Features
- Console keyboard and printer devices (in progress)
- Card reader/punch devices
- Disk storage devices
- Tape devices
- Line printer
- Vector graphics displays (planned)
- Web-based UI using Yew framework

## Architecture

The project is organized as a Rust workspace with multiple crates:

- **s1130-core**: Core emulator logic (CPU, memory, devices, assembler)
- **s1130-wasm**: WebAssembly bindings
- **s1130-ui**: Yew-based web frontend

## Setup

### Installing Rust

If you don't have Rust installed, install it using [rustup](https://rustup.rs/):

```bash
# On macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows, download and run rustup-init.exe from https://rustup.rs/
```

After installation, restart your terminal and verify:
```bash
rustc --version
cargo --version
```

### Installing WebAssembly Tools

For WebAssembly development, install `wasm-pack`:

```bash
cargo install wasm-pack
```

### Installing Development Server

For local testing, install `basic-http-server`:

```bash
cargo install basic-http-server
```

## Quick Start

After installing the prerequisites (see [Setup](#setup) section below):

```bash
# Clone the repository
git clone https://github.com/wrightmikea/S1130-rs.git
cd S1130-rs

# Run tests to verify everything works
cargo test

# Build WebAssembly (optional - only needed for web UI)
./scripts/build.sh

# Start development server (optional - only for web UI)
./scripts/serve.sh
# Then open http://localhost:1130 in your browser
```

## Building

### Prerequisites
- Rust 1.70 or later
- wasm-pack (for WebAssembly builds - optional)
- basic-http-server (for development server - optional)

### Native Build

```bash
# Build all crates (native/library only)
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Run with no warnings from clippy
cargo clippy --all -- -D warnings

# Format code
cargo fmt --all
```

### WebAssembly Build

The project includes helper scripts in the `./scripts/` directory:

#### `./scripts/build.sh`
Builds the WebAssembly package using wasm-pack:
- Compiles `s1130-wasm` crate to WebAssembly
- Generates JavaScript bindings
- Outputs to `./pkg/` directory
- Target: web (ES modules)

```bash
./scripts/build.sh
```

Manual equivalent:
```bash
cd crates/s1130-wasm
wasm-pack build --target web --out-dir ../../pkg
```

#### `./scripts/serve.sh`
Starts a local development server:
- Serves the `s1130-ui` crate on port **1130** (matching the IBM 1130!)
- Accessible at `http://localhost:1130`
- Binds to `0.0.0.0` (accessible from network)
- Requires `basic-http-server` to be installed

```bash
./scripts/serve.sh
```

**Note**: Install the server first if needed:
```bash
cargo install basic-http-server
```

## Documentation

- [Implementation Plan](./ImplementationPlan.md) - Detailed phase-by-phase implementation roadmap
- [Architecture Documentation](./docs/DMS-Architecture.md) - System architecture overview
- [Phase 0 Documentation](./docs/phase-0-workspace.md) - Workspace setup details
- [API Documentation](./docs/) - Additional technical documentation

## Original Project

This is a port of the original S1130 emulator. The original C#/JavaScript implementation and documentation can be found in the [`archive/`](./archive/) directory.

**Original Repository**: [IBM 1130 Emulator (C#/JS)](https://github.com/wrightmikea/S1130) *(if publicly available)*

## License

MIT License - See [LICENSE](./LICENSE) file for details.

## Contributing

This project is currently under active development. Contributions, bug reports, and suggestions are welcome!

## References

- IBM 1130 Functional Characteristics Manual
- IBM 1130 Principles of Operation
- Original S1130 emulator documentation (see `archive/`)
