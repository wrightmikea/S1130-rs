//! Instruction Decoding and Execution
//!
//! This module handles instruction decoding for the IBM 1130.
//! The 1130 uses two instruction formats:
//! - Short format (16-bit): Most common instructions
//! - Long format (32-bit): Instructions requiring displacement

use crate::error::InstructionError;
use serde::{Deserialize, Serialize};

/// Result type for instruction operations
pub type Result<T> = std::result::Result<T, InstructionError>;

/// IBM 1130 Operation Codes
///
/// The 1130 has 28 primary instructions, identified by the opcode field
/// in bits 0-4 of the instruction word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    /// Load Accumulator (0x60)
    LD = 0x60,
    /// Load Accumulator with Double (0x68)
    LDD = 0x68,
    /// Store Accumulator (0x70)
    STO = 0x70,
    /// Store Accumulator and Decrement (0x78)
    STD = 0x78,

    /// Add (0xE0)
    A = 0xE0,
    /// Add Double (0xE8)
    AD = 0xE8,
    /// Subtract (0xC0)
    S = 0xC0,
    /// Subtract Double (0xC8)
    SD = 0xC8,
    /// Multiply (0xF0)
    M = 0xF0,
    /// Divide (0xF8)
    D = 0xF8,

    /// AND (0x80)
    AND = 0x80,
    /// OR (0x90)
    OR = 0x90,
    /// Exclusive OR (0x98)
    EOR = 0x98,

    /// Shift Left Accumulator (0x20)
    SLA = 0x20,
    /// Shift Left Accumulator and Extension (0x28)
    SLCA = 0x28,
    /// Shift Right Accumulator (0x30)
    SRA = 0x30,
    /// Shift Right Accumulator and Extension (0x38)
    SRT = 0x38,

    /// Branch and Store IAR (0x48)
    BSI = 0x48,
    /// Branch on Condition (0x40)
    BC = 0x40,
    /// Branch and Store IAR on Condition (0x50)
    BSC = 0x50,

    /// Load Index Register (0x74)
    LDX = 0x74,
    /// Store Index Register (0x54)
    STX = 0x54,
    /// Modify Index and Skip (0x58)
    MDX = 0x58,

    /// Wait (0xB0)
    WAIT = 0xB0,

    /// Load Status (0xC4)
    LDS = 0xC4,
    /// Store Status (0xCC)
    STS = 0xCC,

    /// Execute (0x44)
    XIO = 0x44,
    /// Sense Device Status (0x4C)
    SDS = 0x4C,
}

impl OpCode {
    /// Decode opcode from instruction word
    ///
    /// The opcode is in bits 0-7 (upper byte) of the instruction word
    pub fn from_word(word: u16) -> Result<Self> {
        let opcode = (word >> 8) as u8;

        match opcode {
            0x60 => Ok(OpCode::LD),
            0x68 => Ok(OpCode::LDD),
            0x70 => Ok(OpCode::STO),
            0x78 => Ok(OpCode::STD),
            0xE0 => Ok(OpCode::A),
            0xE8 => Ok(OpCode::AD),
            0xC0 => Ok(OpCode::S),
            0xC8 => Ok(OpCode::SD),
            0xF0 => Ok(OpCode::M),
            0xF8 => Ok(OpCode::D),
            0x80 => Ok(OpCode::AND),
            0x90 => Ok(OpCode::OR),
            0x98 => Ok(OpCode::EOR),
            0x20 => Ok(OpCode::SLA),
            0x28 => Ok(OpCode::SLCA),
            0x30 => Ok(OpCode::SRA),
            0x38 => Ok(OpCode::SRT),
            0x48 => Ok(OpCode::BSI),
            0x40 => Ok(OpCode::BC),
            0x50 => Ok(OpCode::BSC),
            0x74 => Ok(OpCode::LDX),
            0x54 => Ok(OpCode::STX),
            0x58 => Ok(OpCode::MDX),
            0xB0 => Ok(OpCode::WAIT),
            0xC4 => Ok(OpCode::LDS),
            0xCC => Ok(OpCode::STS),
            0x44 => Ok(OpCode::XIO),
            0x4C => Ok(OpCode::SDS),
            _ => Err(InstructionError::InvalidOpcode(opcode)),
        }
    }

