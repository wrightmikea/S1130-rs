# S1130 Rust + Yew Architecture

## Overview

This document describes the architecture for the Rust + Yew (WASM) port of the S1130 IBM 1130 emulator. The goal is to eliminate all C# and JavaScript code, implementing the complete emulator and user interface in Rust, compiled to WebAssembly for browser execution.

## Architecture Principles

### Core Design Goals

1. **Pure Rust Implementation**: All domain logic and presentation logic in Rust
2. **Zero JavaScript**: Leverage Yew framework for UI, compiled to WASM
3. **Maintainability**: Code quality via `cargo fmt`, `cargo clippy`, and comprehensive testing
4. **Test-Driven Development**: Red/Green testing methodology throughout
5. **Rust 2024 Edition**: Use latest stable Rust features and patterns
6. **Browser-Native**: Single-page application running entirely in the browser
7. **Faithful Emulation**: Maintain functional accuracy of IBM 1130 behavior

### Key Architectural Decisions

**Decision 1: Single WASM Binary**
- Entire application (emulator core + UI) compiles to one WASM module
- No server-side component required
- Static file hosting sufficient for deployment

**Decision 2: Yew for UI Framework**
- Component-based architecture similar to React
- Type-safe HTML/CSS via `html!` macro
- Message-based state management
- Efficient virtual DOM with WASM performance

**Decision 3: Shared State Management**
- `Rc<RefCell<Cpu>>` pattern for shared emulator state
- Yew contexts for global state distribution
- Message passing for component communication
- Immutable data structures where practical

**Decision 4: Module Organization**
- Workspace structure separating concerns:
  - `s1130-core`: Emulator logic (CPU, devices, assembler)
  - `s1130-wasm`: WASM bindings and utilities
  - `s1130-ui`: Yew components and application
- Clear separation of concerns and testability

## System Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser                               │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              WASM Runtime                              │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │           s1130-ui (Yew Application)             │ │ │
│  │  │  ┌────────────────────────────────────────────┐  │ │ │
│  │  │  │  App Component (Root)                      │  │ │ │
│  │  │  │  ├─ AssemblerEditor                        │  │ │ │
│  │  │  │  ├─ CpuConsole (Registers Display)         │  │ │ │
│  │  │  │  ├─ ControlPanel (Run/Step/Reset)          │  │ │ │
│  │  │  │  ├─ MemoryViewer                           │  │ │ │
│  │  │  │  └─ DevicePanel                            │  │ │ │
│  │  │  └────────────────────────────────────────────┘  │ │ │
│  │  │                      │                            │ │ │
│  │  │                      ▼                            │ │ │
│  │  │  ┌────────────────────────────────────────────┐  │ │ │
│  │  │  │     EmulatorContext (Shared State)         │  │ │ │
│  │  │  │     Rc<RefCell<EmulatorState>>             │  │ │ │
│  │  │  └────────────────────────────────────────────┘  │ │ │
│  │  │                      │                            │ │ │
│  │  └──────────────────────┼────────────────────────────┘ │ │
│  │                         ▼                              │ │
│  │  ┌──────────────────────────────────────────────────┐ │ │
│  │  │         s1130-core (Emulator Logic)              │ │ │
│  │  │  ┌────────────────────────────────────────────┐  │ │ │
│  │  │  │  Cpu                                       │  │ │ │
│  │  │  │  ├─ Registers (Acc, Ext, Iar, XR1-3)      │  │ │ │
│  │  │  │  ├─ Memory (Vec<u16>, 32K words)          │  │ │ │
│  │  │  │  ├─ InstructionSet (28 instructions)      │  │ │ │
│  │  │  │  ├─ InterruptSystem                        │  │ │ │
│  │  │  │  └─ DeviceController                       │  │ │ │
│  │  │  └────────────────────────────────────────────┘  │ │ │
│  │  │  ┌────────────────────────────────────────────┐  │ │ │
│  │  │  │  Assembler                                 │  │ │ │
│  │  │  │  ├─ Lexer (Tokenization)                   │  │ │ │
│  │  │  │  ├─ Parser (Two-pass assembly)             │  │ │ │
│  │  │  │  └─ CodeGenerator                          │  │ │ │
│  │  │  └────────────────────────────────────────────┘  │ │ │
│  │  │  ┌────────────────────────────────────────────┐  │ │ │
│  │  │  │  Devices                                   │  │ │ │
│  │  │  │  ├─ Device2501 (Card Reader)               │  │ │ │
│  │  │  │  ├─ Device1442 (Card Punch)                │  │ │ │
│  │  │  │  ├─ Device2310 (Disk Drive)                │  │ │ │
│  │  │  │  └─ DeviceConsoleKeyboard                  │  │ │ │
│  │  │  └────────────────────────────────────────────┘  │ │ │
│  │  └──────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

