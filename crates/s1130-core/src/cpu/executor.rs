//! Instruction Execution
//!
//! This module contains the execution logic for all IBM 1130 instructions.
//! Each instruction is implemented as a method that modifies CPU state.

use super::Cpu;
use crate::error::{CpuError, Result};
use crate::instructions::{InstructionInfo, OpCode};

impl Cpu {
    /// Execute a decoded instruction
    ///
    /// This is the main dispatch method that calls the appropriate instruction handler
    /// based on the opcode.
    pub fn execute_instruction(
        &mut self,
        instr: &InstructionInfo,
        effective_address: u16,
    ) -> Result<()> {
        match instr.opcode {
            // Load/Store Instructions
            OpCode::LD => self.execute_ld(effective_address),
            OpCode::LDD => self.execute_ldd(effective_address),
            OpCode::STO => self.execute_sto(effective_address),
            OpCode::STD => self.execute_std(effective_address),

            // Arithmetic Instructions
            OpCode::A => self.execute_a(effective_address),
            OpCode::AD => self.execute_ad(effective_address),
            OpCode::S => self.execute_s(effective_address),
            OpCode::SD => self.execute_sd(effective_address),
            OpCode::M => self.execute_m(effective_address),
            OpCode::D => self.execute_d(effective_address),

            // Logical Instructions
            OpCode::AND => self.execute_and(effective_address),
            OpCode::OR => self.execute_or(effective_address),
            OpCode::EOR => self.execute_eor(effective_address),

            // Shift Instructions
            OpCode::SLA => self.execute_sla(instr.displacement),
            OpCode::SLCA => self.execute_slca(instr.displacement),
            OpCode::SRA => self.execute_sra(instr.displacement),
            OpCode::SRT => self.execute_srt(instr.displacement),

            // Branch Instructions
            OpCode::BSI => self.execute_bsi(effective_address),
            OpCode::BC => self.execute_bc(effective_address, instr.tag),
            OpCode::BSC => self.execute_bsc(effective_address, instr.tag),

            // Index Register Instructions
            OpCode::LDX => self.execute_ldx(effective_address, instr.tag),
            OpCode::STX => self.execute_stx(effective_address, instr.tag),
            OpCode::MDX => self.execute_mdx(effective_address, instr.tag),

            // Status Instructions
            OpCode::LDS => self.execute_lds(effective_address),
            OpCode::STS => self.execute_sts(effective_address),

            // Control Instructions
            OpCode::WAIT => self.execute_wait(),

            // I/O Instructions
            OpCode::XIO => self.execute_xio(effective_address),
            OpCode::SDS => Err(CpuError::InvalidInstruction(self.iar)), // TODO: Phase 4
        }
    }

    // === Load/Store Instructions ===

    /// LD - Load Accumulator
    ///
    /// Loads a word from memory into the accumulator.
    /// Flags affected: None
    fn execute_ld(&mut self, address: u16) -> Result<()> {
        let value = self.read_memory(address as usize)?;
        self.set_acc(value);
        Ok(())
    }

    /// LDD - Load Accumulator Double
    ///
    /// Loads two consecutive words from memory into ACC and EXT.
    /// First word goes to ACC, second word goes to EXT.
    /// Flags affected: None
    fn execute_ldd(&mut self, address: u16) -> Result<()> {
        let acc_value = self.read_memory(address as usize)?;
        let ext_value = self.read_memory((address + 1) as usize)?;
        self.set_acc(acc_value);
        self.set_ext(ext_value);
        Ok(())
    }

    /// STO - Store Accumulator
    ///
    /// Stores the accumulator value to memory.
    /// Flags affected: None
    fn execute_sto(&mut self, address: u16) -> Result<()> {
        let value = self.get_acc();
        self.write_memory(address as usize, value)?;
        Ok(())
    }

    /// STD - Store Accumulator Double
    ///
    /// Stores ACC and EXT to two consecutive memory locations.
    /// ACC goes to first location, EXT goes to second location.
    /// Flags affected: None
    fn execute_std(&mut self, address: u16) -> Result<()> {
        let acc_value = self.get_acc();
        let ext_value = self.get_ext();
        self.write_memory(address as usize, acc_value)?;
        self.write_memory((address + 1) as usize, ext_value)?;
        Ok(())
    }

