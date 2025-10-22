# Product Requirements Document: S1130 Rust + Yew Port

## Executive Summary

### Project Name
S1130 Rust Port - IBM 1130 Emulator in Pure Rust + WebAssembly

### Project Goal
Port the existing S1130 IBM 1130 emulator from C# + planned JavaScript frontend to a pure Rust implementation with Yew (WASM) frontend, eliminating all C# and JavaScript code while maintaining full functional accuracy and educational value.

### Target Audience
- Computer history enthusiasts
- Students learning computer architecture
- Educators teaching historical computing systems
- Retro computing hobbyists
- Software engineers interested in emulation

### Success Criteria
1. **Functional Parity**: 100% of C# emulator features implemented in Rust
2. **Test Coverage**: All 395+ unit tests ported and passing
3. **Code Quality**: Zero `cargo clippy` warnings, formatted with `cargo fmt`
4. **Performance**: Execute 100,000+ instructions/second in WASM
5. **Browser Compatibility**: Works in Chrome, Firefox, Safari (latest versions)
6. **Zero External Dependencies**: No JavaScript or C# code in final artifact

## Background

### Current State

**Existing Implementation**:
- **Backend**: C# .NET 8 library (11,370 lines)
- **Frontend**: Planned ASP.NET Core + React (not implemented)
- **Testing**: xUnit with 395+ tests, comprehensive coverage
- **Deployment**: Requires .NET runtime, server infrastructure

**Key Features**:
- Complete IBM 1130 CPU emulation (28 instructions)
- Full two-pass assembler
- I/O device simulation (card reader, disk, printer, keyboard)
- Interrupt system (6 priority levels)
- Debugging capabilities (breakpoints, memory inspection)

### Motivation for Port

**Technical Reasons**:
1. **Browser-Native**: Run entirely in browser, no server required
2. **Performance**: Near-native speed with WebAssembly
3. **Memory Safety**: Rust's ownership model prevents entire classes of bugs
4. **Portability**: Static file deployment, works anywhere
5. **Modern Toolchain**: Leverage Rust 2024 edition features

**Strategic Reasons**:
1. **Accessibility**: Lower barrier to entry (just open a URL)
2. **Maintenance**: Single codebase (no backend/frontend split)
3. **Community**: Engage Rust community, educational resource
4. **Longevity**: WASM is a W3C standard, future-proof

## Requirements

### Functional Requirements

#### FR-1: CPU Emulation

**Priority**: Critical

**Description**: Implement complete IBM 1130 CPU with all registers, memory, and execution logic.

**Requirements**:
- FR-1.1: 16-bit accumulator (ACC)
- FR-1.2: 16-bit extension register (EXT)
- FR-1.3: 16-bit instruction address register (IAR)
- FR-1.4: Three 16-bit index registers (XR1, XR2, XR3)
- FR-1.5: Status flags (Carry, Overflow, Wait)
- FR-1.6: Configurable memory (default 32,768 words)
- FR-1.7: Memory-mapped index registers (addresses 0x0001-0x0003)
- FR-1.8: Instruction counter for performance metrics

**Acceptance Criteria**:
- All CPU unit tests from C# version pass
- Memory access is bounds-checked
- Register state is observable via public API

#### FR-2: Instruction Set

**Priority**: Critical

**Description**: Implement all 28 IBM 1130 instructions with correct behavior.

**Instruction Categories**:
- Load/Store: LD, LDD, STO, STD, LDX, STX, LDS, STS
- Arithmetic: A, AD, S, SD, M, D
- Logical: AND, OR, EOR
- Shift: SLA, SRA, SLT, SRT, SLC, SLCA
- Branch: BSC, BSI, MDX
- I/O: XIO
- Control: WAIT

**Requirements**:
- FR-2.1: Short format (1 word, 8-bit signed displacement)
- FR-2.2: Long format (2 words, 16-bit address)
- FR-2.3: Indexed addressing (tag field 1-3)
- FR-2.4: Indirect addressing (I bit)
- FR-2.5: Correct flag updates (Carry, Overflow)
- FR-2.6: Proper sign extension for short format

**Acceptance Criteria**:
- Each instruction has dedicated test file
- All addressing modes tested
- Edge cases covered (overflow, underflow, etc.)

#### FR-3: Assembler

**Priority**: Critical

**Description**: Two-pass assembler supporting IBM 1130 assembly language.

