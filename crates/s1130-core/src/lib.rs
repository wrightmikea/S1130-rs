//! S1130 Core - IBM 1130 Emulator Library
//!
//! This crate provides a complete emulation of the IBM 1130 minicomputer,
//! including the CPU, instruction set, assembler, and I/O devices.
//!
//! # Architecture
//!
//! - **CPU**: 16-bit processor with accumulator, extension, and index registers
//! - **Memory**: 32K words (configurable)
//! - **Instructions**: Complete 28-instruction set
//! - **Assembler**: Two-pass assembler with full IBM 1130 syntax support
//! - **Devices**: I/O device emulation (card reader, disk, etc.)
//!
//! # Example
//!
//! ```no_run
//! use s1130_core::Cpu;
//!
//! let mut cpu = Cpu::new();
//! // Load program into memory
//! // Execute instructions
//! ```

pub mod assembler;
pub mod cpu;
pub mod devices;
pub mod error;
pub mod instructions;

// Re-export commonly used types
pub use cpu::{Cpu, CpuState};
pub use error::{AssemblerError, CpuError, DeviceError, InstructionError, Result};
pub use instructions::{InstructionFormat, InstructionInfo, OpCode};