    // === Arithmetic Instructions ===

    /// A - Add
    ///
    /// Adds a memory word to the accumulator.
    /// Flags affected: Carry, Overflow
    fn execute_a(&mut self, address: u16) -> Result<()> {
        let operand = self.read_memory(address as usize)? as i16;
        let acc = self.get_acc() as i16;

        let (result, overflow) = acc.overflowing_add(operand);
        let carry = (self.get_acc() as u32 + operand as u32) > 0xFFFF;

        self.set_acc(result as u16);
        self.set_carry(carry);
        self.set_overflow(overflow);
        Ok(())
    }

    /// AD - Add Double
    ///
    /// Adds a 32-bit value from memory to ACC:EXT.
    /// Flags affected: Carry, Overflow
    fn execute_ad(&mut self, address: u16) -> Result<()> {
        let high = self.read_memory(address as usize)? as u32;
        let low = self.read_memory((address + 1) as usize)? as u32;
        let operand = (high << 16) | low;

        let acc_ext = self.get_acc_ext();
        let result = acc_ext.wrapping_add(operand);

        // Check for carry (unsigned overflow)
        let carry = acc_ext > u32::MAX - operand;

        // Check for signed overflow
        let overflow = {
            let acc_ext_signed = acc_ext as i32;
            let operand_signed = operand as i32;
            let result_signed = result as i32;
            (acc_ext_signed > 0 && operand_signed > 0 && result_signed < 0)
                || (acc_ext_signed < 0 && operand_signed < 0 && result_signed > 0)
        };

        self.set_acc_ext(result);
        self.set_carry(carry);
        self.set_overflow(overflow);
        Ok(())
    }

    /// S - Subtract
    ///
    /// Subtracts a memory word from the accumulator.
    /// Flags affected: Carry, Overflow
    fn execute_s(&mut self, address: u16) -> Result<()> {
        let operand = self.read_memory(address as usize)? as i16;
        let acc = self.get_acc() as i16;

        let (result, overflow) = acc.overflowing_sub(operand);
        let carry = (self.get_acc() as u32) < (operand as u32);

        self.set_acc(result as u16);
        self.set_carry(carry);
        self.set_overflow(overflow);
        Ok(())
    }

    /// SD - Subtract Double
    ///
    /// Subtracts a 32-bit value from ACC:EXT.
    /// Flags affected: Carry, Overflow
    fn execute_sd(&mut self, address: u16) -> Result<()> {
        let high = self.read_memory(address as usize)? as u32;
        let low = self.read_memory((address + 1) as usize)? as u32;
        let operand = (high << 16) | low;

        let acc_ext = self.get_acc_ext();
        let result = acc_ext.wrapping_sub(operand);

        let carry = acc_ext < operand;

        let overflow = {
            let acc_ext_signed = acc_ext as i32;
            let operand_signed = operand as i32;
            let result_signed = result as i32;
            (acc_ext_signed > 0 && operand_signed < 0 && result_signed < 0)
                || (acc_ext_signed < 0 && operand_signed > 0 && result_signed > 0)
        };

        self.set_acc_ext(result);
        self.set_carry(carry);
        self.set_overflow(overflow);
        Ok(())
    }

    /// M - Multiply
    ///
    /// Multiplies ACC by memory word, result in ACC:EXT.
    /// Flags affected: None
    fn execute_m(&mut self, address: u16) -> Result<()> {
        let operand = self.read_memory(address as usize)? as i16;
        let acc = self.get_acc() as i16;

        let result = (acc as i32) * (operand as i32);
        self.set_acc_ext(result as u32);
        Ok(())
    }

