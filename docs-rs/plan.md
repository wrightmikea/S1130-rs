# S1130 Rust + Yew Implementation Plan

## Overview

This document outlines the detailed implementation plan for porting the S1130 IBM 1130 emulator from C# to Rust + Yew (WASM). The plan follows a test-driven development (TDD) approach with incremental milestones to ensure quality and maintainability throughout the porting process.

## Guiding Principles

1. **Test-Driven Development**: Write tests first (Red), implement to pass (Green), refactor for quality
2. **Incremental Progress**: Small, testable changes that build on each other
3. **Code Quality**: Maintain zero `cargo clippy` warnings and proper `cargo fmt` formatting
4. **Frequent Integration**: Run full test suite after each major change
5. **Documentation**: Update docs as code evolves
6. **Reversibility**: Each phase can be committed independently

## Development Environment Setup

### Prerequisites

Before starting, ensure you have:
- Rust 2024 Edition (stable)
- `rustup` for toolchain management
- `cargo` for build management
- Node.js 18+ (for `trunk`)
- Git for version control

### Initial Setup Commands

```bash
# Install Rust toolchain
rustup install stable
rustup default stable
rustup target add wasm32-unknown-unknown

# Install development tools
cargo install trunk
cargo install wasm-pack
cargo install wasm-bindgen-cli
cargo install cargo-watch
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit       # Security audits
cargo install criterion         # Benchmarking

# Create workspace
cd /Users/mike/github/wrightmikea/S1130-rs
cargo init --lib crates/s1130-core
cargo init --lib crates/s1130-wasm
cargo init crates/s1130-ui

# Initialize workspace Cargo.toml
# (see Phase 0 for contents)
```

## Phase 0: Project Scaffolding (Week 1)

### Goal
Set up Rust workspace with proper structure, CI/CD, and tooling.

### Tasks

#### Task 0.1: Create Workspace Structure

**Commands**:
```bash
# Create directory structure
mkdir -p crates/{s1130-core,s1130-wasm,s1130-ui}/src
mkdir -p crates/s1130-core/tests
mkdir -p crates/s1130-ui/static
mkdir -p docs-rs
```

**Create workspace `Cargo.toml`**:
```toml
[workspace]
members = [
    "crates/s1130-core",
    "crates/s1130-wasm",
    "crates/s1130-ui",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/wrightmikea/S1130-rs"

[workspace.dependencies]
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Acceptance Criteria**:
- [ ] `cargo build --workspace` succeeds
- [ ] Directory structure matches design document

#### Task 0.2: Configure `s1130-core` Crate

**Create `crates/s1130-core/Cargo.toml`**:
```toml
[package]
name = "s1130-core"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror.workspace = true
serde.workspace = true

[dev-dependencies]
proptest = "1.4"
criterion = "0.5"

[[bench]]
name = "cpu_benchmark"
harness = false
```

**Create basic module structure**:
```bash
cd crates/s1130-core/src
touch lib.rs cpu.rs instructions.rs devices.rs assembler.rs error.rs
```

**Acceptance Criteria**:
- [ ] `cargo build -p s1130-core` succeeds
- [ ] `cargo test -p s1130-core` runs (no tests yet)

#### Task 0.3: Configure `s1130-wasm` Crate

**Create `crates/s1130-wasm/Cargo.toml`**:
```toml
[package]
name = "s1130-wasm"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
s1130-core = { path = "../s1130-core" }
wasm-bindgen = "0.2"
serde-wasm-bindgen = "0.6"
console_error_panic_hook = "0.1"
serde.workspace = true

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

**Acceptance Criteria**:
- [ ] `cargo build -p s1130-wasm --target wasm32-unknown-unknown` succeeds

#### Task 0.4: Configure `s1130-ui` Crate

**Create `crates/s1130-ui/Cargo.toml`**:
```toml
[package]
name = "s1130-ui"
version.workspace = true
edition.workspace = true

[dependencies]
s1130-wasm = { path = "../s1130-wasm" }
yew = { version = "0.21", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = "0.3"
gloo = "0.11"
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

**Create `crates/s1130-ui/index.html`**:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IBM 1130 Emulator</title>
    <link rel="stylesheet" href="/static/styles.css">
</head>
<body>
    <div id="app"></div>
</body>
</html>
```

