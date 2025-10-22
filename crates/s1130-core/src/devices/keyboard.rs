//! Console Keyboard Device
//!
//! This device emulates a console keyboard for the IBM 1130.
//! It's a character-mode device that provides keyboard input to programs.
//!
//! Device code: 1 (standard console keyboard)
//!
//! Operations:
//! - Sense: Check if a key is ready
//! - Read: Read a character from keyboard buffer

use crate::devices::{Device, DeviceFunction, Iocc};
use crate::error::CpuError;
use std::collections::VecDeque;

/// Console Keyboard Device
///
/// This is a character-mode input device. Programs use XIO to:
/// 1. Sense if a character is ready
/// 2. Read characters one at a time
pub struct DeviceConsoleKeyboard {
    /// Input buffer (characters waiting to be read)
    input_buffer: VecDeque<u16>,

    /// Device status flags
    busy: bool,
}

impl DeviceConsoleKeyboard {
    /// Create a new console keyboard device
    pub fn new() -> Self {
        Self {
            input_buffer: VecDeque::new(),
            busy: false,
        }
    }

    /// Add a character to the input buffer
    ///
    /// This simulates a user typing a key. In a real system, this would
    /// be triggered by actual keyboard hardware.
    ///
    /// # Arguments
    /// * `ch` - The character to add (as a 16-bit word, typically ASCII in low byte)
    pub fn type_char(&mut self, ch: u16) {
        self.input_buffer.push_back(ch);
    }

    /// Type a string of characters
    ///
    /// Convenience method to simulate typing multiple characters.
    ///
    /// # Arguments
    /// * `s` - The string to type
    pub fn type_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.input_buffer.push_back(ch as u16);
        }
    }

    /// Check if a character is available
    pub fn has_char(&self) -> bool {
        !self.input_buffer.is_empty()
    }

    /// Read a character from the buffer
    fn read_char(&mut self) -> Option<u16> {
        self.input_buffer.pop_front()
    }
}

impl Default for DeviceConsoleKeyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for DeviceConsoleKeyboard {
    fn device_code(&self) -> u8 {
        1 // Console keyboard
    }

    fn device_name(&self) -> &'static str {
        "Console Keyboard"
    }

    fn execute_iocc(&mut self, iocc: &Iocc, memory: &mut [u16]) -> Result<(), CpuError> {
        match iocc.function {
            DeviceFunction::Sense => {
                // Sense operation: return status in WCA location
                // Bit 15 (LSB) = 1 if character ready
                let status = if self.has_char() { 1 } else { 0 };
                if (iocc.wca as usize) < memory.len() {
                    memory[iocc.wca as usize] = status;
                }
                Ok(())
            }

            DeviceFunction::Read => {
                // Read operation: read one character into WCA location
                if let Some(ch) = self.read_char() {
                    if (iocc.wca as usize) < memory.len() {
                        memory[iocc.wca as usize] = ch;
                    }
                    Ok(())
                } else {
                    Err(CpuError::DeviceError(
                        "Keyboard: No character available".to_string(),
                    ))
                }
            }

            _ => Err(CpuError::DeviceError(format!(
                "Keyboard: Unsupported function {:?}",
                iocc.function
            ))),
        }
    }

    fn is_busy(&self) -> bool {
        self.busy
    }

    fn reset(&mut self) {
        self.input_buffer.clear();
        self.busy = false;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_creation() {
        let kb = DeviceConsoleKeyboard::new();
        assert_eq!(kb.device_code(), 1);
        assert_eq!(kb.device_name(), "Console Keyboard");
        assert!(!kb.has_char());
    }

    #[test]
    fn test_type_char() {
        let mut kb = DeviceConsoleKeyboard::new();
        kb.type_char(b'A' as u16);
        assert!(kb.has_char());
    }

    #[test]
    fn test_type_string() {
        let mut kb = DeviceConsoleKeyboard::new();
        kb.type_string("hello");
        assert!(kb.has_char());
        assert_eq!(kb.read_char(), Some(b'h' as u16));
        assert_eq!(kb.read_char(), Some(b'e' as u16));
    }

    #[test]
    fn test_sense_operation() {
        let mut kb = DeviceConsoleKeyboard::new();
        let mut memory = vec![0u16; 100];

        // Sense with no character ready
        let iocc = Iocc {
            wca: 50,
            device_code: 1,
            function: DeviceFunction::Sense,
            modifiers: 0,
        };
        kb.execute_iocc(&iocc, &mut memory).unwrap();
        assert_eq!(memory[50], 0); // No character ready

        // Add a character and sense again
        kb.type_char(b'X' as u16);
        kb.execute_iocc(&iocc, &mut memory).unwrap();
        assert_eq!(memory[50], 1); // Character ready
    }

    #[test]
    fn test_read_operation() {
        let mut kb = DeviceConsoleKeyboard::new();
        let mut memory = vec![0u16; 100];

        kb.type_char(b'A' as u16);

        let iocc = Iocc {
            wca: 50,
            device_code: 1,
            function: DeviceFunction::Read,
            modifiers: 0,
        };

        kb.execute_iocc(&iocc, &mut memory).unwrap();
        assert_eq!(memory[50], b'A' as u16);
        assert!(!kb.has_char()); // Buffer should be empty now
    }
}
