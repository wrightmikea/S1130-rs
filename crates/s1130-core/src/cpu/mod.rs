//! IBM 1130 CPU Module
//!
//! This module orchestrates the CPU components:
//! - Registers (accumulator, extension, index registers, flags)
//! - Memory (word-addressable, 32K default)
//! - State snapshots for external observation

pub mod memory;
pub mod registers;
pub mod state;

pub use memory::Memory;
pub use registers::{IndexRegisters, StatusFlags};
pub use state::CpuState;

use crate::error::Result;

/// IBM 1130 Central Processing Unit
///
/// The CPU coordinates execution of instructions, manages registers,
/// and provides access to memory. It maintains minimal state and delegates
/// responsibilities to focused submodules.
pub struct Cpu {
    /// Main accumulator (16-bit)
    acc: u16,

    /// Extension register (16-bit, for 32-bit operations)
    ext: u16,

    /// Instruction Address Register (program counter)
    iar: u16,

    /// Index registers (XR1, XR2, XR3)
    index_registers: IndexRegisters,

    /// Status flags (carry, overflow, wait)
    status_flags: StatusFlags,

    /// Main memory
    memory: Memory,

    /// Instruction execution counter
    instruction_count: u64,
}

impl Cpu {
    /// Create a new CPU with default configuration (32K memory)
    pub fn new() -> Self {
        Self::with_memory_size(32768)
    }

    /// Create a CPU with specific memory size (in words)
    pub fn with_memory_size(size: usize) -> Self {
        Self {
            acc: 0,
            ext: 0,
            iar: 0,
            index_registers: IndexRegisters::new(),
            status_flags: StatusFlags::new(),
            memory: Memory::with_size(size),
            instruction_count: 0,
        }
    }

    /// Reset CPU to initial state
    ///
    /// Clears all registers and flags, but preserves memory contents
    pub fn reset(&mut self) {
        self.acc = 0;
        self.ext = 0;
        self.iar = 0;
        self.index_registers.reset();
        self.status_flags.reset();
        self.instruction_count = 0;
        // Memory is NOT cleared - programs remain loaded
    }

    /// Get current CPU state snapshot
    pub fn get_state(&self) -> CpuState {
        CpuState {
            acc: self.acc,
            ext: self.ext,
            iar: self.iar,
            xr1: self.index_registers.xr1,
            xr2: self.index_registers.xr2,
            xr3: self.index_registers.xr3,
            carry: self.status_flags.carry,
            overflow: self.status_flags.overflow,
            wait: self.status_flags.wait,
            instruction_count: self.instruction_count,
            current_interrupt_level: None, // TODO: implement interrupt system
        }
    }

    // === Accumulator Methods ===

    pub fn get_acc(&self) -> u16 {
        self.acc
    }

    pub fn set_acc(&mut self, value: u16) {
        self.acc = value;
    }

    // === Extension Register Methods ===

    pub fn get_ext(&self) -> u16 {
        self.ext
    }

    pub fn set_ext(&mut self, value: u16) {
        self.ext = value;
    }

    /// Get combined ACC:EXT as 32-bit value
    pub fn get_acc_ext(&self) -> u32 {
        ((self.acc as u32) << 16) | (self.ext as u32)
    }

    /// Set combined ACC:EXT from 32-bit value
    pub fn set_acc_ext(&mut self, value: u32) {
        self.acc = (value >> 16) as u16;
        self.ext = value as u16;
    }

    // === IAR (Program Counter) Methods ===

    pub fn get_iar(&self) -> u16 {
        self.iar
    }

    pub fn set_iar(&mut self, value: u16) {
        self.iar = value;
    }

    /// Increment IAR by specified amount
    pub fn increment_iar(&mut self, amount: u16) {
        self.iar = self.iar.wrapping_add(amount);
    }

    // === Index Register Methods ===

    pub fn get_index_register(&self, tag: u8) -> u16 {
        self.index_registers.get(tag)
    }

    pub fn set_index_register(&mut self, tag: u8, value: u16) {
        self.index_registers.set(tag, value);

        // Update memory-mapped locations (0x0001-0x0003)
        match tag {
            1 => {
                let _ = self.memory.write(0x0001, value);
            }
            2 => {
                let _ = self.memory.write(0x0002, value);
            }
            3 => {
                let _ = self.memory.write(0x0003, value);
            }
            _ => {}
        }
    }

    // === Status Flag Methods ===

    pub fn get_carry(&self) -> bool {
        self.status_flags.carry
    }

    pub fn set_carry(&mut self, value: bool) {
        self.status_flags.carry = value;
    }

    pub fn get_overflow(&self) -> bool {
        self.status_flags.overflow
    }

    pub fn set_overflow(&mut self, value: bool) {
        self.status_flags.overflow = value;
    }

    pub fn get_wait(&self) -> bool {
        self.status_flags.wait
    }

    pub fn set_wait(&mut self, value: bool) {
        self.status_flags.wait = value;
    }

    // === Memory Methods ===

    /// Read word from memory with bounds checking
    pub fn read_memory(&self, address: usize) -> Result<u16> {
        self.memory.read(address)
    }

    /// Write word to memory with bounds checking and memory-mapped register handling
    pub fn write_memory(&mut self, address: usize, value: u16) -> Result<()> {
        self.memory.write(address, value)?;

        // Handle memory-mapped index registers
        match address {
            0x0001 => self.index_registers.xr1 = value,
            0x0002 => self.index_registers.xr2 = value,
            0x0003 => self.index_registers.xr3 = value,
            _ => {}
        }

        Ok(())
    }