**Requirements**:
- FR-3.1: Lexer for tokenization
- FR-3.2: Two-pass assembly (symbol collection, code generation)
- FR-3.3: Directives: ORG, DC, EQU, BSS, BES
- FR-3.4: Symbolic labels with forward references
- FR-3.5: Expression evaluation (current address operator `*`)
- FR-3.6: Error reporting with line numbers
- FR-3.7: Assembly listing generation
- FR-3.8: Symbol table export

**Acceptance Criteria**:
- All assembler tests from C# version pass (39,478 bytes of tests)
- Errors include line numbers and clear messages
- Listing format matches IBM 1130 conventions

#### FR-4: Device Emulation

**Priority**: High

**Description**: Simulate I/O devices for realistic program execution.

**Devices**:
- Device2501 (Card Reader, code 0x09)
- Device1442 (Card Punch, code 0x0A)
- Device2310 (Disk Drive, code 0x06)
- DeviceConsoleKeyboard
- Device1053 (Console Printer)

**Requirements**:
- FR-4.1: Device trait with execute_iocc() method
- FR-4.2: IOCC (I/O Channel Command) structure support
- FR-4.3: Block-mode devices (2501, 2310)
- FR-4.4: Character-mode devices (1442)
- FR-4.5: Interrupt generation on completion
- FR-4.6: Status word management

**Acceptance Criteria**:
- All device tests from C# version pass
- Devices can be attached/detached dynamically
- Interrupt timing is functionally correct

#### FR-5: Interrupt System

**Priority**: High

**Description**: Six-level priority interrupt system.

**Requirements**:
- FR-5.1: Six interrupt levels (0-5)
- FR-5.2: Interrupt vectors at addresses 0x0008-0x000D
- FR-5.3: FIFO queue per interrupt level
- FR-5.4: Interrupt pooling for efficiency
- FR-5.5: Nested interrupt support
- FR-5.6: ILSW (Interrupt Level Status Word) per device

**Acceptance Criteria**:
- Interrupt tests pass
- Higher priority interrupts preempt lower priority
- Interrupt return (BOSC) restores state correctly

#### FR-6: User Interface

**Priority**: Critical

**Description**: Yew-based web interface for emulator interaction.

**Components**:
- FR-6.1: AssemblerEditor (textarea, assemble button, error display)
- FR-6.2: CpuConsole (register display with live updates)
- FR-6.3: ControlPanel (Reset, Step, Run, Stop buttons)
- FR-6.4: MemoryViewer (paginated memory inspection)
- FR-6.5: DevicePanel (device status cards)

**Requirements**:
- FR-6.6: Real-time register updates during execution
- FR-6.7: Syntax highlighting in assembler editor (optional)
- FR-6.8: Responsive layout (desktop and tablet)
- FR-6.9: Keyboard shortcuts (Space=Step, R=Run, S=Stop)
- FR-6.10: Error messages displayed inline

**Acceptance Criteria**:
- Complete workflow: Write code → Assemble → Run → Inspect
- UI updates at 60 FPS during continuous execution
- Mobile devices show readable content (graceful degradation)

### Non-Functional Requirements

#### NFR-1: Performance

**Requirements**:
- NFR-1.1: Execute 100,000+ instructions/second in WASM
- NFR-1.2: Assembler completes < 100ms for 500-line programs
- NFR-1.3: UI renders at 60 FPS during continuous run
- NFR-1.4: WASM binary size < 2 MB (compressed)
- NFR-1.5: Initial load time < 3 seconds on broadband

**Measurement**:
- `criterion` benchmarks for core library
- Browser DevTools Performance tab for UI
- Lighthouse score > 90 for performance

#### NFR-2: Code Quality

**Requirements**:
- NFR-2.1: Zero `cargo clippy` warnings
- NFR-2.2: All code formatted with `cargo fmt`
- NFR-2.3: No `unsafe` blocks except where absolutely necessary (document all)
- NFR-2.4: Public APIs have rustdoc comments
- NFR-2.5: Test coverage > 80% line coverage

**Enforcement**:
- CI pipeline runs clippy and fmt checks
- PRs rejected if warnings present
- `cargo tarpaulin` for coverage reporting

#### NFR-3: Testing