**Acceptance Criteria**:
- [ ] `trunk build` succeeds
- [ ] `trunk serve` starts dev server

#### Task 0.5: Set Up CI/CD Pipeline

**Create `.github/workflows/ci.yml`**:
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo fmt --all -- --check

  wasm:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - uses: jetli/trunk-action@v0.5.0
      - run: trunk build --release
```

**Acceptance Criteria**:
- [ ] CI pipeline runs on GitHub
- [ ] All checks pass (even with minimal code)

---

## Phase 1: CPU Core Implementation (Weeks 2-4)

### Goal
Implement the CPU core with registers, memory, and basic instruction execution loop. Port existing C# tests.

### Tasks

#### Task 1.1: Define Core Types and Errors

**File**: `crates/s1130-core/src/error.rs`

**Code**:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CpuError {
    #[error("Invalid instruction at address {0:#06x}")]
    InvalidInstruction(u16),

    #[error("Memory access violation at {0:#06x}")]
    MemoryViolation(u16),

    #[error("Device error: {0}")]
    DeviceError(String),

    #[error("Execution halted by WAIT instruction")]
    WaitState,

    #[error("No instruction loaded for execution")]
    NoInstructionLoaded,
}

#[derive(Debug, Error)]
pub enum InstructionError {
    #[error("Invalid opcode: {0:#04x}")]
    InvalidOpcode(u8),

    #[error("Memory access error: {0}")]
    MemoryError(#[from] CpuError),
}

pub type Result<T> = std::result::Result<T, CpuError>;
```

**Test** (TDD - Red):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CpuError::MemoryViolation(0x1234);
        assert_eq!(err.to_string(), "Memory access violation at 0x1234");
    }
}
```

**Acceptance Criteria**:
- [ ] All error types defined
- [ ] Tests pass
- [ ] `cargo clippy` clean
- [ ] `cargo fmt` applied

#### Task 1.2: Implement CPU Registers and Memory

**File**: `crates/s1130-core/src/cpu.rs`

**Code** (based on design.md):
```rust
use crate::error::*;
use std::collections::HashMap;

pub struct Cpu {
    acc: u16,
    ext: u16,
    iar: u16,
    xr1: u16,
    xr2: u16,
    xr3: u16,
    carry: bool,
    overflow: bool,
    wait: bool,
    memory: Vec<u16>,
    instruction_count: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            acc: 0,
            ext: 0,
            iar: 0,
            xr1: 0,
            xr2: 0,
            xr3: 0,
            carry: false,
            overflow: false,
            wait: false,
            memory: vec![0; 32768],
            instruction_count: 0,
        }
    }

    pub fn read_memory(&self, address: usize) -> Result<u16> {
        self.memory.get(address)
            .copied()
            .ok_or(CpuError::MemoryViolation(address as u16))
    }

    pub fn write_memory(&mut self, address: usize, value: u16) -> Result<()> {
        if address < self.memory.len() {
            self.memory[address] = value;

            // Memory-mapped index registers
            match address {
                0x0001 => self.xr1 = value,
                0x0002 => self.xr2 = value,
                0x0003 => self.xr3 = value,
                _ => {}
            }

            Ok(())
        } else {
            Err(CpuError::MemoryViolation(address as u16))
        }
    }
}
```

**Tests** (from C# CpuTests.cs):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.ext, 0);
        assert_eq!(cpu.iar, 0);
        assert_eq!(cpu.memory.len(), 32768);
    }

    #[test]
    fn test_memory_read_write() {
        let mut cpu = Cpu::new();
        cpu.write_memory(0x100, 0x1234).unwrap();
        assert_eq!(cpu.read_memory(0x100).unwrap(), 0x1234);
    }

    #[test]
    fn test_memory_bounds_check() {
        let cpu = Cpu::new();
        assert!(cpu.read_memory(0x10000).is_err());
    }

    #[test]
    fn test_memory_mapped_index_registers() {
        let mut cpu = Cpu::new();
        cpu.write_memory(0x0001, 0xABCD).unwrap();
        assert_eq!(cpu.xr1, 0xABCD);
        assert_eq!(cpu.read_memory(0x0001).unwrap(), 0xABCD);
    }
}
```