### Workspace Organization

```
S1130-rs/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── s1130-core/              # Emulator core library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── cpu/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── registers.rs
│   │   │   │   ├── memory.rs
│   │   │   │   └── interrupt.rs
│   │   │   ├── instructions/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── base.rs
│   │   │   │   ├── load.rs
│   │   │   │   ├── arithmetic.rs
│   │   │   │   ├── logical.rs
│   │   │   │   ├── shift.rs
│   │   │   │   ├── branch.rs
│   │   │   │   └── io.rs
│   │   │   ├── devices/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── base.rs
│   │   │   │   ├── device2501.rs
│   │   │   │   ├── device1442.rs
│   │   │   │   ├── device2310.rs
│   │   │   │   └── console.rs
│   │   │   ├── assembler/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── lexer.rs
│   │   │   │   ├── parser.rs
│   │   │   │   └── codegen.rs
│   │   │   └── utility/
│   │   │       ├── mod.rs
│   │   │       ├── conversion_codes.rs
│   │   │       └── ipl_cards.rs
│   │   └── tests/
│   │       ├── cpu_tests.rs
│   │       ├── instruction_tests/
│   │       ├── assembler_tests.rs
│   │       └── device_tests/
│   ├── s1130-wasm/              # WASM bindings
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── bindings.rs      # wasm-bindgen exports
│   │       └── utils.rs
│   └── s1130-ui/                # Yew frontend
│       ├── Cargo.toml
│       ├── index.html
│       ├── src/
│       │   ├── main.rs
│       │   ├── app.rs
│       │   ├── components/
│       │   │   ├── mod.rs
│       │   │   ├── assembler_editor.rs
│       │   │   ├── cpu_console.rs
│       │   │   ├── control_panel.rs
│       │   │   ├── register_display.rs
│       │   │   ├── memory_viewer.rs
│       │   │   └── device_panel.rs
│       │   ├── context/
│       │   │   ├── mod.rs
│       │   │   └── emulator.rs
│       │   └── services/
│       │       ├── mod.rs
│       │       └── execution.rs
│       └── static/
│           ├── styles.css
│           └── assets/
├── docs-rs/                     # Port documentation
└── target/                      # Build output
```

### Module Responsibilities

#### `s1130-core` (Emulator Core)

**Purpose**: Platform-agnostic IBM 1130 emulator implementation

**Key Traits**:
```rust
pub trait Cpu {
    fn step(&mut self) -> Result<(), CpuError>;
    fn execute_instruction(&mut self) -> Result<(), CpuError>;
    fn reset(&mut self);
    fn get_state(&self) -> CpuState;
}

pub trait Instruction {
    fn opcode(&self) -> OpCode;
    fn execute(&self, cpu: &mut dyn Cpu) -> Result<(), InstructionError>;
    fn has_long_format(&self) -> bool;
}

pub trait Device {
    fn device_code(&self) -> u8;
    fn execute_iocc(&mut self, cpu: &mut dyn Cpu) -> Result<(), DeviceError>;
    fn reset(&mut self);
}

pub trait Assembler {
    fn assemble(&mut self, source: &str) -> Result<AssemblyResult, AssemblerError>;
}
```

**Dependencies**:
- `std` collections (`Vec`, `HashMap`, etc.)
- `thiserror` for error handling
- No WASM-specific dependencies