    /// Check if this instruction requires long format (displacement)
    pub fn is_long_format(self) -> bool {
        matches!(
            self,
            OpCode::LD
                | OpCode::LDD
                | OpCode::STO
                | OpCode::STD
                | OpCode::A
                | OpCode::AD
                | OpCode::S
                | OpCode::SD
                | OpCode::M
                | OpCode::D
                | OpCode::AND
                | OpCode::OR
                | OpCode::EOR
                | OpCode::BSI
                | OpCode::LDX
                | OpCode::STX
                | OpCode::MDX
        )
    }
}

/// Instruction format (short or long)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionFormat {
    /// Short format: 16-bit instruction (1 word)
    Short,
    /// Long format: 32-bit instruction (2 words)
    Long,
}

/// Decoded instruction information
///
/// Contains all fields extracted from the instruction word(s):
/// - Opcode
/// - Tag (index register selector: 0, 1, 2, or 3)
/// - Indirect flag
/// - Displacement (for long format) or address (for short format)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstructionInfo {
    /// Operation code
    pub opcode: OpCode,

    /// Instruction format
    pub format: InstructionFormat,

    /// Index register tag (0 = none, 1 = XR1, 2 = XR2, 3 = XR3)
    pub tag: u8,

    /// Indirect addressing flag
    pub indirect: bool,

    /// Displacement (long format) or direct address (short format)
    pub displacement: u16,

    /// Effective address (calculated during execution)
    pub effective_address: Option<u16>,
}

impl InstructionInfo {
    /// Decode instruction from one or two words
    ///
    /// # Arguments
    /// * `word1` - First instruction word (always present)
    /// * `word2` - Second instruction word (present only for long format)
    ///
    /// # Returns
    /// Decoded instruction information
    pub fn decode(word1: u16, word2: Option<u16>) -> Result<Self> {
        let opcode = OpCode::from_word(word1)?;

        // Extract tag (bits 8-9)
        let tag = ((word1 >> 6) & 0x03) as u8;

        // Extract indirect flag (bit 10)
        let indirect = (word1 & 0x20) != 0;

        if opcode.is_long_format() {
            // Long format: requires displacement word
            let displacement = word2.ok_or(InstructionError::MissingDisplacement)?;

            Ok(InstructionInfo {
                opcode,
                format: InstructionFormat::Long,
                tag,
                indirect,
                displacement,
                effective_address: None,
            })
        } else {
            // Short format: address is in bits 11-15 (lower 5 bits)
            let displacement = word1 & 0x1F;

            Ok(InstructionInfo {
                opcode,
                format: InstructionFormat::Short,
                tag,
                indirect,
                displacement,
                effective_address: None,
            })
        }
    }

    /// Calculate effective address from base address, tag, and indirect flag
    ///
    /// Effective address calculation:
    /// 1. Start with displacement (long format) or direct address (short format)
    /// 2. Add index register value if tag != 0
    /// 3. If indirect flag set, read memory at calculated address to get final address
    ///
    /// # Arguments
    /// * `index_register_value` - Value of the index register (or 0 if tag == 0)
    /// * `memory_read` - Closure to read memory for indirect addressing
    pub fn calculate_effective_address<F>(
        &mut self,
        index_register_value: u16,
        mut memory_read: F,
    ) -> Result<u16>
    where
        F: FnMut(u16) -> Result<u16>,
    {
        // Step 1: Start with displacement
        let mut address = self.displacement;

        // Step 2: Add index register if tag != 0
        if self.tag != 0 {
            address = address.wrapping_add(index_register_value);
        }

        // Step 3: Handle indirect addressing
        if self.indirect {
            address = memory_read(address)?;
        }

        self.effective_address = Some(address);
        Ok(address)
    }