**Requirements**:
- NFR-3.1: All 395+ C# unit tests ported to Rust
- NFR-3.2: Red/Green TDD methodology followed
- NFR-3.3: Integration tests for WASM bindings
- NFR-3.4: Property-based tests for instruction correctness (using `proptest`)
- NFR-3.5: Regression tests for bug fixes

**Acceptance Criteria**:
- `cargo test --all` passes 100% of tests
- Test execution time < 10 seconds
- Tests run in CI on every commit

#### NFR-4: Browser Compatibility

**Requirements**:
- NFR-4.1: Chrome 90+ (stable)
- NFR-4.2: Firefox 88+ (stable)
- NFR-4.3: Safari 14+ (stable)
- NFR-4.4: Edge 90+ (stable)
- NFR-4.5: Detect WASM support, show error if unavailable

**Testing**:
- Manual testing on all target browsers
- BrowserStack for automated cross-browser testing

#### NFR-5: Maintainability

**Requirements**:
- NFR-5.1: Clear module boundaries (core, wasm, ui)
- NFR-5.2: Dependency versions pinned in Cargo.lock
- NFR-5.3: No circular dependencies
- NFR-5.4: Maximum cyclomatic complexity: 15 per function
- NFR-5.5: README with quick start, architecture overview

**Metrics**:
- `cargo-geiger` for unsafe code audit
- `cargo-bloat` for binary size analysis
- `cargo-outdated` for dependency management

### Out of Scope (Future Enhancements)

The following are **not** required for initial release:

1. **Cycle-Accurate Timing**: Functional emulation only, not timing-accurate
2. **DMS Operating System**: Emulator only, no OS bootstrap
3. **Networking**: No remote debugging or multi-user features
4. **File System**: No local file save/load (use localStorage for v1)
5. **Audio**: No sound effects for devices
6. **Mobile Apps**: Web-only, no native mobile apps
7. **Multi-Threading**: Single-threaded execution (Web Workers future)
8. **Historical Accuracy**: Colors, fonts, exact hardware appearance

## User Stories

### US-1: First-Time User

**As a** computer history enthusiast
**I want to** run a simple IBM 1130 program
**So that** I can understand how 1960s computers worked

**Acceptance Criteria**:
- Visit URL, emulator loads in < 5 seconds
- Example program pre-loaded in editor
- Click "Assemble" → "Run" → see results
- No errors, no installation, no configuration

### US-2: Student Learning Assembly

**As a** computer science student
**I want to** write and debug IBM 1130 assembly code
**So that** I can learn low-level programming concepts

**Acceptance Criteria**:
- Type assembly code in editor
- Click "Assemble" to compile
- See clear error messages if code is invalid
- Step through execution line by line
- Inspect registers and memory at each step

### US-3: Educator Teaching Architecture

**As an** instructor teaching computer architecture
**I want to** demonstrate CPU internals with live examples
**So that** students can see registers change in real-time

**Acceptance Criteria**:
- Load example program demonstrating concept (e.g., subroutines)
- Run program, show register updates visually
- Pause at key points to explain
- Reset and re-run without refreshing page

### US-4: Developer Porting Software

**As a** retro computing developer
**I want to** port historical IBM 1130 programs to the emulator
**So that** I can preserve and share vintage software

**Acceptance Criteria**:
- Paste existing assembly source code
- Assemble without modification
- Run program and verify output
- Export memory state for later analysis

### US-5: Researcher Studying Emulation

**As a** researcher studying emulator design
**I want to** examine the source code and architecture
**So that** I can learn emulation techniques

**Acceptance Criteria**:
- Access source code on GitHub
- Read architecture documentation
- Understand module boundaries
- Build project locally, run tests
- Contribute improvements via PRs

## Technical Constraints

### TC-1: Technology Stack

**Mandatory**:
- Rust 2024 Edition
- Yew 0.21+ (latest stable)
- wasm-bindgen for WASM bindings
- trunk for build tooling

**Prohibited**:
- No JavaScript (except generated glue code from wasm-bindgen)
- No C# (completely removed in port)
- No server-side components (static hosting only)

### TC-2: Browser APIs

**Allowed**:
- `web-sys` for DOM manipulation
- `gloo` for browser utilities (timers, events)
- LocalStorage for persistence

**Restricted**:
- No network requests (no fetch/XHR)
- No WebGL (future enhancement)
- No WebRTC (not needed)

### TC-3: Build System