**Testing**: Extensive unit tests mirroring C# test suite (395+ tests)

#### `s1130-wasm` (WASM Bindings)

**Purpose**: Bridge between Rust core and JavaScript/WASM runtime

**Key Exports**:
```rust
#[wasm_bindgen]
pub struct WasmCpu {
    inner: Cpu,
}

#[wasm_bindgen]
impl WasmCpu {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self;

    pub fn step(&mut self) -> Result<JsValue, JsValue>;
    pub fn reset(&mut self);
    pub fn get_state(&self) -> JsValue;
    pub fn assemble(&mut self, source: &str) -> JsValue;
}
```

**Dependencies**:
- `wasm-bindgen` for JS interop
- `serde-wasm-bindgen` for serialization
- `web-sys` for browser APIs
- `s1130-core` (local workspace dependency)

#### `s1130-ui` (Yew Application)

**Purpose**: Browser-based UI for emulator interaction

**Component Hierarchy**:
```
App (Root)
├─ EmulatorProvider (Context)
│   ├─ AssemblerEditor
│   │   ├─ CodeEditor (textarea)
│   │   └─ ErrorDisplay
│   ├─ CpuConsole
│   │   ├─ RegisterDisplay (IAR)
│   │   ├─ RegisterDisplay (ACC)
│   │   ├─ RegisterDisplay (EXT)
│   │   ├─ RegisterDisplay (XR1-3)
│   │   └─ StatusFlags (Carry, Overflow, Wait)
│   ├─ ControlPanel
│   │   ├─ ResetButton
│   │   ├─ StepButton
│   │   ├─ RunButton
│   │   └─ StopButton
│   ├─ MemoryViewer
│   │   └─ MemoryTable
│   └─ DevicePanel
│       └─ DeviceCard (for each device)
```

**State Management**:
```rust
pub struct EmulatorState {
    cpu: Rc<RefCell<WasmCpu>>,
    is_running: bool,
    execution_handle: Option<IntervalHandle>,
}

pub enum EmulatorAction {
    Reset,
    Step,
    Run { instructions_per_second: u32 },
    Stop,
    Assemble { source_code: String },
    UpdateState,
}
```

**Dependencies**:
- `yew` (UI framework)
- `wasm-bindgen` (JS interop)
- `web-sys` (browser APIs)
- `gloo` (utilities for WASM)
- `s1130-wasm` (emulator bindings)

## Data Flow Architecture

### Execution Flow

```
User Action (Button Click)
    │
    ▼
Yew Component (emit message)
    │
    ▼
Message Dispatch (update method)
    │
    ▼
EmulatorContext (modify shared state)
    │
    ▼
s1130-wasm (call WASM binding)
    │
    ▼
s1130-core (execute emulator logic)
    │
    ▼
Return Result
    │
    ▼
Update Component State
    │
    ▼
Yew Virtual DOM Diff
    │
    ▼
Browser DOM Update
```

### State Update Flow

**Synchronous Updates (Step, Reset)**:
1. User clicks "Step" button
2. `ControlPanel` component emits `EmulatorAction::Step`
3. `EmulatorProvider` context handles action
4. Calls `cpu.step()` via WASM binding
5. Retrieves updated `CpuState`
6. Notifies all subscribed components
7. Components re-render with new state

**Asynchronous Updates (Run)**:
1. User clicks "Run" button
2. `ControlPanel` emits `EmulatorAction::Run { ips: 1000 }`
3. `EmulatorProvider` creates interval using `gloo::timers::callback::Interval`
4. Interval callback executes steps at specified rate
5. After each step, broadcasts state update
6. Components update in real-time
7. "Stop" button or WAIT instruction cancels interval

### Assembler Flow