    /// Read multiple words from memory
    pub fn read_memory_range(&self, address: usize, count: usize) -> Vec<u16> {
        self.memory.read_range(address, count)
    }

    /// Write multiple words to memory
    pub fn write_memory_range(&mut self, address: usize, values: &[u16]) -> Result<()> {
        self.memory.write_range(address, values)?;

        // Update memory-mapped registers if affected
        for (offset, &value) in values.iter().enumerate() {
            match address + offset {
                0x0001 => self.index_registers.xr1 = value,
                0x0002 => self.index_registers.xr2 = value,
                0x0003 => self.index_registers.xr3 = value,
                _ => {}
            }
        }

        Ok(())
    }

    // === Performance Methods ===

    pub fn get_instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Increment instruction counter (called after each instruction execution)
    pub fn increment_instruction_count(&mut self) {
        self.instruction_count += 1;
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_new() {
        let cpu = Cpu::new();
        assert_eq!(cpu.get_acc(), 0);
        assert_eq!(cpu.get_ext(), 0);
        assert_eq!(cpu.get_iar(), 0);
        assert!(!cpu.get_carry());
        assert!(!cpu.get_overflow());
        assert!(!cpu.get_wait());
    }

    #[test]
    fn test_cpu_with_custom_memory_size() {
        let cpu = Cpu::with_memory_size(8192);
        let state = cpu.get_state();
        // Verify it doesn't panic accessing memory within bounds
        assert_eq!(state.acc, 0);
    }

    #[test]
    fn test_accumulator_operations() {
        let mut cpu = Cpu::new();
        cpu.set_acc(0x1234);
        assert_eq!(cpu.get_acc(), 0x1234);
    }

    #[test]
    fn test_extension_register_operations() {
        let mut cpu = Cpu::new();
        cpu.set_ext(0x5678);
        assert_eq!(cpu.get_ext(), 0x5678);
    }

    #[test]
    fn test_acc_ext_combined() {
        let mut cpu = Cpu::new();
        cpu.set_acc_ext(0x12345678);
        assert_eq!(cpu.get_acc(), 0x1234);
        assert_eq!(cpu.get_ext(), 0x5678);
        assert_eq!(cpu.get_acc_ext(), 0x12345678);
    }

    #[test]
    fn test_iar_operations() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);
        assert_eq!(cpu.get_iar(), 0x0100);

        cpu.increment_iar(2);
        assert_eq!(cpu.get_iar(), 0x0102);
    }

    #[test]
    fn test_index_registers() {
        let mut cpu = Cpu::new();

        cpu.set_index_register(1, 0x1111);
        cpu.set_index_register(2, 0x2222);
        cpu.set_index_register(3, 0x3333);

        assert_eq!(cpu.get_index_register(1), 0x1111);
        assert_eq!(cpu.get_index_register(2), 0x2222);
        assert_eq!(cpu.get_index_register(3), 0x3333);
    }

    #[test]
    fn test_status_flags() {
        let mut cpu = Cpu::new();

        cpu.set_carry(true);
        assert!(cpu.get_carry());

        cpu.set_overflow(true);
        assert!(cpu.get_overflow());

        cpu.set_wait(true);
        assert!(cpu.get_wait());
    }

    #[test]
    fn test_memory_operations() {
        let mut cpu = Cpu::new();
        cpu.write_memory(0x100, 0x1234).unwrap();
        assert_eq!(cpu.read_memory(0x100).unwrap(), 0x1234);
    }

    #[test]
    fn test_memory_bounds_check() {
        let cpu = Cpu::new();
        let result = cpu.read_memory(0x10000);
        assert!(result.is_err());
    }

    #[test]
    fn test_memory_mapped_index_registers() {
        let mut cpu = Cpu::new();

        // Writing to memory location should update index register
        cpu.write_memory(0x0001, 0xABCD).unwrap();
        assert_eq!(cpu.index_registers.xr1, 0xABCD);
        assert_eq!(cpu.read_memory(0x0001).unwrap(), 0xABCD);

        // Setting index register should update memory
        cpu.set_index_register(2, 0x1234);
        assert_eq!(cpu.read_memory(0x0002).unwrap(), 0x1234);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();
        cpu.set_acc(0x1234);
        cpu.set_iar(0x0100);
        cpu.set_carry(true);
        cpu.write_memory(0x200, 0xABCD).unwrap();
        cpu.instruction_count = 42;

        cpu.reset();

        assert_eq!(cpu.get_acc(), 0);
        assert_eq!(cpu.get_iar(), 0);
        assert!(!cpu.get_carry());
        assert_eq!(cpu.instruction_count, 0);

        // Memory should NOT be cleared
        assert_eq!(cpu.read_memory(0x200).unwrap(), 0xABCD);
    }

    #[test]
    fn test_get_state() {
        let mut cpu = Cpu::new();
        cpu.set_acc(0x1234);
        cpu.set_ext(0x5678);
        cpu.set_iar(0x0100);
        cpu.set_carry(true);
        cpu.set_index_register(1, 0xAAAA);

        let state = cpu.get_state();
        assert_eq!(state.acc, 0x1234);
        assert_eq!(state.ext, 0x5678);
        assert_eq!(state.iar, 0x0100);
        assert!(state.carry);
        assert_eq!(state.xr1, 0xAAAA);
    }

    #[test]
    fn test_instruction_count() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.get_instruction_count(), 0);

        cpu.increment_instruction_count();
        assert_eq!(cpu.get_instruction_count(), 1);

        cpu.increment_instruction_count();
        assert_eq!(cpu.get_instruction_count(), 2);
    }
}