**Acceptance Criteria**:
- [ ] All tests pass
- [ ] Memory access is bounds-checked
- [ ] Index registers are memory-mapped
- [ ] Code coverage > 80%

#### Task 1.3: Implement Instruction Decoding

**File**: `crates/s1130-core/src/instructions.rs`

**Code**:
```rust
use crate::error::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    Load = 0x18,
    Store = 0x1A,
    Add = 0x10,
    // ... (all 28 opcodes)
}

impl OpCode {
    pub fn from_u8(value: u8) -> Result<Self, InstructionError> {
        match value {
            0x18 => Ok(OpCode::Load),
            0x1A => Ok(OpCode::Store),
            // ...
            _ => Err(InstructionError::InvalidOpcode(value))
        }
    }
}

#[derive(Debug, Clone)]
pub enum InstructionFormat {
    Short,
    Long,
}

#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub opcode: OpCode,
    pub format: InstructionFormat,
    pub tag: u8,
    pub displacement: u16,
    pub indirect: bool,
    pub modifiers: u8,
}
```

**Add to `cpu.rs`**:
```rust
impl Cpu {
    pub fn next_instruction(&mut self) -> Result<()> {
        let first_word = self.read_memory(self.iar as usize)?;
        self.iar = self.iar.wrapping_add(1);

        let opcode_bits = ((first_word >> 11) & 0x1F) as u8;
        let opcode = OpCode::from_u8(opcode_bits)?;

        let format_bit = (first_word & 0x0400) != 0;
        let tag = ((first_word & 0x0300) >> 8) as u8;
        let modifiers = (first_word & 0x00FF) as u8;

        let (format, displacement, indirect) = if format_bit {
            let second_word = self.read_memory(self.iar as usize)?;
            self.iar = self.iar.wrapping_add(1);
            let indirect = (first_word & 0x0080) != 0;
            (InstructionFormat::Long, second_word, indirect)
        } else {
            (InstructionFormat::Short, modifiers as u16, false)
        };

        // Store decoded instruction (implement storage later)
        Ok(())
    }
}
```