```
User enters assembly code
    │
    ▼
AssemblerEditor component
    │
    ▼
EmulatorAction::Assemble { source_code }
    │
    ▼
EmulatorProvider context
    │
    ▼
cpu.assemble(source_code)
    │
    ▼
Assembler::assemble (two-pass)
    │
    ├─ Pass 1: Build symbol table
    │
    └─ Pass 2: Generate machine code
    │
    ▼
Write to cpu.memory
    │
    ▼
Return AssemblyResult { success, errors, symbols }
    │
    ▼
Update AssemblerEditor state
    │
    ▼
Display errors or success message
```

## Memory Model

### Rust Memory Safety

**Key Patterns**:

1. **Ownership**: CPU owns memory array, instructions don't copy data
2. **Borrowing**: Devices borrow mutable reference to CPU during I/O
3. **Interior Mutability**: `RefCell<Cpu>` for shared mutable access in UI
4. **Reference Counting**: `Rc<RefCell<Cpu>>` for multi-component access

**Example**:
```rust
// EmulatorProvider holds Rc<RefCell<Cpu>>
let cpu = Rc::new(RefCell::new(Cpu::new()));

// Components receive Rc clone
let cpu_clone = cpu.clone();

// Mutable access when needed
cpu.borrow_mut().step()?;

// Immutable access for display
let state = cpu.borrow().get_state();
```

### WASM Memory

**Linear Memory**: All Rust data structures stored in WASM linear memory

**Pointer Safety**: No raw pointers exposed to JavaScript

**Serialization**: `serde` for complex type conversion between Rust and JS

## Error Handling

### Error Taxonomy

```rust
#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("Invalid instruction at address {0:#06x}")]
    InvalidInstruction(u16),

    #[error("Memory access violation at {0:#06x}")]
    MemoryViolation(u16),

    #[error("Device error: {0}")]
    DeviceError(#[from] DeviceError),

    #[error("Execution stopped by WAIT instruction")]
    WaitState,
}

#[derive(Debug, thiserror::Error)]
pub enum AssemblerError {
    #[error("Syntax error on line {line}: {message}")]
    SyntaxError { line: usize, message: String },

    #[error("Undefined symbol: {0}")]
    UndefinedSymbol(String),

    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),
}
```

### Error Propagation

**Core Library**: Use `Result<T, E>` throughout
**WASM Boundary**: Convert to `JsValue` via `serde_wasm_bindgen`
**UI Layer**: Display user-friendly error messages in components

## Performance Considerations

### Optimization Strategies

1. **Zero-Copy Operations**: Avoid unnecessary clones, use references
2. **Inline Hot Paths**: `#[inline]` on instruction execution
3. **Lazy Evaluation**: Defer expensive operations until needed
4. **Efficient Collections**: Use `Vec` over `HashMap` where appropriate
5. **WASM Optimization**: `wasm-opt` for size and speed

### Benchmarking

**Target Performance**:
- CPU instruction execution: 100K+ instructions/second in WASM
- UI responsiveness: 60 FPS during continuous run
- Memory viewer: Render 1000+ words without lag
- Assembler: < 100ms for typical programs (< 500 lines)

**Measurement Tools**:
- `criterion` for Rust benchmarks
- Browser DevTools Performance tab for WASM
- `wasm-pack test` for integration benchmarks

## Browser Compatibility

### Target Browsers

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

### Required Features

- **WebAssembly MVP**: All target browsers support
- **ES Modules**: For `wasm-bindgen` output
- **SharedArrayBuffer**: Not required (no threading)

### Graceful Degradation

- Detect WASM support, show error if unavailable
- Fallback message for unsupported browsers
- Progressive enhancement for optional features

## Build and Deployment

### Development Build

```bash
# Install dependencies
cargo install trunk wasm-pack

# Dev server with hot reload
trunk serve --open
```

### Production Build

```bash
# Build optimized WASM
trunk build --release

# Output: dist/
#   index.html
#   s1130-ui-*.wasm (optimized)
#   s1130-ui-*.js (glue code)
```

### Deployment Options

1. **Static Hosting**: GitHub Pages, Netlify, Vercel
2. **CDN**: CloudFlare, AWS S3 + CloudFront
3. **Self-Hosted**: nginx, Apache

