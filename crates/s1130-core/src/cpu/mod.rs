//! IBM 1130 CPU Module
//!
//! This module orchestrates the CPU components:
//! - Registers (accumulator, extension, index registers, flags)
//! - Memory (word-addressable, 32K default)
//! - State snapshots for external observation

pub mod executor;
pub mod memory;
pub mod registers;
pub mod state;

pub use memory::Memory;
pub use registers::{IndexRegisters, StatusFlags};
pub use state::CpuState;

use crate::error::{CpuError, Result};
use crate::instructions::{InstructionInfo, OpCode};

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

    // === Fetch-Decode-Execute Cycle ===

    /// Fetch instruction from memory at current IAR
    ///
    /// Fetches one or two words depending on instruction format.
    /// Does NOT increment IAR (that happens after execution).
    ///
    /// # Returns
    /// Tuple of (word1, Option<word2>) where word2 is present for long format instructions
    pub fn fetch_instruction(&self) -> Result<(u16, Option<u16>)> {
        let word1 = self.read_memory(self.iar as usize)?;

        // Peek at opcode to determine if we need a second word
        let opcode =
            OpCode::from_word(word1).map_err(|_| CpuError::InvalidInstruction(self.iar))?;

        let word2 = if opcode.is_long_format() {
            Some(self.read_memory((self.iar + 1) as usize)?)
        } else {
            None
        };

        Ok((word1, word2))
    }

    /// Fetch and decode instruction at current IAR
    ///
    /// # Returns
    /// Decoded instruction information ready for execution
    pub fn fetch_and_decode(&self) -> Result<InstructionInfo> {
        let (word1, word2) = self.fetch_instruction()?;

        InstructionInfo::decode(word1, word2).map_err(|_| CpuError::InvalidInstruction(self.iar))
    }

    /// Calculate effective address for an instruction
    ///
    /// This helper method calculates the effective address by:
    /// 1. Starting with the instruction's displacement
    /// 2. Adding the index register value if tag != 0
    /// 3. Following indirect addressing if the indirect flag is set
    pub fn calculate_effective_address(&self, instr: &mut InstructionInfo) -> Result<u16> {
        use crate::error::InstructionError;

        let index_value = self.index_registers.get(instr.tag);

        instr
            .calculate_effective_address(index_value, |addr| {
                self.read_memory(addr as usize)
                    .map_err(InstructionError::MemoryError)
            })
            .map_err(|e| match e {
                InstructionError::MemoryError(cpu_err) => cpu_err,
                _ => CpuError::InvalidInstruction(self.iar),
            })
    }

    /// Execute one instruction at current IAR
    ///
    /// This is the main execution method that:
    /// 1. Fetches instruction from memory at IAR
    /// 2. Decodes the instruction
    /// 3. Calculates effective address
    /// 4. Executes the instruction (to be implemented in Phase 2)
    /// 5. Increments IAR
    /// 6. Increments instruction counter
    ///
    /// # Returns
    /// Ok(()) if instruction executed successfully, Err if execution failed
    pub fn step(&mut self) -> Result<()> {
        // Check if CPU is in wait state
        if self.status_flags.wait {
            return Err(CpuError::WaitState);
        }

        // Fetch and decode
        let mut instr = self.fetch_and_decode()?;

        // Calculate effective address
        // For index register instructions (LDX, STX, MDX), don't use tag for address calculation
        let effective_address = match instr.opcode {
            OpCode::LDX | OpCode::STX | OpCode::MDX => {
                // For these instructions, tag specifies WHICH register to operate on,
                // not which register to use for addressing. Calculate EA without tag.
                let saved_tag = instr.tag;
                instr.tag = 0;
                let ea = self.calculate_effective_address(&mut instr)?;
                instr.tag = saved_tag;
                ea
            }
            _ => self.calculate_effective_address(&mut instr)?,
        };

        // Increment IAR by instruction size BEFORE execution
        // (branch instructions will override this)
        let instruction_size = instr.size_in_words();
        self.increment_iar(instruction_size);

        // Execute instruction
        self.execute_instruction(&instr, effective_address)?;

        // Increment instruction counter
        self.increment_instruction_count();

        Ok(())
    }

    /// Run CPU for a specified number of steps or until WAIT
    ///
    /// # Arguments
    /// * `max_steps` - Maximum number of instructions to execute
    ///
    /// # Returns
    /// Number of instructions actually executed
    pub fn run(&mut self, max_steps: u64) -> u64 {
        let mut steps = 0;

        for _ in 0..max_steps {
            match self.step() {
                Ok(()) => steps += 1,
                Err(CpuError::WaitState) => break,
                Err(_) => break,
            }
        }

        steps
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

    #[test]
    fn test_fetch_instruction_short_format() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write WAIT instruction (short format)
        cpu.write_memory(0x0100, 0xB000).unwrap();

        let (word1, word2) = cpu.fetch_instruction().unwrap();
        assert_eq!(word1, 0xB000);
        assert_eq!(word2, None);
    }

    #[test]
    fn test_fetch_instruction_long_format() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write LD instruction (long format) at 0x0100
        cpu.write_memory(0x0100, 0x6000).unwrap(); // LD opcode
        cpu.write_memory(0x0101, 0x1234).unwrap(); // displacement

        let (word1, word2) = cpu.fetch_instruction().unwrap();
        assert_eq!(word1, 0x6000);
        assert_eq!(word2, Some(0x1234));
    }

    #[test]
    fn test_fetch_and_decode() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write LD instruction
        cpu.write_memory(0x0100, 0x6000).unwrap();
        cpu.write_memory(0x0101, 0x5678).unwrap();

        let instr = cpu.fetch_and_decode().unwrap();
        assert_eq!(instr.opcode, OpCode::LD);
        assert_eq!(instr.displacement, 0x5678);
    }

    #[test]
    fn test_calculate_effective_address() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write LD with tag=1 (XR1)
        cpu.write_memory(0x0100, 0x6040).unwrap(); // LD with tag=1
        cpu.write_memory(0x0101, 0x0200).unwrap(); // displacement

        // Set XR1 to 0x0050
        cpu.set_index_register(1, 0x0050);

        let mut instr = cpu.fetch_and_decode().unwrap();
        let ea = cpu.calculate_effective_address(&mut instr).unwrap();

        // EA should be 0x0200 + 0x0050 = 0x0250
        assert_eq!(ea, 0x0250);
    }

    #[test]
    fn test_step_wait_instruction() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write WAIT instruction
        cpu.write_memory(0x0100, 0xB000).unwrap();

        // Execute one step
        let result = cpu.step();
        assert!(result.is_ok());

        // CPU should be in wait state
        assert!(cpu.get_wait());

        // IAR should have advanced by 1 (short format)
        assert_eq!(cpu.get_iar(), 0x0101);

        // Instruction count should be 1
        assert_eq!(cpu.get_instruction_count(), 1);

        // Second step should fail with WaitState
        let result = cpu.step();
        assert!(result.is_err());
        match result {
            Err(CpuError::WaitState) => {}
            _ => panic!("Expected WaitState error"),
        }
    }

    #[test]
    fn test_run_until_wait() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write multiple WAIT instructions
        cpu.write_memory(0x0100, 0xB000).unwrap();
        cpu.write_memory(0x0101, 0xB000).unwrap();
        cpu.write_memory(0x0102, 0xB000).unwrap();

        // Run for up to 10 steps (should stop at first WAIT)
        let steps = cpu.run(10);
        assert_eq!(steps, 1);
        assert!(cpu.get_wait());
    }

    #[test]
    fn test_fetch_instruction_invalid_opcode() {
        let mut cpu = Cpu::new();
        cpu.set_iar(0x0100);

        // Write invalid opcode
        cpu.write_memory(0x0100, 0xFF00).unwrap();

        let result = cpu.fetch_instruction();
        assert!(result.is_err());
    }
}
