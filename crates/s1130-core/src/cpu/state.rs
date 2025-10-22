//! CPU State Representation
//!
//! This module defines the external view of CPU state for debugging and UI.
//! The state is a snapshot that can be serialized and sent across boundaries.

use serde::{Deserialize, Serialize};

/// Snapshot of CPU state at a point in time
///
/// This struct represents all observable CPU state for:
/// - Debugging
/// - UI display
/// - Saving/loading emulator state
/// - Testing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuState {
    /// Accumulator (16-bit)
    pub acc: u16,

    /// Extension register (16-bit)
    pub ext: u16,

    /// Instruction Address Register / Program Counter (16-bit)
    pub iar: u16,

    /// Index Register 1
    pub xr1: u16,

    /// Index Register 2
    pub xr2: u16,

    /// Index Register 3
    pub xr3: u16,

    /// Carry flag
    pub carry: bool,

    /// Overflow flag
    pub overflow: bool,

    /// Wait state (CPU halted)
    pub wait: bool,

    /// Number of instructions executed
    pub instruction_count: u64,

    /// Current interrupt level being serviced (0-5, None if not in interrupt)
    pub current_interrupt_level: Option<u8>,
}

impl CpuState {
    /// Create a new zeroed state
    pub fn new() -> Self {
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
            instruction_count: 0,
            current_interrupt_level: None,
        }
    }

    /// Get combined 32-bit ACC:EXT value
    pub fn acc_ext(&self) -> u32 {
        ((self.acc as u32) << 16) | (self.ext as u32)
    }

    /// Check if CPU is in a halted state
    pub fn is_halted(&self) -> bool {
        self.wait
    }

    /// Check if any status flag is set
    pub fn has_status_flags(&self) -> bool {
        self.carry || self.overflow
    }
}

impl Default for CpuState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_state_new() {
        let state = CpuState::new();
        assert_eq!(state.acc, 0);
        assert_eq!(state.ext, 0);
        assert_eq!(state.iar, 0);
        assert!(!state.carry);
        assert!(!state.overflow);
        assert!(!state.wait);
        assert_eq!(state.instruction_count, 0);
        assert_eq!(state.current_interrupt_level, None);
    }

    #[test]
    fn test_cpu_state_acc_ext() {
        let mut state = CpuState::new();
        state.acc = 0x1234;
        state.ext = 0x5678;
        assert_eq!(state.acc_ext(), 0x12345678);
    }

    #[test]
    fn test_cpu_state_is_halted() {
        let mut state = CpuState::new();
        assert!(!state.is_halted());

        state.wait = true;
        assert!(state.is_halted());
    }

    #[test]
    fn test_cpu_state_has_status_flags() {
        let mut state = CpuState::new();
        assert!(!state.has_status_flags());

        state.carry = true;
        assert!(state.has_status_flags());

        state.carry = false;
        state.overflow = true;
        assert!(state.has_status_flags());

        state.overflow = false;
        assert!(!state.has_status_flags());
    }

    #[test]
    fn test_cpu_state_serialization() {
        let state = CpuState {
            acc: 0x1234,
            ext: 0x5678,
            iar: 0x0100,
            xr1: 0xABCD,
            xr2: 0xEF01,
            xr3: 0x2345,
            carry: true,
            overflow: false,
            wait: false,
            instruction_count: 42,
            current_interrupt_level: Some(4),
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: CpuState = serde_json::from_str(&json).unwrap();

        assert_eq!(state, deserialized);
    }
}