    /// D - Divide
    ///
    /// Divides ACC:EXT by memory word.
    /// Quotient in ACC, remainder in EXT.
    /// Flags affected: Overflow (on divide by zero or overflow)
    fn execute_d(&mut self, address: u16) -> Result<()> {
        let divisor = self.read_memory(address as usize)? as i16;

        if divisor == 0 {
            self.set_overflow(true);
            return Ok(());
        }

        let dividend = self.get_acc_ext() as i32;
        let quotient = dividend / (divisor as i32);
        let remainder = dividend % (divisor as i32);

        // Check if quotient fits in 16 bits
        if quotient > i16::MAX as i32 || quotient < i16::MIN as i32 {
            self.set_overflow(true);
            return Ok(());
        }

        self.set_acc(quotient as u16);
        self.set_ext(remainder as u16);
        self.set_overflow(false);
        Ok(())
    }

    // === Logical Instructions ===

    /// AND - Logical AND
    ///
    /// Performs bitwise AND between ACC and memory.
    /// Flags affected: None
    fn execute_and(&mut self, address: u16) -> Result<()> {
        let operand = self.read_memory(address as usize)?;
        let result = self.get_acc() & operand;
        self.set_acc(result);
        Ok(())
    }

    /// OR - Logical OR
    ///
    /// Performs bitwise OR between ACC and memory.
    /// Flags affected: None
    fn execute_or(&mut self, address: u16) -> Result<()> {
        let operand = self.read_memory(address as usize)?;
        let result = self.get_acc() | operand;
        self.set_acc(result);
        Ok(())
    }

    /// EOR - Exclusive OR
    ///
    /// Performs bitwise XOR between ACC and memory.
    /// Flags affected: None
    fn execute_eor(&mut self, address: u16) -> Result<()> {
        let operand = self.read_memory(address as usize)?;
        let result = self.get_acc() ^ operand;
        self.set_acc(result);
        Ok(())
    }

    // === Shift Instructions ===

    /// SLA - Shift Left Accumulator
    ///
    /// Shifts ACC left by specified count.
    /// Flags affected: Carry (last bit shifted out)
    fn execute_sla(&mut self, count: u16) -> Result<()> {
        let shift_count = (count & 0x1F) as u32; // Use lower 5 bits
        if shift_count == 0 {
            return Ok(());
        }

        let acc = self.get_acc();
        let carry = (acc >> (16 - shift_count)) & 1 != 0;
        let result = acc << shift_count;

        self.set_acc(result);
        self.set_carry(carry);
        Ok(())
    }

    /// SLCA - Shift Left Combined Accumulator
    ///
    /// Shifts ACC:EXT left by specified count.
    /// Flags affected: Carry (last bit shifted out)
    fn execute_slca(&mut self, count: u16) -> Result<()> {
        let shift_count = (count & 0x1F) as u32;
        if shift_count == 0 {
            return Ok(());
        }

        let acc_ext = self.get_acc_ext() as u64;
        let carry = (acc_ext >> (32 - shift_count)) & 1 != 0;
        let result = (acc_ext << shift_count) as u32;

        self.set_acc_ext(result);
        self.set_carry(carry);
        Ok(())
    }

    /// SRA - Shift Right Accumulator
    ///
    /// Arithmetic right shift of ACC (sign extends).
    /// Flags affected: Carry (last bit shifted out)
    fn execute_sra(&mut self, count: u16) -> Result<()> {
        let shift_count = (count & 0x1F) as u32;
        if shift_count == 0 {
            return Ok(());
        }

        let acc = self.get_acc() as i16;
        let carry = (acc >> (shift_count - 1)) & 1 != 0;
        let result = acc >> shift_count;

        self.set_acc(result as u16);
        self.set_carry(carry);
        Ok(())
    }

    /// SRT - Shift Right Combined (logical)
    ///
    /// Logical right shift of ACC:EXT (zero fill).
    /// Flags affected: Carry (last bit shifted out)
    fn execute_srt(&mut self, count: u16) -> Result<()> {
        let shift_count = (count & 0x1F) as u32;
        if shift_count == 0 {
            return Ok(());
        }

        let acc_ext = self.get_acc_ext();
        let carry = (acc_ext >> (shift_count - 1)) & 1 != 0;
        let result = acc_ext >> shift_count;

        self.set_acc_ext(result);
        self.set_carry(carry);
        Ok(())
    }