**Requirements**:
- Single `trunk build --release` command
- Output to `dist/` directory
- Source maps for debugging (dev build)
- Optimized WASM (wasm-opt in release)

### TC-4: Deployment

**Targets**:
- Static file hosting (GitHub Pages, Netlify, Vercel)
- CDN-friendly (immutable assets, cache headers)
- HTTPS required (WASM security)

## Success Metrics

### Quantitative Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Pass Rate | 100% | `cargo test --all` |
| Code Coverage | > 80% | `cargo tarpaulin` |
| Clippy Warnings | 0 | `cargo clippy --all-targets` |
| Performance (IPS) | > 100,000 | `criterion` benchmarks |
| WASM Size (gzip) | < 500 KB | `wasm-pack build --release` |
| Lighthouse Score | > 90 | Chrome DevTools |
| Browser Support | 4/4 | Manual testing |
| Load Time (3G) | < 5s | Chrome DevTools throttling |

### Qualitative Metrics

| Metric | Success Criteria |
|--------|------------------|
| Code Readability | New contributor can understand module in < 30 min |
| Documentation | All public APIs have rustdoc, README complete |
| User Experience | Non-technical user can run program in < 2 min |
| Error Messages | User can fix error without reading docs |
| Community Engagement | > 10 stars on GitHub within 3 months |

## Risks and Mitigations

### Risk 1: WASM Performance

**Risk**: WASM may be slower than native .NET, impacting user experience.

**Likelihood**: Medium
**Impact**: High

**Mitigation**:
- Profile early and often with `criterion`
- Optimize hot paths (instruction execution loop)
- Use `wasm-opt` for size and speed
- Benchmark against C# version to validate

**Fallback**: If performance inadequate, consider Rust native app (not browser-based)

### Risk 2: Yew Ecosystem Maturity

**Risk**: Yew is less mature than React, may lack features or have bugs.

**Likelihood**: Medium
**Impact**: Medium

**Mitigation**:
- Use stable Yew releases only
- Test thoroughly across browsers
- Engage with Yew community for support
- Keep UI simple to minimize framework dependency

**Fallback**: Migrate to alternative (e.g., Leptos, Dioxus) if Yew proves inadequate

### Risk 3: Testing Complexity

**Risk**: Porting 395+ tests is time-consuming and error-prone.

**Likelihood**: High
**Impact**: Medium

**Mitigation**:
- Prioritize tests by criticality (CPU first, devices later)
- Automate test generation where possible
- Use TDD to catch regressions early
- Incremental approach: port tests as features implemented

**Fallback**: Accept lower initial coverage, add tests as bugs found

### Risk 4: Browser Compatibility Issues

**Risk**: WASM/JS interop may behave differently across browsers.

**Likelihood**: Low
**Impact**: High

**Mitigation**:
- Test on all target browsers regularly
- Use feature detection, not browser detection
- Polyfill missing APIs if needed
- Continuous testing in CI with BrowserStack

**Fallback**: Document unsupported browsers, recommend Chrome/Firefox

### Risk 5: Learning Curve

**Risk**: Team unfamiliar with Rust or WASM, slowing development.

**Likelihood**: Medium (depends on team experience)
**Impact**: Medium

**Mitigation**:
- Allocate time for learning Rust basics
- Pair programming for knowledge sharing
- Code reviews to enforce best practices
- Reference existing Rust emulators (chip8, NES)

**Fallback**: Hire Rust consultant for critical phases

## Timeline and Milestones

### Phase 1: Foundation (Weeks 1-4)

**Deliverables**:
- Workspace setup (`s1130-core`, `s1130-wasm`, `s1130-ui`)
- CPU core implementation
- Basic instruction set (Load, Store, Add, Subtract)
- Unit tests for core functionality
- CI pipeline (GitHub Actions)

**Success Criteria**:
- 50+ tests passing
- CPU can execute simple programs
- `cargo build --release` succeeds

### Phase 2: Complete Emulator (Weeks 5-10)

**Deliverables**:
- All 28 instructions implemented
- Two-pass assembler complete
- Device emulation (2501, 1442, 2310)
- Interrupt system functional
- All C# tests ported and passing

**Success Criteria**:
- 395+ tests passing
- Assembler can compile non-trivial programs
- Devices respond to XIO instructions

### Phase 3: WASM Integration (Weeks 11-13)

