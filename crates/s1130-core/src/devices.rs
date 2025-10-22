//! I/O Device System for IBM 1130
//!
//! The IBM 1130 supports two types of devices:
//! 1. Block-mode devices (DMA-like): 2501 Card Reader, 2310 Disk Drive
//!    - CPU issues single command, device transfers entire block
//!    - Uses IOCC (I/O Channel Command) structure
//!    - Generates completion interrupt
//! 2. Character-mode devices (CPU-intensive): 1442 Card Read Punch
//!    - CPU issues command for each character
//!    - Device generates interrupt for each character
//!    - High CPU overhead

use crate::error::CpuError;

/// Device function codes (3 bits, values 0-7)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceFunction {
    /// Sense Device Status (check if ready, busy, etc.)
    Sense = 0,
    /// Control operation (start, stop, reset)
    Control = 1,
    /// Initiate Read operation
    InitRead = 2,
    /// Read data (character-mode devices)
    Read = 3,
    /// Initiate Write operation
    InitWrite = 4,
    /// Write data (character-mode devices)
    Write = 5,
    /// Sense Interrupt Level Status Word (ILSW)
    SenseIlsw = 6,
    /// Reserved/undefined
    Reserved = 7,
}

impl DeviceFunction {
    /// Convert from 3-bit function code
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits & 0x07 {
            0 => Some(DeviceFunction::Sense),
            1 => Some(DeviceFunction::Control),
            2 => Some(DeviceFunction::InitRead),
            3 => Some(DeviceFunction::Read),
            4 => Some(DeviceFunction::InitWrite),
            5 => Some(DeviceFunction::Write),
            6 => Some(DeviceFunction::SenseIlsw),
            7 => Some(DeviceFunction::Reserved),
            _ => None,
        }
    }

    /// Convert to 3-bit function code
    pub fn to_bits(self) -> u8 {
        self as u8
    }
}

/// IOCC (I/O Channel Command) structure
///
/// This is a 2-word structure in memory used by block-mode devices:
/// - Word 0 (even address): Word Count Address (WCA)
/// - Word 1 (odd address): Device code (bits 0-4), Function (bits 5-7), Modifiers (bits 8-15)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iocc {
    /// Word Count Address - points to word count and data buffer
    pub wca: u16,
    /// Device code (5 bits, 0-31)
    pub device_code: u8,
    /// Function code (3 bits, 0-7)
    pub function: DeviceFunction,
    /// Modifier bits (8 bits)
    pub modifiers: u8,
}

impl Iocc {
    /// Decode IOCC from two memory words
    ///
    /// # Arguments
    /// * `word1` - First word (WCA - Word Count Address)
    /// * `word2` - Second word (device code + function + modifiers)
    pub fn decode(word1: u16, word2: u16) -> Result<Self, CpuError> {
        let device_code = ((word2 & 0xF800) >> 11) as u8; // Bits 0-4
        let function_bits = ((word2 & 0x0700) >> 8) as u8; // Bits 5-7
        let modifiers = (word2 & 0x00FF) as u8; // Bits 8-15

        let function =
            DeviceFunction::from_bits(function_bits).ok_or(CpuError::InvalidDevice(device_code))?;

        Ok(Iocc {
            wca: word1,
            device_code,
            function,
            modifiers,
        })
    }

    /// Encode IOCC into two memory words
    pub fn encode(&self) -> (u16, u16) {
        let word1 = self.wca;
        let word2 = ((self.device_code as u16) << 11)
            | ((self.function.to_bits() as u16) << 8)
            | (self.modifiers as u16);
        (word1, word2)
    }
}

/// Device trait - all I/O devices must implement this
pub trait Device: Send + Sync {
    /// Get the device code (5-bit identifier, 0-31)
    fn device_code(&self) -> u8;

    /// Get the device name (for debugging/display)
    fn device_name(&self) -> &'static str;

    /// Execute an IOCC command
    ///
    /// This is called by the XIO instruction when the device code matches.
    /// The device should process the command and update its internal state.
    ///
    /// # Arguments
    /// * `iocc` - The decoded IOCC structure
    /// * `memory` - Mutable reference to CPU memory for DMA transfers
    ///
    /// # Returns
    /// * `Ok(())` if command executed successfully
    /// * `Err(CpuError)` if command failed
    fn execute_iocc(&mut self, iocc: &Iocc, memory: &mut [u16]) -> Result<(), CpuError>;

    /// Check if device is busy
    fn is_busy(&self) -> bool;

    /// Reset device to initial state
    fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_function_conversion() {
        assert_eq!(DeviceFunction::from_bits(0), Some(DeviceFunction::Sense));
        assert_eq!(DeviceFunction::from_bits(1), Some(DeviceFunction::Control));
        assert_eq!(DeviceFunction::from_bits(2), Some(DeviceFunction::InitRead));
        assert_eq!(DeviceFunction::from_bits(3), Some(DeviceFunction::Read));
        assert_eq!(
            DeviceFunction::from_bits(4),
            Some(DeviceFunction::InitWrite)
        );
        assert_eq!(DeviceFunction::from_bits(5), Some(DeviceFunction::Write));
        assert_eq!(
            DeviceFunction::from_bits(6),
            Some(DeviceFunction::SenseIlsw)
        );
        assert_eq!(DeviceFunction::from_bits(7), Some(DeviceFunction::Reserved));

        assert_eq!(DeviceFunction::Sense.to_bits(), 0);
        assert_eq!(DeviceFunction::Control.to_bits(), 1);
    }

    #[test]
    fn test_iocc_decode_encode() {
        // Test IOCC structure:
        // Device code = 5 (2501 Card Reader)
        // Function = InitRead (2)
        // Modifiers = 0x42
        let word1 = 0x1000; // WCA
        let word2 = 0x2A42; // (5 << 11) | (2 << 8) | 0x42 = 0x2800 | 0x0200 | 0x42

        let iocc = Iocc::decode(word1, word2).unwrap();
        assert_eq!(iocc.wca, 0x1000);
        assert_eq!(iocc.device_code, 5);
        assert_eq!(iocc.function, DeviceFunction::InitRead);
        assert_eq!(iocc.modifiers, 0x42);

        let (encoded1, encoded2) = iocc.encode();
        assert_eq!(encoded1, word1);
        assert_eq!(encoded2, word2);
    }

    #[test]
    fn test_iocc_decode_all_functions() {
        for func in 0..8 {
            let word1 = 0x0100;
            let word2 = (func as u16) << 8;
            let iocc = Iocc::decode(word1, word2).unwrap();
            assert_eq!(iocc.function.to_bits(), func);
        }
    }
}