    /// Get the size of this instruction in words
    pub fn size_in_words(&self) -> u16 {
        match self.format {
            InstructionFormat::Short => 1,
            InstructionFormat::Long => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_from_word_valid() {
        assert_eq!(OpCode::from_word(0x6000).unwrap(), OpCode::LD);
        assert_eq!(OpCode::from_word(0xE000).unwrap(), OpCode::A);
        assert_eq!(OpCode::from_word(0xC000).unwrap(), OpCode::S);
        assert_eq!(OpCode::from_word(0x4800).unwrap(), OpCode::BSI);
        assert_eq!(OpCode::from_word(0xB000).unwrap(), OpCode::WAIT);
    }

    #[test]
    fn test_opcode_from_word_invalid() {
        let result = OpCode::from_word(0xFF00);
        assert!(result.is_err());
        match result {
            Err(InstructionError::InvalidOpcode(0xFF)) => {}
            _ => panic!("Expected InvalidOpcode error"),
        }
    }

    #[test]
    fn test_opcode_is_long_format() {
        assert!(OpCode::LD.is_long_format());
        assert!(OpCode::A.is_long_format());
        assert!(OpCode::BSI.is_long_format());

        assert!(!OpCode::WAIT.is_long_format());
        assert!(!OpCode::SLA.is_long_format());
        assert!(!OpCode::BC.is_long_format());
    }

    #[test]
    fn test_decode_short_format() {
        // WAIT instruction (0xB000): opcode=B0, no tag, no indirect, address=0
        let instr = InstructionInfo::decode(0xB000, None).unwrap();
        assert_eq!(instr.opcode, OpCode::WAIT);
        assert_eq!(instr.format, InstructionFormat::Short);
        assert_eq!(instr.tag, 0);
        assert!(!instr.indirect);
        assert_eq!(instr.displacement, 0);
    }

    #[test]
    fn test_decode_short_format_with_tag() {
        // SLA with tag=1 (XR1): 0x2040 = 0010 0000 0100 0000
        // Opcode=20, tag=01, indirect=0, address=00000
        let instr = InstructionInfo::decode(0x2040, None).unwrap();
        assert_eq!(instr.opcode, OpCode::SLA);
        assert_eq!(instr.tag, 1);
        assert!(!instr.indirect);
    }

    #[test]
    fn test_decode_short_format_with_indirect() {
        // BC with indirect: 0x4020 = 0100 0000 0010 0000
        // Opcode=40, tag=00, indirect=1, address=00000
        let instr = InstructionInfo::decode(0x4020, None).unwrap();
        assert_eq!(instr.opcode, OpCode::BC);
        assert_eq!(instr.tag, 0);
        assert!(instr.indirect);
    }

    #[test]
    fn test_decode_long_format() {
        // LD 0x1234: word1=0x6000, word2=0x1234
        let instr = InstructionInfo::decode(0x6000, Some(0x1234)).unwrap();
        assert_eq!(instr.opcode, OpCode::LD);
        assert_eq!(instr.format, InstructionFormat::Long);
        assert_eq!(instr.tag, 0);
        assert!(!instr.indirect);
        assert_eq!(instr.displacement, 0x1234);
    }

    #[test]
    fn test_decode_long_format_with_tag() {
        // LD with tag=2 (XR2): word1=0x6080 = 0110 0000 1000 0000
        // Opcode=60, tag=10, indirect=0, displacement in word2
        let instr = InstructionInfo::decode(0x6080, Some(0x5678)).unwrap();
        assert_eq!(instr.opcode, OpCode::LD);
        assert_eq!(instr.tag, 2);
        assert_eq!(instr.displacement, 0x5678);
    }

    #[test]
    fn test_decode_long_format_with_indirect() {
        // LD with indirect: word1=0x6020
        let instr = InstructionInfo::decode(0x6020, Some(0xABCD)).unwrap();
        assert_eq!(instr.opcode, OpCode::LD);
        assert!(instr.indirect);
        assert_eq!(instr.displacement, 0xABCD);
    }

    #[test]
    fn test_decode_long_format_missing_displacement() {
        // LD without second word should fail
        let result = InstructionInfo::decode(0x6000, None);
        assert!(result.is_err());
        match result {
            Err(InstructionError::MissingDisplacement) => {}
            _ => panic!("Expected MissingDisplacement error"),
        }
    }

    #[test]
    fn test_calculate_effective_address_direct() {
        let mut instr = InstructionInfo::decode(0x6000, Some(0x1234)).unwrap();

        // No tag, no indirect: effective address = displacement
        let ea = instr.calculate_effective_address(0, |_| Ok(0)).unwrap();
        assert_eq!(ea, 0x1234);
        assert_eq!(instr.effective_address, Some(0x1234));
    }

    #[test]
    fn test_calculate_effective_address_with_index() {
        let mut instr = InstructionInfo::decode(0x6040, Some(0x0100)).unwrap();
        assert_eq!(instr.tag, 1); // XR1

        // displacement=0x0100, XR1=0x0050 -> EA = 0x0150
        let ea = instr
            .calculate_effective_address(0x0050, |_| Ok(0))
            .unwrap();
        assert_eq!(ea, 0x0150);
    }

    #[test]
    fn test_calculate_effective_address_indirect() {
        let mut instr = InstructionInfo::decode(0x6020, Some(0x0100)).unwrap();
        assert!(instr.indirect);

        // displacement=0x0100, memory[0x0100]=0x0200 -> EA = 0x0200
        let ea = instr
            .calculate_effective_address(0, |addr| {
                assert_eq!(addr, 0x0100);
                Ok(0x0200)
            })
            .unwrap();
        assert_eq!(ea, 0x0200);
    }

    #[test]
    fn test_calculate_effective_address_with_index_and_indirect() {
        let mut instr = InstructionInfo::decode(0x6060, Some(0x0100)).unwrap();
        assert_eq!(instr.tag, 1); // XR1
        assert!(instr.indirect);

        // displacement=0x0100, XR1=0x0050 -> 0x0150
        // memory[0x0150]=0x0300 -> EA = 0x0300
        let ea = instr
            .calculate_effective_address(0x0050, |addr| {
                assert_eq!(addr, 0x0150);
                Ok(0x0300)
            })
            .unwrap();
        assert_eq!(ea, 0x0300);
    }

    #[test]
    fn test_size_in_words() {
        let short = InstructionInfo::decode(0xB000, None).unwrap();
        assert_eq!(short.size_in_words(), 1);

        let long = InstructionInfo::decode(0x6000, Some(0x1234)).unwrap();
        assert_eq!(long.size_in_words(), 2);
    }

    #[test]
    fn test_all_opcodes_decode() {
        // Test that all defined opcodes can be decoded
        let opcodes = [
            (0x6000, OpCode::LD),
            (0x6800, OpCode::LDD),
            (0x7000, OpCode::STO),
            (0x7800, OpCode::STD),
            (0xE000, OpCode::A),
            (0xE800, OpCode::AD),
            (0xC000, OpCode::S),
            (0xC800, OpCode::SD),
            (0xF000, OpCode::M),
            (0xF800, OpCode::D),
            (0x8000, OpCode::AND),
            (0x9000, OpCode::OR),
            (0x9800, OpCode::EOR),
            (0x2000, OpCode::SLA),
            (0x2800, OpCode::SLCA),
            (0x3000, OpCode::SRA),
            (0x3800, OpCode::SRT),
            (0x4800, OpCode::BSI),
            (0x4000, OpCode::BC),
            (0x5000, OpCode::BSC),
            (0x7400, OpCode::LDX),
            (0x5400, OpCode::STX),
            (0x5800, OpCode::MDX),
            (0xB000, OpCode::WAIT),
            (0xC400, OpCode::LDS),
            (0xCC00, OpCode::STS),
            (0x4400, OpCode::XIO),
            (0x4C00, OpCode::SDS),
        ];

        for (word, expected_op) in &opcodes {
            let decoded_op = OpCode::from_word(*word).unwrap();
            assert_eq!(decoded_op, *expected_op, "Failed for word {:#06x}", word);
        }
    }
}