**Deliverables**:
- WASM bindings for CPU and assembler
- Serialization/deserialization for JS interop
- WASM-specific tests
- Performance benchmarks

**Success Criteria**:
- `wasm-pack build --release` succeeds
- Emulator callable from JavaScript
- Benchmarks show > 100K IPS

### Phase 4: User Interface (Weeks 14-18)

**Deliverables**:
- Yew components (AssemblerEditor, CpuConsole, ControlPanel)
- EmulatorContext for state management
- Real-time register updates
- Responsive layout

**Success Criteria**:
- Complete user workflow functional
- UI renders at 60 FPS
- Works on Chrome, Firefox, Safari

### Phase 5: Polish and Release (Weeks 19-20)

**Deliverables**:
- Documentation (README, architecture.md, user guide)
- Example programs included
- Performance optimization
- Browser testing complete
- Deployment to GitHub Pages

**Success Criteria**:
- All acceptance criteria met
- Zero critical bugs
- Lighthouse score > 90
- Positive user feedback

## Acceptance Criteria

### Emulator Core

- [ ] All 28 instructions implemented and tested
- [ ] 395+ unit tests passing
- [ ] Assembler compiles complex programs (e.g., subroutines, loops)
- [ ] Devices respond correctly to I/O operations
- [ ] Interrupts handled at correct priority levels
- [ ] Memory bounds checked, no panics

### User Interface

- [ ] User can enter assembly code in editor
- [ ] Assemble button compiles code, displays errors
- [ ] Step button executes one instruction
- [ ] Run button starts continuous execution
- [ ] Stop button halts execution
- [ ] Reset button clears CPU and memory
- [ ] Registers display updates in real-time
- [ ] Memory viewer shows current IAR location
- [ ] Device panel shows status of attached devices

### Code Quality

- [ ] Zero `cargo clippy` warnings
- [ ] All code formatted with `cargo fmt`
- [ ] No `unsafe` blocks (or all documented)
- [ ] Test coverage > 80%
- [ ] Public APIs have rustdoc
- [ ] README explains how to build and run

### Performance

- [ ] Execute > 100,000 instructions/second
- [ ] Assemble 500-line program in < 100ms
- [ ] WASM binary < 2 MB (compressed)
- [ ] UI renders at 60 FPS
- [ ] Initial load < 3 seconds

### Browser Compatibility

- [ ] Works in Chrome 90+
- [ ] Works in Firefox 88+
- [ ] Works in Safari 14+
- [ ] Works in Edge 90+
- [ ] Graceful error for unsupported browsers

### Deployment

- [ ] `trunk build --release` produces optimized build
- [ ] Deployed to GitHub Pages
- [ ] HTTPS enabled
- [ ] Cache headers configured
- [ ] No server-side components required

## Appendices

### Appendix A: Glossary

- **ACC**: Accumulator, primary 16-bit register
- **EXT**: Extension register, used for 32-bit operations
- **IAR**: Instruction Address Register (program counter)
- **IOCC**: I/O Channel Command, structure for device operations
- **ILSW**: Interrupt Level Status Word
- **XIO**: Execute I/O instruction
- **WASM**: WebAssembly, binary format for browsers
- **Yew**: Rust framework for building web UIs

### Appendix B: References

- [IBM 1130 Functional Characteristics](http://bitsavers.org/pdf/ibm/1130/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Yew Documentation](https://yew.rs/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Original C# Implementation](../src/S1130.SystemObjects/)

### Appendix C: Related Work

- **Rust IBM 1130 Emulators**: None found (this is first)
- **Other Rust Emulators**: chip8-rs, nes-rs, gameboy-rs
- **WASM Emulators**: em-dosbox, v86 (x86), jslinux

### Appendix D: Open Questions

1. Should we support DMS (Disk Monitor System) in v1?
   - **Decision**: Deferred to v2, focus on assembler workflow
2. Should we include assembler macros?
   - **Decision**: Deferred to v2, basic assembly only
3. Should we support file upload/download?
   - **Decision**: Use LocalStorage for v1, file I/O in v2
4. Should we target mobile devices?
   - **Decision**: Desktop first, mobile graceful degradation

## Sign-Off

This PRD represents the agreed-upon scope and requirements for the S1130 Rust + Yew port. Changes to requirements must be documented and approved.

**Product Owner**: _[Your Name]_
**Engineering Lead**: _[Your Name]_
**Date**: 2025-10-22