**Tests** (from C# InstructionTests):
```rust
#[test]
fn test_decode_short_format() {
    let mut cpu = Cpu::new();
    cpu.iar = 0x100;

    // LD 5 (short format)
    cpu.write_memory(0x100, 0xC005).unwrap();
    cpu.next_instruction().unwrap();

    assert_eq!(cpu.iar, 0x101);
}

#[test]
fn test_decode_long_format() {
    let mut cpu = Cpu::new();
    cpu.iar = 0x100;

    // LD L 0x0400
    cpu.write_memory(0x100, 0xC400).unwrap();
    cpu.write_memory(0x101, 0x0400).unwrap();
    cpu.next_instruction().unwrap();

    assert_eq!(cpu.iar, 0x102);
}
```

**Acceptance Criteria**:
- [ ] Instruction decoding tests pass
- [ ] Both short and long formats supported
- [ ] IAR updated correctly

#### Task 1.4: Port Remaining CPU Tests

**Goal**: Port all tests from `UnitTests.S1130.SystemObjects/CpuTests.cs`

**Process**:
1. Read C# test
2. Write equivalent Rust test (Red)
3. Implement feature to pass test (Green)
4. Refactor if needed
5. Run full suite

**Acceptance Criteria**:
- [ ] All CPU-level tests from C# ported
- [ ] All tests pass
- [ ] Code coverage > 85%

---

## Phase 2: Instruction Set Implementation (Weeks 5-8)

### Goal
Implement all 28 instructions with full addressing mode support. Port all instruction tests from C#.

### Strategy

Each instruction follows this pattern:
1. Create test file (e.g., `tests/instruction_tests/load_tests.rs`)
2. Port C# tests → Red
3. Implement instruction → Green
4. Refactor → Maintain Green
5. Move to next instruction

### Task 2.1: Implement Load/Store Instructions

**Instructions**: LD, LDD, STO, STD, LDX, STX, LDS, STS

**File**: `crates/s1130-core/src/instructions/load_store.rs`

**Example** (Load instruction):
```rust
use crate::cpu::Cpu;
use crate::error::*;
use crate::instructions::{OpCode, InstructionInfo, InstructionFormat};

pub struct Load;

impl Load {
    pub fn execute(cpu: &mut Cpu, info: &InstructionInfo) -> Result<()> {
        let address = Self::get_effective_address(cpu, info)?;
        let value = cpu.read_memory(address as usize)?;
        cpu.set_acc(value);
        Ok(())
    }

    fn get_effective_address(cpu: &Cpu, info: &InstructionInfo) -> Result<u16> {
        let base_address = match info.format {
            InstructionFormat::Long => info.displacement,
            InstructionFormat::Short => {
                let signed_disp = if info.displacement & 0x80 != 0 {
                    (info.displacement as i16) | 0xFF00
                } else {
                    info.displacement as i16
                };
                cpu.get_iar().wrapping_add_signed(signed_disp)
            }
        };

        let indexed_address = if info.tag > 0 {
            base_address.wrapping_add(cpu.get_index_register(info.tag))
        } else {
            base_address
        };

        if info.indirect {
            let pointer = cpu.read_memory(indexed_address as usize)?;
            Ok(pointer)
        } else {
            Ok(indexed_address)
        }
    }
}
```

**Tests** (port from `LoadTests.cs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_load_short_format_loads_accumulator() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x100);
        cpu.write_memory(0x105, 0x1234).unwrap();

        // LD 5
        let info = build_short_instruction(OpCode::Load, 0, 5);
        Load::execute(&mut cpu, &info).unwrap();

        assert_eq!(cpu.get_acc(), 0x1234);
    }

    #[test]
    fn execute_load_long_format_loads_accumulator() {
        let mut cpu = Cpu::new();
        cpu.write_memory(0x0400, 0xABCD).unwrap();

        // LD L 0x0400
        let info = build_long_instruction(OpCode::Load, 0, 0x0400, false);
        Load::execute(&mut cpu, &info).unwrap();

        assert_eq!(cpu.get_acc(), 0xABCD);
    }

    #[test]
    fn execute_load_with_index_register() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x100);
        cpu.set_index_register(1, 0x0010);
        cpu.write_memory(0x0115, 0x5678).unwrap();

        // LD 5,X1
        let info = build_short_instruction(OpCode::Load, 1, 5);
        Load::execute(&mut cpu, &info).unwrap();

        assert_eq!(cpu.get_acc(), 0x5678);
    }

    #[test]
    fn execute_load_indirect() {
        let mut cpu = Cpu::new();
        cpu.write_memory(0x0400, 0x0500).unwrap();
        cpu.write_memory(0x0500, 0x9999).unwrap();

        // LD L 0x0400 I
        let info = build_long_instruction(OpCode::Load, 0, 0x0400, true);
        Load::execute(&mut cpu, &info).unwrap();

        assert_eq!(cpu.get_acc(), 0x9999);
    }
}
```

**Acceptance Criteria** (per instruction):
- [ ] All addressing modes tested (short, long, indexed, indirect)
- [ ] Edge cases covered (boundary addresses, etc.)
- [ ] Tests ported from C# version
- [ ] Implementation matches IBM 1130 behavior

**Repeat for all Load/Store instructions**:
- [ ] LD (Load)
- [ ] LDD (Load Double)
- [ ] STO (Store)
- [ ] STD (Store Double)
- [ ] LDX (Load Index)
- [ ] STX (Store Index)
- [ ] LDS (Load Status)
- [ ] STS (Store Status)

### Task 2.2-2.5: Implement Remaining Instruction Categories

**Process**: Same TDD approach for each category:

**Task 2.2: Arithmetic Instructions** (2 days)
- [ ] A (Add)
- [ ] AD (Add Double)
- [ ] S (Subtract)
- [ ] SD (Subtract Double)
- [ ] M (Multiply)
- [ ] D (Divide)

**Task 2.3: Logical Instructions** (1 day)
- [ ] AND
- [ ] OR
- [ ] EOR (Exclusive OR)

**Task 2.4: Shift Instructions** (2 days)
- [ ] SLA (Shift Left Accumulator)
- [ ] SRA (Shift Right Accumulator)
- [ ] SLT (Shift Left and Count)
- [ ] SRT (Shift Right and Count)
- [ ] SLC (Shift Left Combined)
- [ ] SLCA (Shift Left Combined Arithmetic)

**Task 2.5: Branch and Control Instructions** (2 days)
- [ ] BSC (Branch or Skip on Condition)
- [ ] BSI (Branch and Store IAR)
- [ ] MDX (Modify Index and Skip)
- [ ] XIO (Execute I/O)
- [ ] WAIT

### Milestone: Complete Instruction Set

**Acceptance Criteria**:
- [ ] All 28 instructions implemented
- [ ] All tests from C# instruction test suite ported
- [ ] 395+ tests passing
- [ ] Code coverage > 90% for instructions
- [ ] `cargo bench` shows > 100K instructions/second
- [ ] Zero clippy warnings

---

## Phase 3: Assembler Implementation (Weeks 9-10)

### Goal
Implement two-pass assembler matching C# functionality. Port assembler tests.

### Task 3.1: Lexer Implementation

**File**: `crates/s1130-core/src/assembler/lexer.rs`

**Tests First** (from `AssemblerTests.cs`):
```rust
#[test]
fn test_tokenize_hex_number() {
    let mut lexer = Lexer::new("/1234");
    let tokens = lexer.tokenize();
    assert_eq!(tokens, vec![Token::HexNumber(0x1234)]);
}

#[test]
fn test_tokenize_decimal_number() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize();
    assert_eq!(tokens, vec![Token::Number(42)]);
}

#[test]
fn test_tokenize_identifier() {
    let mut lexer = Lexer::new("LABEL");
    let tokens = lexer.tokenize();
    assert_eq!(tokens, vec![Token::Identifier("LABEL".to_string())]);
}
```

**Implementation**:
```rust
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        // Implementation
    }
}
```

**Acceptance Criteria**:
- [ ] All token types recognized
- [ ] Hex, octal, decimal numbers parsed
- [ ] Identifiers and operators tokenized
- [ ] Comments handled correctly

### Task 3.2: Two-Pass Assembler

**Pass 1: Symbol Collection**

**Tests**:
```rust
#[test]
fn test_pass1_builds_symbol_table() {
    let source = r#"
          ORG  /100
    START LD   L VALUE
    VALUE DC   42
    "#;

    let mut assembler = Assembler::new();
    assembler.pass1(source);

    assert_eq!(assembler.symbols.get("START"), Some(&0x100));
    assert_eq!(assembler.symbols.get("VALUE"), Some(&0x102));
    assert!(assembler.errors.is_empty());
}
```

**Pass 2: Code Generation**

**Tests** (from `AssemblerTests.cs`):
```rust
#[test]
fn test_assemble_simple_program() {
    let source = r#"
          ORG  /100
          LD   L /200
          A    L /201
          STO  L /202
          WAIT
    "#;

    let mut cpu = Cpu::new();
    let mut assembler = Assembler::new();
    let result = assembler.assemble(source, &mut cpu);

    assert!(result.success);
    assert_eq!(cpu.read_memory(0x100).unwrap(), 0xC400);  // LD L
    assert_eq!(cpu.read_memory(0x101).unwrap(), 0x0200);  // address
}
```

**Acceptance Criteria**:
- [ ] All directive tests pass (ORG, DC, EQU, BSS, BES)
- [ ] Symbol resolution works (forward references)
- [ ] Error reporting includes line numbers
- [ ] All tests from C# `AssemblerTests.cs` ported and passing
- [ ] Listing generation functional

---

## Phase 4: Device System (Weeks 11-13)

### Goal
Implement device trait and key devices. Port device tests.

### Task 4.1: Device Trait and Base

**File**: `crates/s1130-core/src/devices/mod.rs`

**Code**:
```rust
pub trait Device: Send + Sync {
    fn device_code(&self) -> u8;
    fn name(&self) -> &'static str;
    fn execute_iocc(&mut self, cpu: &mut Cpu) -> Result<(), DeviceError>;
    fn reset(&mut self);
    fn get_status(&self) -> DeviceStatus;
}
```

**Acceptance Criteria**:
- [ ] Device trait defined
- [ ] IOCC structure implemented
- [ ] Device attachment to CPU works

### Task 4.2: Implement Device2501 (Card Reader)

**Tests** (from `Device2501Tests.cs`):
```rust
#[test]
fn test_2501_reads_card() {
    let mut cpu = Cpu::new();
    let mut reader = Device2501::new();

    let mut card = Card::new();
    card.columns[0] = 0x9000;  // 'A' in card code
    reader.load_card(card);

    // Set up IOCC
    cpu.write_memory(0x300, 80).unwrap();  // Word count

    // Execute InitRead
    reader.execute_iocc(&mut cpu).unwrap();

    assert_eq!(cpu.read_memory(0x301).unwrap(), 0x9000);
}
```

**Acceptance Criteria**:
- [ ] All Device2501 tests ported
- [ ] Card reading functional
- [ ] Interrupts generated correctly

### Task 4.3-4.5: Implement Remaining Devices

**Task 4.3**: Device1442 (Card Punch)
**Task 4.4**: Device2310 (Disk Drive)
**Task 4.5**: DeviceConsoleKeyboard

**Acceptance Criteria** (overall):
- [ ] All device tests from C# ported
- [ ] Block-mode devices work (2501, 2310)
- [ ] Character-mode devices work (1442)
- [ ] Interrupt system integrated

---

## Phase 5: Interrupt System (Week 14)

### Goal
Implement 6-level interrupt system with proper priority handling.

### Task 5.1: Interrupt Manager

**File**: `crates/s1130-core/src/cpu/interrupt.rs`

**Tests**:
```rust
#[test]
fn test_interrupt_priority() {
    let mut cpu = Cpu::new();

    cpu.add_interrupt(Interrupt::new(4, 0x0800, 0x09));
    cpu.add_interrupt(Interrupt::new(0, 0x0001, 0x09));

    let next = cpu.get_next_interrupt();
    assert_eq!(next.unwrap().level, 0);  // Higher priority
}
```

**Acceptance Criteria**:
- [ ] Six interrupt queues (0-5)
- [ ] Priority-based handling
- [ ] Interrupt pooling implemented
- [ ] All interrupt tests pass

---

## Phase 6: WASM Bindings (Week 15)

### Goal
Create WASM bindings for browser use.

### Task 6.1: WASM Wrapper

**File**: `crates/s1130-wasm/src/lib.rs`

**Implementation**:
```rust
#[wasm_bindgen]
pub struct WasmCpu {
    inner: Cpu,
}

#[wasm_bindgen]
impl WasmCpu {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self { inner: Cpu::new() }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        self.inner.step()
            .map(|_| JsValue::NULL)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> JsValue {
        let state = self.inner.get_state();
        serde_wasm_bindgen::to_value(&state).unwrap()
    }
}
```

**Tests**:
```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_cpu_step() {
    let mut cpu = WasmCpu::new();
    // Build simple program
    // Test step
}
```

**Acceptance Criteria**:
- [ ] All core functions exposed to WASM
- [ ] Serialization works (Rust ↔ JS)
- [ ] WASM tests pass
- [ ] `wasm-pack build --release` succeeds

---

## Phase 7: Yew UI Components (Weeks 16-18)

### Goal
Build Yew components for emulator interaction.

### Task 7.1: Application Shell

**File**: `crates/s1130-ui/src/app.rs`

**Code**:
```rust
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <EmulatorProvider>
            <div class="app-container">
                <header>
                    <h1>{ "IBM 1130 Emulator" }</h1>
                </header>
                <main>
                    <AssemblerEditor />
                    <CpuConsole />
                    <ControlPanel />
                </main>
            </div>
        </EmulatorProvider>
    }
}
```

**Acceptance Criteria**:
- [ ] App renders in browser
- [ ] Layout is responsive
- [ ] No console errors

### Task 7.2: EmulatorContext (State Management)

**File**: `crates/s1130-ui/src/context/emulator.rs`

**Implementation**: See design.md

**Acceptance Criteria**:
- [ ] Context provides CPU access to all components
- [ ] Actions dispatch correctly
- [ ] State updates propagate

### Task 7.3: AssemblerEditor Component

**File**: `crates/s1130-ui/src/components/assembler_editor.rs`

**Features**:
- Textarea for code entry
- "Assemble" button
- Error display with line numbers

**Acceptance Criteria**:
- [ ] User can enter assembly code
- [ ] Assemble button calls WASM binding
- [ ] Errors display clearly

### Task 7.4: CpuConsole Component

**File**: `crates/s1130-ui/src/components/cpu_console.rs`

**Features**:
- Display all registers (IAR, ACC, EXT, XR1-3)
- Status flags (Carry, Overflow, Wait)
- Interrupt level indicator

**Acceptance Criteria**:
- [ ] Registers update in real-time
- [ ] Multiple formats (hex, octal, decimal)
- [ ] Bit display for visual feedback

### Task 7.5: ControlPanel Component

**Buttons**: Reset, Step, Run, Stop

**Acceptance Criteria**:
- [ ] All buttons functional
- [ ] Buttons disabled appropriately
- [ ] Run/Stop toggles correctly

### Task 7.6: MemoryViewer Component

**Features**:
- Display memory around IAR
- Paginated view
- Highlight current instruction

**Acceptance Criteria**:
- [ ] Memory displays correctly
- [ ] IAR row highlighted
- [ ] Pagination works

---

## Phase 8: Integration and Polish (Weeks 19-20)

### Goal
Integrate all components, test end-to-end, optimize performance.

### Task 8.1: End-to-End Testing

**Test Workflow**:
1. User enters simple program
2. Click Assemble → verify no errors
3. Click Step → verify registers update
4. Click Run → verify continuous execution
5. Click Stop → verify execution halts
6. Click Reset → verify state clears

**Acceptance Criteria**:
- [ ] Complete workflow functions correctly
- [ ] No browser console errors
- [ ] Performance acceptable (60 FPS)

### Task 8.2: Performance Optimization

**Tasks**:
- [ ] Profile with browser DevTools
- [ ] Optimize hot paths (instruction execution)
- [ ] Run `wasm-opt` on release build
- [ ] Benchmark: > 100K IPS in WASM

**Acceptance Criteria**:
- [ ] Lighthouse score > 90
- [ ] WASM binary < 2MB compressed
- [ ] Load time < 3 seconds

### Task 8.3: Documentation

**Create**:
- [ ] README.md (quick start, features)
- [ ] CONTRIBUTING.md (how to build, test, contribute)
- [ ] User guide (how to use emulator)
- [ ] Example programs (included in UI)

**Acceptance Criteria**:
- [ ] New user can build from source
- [ ] Documentation is clear and accurate

### Task 8.4: Deployment

**Steps**:
1. Configure GitHub Pages
2. Set up automated deployment on `main` branch push
3. Add custom domain (optional)
4. Configure cache headers

**Acceptance Criteria**:
- [ ] Deployed to GitHub Pages
- [ ] HTTPS enabled
- [ ] Publicly accessible

---

## Phase 9: Release (Week 21)

### Goal
Prepare for public release, announce project.

### Checklist

**Code Quality**:
- [ ] All tests passing (395+)
- [ ] Zero clippy warnings
- [ ] Code formatted with `cargo fmt`
- [ ] No `unsafe` blocks (or all documented)
- [ ] Code coverage > 80%

**Functionality**:
- [ ] All 28 instructions work
- [ ] Assembler compiles complex programs
- [ ] Devices respond correctly
- [ ] Interrupts handled properly
- [ ] UI fully functional

**Performance**:
- [ ] > 100K instructions/second
- [ ] WASM binary < 2MB
- [ ] 60 FPS during execution
- [ ] Load time < 3 seconds

**Documentation**:
- [ ] README complete
- [ ] API docs (rustdoc) generated
- [ ] User guide written
- [ ] Example programs included

**Deployment**:
- [ ] Deployed to GitHub Pages
- [ ] CI/CD pipeline working
- [ ] Security audit clean (`cargo audit`)

### Release Steps

1. **Tag Release**: `git tag -a v1.0.0 -m "Initial release"`
2. **Create GitHub Release**: Include changelog, binaries
3. **Announce**: Share on Reddit (r/rust, r/emulation), Hacker News
4. **Monitor**: Watch for issues, respond to feedback

---

## Appendix A: Testing Strategy

### Test Types

**Unit Tests**:
- One test file per source file
- Test all public functions
- Cover edge cases
- Target: > 90% line coverage

**Integration Tests**:
- Test cross-module interactions
- Assemble → Execute workflows
- Device I/O operations
- Target: All critical paths covered

**Property-Based Tests**:
- Use `proptest` for arithmetic operations
- Verify commutativity, associativity
- Fuzz inputs for robustness

**WASM Tests**:
- Use `wasm-bindgen-test`
- Test serialization
- Verify browser APIs work

### Test Commands

```bash
# Run all tests
cargo test --workspace

# Run tests with coverage
cargo tarpaulin --workspace --out Html

# Run benchmarks
cargo bench

# Run WASM tests
wasm-pack test --headless --chrome crates/s1130-wasm
```

---

## Appendix B: Code Quality Checklist

**Before Every Commit**:
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo build --release`

**Weekly**:
- [ ] `cargo audit`
- [ ] `cargo outdated`
- [ ] Review code coverage report
- [ ] Update dependencies if needed

---

## Appendix C: Milestones Summary

| Phase | Milestone | Duration | Completion Criteria |
|-------|-----------|----------|---------------------|
| 0 | Project Setup | Week 1 | Workspace builds, CI runs |
| 1 | CPU Core | Weeks 2-4 | CPU tests pass |
| 2 | Instructions | Weeks 5-8 | All 28 instructions, 395+ tests |
| 3 | Assembler | Weeks 9-10 | Assembler tests pass |
| 4 | Devices | Weeks 11-13 | Device tests pass |
| 5 | Interrupts | Week 14 | Interrupt tests pass |
| 6 | WASM | Week 15 | WASM builds, JS interop works |
| 7 | UI | Weeks 16-18 | All components functional |
| 8 | Integration | Weeks 19-20 | E2E tests pass, optimized |
| 9 | Release | Week 21 | Public release |

**Total Estimated Duration**: 21 weeks (5 months)

---

## Appendix D: Risk Mitigation

### Risk: Falling Behind Schedule

**Mitigation**:
- Prioritize critical features (CPU, instructions, assembler)
- Defer nice-to-have features (advanced UI, devices)
- Review progress weekly, adjust scope if needed

### Risk: Test Porting Takes Longer Than Expected

**Mitigation**:
- Focus on high-value tests first
- Use test automation tools where possible
- Accept lower coverage initially, add tests incrementally

### Risk: Performance Issues in WASM

**Mitigation**:
- Profile early and often
- Optimize hot paths first
- Use `wasm-opt` aggressively
- Consider Web Workers if single-threaded insufficient

---

## Summary

This implementation plan provides a detailed, phase-by-phase approach to porting the S1130 IBM 1130 emulator from C# to Rust + Yew (WASM). By following test-driven development, maintaining code quality standards, and working incrementally, the project will achieve:

- **Functional Parity**: All features from C# version
- **High Quality**: Comprehensive testing, zero warnings
- **Performance**: > 100K instructions/second in WASM
- **Maintainability**: Clear structure, excellent documentation
- **Accessibility**: Browser-based, no installation required

The plan is designed to be flexible, allowing for adjustments as the project progresses while maintaining focus on core objectives and quality standards.
