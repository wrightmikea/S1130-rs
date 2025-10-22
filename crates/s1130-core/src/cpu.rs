//! CPU implementation for the IBM 1130

use crate::error::*;
use serde::{Deserialize, Serialize};

/// IBM 1130 CPU
///
/// The CPU contains all registers, memory, and execution state.
/// It is responsible for fetching, decoding, and executing instructions.
pub struct Cpu {
    /// Accumulator (16-bit)
    acc: u16,

    /// Extension register (16-bit)
    ext: u16,

    /// Instruction Address Register (program counter)
    iar: u16,

    /// Index register 1
    xr1: u16,

    /// Index register 2
    xr2: u16,

    /// Index register 3
    xr3: u16,

    /// Carry flag
    carry: bool,

    /// Overflow flag
    overflow: bool,

    /// Wait state (halted)
    wait: bool,

    /// Main memory (32K words = 65536 bytes)
    memory: Vec<u16>,

    /// Instruction execution counter
    instruction_count: u64,
}

impl Cpu {
    /// Create a new CPU with default configuration
    ///
    /// # Example
    ///
    /// ```
    /// use s1130_core::Cpu;
    ///
    /// let cpu = Cpu::new();
    /// assert_eq!(cpu.get_acc(), 0);
    /// ```
    pub fn new() -> Self {
        Self::with_memory_size(32768)
    }

    /// Create a CPU with a specific memory size (in words)
    pub fn with_memory_size(size: usize) -> Self {
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
            memory: vec![0; size],
            instruction_count: 0,
        }
    }

    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.acc = 0;
        self.ext = 0;
        self.iar = 0;
        self.xr1 = 0;
        self.xr2 = 0;
        self.xr3 = 0;
        self.carry = false;
        self.overflow = false;
        self.wait = false;
        self.instruction_count = 0;
        // Don't clear memory - programs remain loaded
    }

    /// Read from memory with bounds checking
    ///
    /// # Errors
    ///
    /// Returns `CpuError::MemoryViolation` if address is out of bounds
    pub fn read_memory(&self, address: usize) -> Result<u16> {
        self.memory
            .get(address)
            .copied()
            .ok_or(CpuError::MemoryViolation(address as u16))
    }

    /// Write to memory with bounds checking
    ///
    /// Handles memory-mapped index registers (0x0001-0x0003)
    ///
    /// # Errors
    ///
    /// Returns `CpuError::MemoryViolation` if address is out of bounds
    pub fn write_memory(&mut self, address: usize, value: u16) -> Result<()> {
        if address < self.memory.len() {
            self.memory[address] = value;

            // Handle memory-mapped index registers
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

    /// Get current CPU state (for UI display and debugging)
    pub fn get_state(&self) -> CpuState {
        CpuState {
            acc: self.acc,
            ext: self.ext,
            iar: self.iar,
            xr1: self.xr1,
            xr2: self.xr2,
            xr3: self.xr3,
            carry: self.carry,
            overflow: self.overflow,
            wait: self.wait,
            instruction_count: self.instruction_count,
            current_interrupt_level: None, // TODO: implement interrupt system
        }
    }

    // Accessor methods
    pub fn get_acc(&self) -> u16 {
        self.acc
    }

    pub fn set_acc(&mut self, value: u16) {
        self.acc = value;
    }

    pub fn get_ext(&self) -> u16 {
        self.ext
    }

    pub fn set_ext(&mut self, value: u16) {
        self.ext = value;
    }

    pub fn get_iar(&self) -> u16 {
        self.iar
    }

    pub fn set_iar(&mut self, value: u16) {
        self.iar = value;
    }

    pub fn get_index_register(&self, tag: u8) -> u16 {
        match tag {
            1 => self.xr1,
            2 => self.xr2,
            3 => self.xr3,
            _ => 0, // Tag 0 means no index register
        }
    }

    pub fn set_index_register(&mut self, tag: u8, value: u16) {
        match tag {
            1 => self.xr1 = value,
            2 => self.xr2 = value,
            3 => self.xr3 = value,
            _ => {}
        }
    }

    pub fn get_carry(&self) -> bool {
        self.carry
    }

    pub fn set_carry(&mut self, value: bool) {
        self.carry = value;
    }

    pub fn get_overflow(&self) -> bool {
        self.overflow
    }

    pub fn set_overflow(&mut self, value: bool) {
        self.overflow = value;
    }

    pub fn get_wait(&self) -> bool {
        self.wait
    }

    pub fn set_wait(&mut self, value: bool) {
        self.wait = value;
    }

    pub fn get_instruction_count(&self) -> u64 {
        self.instruction_count
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU state snapshot for external observation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CpuState {
    pub acc: u16,
    pub ext: u16,
    pub iar: u16,
    pub xr1: u16,
    pub xr2: u16,
    pub xr3: u16,
    pub carry: bool,
    pub overflow: bool,
    pub wait: bool,
    pub instruction_count: u64,
    pub current_interrupt_level: Option<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_initialization() {
        let cpu = Cpu::new();
        assert_eq!(cpu.get_acc(), 0);
        assert_eq!(cpu.get_ext(), 0);
        assert_eq!(cpu.get_iar(), 0);
        assert_eq!(cpu.memory.len(), 32768);
        assert!(!cpu.get_carry());
        assert!(!cpu.get_overflow());
        assert!(!cpu.get_wait());
    }

    #[test]
    fn test_cpu_with_custom_memory_size() {
        let cpu = Cpu::with_memory_size(8192);
        assert_eq!(cpu.memory.len(), 8192);
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
        let result = cpu.read_memory(0x10000);
        assert!(result.is_err());
        match result.unwrap_err() {
            CpuError::MemoryViolation(addr) => assert_eq!(addr, 0), // Wraps around
            _ => panic!("Expected MemoryViolation"),
        }
    }

    #[test]
    fn test_memory_mapped_index_registers() {
        let mut cpu = Cpu::new();

        cpu.write_memory(0x0001, 0xABCD).unwrap();
        assert_eq!(cpu.xr1, 0xABCD);
        assert_eq!(cpu.read_memory(0x0001).unwrap(), 0xABCD);

        cpu.write_memory(0x0002, 0x1234).unwrap();
        assert_eq!(cpu.xr2, 0x1234);

        cpu.write_memory(0x0003, 0x5678).unwrap();
        assert_eq!(cpu.xr3, 0x5678);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();
        cpu.set_acc(0x1234);
        cpu.set_iar(0x100);
        cpu.set_carry(true);
        cpu.write_memory(0x200, 0xABCD).unwrap();

        cpu.reset();

        assert_eq!(cpu.get_acc(), 0);
        assert_eq!(cpu.get_iar(), 0);
        assert!(!cpu.get_carry());
        // Memory should not be cleared
        assert_eq!(cpu.read_memory(0x200).unwrap(), 0xABCD);
    }

    #[test]
    fn test_get_state() {
        let mut cpu = Cpu::new();
        cpu.set_acc(0x1234);
        cpu.set_ext(0x5678);
        cpu.set_iar(0x0100);
        cpu.set_carry(true);

        let state = cpu.get_state();
        assert_eq!(state.acc, 0x1234);
        assert_eq!(state.ext, 0x5678);
        assert_eq!(state.iar, 0x0100);
        assert!(state.carry);
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
        assert_eq!(cpu.get_index_register(0), 0); // Tag 0 = no index
    }
}