**Configuration**:
- WASM MIME type: `application/wasm`
- Gzip/Brotli compression for `.wasm` files
- Cache headers for immutable assets

## Testing Strategy

### Unit Tests (`s1130-core`)

- Test each instruction independently
- Mock CPU state for isolated testing
- Cover all addressing modes (short, long, indexed, indirect)
- Test edge cases (overflow, carry, interrupts)

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_add_short_format_no_overflow() {
        let mut cpu = Cpu::new();
        cpu.acc = 0x1234;
        cpu[0x105] = 0x0042;

        let add = Add;
        build_short_instruction(&add, 0, 5, &mut cpu, 0x100);

        cpu.next_instruction();
        cpu.execute_instruction().unwrap();

        assert_eq!(cpu.acc, 0x1276);
        assert!(!cpu.carry);
        assert!(!cpu.overflow);
    }
}
```

### Integration Tests (`s1130-wasm`)

- Test WASM bindings with `wasm-pack test`
- Verify serialization between Rust and JS
- Test error handling across boundary

### End-to-End Tests (`s1130-ui`)

- Use `wasm-bindgen-test` for component tests
- Test user workflows (assemble → run → inspect)
- Visual regression tests with screenshot comparison

### Test Coverage Goals

- Core library: 90%+ line coverage
- WASM bindings: 80%+ (hard to test some browser APIs)
- UI components: 70%+ (focus on logic, not styling)

## Security Considerations

### WASM Sandbox

- No file system access
- No network access (unless explicitly enabled via `web-sys`)
- Runs in browser security context

### Input Validation

- Validate all user input (assembly source, memory addresses)
- Bounds checking on memory access
- Sanitize error messages to prevent XSS

### Dependency Management

- Regular `cargo audit` for vulnerabilities
- Pin dependencies in `Cargo.lock`
- Review all dependencies for supply chain security

## Future Enhancements

### Potential Features

1. **Offline Support**: Service worker for PWA capabilities
2. **Local Storage**: Persist programs between sessions
3. **Export/Import**: Save/load emulator state as JSON
4. **Debugger**: Breakpoints, watchpoints, call stack
5. **Disassembler**: Convert memory back to assembly
6. **Performance Profiling**: Instruction histogram, hotspots
7. **Multi-Threading**: Web Workers for background execution
8. **WebGPU**: Accelerated rendering for large displays

### Extensibility

- Plugin system for custom devices
- Macro support in assembler
- Scripting interface for automated testing
- HTTP API via `wasm-bindgen-futures` for server integration

## Comparison to C# Implementation

### Advantages of Rust + Yew

1. **Memory Safety**: No null references, no garbage collection pauses
2. **Performance**: Near-native speed, smaller binary size
3. **Type Safety**: Stronger type system, compile-time guarantees
4. **Concurrency**: Fearless concurrency with ownership model
5. **Deployment**: Static files only, no server required
6. **Portability**: Runs in any modern browser

### Challenges

1. **Learning Curve**: Steeper than C#/JavaScript
2. **Ecosystem Maturity**: WASM/Yew less mature than ASP.NET/React
3. **Debugging**: WASM debugging still improving
4. **Build Time**: Rust compile times longer than C#
5. **Interop**: Crossing WASM boundary has overhead

### Migration Path

- Port core emulator first (testable in Rust, no UI)
- Add WASM bindings incrementally
- Build UI components one at a time
- Maintain functional parity with C# version throughout

## Summary

The Rust + Yew architecture provides a robust, performant, and maintainable foundation for the S1130 emulator. By leveraging Rust's safety guarantees and Yew's component model, we achieve a modern web application with no JavaScript dependencies, running entirely in the browser as WebAssembly.

Key architectural benefits:
- **Type Safety**: Compile-time verification of correctness
- **Performance**: Near-native execution speed in WASM
- **Maintainability**: Clear module boundaries, comprehensive testing
- **Portability**: Runs anywhere modern browsers are available
- **Security**: WASM sandbox provides strong isolation

This architecture positions the project for long-term success while maintaining the functional accuracy and educational value of the original IBM 1130 emulator.
