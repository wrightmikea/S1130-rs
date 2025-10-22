//! CPU Register Management
//!
//! This module handles all CPU registers in isolation.
//! Each register has clear getter/setter methods with no side effects.

use serde::{Deserialize, Serialize};

/// Index registers (XR1, XR2, XR3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexRegisters {
    pub xr1: u16,
    pub xr2: u16,
    pub xr3: u16,
}

impl IndexRegisters {
    /// Create new index registers, all initialized to zero
    pub fn new() -> Self {
        Self {
            xr1: 0,
            xr2: 0,
            xr3: 0,
        }
    }

    /// Get index register by tag (1=XR1, 2=XR2, 3=XR3, 0=none)
    ///
    /// Returns 0 for invalid tags (including tag 0, which means "no index register")
    pub fn get(&self, tag: u8) -> u16 {
        match tag {
            1 => self.xr1,
            2 => self.xr2,
            3 => self.xr3,
            _ => 0,
        }
    }

    /// Set index register by tag (1=XR1, 2=XR2, 3=XR3)
    ///
    /// Silently ignores invalid tags
    pub fn set(&mut self, tag: u8, value: u16) {
        match tag {
            1 => self.xr1 = value,
            2 => self.xr2 = value,
            3 => self.xr3 = value,
            _ => {}
        }
    }

    /// Reset all index registers to zero
    pub fn reset(&mut self) {
        self.xr1 = 0;
        self.xr2 = 0;
        self.xr3 = 0;
    }
}

impl Default for IndexRegisters {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU Status Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusFlags {
    pub carry: bool,
    pub overflow: bool,
    pub wait: bool,
}

impl StatusFlags {
    /// Create new status flags, all cleared
    pub fn new() -> Self {
        Self {
            carry: false,
            overflow: false,
            wait: false,
        }
    }

    /// Reset all flags to cleared state
    pub fn reset(&mut self) {
        self.carry = false;
        self.overflow = false;
        self.wait = false;
    }

    /// Pack flags into a 16-bit word (for LDS/STS instructions)
    ///
    /// Bit layout: [15:Carry, 14:Overflow, ... ]
    pub fn to_word(&self) -> u16 {
        let mut word = 0u16;
        if self.carry {
            word |= 0x8000;
        }
        if self.overflow {
            word |= 0x4000;
        }
        // Wait flag is not typically stored in status word
        word
    }

    /// Unpack flags from a 16-bit word
    pub fn from_word(word: u16) -> Self {
        Self {
            carry: (word & 0x8000) != 0,
            overflow: (word & 0x4000) != 0,
            wait: false, // Not stored in status word
        }
    }
}

impl Default for StatusFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_registers_new() {
        let xr = IndexRegisters::new();
        assert_eq!(xr.xr1, 0);
        assert_eq!(xr.xr2, 0);
        assert_eq!(xr.xr3, 0);
    }

    #[test]
    fn test_index_register_get() {
        let mut xr = IndexRegisters::new();
        xr.xr1 = 0x1111;
        xr.xr2 = 0x2222;
        xr.xr3 = 0x3333;

        assert_eq!(xr.get(1), 0x1111);
        assert_eq!(xr.get(2), 0x2222);
        assert_eq!(xr.get(3), 0x3333);
        assert_eq!(xr.get(0), 0); // Tag 0 = no index
        assert_eq!(xr.get(4), 0); // Invalid tag
    }

    #[test]
    fn test_index_register_set() {
        let mut xr = IndexRegisters::new();

        xr.set(1, 0xABCD);
        assert_eq!(xr.xr1, 0xABCD);

        xr.set(2, 0x1234);
        assert_eq!(xr.xr2, 0x1234);

        xr.set(3, 0x5678);
        assert_eq!(xr.xr3, 0x5678);

        // Invalid tags should be ignored
        xr.set(0, 0xFFFF);
        xr.set(4, 0xFFFF);
        assert_eq!(xr.xr1, 0xABCD); // Unchanged
    }

    #[test]
    fn test_index_registers_reset() {
        let mut xr = IndexRegisters::new();
        xr.xr1 = 0x1111;
        xr.xr2 = 0x2222;
        xr.xr3 = 0x3333;

        xr.reset();

        assert_eq!(xr.xr1, 0);
        assert_eq!(xr.xr2, 0);
        assert_eq!(xr.xr3, 0);
    }

    #[test]
    fn test_status_flags_new() {
        let flags = StatusFlags::new();
        assert!(!flags.carry);
        assert!(!flags.overflow);
        assert!(!flags.wait);
    }

    #[test]
    fn test_status_flags_reset() {
        let mut flags = StatusFlags::new();
        flags.carry = true;
        flags.overflow = true;
        flags.wait = true;

        flags.reset();

        assert!(!flags.carry);
        assert!(!flags.overflow);
        assert!(!flags.wait);
    }

    #[test]
    fn test_status_flags_to_word() {
        let mut flags = StatusFlags::new();
        assert_eq!(flags.to_word(), 0x0000);

        flags.carry = true;
        assert_eq!(flags.to_word(), 0x8000);

        flags.overflow = true;
        assert_eq!(flags.to_word(), 0xC000);
    }

    #[test]
    fn test_status_flags_from_word() {
        let flags = StatusFlags::from_word(0x0000);
        assert!(!flags.carry);
        assert!(!flags.overflow);

        let flags = StatusFlags::from_word(0x8000);
        assert!(flags.carry);
        assert!(!flags.overflow);

        let flags = StatusFlags::from_word(0xC000);
        assert!(flags.carry);
        assert!(flags.overflow);
    }
}