    // === Branch Instructions ===

    /// BSI - Branch and Store IAR
    ///
    /// Stores return address and branches to subroutine.
    fn execute_bsi(&mut self, address: u16) -> Result<()> {
        let return_address = self.get_iar();
        self.write_memory(address as usize, return_address)?;
        self.set_iar(address.wrapping_add(1));
        Ok(())
    }

    /// BC - Branch on Condition
    ///
    /// Conditional branch based on tag bits.
    fn execute_bc(&mut self, address: u16, tag: u8) -> Result<()> {
        if self.check_branch_condition(tag) {
            self.set_iar(address);
        }
        Ok(())
    }

    /// BSC - Branch and Store on Condition
    ///
    /// Conditional BSI.
    fn execute_bsc(&mut self, address: u16, tag: u8) -> Result<()> {
        if self.check_branch_condition(tag) {
            let return_address = self.get_iar();
            self.write_memory(address as usize, return_address)?;
            self.set_iar(address.wrapping_add(1));
        }
        Ok(())
    }

    /// Check branch condition based on tag bits
    fn check_branch_condition(&self, tag: u8) -> bool {
        match tag {
            0 => true,                // Unconditional
            1 => self.get_carry(),    // Carry set
            2 => self.get_overflow(), // Overflow set
            3 => !self.get_carry(),   // Carry clear
            _ => false,
        }
    }

    // === Index Register Instructions ===

    /// LDX - Load Index Register
    fn execute_ldx(&mut self, address: u16, tag: u8) -> Result<()> {
        if tag == 0 {
            return Ok(()); // No operation for tag 0
        }
        let value = self.read_memory(address as usize)?;
        self.set_index_register(tag, value);
        Ok(())
    }

    /// STX - Store Index Register
    fn execute_stx(&mut self, address: u16, tag: u8) -> Result<()> {
        if tag == 0 {
            return Ok(());
        }
        let value = self.get_index_register(tag);
        self.write_memory(address as usize, value)?;
        Ok(())
    }

    /// MDX - Modify Index and Skip
    fn execute_mdx(&mut self, address: u16, tag: u8) -> Result<()> {
        if tag == 0 {
            return Ok(());
        }

        let operand = self.read_memory(address as usize)? as i16;
        let index_value = self.get_index_register(tag) as i16;
        let result = index_value.wrapping_add(operand);

        self.set_index_register(tag, result as u16);

        // Skip next instruction if result is zero
        if result == 0 {
            let next_instr = self.fetch_and_decode()?;
            self.increment_iar(next_instr.size_in_words());
        }

        Ok(())
    }

    // === Status Instructions ===

    /// LDS - Load Status
    fn execute_lds(&mut self, address: u16) -> Result<()> {
        let value = self.read_memory(address as usize)?;
        let flags = crate::cpu::registers::StatusFlags::from_word(value);
        self.set_carry(flags.carry);
        self.set_overflow(flags.overflow);
        Ok(())
    }

    /// STS - Store Status
    fn execute_sts(&mut self, address: u16) -> Result<()> {
        let flags = crate::cpu::registers::StatusFlags {
            carry: self.get_carry(),
            overflow: self.get_overflow(),
            wait: self.get_wait(),
        };
        let word = flags.to_word();
        self.write_memory(address as usize, word)?;
        Ok(())
    }

    // === Control Instructions ===

    /// WAIT - Halt CPU
    fn execute_wait(&mut self) -> Result<()> {
        self.set_wait(true);
        Ok(())
    }

    // === I/O Instructions ===

    /// XIO - Execute I/O
    ///
    /// This instruction decodes an IOCC structure from memory and executes
    /// the I/O operation on the specified device.
    ///
    /// The effective address points to the IOCC structure (2 words):
    /// - Word 0: WCA (Word Count Address)
    /// - Word 1: Device code + Function + Modifiers
    fn execute_xio(&mut self, address: u16) -> Result<()> {
        // Decode IOCC from memory
        self.decode_iocc(address)?;

        // Execute the I/O operation
        self.execute_iocc()?;

        Ok(())
    }
}
